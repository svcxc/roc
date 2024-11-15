#[macro_use]
extern crate pretty_assertions;

#[macro_use]
extern crate indoc;

extern crate dircpy;
extern crate roc_collections;

mod helpers;

#[cfg(test)]
mod glue_cli_run {
    use crate::helpers::fixtures_dir;
    use cli_utils::helpers::{has_error, run_glue, run_roc, Out};
    use std::fs;
    use std::path::{Path, PathBuf};

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    const TEST_LEGACY_LINKER: bool = true;

    // Surgical linker currently only supports linux x86_64,
    // so we're always testing the legacy linker on other targets.
    #[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
    const TEST_LEGACY_LINKER: bool = false;

    /// This macro does two things.
    ///
    /// First, it generates and runs a separate test for each of the given
    /// expected stdout endings. Each of these should test a particular .roc file
    /// in the fixtures/ directory. The fixtures themselves run assertions too, but
    /// the stdout check verifies that we're actually running the code we think we are;
    /// without it, it would be possible that the fixtures are just exiting without running
    /// any assertions, and we would have no way to find out!
    ///
    /// Second, this generates an extra test which (non-recursively) traverses the
    /// fixtures/ directory and verifies that each of the .roc files in there
    /// has had a corresponding test generated in the previous step. This test
    /// will fail if we ever add a new .roc file to fixtures/ and forget to
    /// add a test for it here!
    macro_rules! fixtures {
        ($($test_name:ident:$fixture_dir:expr => $ends_with:expr,)+) => {
            $(
                #[test]
                #[allow(non_snake_case)]
                fn $test_name() {
                    let dir = fixtures_dir($fixture_dir);

                    generate_glue_for(&dir, std::iter::empty());

                    fn validate<'a, I: IntoIterator<Item = &'a str>>(dir: PathBuf, args: I) {
                        let out = run_app(&dir.join("app.roc"), args);

                        assert!(out.status.success());
                        let ignorable = "🔨 Rebuilding platform...\n";
                        let stderr = out.stderr.replacen(ignorable, "", 1);
                        assert_eq!(stderr, "");
                        assert!(
                            out.stdout.ends_with($ends_with),
                            "Unexpected stdout ending\n\n  expected:\n\n    {}\n\n  but stdout was:\n\n    {}",
                            $ends_with,
                            out.stdout
                        );
                    }


                    let test_name_str = stringify!($test_name);

                    // TODO after #5924 is fixed; remove this
                    let skip_on_linux_surgical_linker = ["closures", "option", "nullable_wrapped", "nullable_unwrapped", "nonnullable_unwrapped", "enumeration", "nested_record", "advanced_recursive_union"];

                    // Validate linux with the default linker.
                    if !(cfg!(target_os = "linux") && (skip_on_linux_surgical_linker.contains(&test_name_str))) {
                        validate(dir.clone(), std::iter::empty());
                    }

                    if TEST_LEGACY_LINKER {
                        validate(dir, ["--linker=legacy"]);
                    }
                }
            )*

            #[test]
            #[ignore]
            fn all_fixtures_have_tests() {
                use roc_collections::VecSet;

                let mut all_fixtures: VecSet<String> = VecSet::default();

                $(
                    all_fixtures.insert($fixture_dir.to_string());
                )*

                check_for_tests(&mut all_fixtures);
            }
        }
    }

