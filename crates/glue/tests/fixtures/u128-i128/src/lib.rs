use roc_app;
use roc_std::RocStr;

#[no_mangle]
pub extern "C" fn rust_main() {
    let numbers = roc_app::Numbers {
        u128: 170141183460469231731687303715884105727,
        i128: 85070591730234615865843651857942052863,
    };

    println!("u128 was: {}", numbers.u128);
    println!("i128 was: {}", numbers.i128);

    let doubled = roc_app::mainForHost(numbers);

    let expected = roc_app::Numbers {
        u128: 340282366920938463463374607431768211454,
        i128: 170141183460469231731687303715884105726,
    };

    println!("u128 was: {}", doubled.u128);
    println!("i128 was: {}", doubled.i128);

    assert_eq!(doubled, expected);
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
