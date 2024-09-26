app [makeGlue] { pf: platform "../platform/main.roc" }

import pf.Types exposing [Types]
import pf.Shape exposing [Shape, RocFn]
import pf.File exposing [File]
import pf.TypeId exposing [TypeId]
import "../static/Cargo.toml" as rocAppCargoToml : Str
import "../../roc_std/Cargo.toml" as rocStdCargoToml : Str
import "../../roc_std/src/lib.rs" as rocStdLib : Str
import "../../roc_std/src/roc_box.rs" as rocStdBox : Str
import "../../roc_std/src/roc_list.rs" as rocStdList : Str
import "../../roc_std/src/roc_str.rs" as rocStdStr : Str
import "../../roc_std/src/storage.rs" as rocStdStorage : Str

makeGlue : List Types -> Result (List File) Str
makeGlue = \typesByArch ->
    modFileContent =
        List.walk typesByArch fileHeader \content, types ->
            arch = (Types.target types).architecture
            archStr = archName arch

            Str.concat
                content
                """
                #[cfg(target_arch = "$(archStr)")]
                mod $(archStr);
                #[cfg(target_arch = "$(archStr)")]
                pub use $(archStr)::*;

                """

    typesByArch
    |> List.map generateArchFile
    |> List.append { name: "roc_app/src/lib.rs", content: modFileContent }
    |> List.concat staticFiles
    |> Ok

## These are always included, and don't depend on the specifics of the app.
staticFiles : List File
staticFiles = [
    { name: "roc_app/Cargo.toml", content: rocAppCargoToml },
    { name: "roc_std/Cargo.toml", content: rocStdCargoToml },
    { name: "roc_std/src/lib.rs", content: rocStdLib },
    { name: "roc_std/src/roc_box.rs", content: rocStdBox },
    { name: "roc_std/src/roc_list.rs", content: rocStdList },
    { name: "roc_std/src/roc_str.rs", content: rocStdStr },
    { name: "roc_std/src/storage.rs", content: rocStdStorage },
]

Tld : [
    EntryPoint
        {
            symbol : Str,
            type : TypeId,
        },
    RecordStructDef
        {
            symbol : Str,
            fields : List { name : Str, type : TypeId },
        },
    RefcountImpl,
]

generateArchFile : Types -> File
generateArchFile = \types ->
    content =
        types
        |> typesToTlds
        |> Set.map generateTld
        |> Set.toList
        |> Str.joinWith "\n\n"

    arch = (Types.target types).architecture
    archStr = archName arch

    {
        name: "roc_app/src/$(archStr).rs",
        content,
    }

archName = \arch ->
    when arch is
        Aarch32 -> "arm"
        Aarch64 -> "aarch64"
        Wasm32 -> "wasm32"
        X86x32 -> "x86"
        X86x64 -> "x86_64"

typesToTlds : Types -> Set Tld
typesToTlds = \types ->
    Types.walkShapes types (Set.empty {}) \tlds, shape, typeId ->
        when shape is
            Struct { name: recordName, fields } ->
                # this might be a problem, but the previous RustGlue doesn't do anything with this either
                getFields = \list -> List.map list \{ name, id } -> { name, type: id }

                justFields =
                    when fields is
                        HasNoClosure list -> getFields list
                        HasClosure list -> getFields list

                structDef =
                    RecordStructDef {
                        symbol: recordName,
                        fields: justFields,
                    }

                refcountImpl = RefcountImpl

                tlds
                |> Set.insert structDef

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
                # or represented as Rust builtins
                tlds

            _ -> crash "todo"

generateTld : Tld -> Str
generateTld = Inspect.toStr
