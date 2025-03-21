use std::ffi::{c_char, c_void, CString};

use meurglys3_lib;

#[repr(C)]
pub enum Meu3Error {
    NoError = -1,
    StringError,
    PackError,
}

pub type PACKAGE = *mut c_void;

#[no_mangle]
pub unsafe extern "C" fn meu3_package_dir(dir_path: *const c_char) -> PACKAGE {
    if dir_path.is_null() {
        return std::ptr::null_mut::<c_void>();
    }
    let path = CString::from_raw(dir_path as *mut _);
    let path = path.to_str().expect("failed to read dir_path string");
    let pack = meurglys3_lib::package_dir(std::path::PathBuf::from(path));
    let pack = pack.expect("failed to package directory");
    let pack = Box::new(pack);
    Box::into_raw(pack) as *mut c_void
}
#[no_mangle]
pub extern "C" fn meu3_free_package(package: PACKAGE) {
    if !package.is_null() {
        let pack = unsafe { Box::from_raw(package as *mut meurglys3_lib::Package) };
        drop(pack);
    }
}
#[no_mangle]
pub extern "C" fn meu3_load_package(dir_path: *const c_char) -> PACKAGE {
    std::ptr::null_mut()
}
