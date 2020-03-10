#[macro_use]
extern crate pretty_assertions;
// #[macro_use]
// extern crate indoc;

extern crate bumpalo;
extern crate roc_mono;

mod helpers;

// Test monomorphization
#[cfg(test)]
mod test_mono {
    use crate::helpers::{can_expr, infer_expr, test_home, CanExprOut};
    use bumpalo::Bump;
    use roc_collections::all::MutMap;
    use roc_module::ident::TagName::*;
    use roc_module::symbol::{Interns, Symbol};
    use roc_mono::expr::Expr::{self, *};
    use roc_mono::layout::{Builtin, Layout};
    use roc_types::subs::Subs;

    // HELPERS

    fn compiles_to(src: &str, expected: Expr<'_>) {
        compiles_to_with_interns(src, |_| expected)
    }

    fn compiles_to_with_interns<'a, F>(src: &str, get_expected: F)
    where
        F: FnOnce(Interns) -> Expr<'a>,
    {
        let arena = Bump::new();
        let CanExprOut {
            loc_expr,
            var_store,
            var,
            constraint,
            home,
            mut interns,
            ..
        } = can_expr(src);
        let subs = Subs::new(var_store.into());
        let mut unify_problems = Vec::new();
        let (_content, subs) = infer_expr(subs, &mut unify_problems, &constraint, var);

        // Compile and add all the Procs before adding main
        let mut procs = MutMap::default();
        let mut ident_ids = interns.all_ident_ids.remove(&home).unwrap();

        // Populate Procs and Subs, and get the low-level Expr from the canonical Expr
        let mono_expr = Expr::new(
            &arena,
            &subs,
            loc_expr.value,
            &mut procs,
            home,
            &mut ident_ids,
        );

        // Put this module's ident_ids back in the interns
        interns.all_ident_ids.insert(home, ident_ids);

        assert_eq!(mono_expr, get_expected(interns));
    }

    #[test]
    fn int_literal() {
        compiles_to("5", Int(5));
    }

    #[test]
    fn float_literal() {
        compiles_to("0.5", Float(0.5));
    }

    #[test]
    fn bool_literal() {
        let arena = Bump::new();

        compiles_to_with_interns(
            r#"
                x : Bool
                x = True

                x
            "#,
            |interns| {
                let home = test_home();
                let var_x = interns.symbol(home, "x".into());

                let stores = [(
                    var_x,
                    Layout::Builtin(Builtin::Bool(Global("False".into()), Global("True".into()))),
                    Bool(true),
                )];

                let load = Load(var_x);

                Store(arena.alloc(stores), arena.alloc(load))
            },
        );
    }

    //    #[test]
    //    fn two_element_enum() {
    //        compiles_to(
    //            r#"
    //            x : [ Yes, No ]
    //            x = No
    //
    //            x
    //            "#,
    //            Int(32),
    //        );
    //    }
    //
    //    #[test]
    //    fn three_element_enum() {
    //        compiles_to(
    //            r#"
    //            # this test is brought to you by fruits.com!
    //            x : [ Apple, Orange, Banana ]
    //            x = Orange
    //
    //            x
    //            "#,
    //            Int(32),
    //        );
    //    }

    #[test]
    fn set_unique_int_list() {
        compiles_to("List.getUnsafe (List.set [ 12, 9, 7, 3 ] 1 42) 1", {
            CallByName(
                Symbol::LIST_GET_UNSAFE,
                &vec![
                    (
                        CallByName(
                            Symbol::LIST_SET,
                            &vec![
                                (
                                    Array {
                                        elem_layout: Layout::Builtin(Builtin::Int64),
                                        elems: &vec![Int(12), Int(9), Int(7), Int(3)],
                                    },
                                    Layout::Builtin(Builtin::List(&Layout::Builtin(
                                        Builtin::Int64,
                                    ))),
                                ),
                                (Int(1), Layout::Builtin(Builtin::Int64)),
                                (Int(42), Layout::Builtin(Builtin::Int64)),
                            ],
                        ),
                        Layout::Builtin(Builtin::List(&Layout::Builtin(Builtin::Int64))),
                    ),
                    (Int(1), Layout::Builtin(Builtin::Int64)),
                ],
            )
        });
    }
}
