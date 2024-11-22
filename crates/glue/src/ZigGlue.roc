app [makeGlue] { pf: platform "../platform/main.roc" }

import pf.Types exposing [Types]
import pf.File exposing [File]
import pf.TypeId exposing [TypeId]
import "../../compiler/builtins/bitcode/src/list.zig" as rocStdList : Str
import "../../compiler/builtins/bitcode/src/str.zig" as rocStdStr : Str
import "../../compiler/builtins/bitcode/src/utils.zig" as rocStdUtils : Str
import "../../compiler/builtins/bitcode/src/sort.zig" as rocStdSort : Str
import "../../compiler/builtins/bitcode/src/panic.zig" as rocStdPanic : Str

makeGlue : List Types -> Result (List File) Str
makeGlue = \typesByArch ->
    main = { name: "roc_app/roc_app.zig", content: mainFileContent typesByArch }

    archFiles = List.map typesByArch generateArchFile

    archFiles
    |> List.append main
    |> List.concat staticFiles
    |> Ok

mainFileContent : List Types -> Str
mainFileContent = \typesByArch ->
    prongs =
        List.walk typesByArch [] \list, types ->
            arch = types |> Types.target |> .architecture |> archName

            prong =
                """
                .$(arch) => @import("$(arch).zig"),
                """

            List.append list prong
        |> Str.joinWith "\n"

    """
    $(fileHeader)

    const builtin = @import("builtin");

    pub usingnamespace switch (builtin.target.cpu.arch) {
        $(prongs |> indentedBy 1)
        else => @compileError("target not supported!"),
    };
    """

## These are always included, and don't depend on the specifics of the app.
staticFiles : List File
staticFiles = [
    { name: "roc_std/list.zig", content: rocStdList },
    { name: "roc_std/str.zig", content: rocStdStr },
    { name: "roc_std/utils.zig", content: rocStdUtils },
    { name: "roc_std/sort.zig", content: rocStdSort },
    { name: "roc_std/panic.zig", content: rocStdPanic },
]

generateArchFile : Types -> File
generateArchFile = \types ->
    content =
        List.concat (typesToItemGroups types) (entrypoints types)
        |> List.map generateItemGroup
        |> Str.joinWith "\n\n"
        |> Str.withPrefix archFileHeader

    archStr =
        types
        |> Types.target
        |> .architecture
        |> archName

    {
        name: "roc_app/$(archStr).zig",
        content,
    }

archName = \arch ->
    when arch is
        Aarch32 -> "arm"
        Aarch64 -> "aarch64"
        Wasm32 -> "wasm32"
        X86x32 -> "x86"
        X86x64 -> "x86_64"

ZigSymbol : Str
ZigType : Str

ItemGroup : [
    Entrypoint
        {
            name : ZigSymbol,
            number : U64,
            args : List ZigType,
            ret : ZigType,
        },
    Record
        {
            name : ZigSymbol,
            fields : List { name : ZigSymbol, type : ZigType },
        },
    TagUnion
        [
            Enumeration
                {
                    name : ZigSymbol,
                    repr : ZigType,
                    tags : List ZigSymbol,
                },
        ],
]

entrypoints : Types -> List ItemGroup
entrypoints = \types ->
    types
    |> Types.entryPoints
    |> List.mapWithIndex \T name type, number ->
        (args, ret) =
            when Types.shape types type is
                Function func ->
                    nonUnitArgTypes = List.keepOks func.args \arg ->
                        when zigTypeName types arg is
                            "void" -> Err WasUnit
                            nonUnitType -> Ok nonUnitType

                    (
                        nonUnitArgTypes,
                        zigTypeName types func.ret,
                    )

                _ -> ([], zigTypeName types type)
        Entrypoint { name, number: number + 1, args, ret }

typesToItemGroups : Types -> List ItemGroup
typesToItemGroups = \types ->
    Types.walkShapes types [] \itemGroups, _, type ->
        List.appendIfOk itemGroups (typeToItemGroup types type)

typeToItemGroup : Types, TypeId -> Result ItemGroup [NoItemsNeeded]
typeToItemGroup = \types, type ->
    shape = Types.shape types type
    when shape is
        Struct { name: recordName, fields } ->
            getFields = \{ name, id } -> { name, type: zigTypeName types id }

            recordFields =
                when fields is
                    HasNoClosure list -> List.map list getFields
                    HasClosure list -> List.map list getFields

            Record {
                name: escape recordName,
                fields: recordFields,
            }
            |> Ok

        TagUnion (Enumeration { name, tags, size }) ->
            Ok (TagUnion (Enumeration { name: escape name, repr: discriminantRepr size, tags }))

        Unit
        | Unsized
        | EmptyTagUnion
        | Num _
        | Bool
        | RocResult _ _
        | RocStr
        | RocDict _ _
        | RocSet _
        | RocList _
        | RocBox _ ->
            # these are taken care of by the Roc std library
            # or represented as Zig builtins
            Err NoItemsNeeded

        other -> crash "todo: $(Inspect.toStr other)"

