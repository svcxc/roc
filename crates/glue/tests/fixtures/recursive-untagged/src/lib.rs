use roc_app;
use roc_std::RocStr;

extern "C" {
    #[link_name = "roc__mainForHost_1_exposed_generic"]
    fn roc_main(_: *mut roc_app::Expr);
}

#[no_mangle]
pub extern "C" fn rust_main() {
    use roc_app::Expr;
    use std::cmp::Ordering;
    use std::collections::hash_set::HashSet;

    let roc_expr = roc_app::mainForHost();
    let rust_expr = Expr::Concat(Expr::String("This is a test".into()), Expr::Tag17());

    assert!(roc_expr == roc_expr);

    // Verify that it has all the expected traits.

    assert!(roc_expr == roc_expr); // PartialEq
    assert!(roc_expr.clone() == roc_expr.clone()); // Clone

    assert!(roc_expr.partial_cmp(&roc_expr) == Some(Ordering::Equal)); // PartialOrd
    assert!(roc_expr.cmp(&roc_expr) == Ordering::Equal); // Ord

    println!("expr constructed in Roc was: {:?}", roc_expr); // Debug
    println!("expr constructed in Rust was: {:?}", rust_expr); // Debug

    let mut set = HashSet::new();

    set.insert(roc_expr.clone()); // Eq, Hash
    set.insert(roc_expr);

    assert_eq!(set.len(), 1);
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
