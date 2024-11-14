use roc_app;
use roc_app::rusty_NonRecursive;
use roc_app::NonRecursive;
use roc_std::RocStr;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(_: *mut NonRecursive);
}

#[no_mangle]
pub extern "C" fn rust_main() {
    use std::cmp::Ordering;
    use std::collections::hash_set::HashSet;

    let tag_union = roc_app::mainForHost();

    // Verify that it has all the expected traits.

    assert!(tag_union == tag_union); // PartialEq
    assert!(tag_union.clone() == tag_union.clone()); // Clone

    assert!(tag_union.partial_cmp(&tag_union) == Some(Ordering::Equal)); // PartialOrd
    assert!(tag_union.cmp(&tag_union) == Ordering::Equal); // Ord

    println!(
        "tag_union was: {:?}\n`Foo \"small str\"` is: {:?}\n`Foo \"A long enough string to not be small\"` is: {:?}\n`Bar 123` is: {:?}\n`Baz` is: {:?}\n`Blah 456` is: {:?}",
        tag_union,
        NonRecursive::Foo("small str".into()),
        NonRecursive::Foo("A long enough string to not be small".into()),
        NonRecursive::Bar(123),
        NonRecursive::Baz(),
        NonRecursive::Blah(456),
    ); // Debug

    let mut set = HashSet::new();

    set.insert(tag_union.clone()); // Eq, Hash
    set.insert(tag_union);

    assert_eq!(set.len(), 1);

    let roc_variants = [
        NonRecursive::Foo("small str".into()),
        NonRecursive::Foo("A long enough string to not be small".into()),
        NonRecursive::Bar(123),
        NonRecursive::Baz(),
        NonRecursive::Blah(456),
    ];

    let rusty_variants = [
        rusty_NonRecursive::Foo("small str".into()),
        rusty_NonRecursive::Foo("A long enough string to not be small".into()),
        rusty_NonRecursive::Bar(123),
        rusty_NonRecursive::Baz,
        rusty_NonRecursive::Blah(456),
    ];

    for (roc_variant, rusty_variant) in roc_variants.iter().zip(rusty_variants.iter()) {
        let into_rusty: rusty_NonRecursive = roc_variant.clone().into();
        let into_roc: NonRecursive = rusty_variant.clone().into();

        println!("roc to rusty: {roc_variant:?} -> {into_rusty:?}");
        println!("rusty to roc: {rusty_variant:?} -> {into_roc:?}");

        assert_eq!(rusty_variant, &into_rusty);
        assert_eq!(roc_variant, &into_roc);
    }
}

// Externs required by roc_std and by the Roc app

use core::ffi::c_void;
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub unsafe extern "C" fn roc_alloc(size: usize, _alignment: u32) -> *mut c_void {
    return libc::malloc(size);
}

#[no_mangle]
pub unsafe extern "C" fn roc_realloc(
    c_ptr: *mut c_void,
    new_size: usize,
    _old_size: usize,
    _alignment: u32,
) -> *mut c_void {
    return libc::realloc(c_ptr, new_size);
}

#[no_mangle]
pub unsafe extern "C" fn roc_dealloc(c_ptr: *mut c_void, _alignment: u32) {
    return libc::free(c_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn roc_panic(msg: *mut RocStr, tag_id: u32) {
    match tag_id {
        0 => {
            eprintln!("Roc standard library hit a panic: {}", &*msg);
        }
        1 => {
            eprintln!("Application hit a panic: {}", &*msg);
        }
        _ => unreachable!(),
    }
    std::process::exit(1);
}

#[no_mangle]
pub unsafe extern "C" fn roc_dbg(loc: *mut RocStr, msg: *mut RocStr, src: *mut RocStr) {
    eprintln!("[{}] {} = {}", &*loc, &*src, &*msg);
}

#[no_mangle]
pub unsafe extern "C" fn roc_memset(dst: *mut c_void, c: i32, n: usize) -> *mut c_void {
    libc::memset(dst, c, n)
}