zigTypeName : Types, TypeId -> ZigType
zigTypeName = \types, type ->
    # helper = \id -> zigTypeName types id

    when Types.shape types type is
        RocStr -> "RocStr"
        Bool -> "bool"
        Num U8 -> "u8"
        Num I8 -> "i8"
        Num U16 -> "u16"
        Num I16 -> "i16"
        Num U32 -> "u32"
        Num I32 -> "i32"
        Num U64 -> "u64"
        Num I64 -> "i64"
        Num U128 -> "u128"
        Num I128 -> "i128"
        Num F32 -> "f32"
        Num F64 -> "f64"
        Num Dec -> "RocDec"
        RocResult a b -> crash "todo"
        RocList a -> crash "todo"
        RocBox a -> crash "todo"
        TagUnion (Enumeration { name }) -> escape name
        TagUnion (NonRecursive { name }) -> escape name
        TagUnion (Recursive { name }) -> escape name
        TagUnion (NullableWrapped { name }) -> escape name
        TagUnion (NonNullableUnwrapped { name }) -> escape name
        TagUnion (SingleTagStruct { name }) -> escape name
        TagUnion (NullableUnwrapped { name }) -> escape name
        EmptyTagUnion -> crash "should never happen"
        Struct { name } -> escape name
        Function { functionName } -> escape functionName
        TagUnionPayload { name } -> escape name
        Unit -> "void"
        RecursivePointer _ | Unsized -> crash "what"
        RocDict _ _ | RocSet _ -> crash "todo"

discriminantRepr : U32 -> ZigType
discriminantRepr = \bytes ->
    when bytes is
        0 -> crash "this is definitely a bug!"
        1 -> "u8"
        2 -> "u16"
        3 | 4 -> "u32"
        5 | 6 | 7 | 8 -> "u64"
        _ -> crash "this is probably a bug!"

reservedKeywords = Set.fromList [
    "addrspace",
    "align",
    "allowzero",
    "and",
    "anyframe",
    "anytype",
    "asm",
    "async",
    "await",
    "break",
    "callconv",
    "catch",
    "comptime",
    "const",
    "continue",
    "defer",
    "else",
    "enum",
    "errdefer",
    "error",
    "export",
    "extern",
    "fn",
    "for",
    "if",
    "inline",
    "linksection",
    "noalias",
    "noinline",
    "nosuspend",
    "opaque",
    "or",
    "orelse",
    "packed",
    "pub",
    "resume",
    "return",
    "struct",
    "suspend",
    "switch",
    "test",
    "threadlocal",
    "try",
    "union",
    "unreachable",
    "usingnamespace",
    "var",
    "volatile",
    "while",
]

escape : Str -> ZigType
escape = \identifier ->
    if Set.contains reservedKeywords identifier then
        """
        @"$(identifier)"
        """
    else
        identifier

## indents all lines except the first so multiline substitions play nice in string interpolation
## e.g:
## """
## fn do_stuff() {
##     $(stuff |> indentedBy 1)
## }
## """
indentedBy : Str, U64 -> Str
indentedBy = \code, amount ->
    indent = Str.repeat "\t" amount

    Str.replaceEach code "\n" "\n$(indent)"

fileHeader =
    """
    // ⚠️ GENERATED CODE ⚠️ 
    //
    // This package is generated by the `roc glue` CLI command
    """

imports =
    """
    pub const RocStr = @import("../roc_std/str.zig").RocStr;
    """

archFileHeader =
    """
    $(fileHeader)

    $(imports)


    """

generateItemGroup : ItemGroup -> Str
generateItemGroup = \itemGroup ->
    when itemGroup is
        Entrypoint { name, number, args, ret } ->
            defArgs =
                args
                |> List.mapWithIndex \type, n -> "arg$(Num.toStr n): $(type)"
                |> Str.joinWith ", "

            callArgs =
                args
                |> List.mapWithIndex \_, n -> "arg$(Num.toStr n)"
                |> Str.joinWith ", "

            """
            extern fn roc__$(name)_$(Num.toStr number)_exposed_generic(ret: *$(ret), $(defArgs)) callconv(.C) void;

            pub fn $(name)($(defArgs)) $(ret) {
                var ret: $(ret) = undefined;

                roc__$(name)_1_exposed_generic(&ret, $(callArgs));

                return ret;
            }
            """

        Record { name, fields } ->
            fieldList =
                fields
                |> List.map \field -> "$(field.name): $(field.type),"
                |> Str.joinWith "\n"

            """
            pub const $(name) = extern struct {
                $(fieldList |> indentedBy 1)
            };
            """

        TagUnion (Enumeration { name, repr, tags }) ->
            tagList =
                tags
                |> List.map \tag -> "$(tag),"
                |> Str.joinWith "\n"

            """
            pub const $(name) = enum($(repr)) {
                $(tagList |> indentedBy 1)
            };
            """
