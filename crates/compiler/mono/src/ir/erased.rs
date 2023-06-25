use bumpalo::{collections::Vec as AVec, Bump};
use roc_module::{low_level::LowLevel, symbol::Symbol};
use roc_types::subs::Variable;

use crate::{
    borrow::Ownership,
    layout::{ErasedIndex, FunctionPointer, InLayout, Layout, LayoutCache, LayoutRepr},
};

use super::{
    with_hole, BranchInfo, Call, CallType, Env, Expr, JoinPointId, Param, Procs, Stmt, UpdateModeId,
};

const ERASED_FUNCTION_FIELD_LAYOUTS: &[InLayout] =
    &[Layout::OPAQUE_PTR, Layout::OPAQUE_PTR, Layout::OPAQUE_PTR];

fn index_erased_function<'a>(
    arena: &'a Bump,
    assign_to: Symbol,
    erased_function: Symbol,
    index: ErasedIndex,
) -> impl FnOnce(Stmt<'a>) -> Stmt<'a> {
    move |rest| {
        Stmt::Let(
            assign_to,
            Expr::StructAtIndex {
                index: index as _,
                structure: erased_function,
                field_layouts: ERASED_FUNCTION_FIELD_LAYOUTS,
            },
            Layout::OPAQUE_PTR,
            arena.alloc(rest),
        )
    }
}

fn cast_erased_callee<'a>(
    arena: &'a Bump,
    assign_to: Symbol,
    erased_function: Symbol,
    fn_ptr_layout: InLayout<'a>,
) -> impl FnOnce(Stmt<'a>) -> Stmt<'a> {
    move |rest| {
        Stmt::Let(
            assign_to,
            Expr::Call(Call {
                call_type: CallType::LowLevel {
                    op: LowLevel::PtrCast,
                    update_mode: UpdateModeId::BACKEND_DUMMY,
                },
                arguments: arena.alloc([erased_function]),
            }),
            fn_ptr_layout,
            arena.alloc(rest),
        )
    }
}

fn call_callee<'a>(
    arena: &'a Bump,
    result_symbol: Symbol,
    result: InLayout<'a>,
    fn_ptr_symbol: Symbol,
    fn_arg_layouts: &'a [InLayout<'a>],
    fn_arguments: &'a [Symbol],
) -> impl FnOnce(Stmt<'a>) -> Stmt<'a> {
    move |rest| {
        Stmt::Let(
            result_symbol,
            Expr::Call(Call {
                call_type: CallType::ByPointer {
                    pointer: fn_ptr_symbol,
                    ret_layout: result,
                    arg_layouts: fn_arg_layouts,
                },
                arguments: fn_arguments,
            }),
            result,
            arena.alloc(rest),
        )
    }
}

fn is_null<'a>(
    env: &mut Env<'a, '_>,
    arena: &'a Bump,
    assign_to: Symbol,
    ptr_symbol: Symbol,
) -> impl FnOnce(Stmt<'a>) -> Stmt<'a> {
    let null_symbol = env.unique_symbol();
    move |rest| {
        Stmt::Let(
            null_symbol,
            Expr::NullPointer,
            Layout::OPAQUE_PTR,
            arena.alloc(Stmt::Let(
                assign_to,
                Expr::Call(Call {
                    call_type: CallType::LowLevel {
                        op: LowLevel::Eq,
                        update_mode: UpdateModeId::BACKEND_DUMMY,
                    },
                    arguments: arena.alloc([ptr_symbol, null_symbol]),
                }),
                Layout::BOOL,
                arena.alloc(rest),
            )),
        )
    }
}

/// Given
///
/// ```
/// Call(f, args)
/// ```
///
/// We generate
///
/// ```
/// f = compile(f)
/// joinpoint join result:
///    <hole>
/// if (f.value) {
///    f = cast(f.callee, (..params, void*) -> ret);
///    result = f ..args f.value
///    jump join result
/// } else {
///    f = cast(f.callee, (..params) -> ret);
///    result = f ..args
///    jump join result
/// }
/// ```
pub fn call_erased_function<'a>(
    env: &mut Env<'a, '_>,
    layout_cache: &mut LayoutCache<'a>,
    procs: &mut Procs<'a>,
    function_expr: roc_can::expr::Expr,
    function_var: Variable,
    function_signature: (&'a [InLayout<'a>], InLayout<'a>),
    function_argument_symbols: &'a [Symbol],
    call_result_symbol: Symbol,
    hole: &'a Stmt<'a>,
    hole_layout: InLayout<'a>,
) -> Stmt<'a> {
    let arena = env.arena;
    let (f_args, f_ret) = function_signature;

    let f = env.unique_symbol();

    let join_point_id = JoinPointId(env.unique_symbol());

    // f_value = f.value
    let f_value = env.unique_symbol();
    let let_f_value = index_erased_function(arena, f_value, f, ErasedIndex::Value);

    // f_callee = f.callee
    let f_callee = env.unique_symbol();
    let let_f_callee = index_erased_function(arena, f_callee, f, ErasedIndex::Callee);

    let mut build_closure_data_branch = |env: &mut Env, pass_closure| {
        // f_callee = cast(f_callee, (..params) -> ret);
        // result = f_callee ..args
        // jump join result

        let (f_args, function_argument_symbols) = if pass_closure {
            // f_args = ...args, f.value
            // function_argument_symbols = ...args, f.value
            let f_args = {
                let mut args = AVec::with_capacity_in(f_args.len() + 1, arena);
                args.extend(f_args.iter().chain(&[Layout::OPAQUE_PTR]).copied());
                args.into_bump_slice()
            };
            let function_argument_symbols = {
                let mut args = AVec::with_capacity_in(function_argument_symbols.len() + 1, arena);
                args.extend(function_argument_symbols.iter().chain(&[f_value]));
                args.into_bump_slice()
            };
            (f_args, function_argument_symbols)
        } else {
            (f_args, function_argument_symbols)
        };

        let fn_ptr_layout =
            layout_cache.put_in_direct_no_semantic(LayoutRepr::FunctionPointer(FunctionPointer {
                args: f_args,
                ret: f_ret,
            }));

        let f_callee_cast = env.unique_symbol();
        let let_f_callee_cast = cast_erased_callee(arena, f_callee_cast, f_callee, fn_ptr_layout);

        let result = env.unique_symbol();
        let let_result = call_callee(
            arena,
            result,
            f_ret,
            f_callee_cast,
            f_args,
            function_argument_symbols,
        );

        let_f_callee_cast(
            //
            let_result(
                //
                Stmt::Jump(join_point_id, arena.alloc([result])),
            ),
        )
    };

    let value_is_null = env.unique_symbol();
    let let_value_is_null = is_null(env, arena, value_is_null, f_value);

    let call_and_jump_on_value = let_value_is_null(
        //
        Stmt::Switch {
            cond_symbol: value_is_null,
            cond_layout: Layout::BOOL,
            // value == null
            branches: arena.alloc([(0, BranchInfo::None, build_closure_data_branch(env, false))]),
            // value != null
            default_branch: (
                BranchInfo::None,
                arena.alloc(build_closure_data_branch(env, true)),
            ),
            ret_layout: hole_layout,
        },
    );

    let joinpoint = {
        let param = Param {
            symbol: call_result_symbol,
            layout: f_ret,
            ownership: Ownership::Owned,
        };

        let remainder =
            // f_value = f.value
            let_f_value(
            // f_callee = f.callee
            let_f_callee(
                //
                call_and_jump_on_value,
            ),
        );

        Stmt::Join {
            id: join_point_id,
            parameters: env.arena.alloc([param]),
            body: hole,
            remainder: arena.alloc(remainder),
        }
    };

    // Compile the function expression into f_val
    with_hole(
        env,
        function_expr,
        function_var,
        procs,
        layout_cache,
        f,
        env.arena.alloc(joinpoint),
    )
}