    fixtures! {
        basic_record:"basic-record" => "Record was: MyRcd { b: 42, a: 1995 }\n",
        nested_record:"nested-record" => "Record was: Outer { y: \"foo\", z: [1, 2], x: Inner { b: 24.0, a: 5 } }\n",
        enumeration:"enumeration" => "tag_union was: MyEnum::Foo, Bar is: MyEnum::Bar, Baz is: MyEnum::Baz\n",
        single_tag_union:"single-tag-union" => indoc!(r#"
            tag_union was: SingleTagUnion::OneTag
        "#),
        union_with_padding:"union-with-padding" => indoc!(r#"
            tag_union was: NonRecursive::Foo("This is a test")
            `Foo "small str"` is: NonRecursive::Foo("small str")
            `Foo "A long enough string to not be small"` is: NonRecursive::Foo("A long enough string to not be small")
            `Bar 123` is: NonRecursive::Bar(123)
            `Baz` is: NonRecursive::Baz(())
            `Blah 456` is: NonRecursive::Blah(456)
            roc to rusty: NonRecursive::Foo("small str") -> Foo("small str")
            rusty to roc: Foo("small str") -> NonRecursive::Foo("small str")
            roc to rusty: NonRecursive::Foo("A long enough string to not be small") -> Foo("A long enough string to not be small")
            rusty to roc: Foo("A long enough string to not be small") -> NonRecursive::Foo("A long enough string to not be small")
            roc to rusty: NonRecursive::Bar(123) -> Bar(123)
            rusty to roc: Bar(123) -> NonRecursive::Bar(123)
            roc to rusty: NonRecursive::Baz(()) -> Baz
            rusty to roc: Baz -> NonRecursive::Baz(())
            roc to rusty: NonRecursive::Blah(456) -> Blah(456)
            rusty to roc: Blah(456) -> NonRecursive::Blah(456)
        "#),
        union_without_padding:"union-without-padding" => indoc!(r#"
            tag_union was: NonRecursive::Foo("This is a test")
            `Foo "small str"` is: NonRecursive::Foo("small str")
            `Bar 123` is: NonRecursive::Bar(123)
            `Baz` is: NonRecursive::Baz(())
            `Blah 456` is: NonRecursive::Blah(456)
            roc to rusty: NonRecursive::Foo("small str") -> Foo("small str")
            rusty to roc: Foo("small str") -> NonRecursive::Foo("small str")
            roc to rusty: NonRecursive::Foo("A long enough string to not be small") -> Foo("A long enough string to not be small")
            rusty to roc: Foo("A long enough string to not be small") -> NonRecursive::Foo("A long enough string to not be small")
            roc to rusty: NonRecursive::Bar(123) -> Bar(123)
            rusty to roc: Bar(123) -> NonRecursive::Bar(123)
            roc to rusty: NonRecursive::Baz(()) -> Baz
            rusty to roc: Baz -> NonRecursive::Baz(())
            roc to rusty: NonRecursive::Blah(456) -> Blah(456)
            rusty to roc: Blah(456) -> NonRecursive::Blah(456)
        "#),
        nullable_wrapped:"nullable-wrapped" => indoc!(r#"
            tag_union was: StrFingerTree::More("foo", StrFingerTree::More("bar", StrFingerTree::Empty))
            `More "small str" (Single "other str")` is: StrFingerTree::More("small str", StrFingerTree::Single("other str"))
            `More "small str" Empty` is: StrFingerTree::More("small str", StrFingerTree::Empty)
            `Single "small str"` is: StrFingerTree::Single("small str")
            `Empty` is: StrFingerTree::Empty
        "#),
        nullable_unwrapped:"nullable-unwrapped" => indoc!(r#"
            tag_union was: StrConsList::Cons("World!", StrConsList::Cons("Hello ", StrConsList::Nil))
            `Cons "small str" Nil` is: StrConsList::Cons("small str", StrConsList::Nil)
            `Nil` is: StrConsList::Nil
        "#),
        nonnullable_unwrapped:"nonnullable-unwrapped" => indoc!(r#"
            tag_union was: StrRoseTree::Tree("root", [StrRoseTree::Tree("leaf1", []), StrRoseTree::Tree("leaf2", [])])
            Tree "foo" [] is: StrRoseTree::Tree("foo", [])
        "#),
        basic_recursive_union:"basic-recursive-union" => indoc!(r#"
            tag_union was: Expr::Concat(Expr::String("Hello, "), Expr::String("World!"))
            `Concat (String "Hello, ") (String "World!")` is: Expr::Concat(Expr::String("Hello, "), Expr::String("World!"))
            `String "this is a test"` is: Expr::String("this is a test")
        "#),
        advanced_recursive_union:"advanced-recursive-union" => indoc!(r#"
            rbt was: Rbt { default: Job::Job(R1 { command: Command::Command(R2 { tool: Tool::SystemTool(R4 { name: "test", num: 42 }) }), inputFiles: ["foo"] }) }
        "#),
        list_recursive_union:"list-recursive-union" => indoc!(r#"
            rbt was: Rbt { default: Job::Job(R1 { command: Command::Command(R2 { args: [], tool: Tool::SystemTool(R3 { name: "test" }) }), inputFiles: ["foo"], job: [] }) }
        "#),
        multiple_modules:"multiple-modules" => indoc!(r#"
            combined was: Combined { s1: DepStr1::S("hello"), s2: DepStr2::R("world") }
        "#),
        // issue https://github.com/roc-lang/roc/issues/6121
        // TODO: re-enable this test. Currently it is flaking on macos x86-64 with a bad exit code.
        // nested_record:"nested-record" => "Record was: Outer { y: \"foo\", z: [1, 2], x: Inner { b: 24.0, a: 5 } }\n",
        // enumeration:"enumeration" => "tag_union was: MyEnum::Foo, Bar is: MyEnum::Bar, Baz is: MyEnum::Baz\n",
        //        arguments:"arguments" => indoc!(r#"
        //            Answer was: 84
        //        "#),
        closures:"closures" => indoc!(r#"
            Answer was: 672
        "#),
        rocresult:"rocresult" => indoc!(r#"
            Answer was: RocOk(ManuallyDrop { value: "Hello World!" })
            Answer was: RocErr(ManuallyDrop { value: 42 })
        "#),
        option:"option" => indoc!(r#"
            Answer was: "Hello World!"
            Answer was: discriminant_U1::None
        "#),
        always_never:"always-never" => indoc!(r#"
            main was: AlwaysNever::Always(306783378)
        "#),
        recursive_untagged:"recursive-untagged" => indoc!(r#"
            expr constructed in Roc was: Expr::Concat(Expr::String("This is a test"), Expr::Tag17())
            expr constructed in Rust was: Expr::Concat(Expr::String("This is a test"), Expr::Tag17())
        "#),
        u128_i128:"u128-i128" => indoc!(r#"
            u128 was: 170141183460469231731687303715884105727
            i128 was: 85070591730234615865843651857942052863
            u128 was: 340282366920938463463374607431768211454
            i128 was: 170141183460469231731687303715884105726
        "#),
    }

    fn check_for_tests(all_fixtures: &mut roc_collections::VecSet<String>) {
        use roc_collections::VecSet;

        let fixtures = fixtures_dir("");
        let entries = std::fs::read_dir(fixtures.as_path()).unwrap_or_else(|err| {
            panic!(
                "Error trying to read {} as a fixtures directory: {}",
                fixtures.to_string_lossy(),
                err
            );
        });

        for entry in entries {
            let entry = entry.unwrap();

            if entry.file_type().unwrap().is_dir() {
                let fixture_dir_name = entry.file_name().into_string().unwrap();

                if !all_fixtures.remove(&fixture_dir_name) {
                    panic!(
                        "The glue fixture directory {} does not have any corresponding tests in test_glue_cli. Please add one, so if it ever stops working, we'll know about it right away!",
                        entry.path().to_string_lossy()
                    );
                }
            }
        }

        assert_eq!(all_fixtures, &mut VecSet::default());
    }

    fn generate_glue_for<'a, I: IntoIterator<Item = &'a str>>(
        platform_dir: &'a Path,
        args: I,
    ) -> Out {
        let platform_module_path = platform_dir.join("platform.roc");
        let glue_dir = platform_dir.join("test_glue");
        let fixture_templates_dir = platform_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixture-templates");

        // Copy the rust template from the templates directory into the fixture dir.
        dircpy::CopyBuilder::new(fixture_templates_dir.join("rust"), platform_dir)
            .overwrite(true) // overwrite any files that were already present
            .run()
            .unwrap();

        // Delete the glue file to make sure we're actually regenerating it!
        if glue_dir.exists() {
            fs::remove_dir_all(&glue_dir)
                .expect("Unable to remove test_glue dir in order to regenerate it in the test");
        }

        let rust_glue_spec = fixture_templates_dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("src")
            .join("RustGlue.roc");

        // Generate a fresh test_glue for this platform
        let parts : Vec<_> =
            // converting these all to String avoids lifetime issues
            std::iter::once("glue".to_string()).chain(
                args.into_iter().map(|arg| arg.to_string()).chain([
                    rust_glue_spec.to_str().unwrap().to_string(),
                    glue_dir.to_str().unwrap().to_string(),
                    platform_module_path.to_str().unwrap().to_string(),
                ]),
            ).collect();
        let glue_out = run_glue(parts.iter());

        if has_error(&glue_out.stderr) {
            panic!(
                "`roc {}` command had unexpected stderr: {}",
                parts.join(" "),
                glue_out.stderr
            );
        }

        assert!(
            glue_out.status.success(),
            "glue exited with bad status {glue_out:?}\n\n{}\n\n{}",
            glue_out.stdout,
            glue_out.status
        );

        glue_out
    }

    fn run_app<'a, 'b, I: IntoIterator<Item = &'a str>>(app_file: &'b Path, args: I) -> Out {
        // Generate test_glue for this platform
        let compile_out = run_roc(
            // converting these all to String avoids lifetime issues
            args.into_iter()
                .map(|arg| arg.to_string())
                .chain([app_file.to_str().unwrap().to_string()]),
            &[],
            &[],
        );

        if has_error(&compile_out.stderr) {
            panic!(
                "`roc` command had unexpected stderr: {}",
                compile_out.stderr
            );
        }

        assert!(
            compile_out.status.success(),
            "app exited with bad status {compile_out:?}\n\n{}\n\n{}",
            compile_out.stdout,
            compile_out.status
        );

        compile_out
    }
}
