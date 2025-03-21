use std::ffi::{c_char, c_void, CString};

use meurglys3_lib;

#[repr(C)]
pub enum Meu3Error {
    NoError = -1,
    StringError,
    PackError
}
//#[repr(C)]
//pub struct MeurglysPackage {
//    pub names: *mut c_void,//HashMap<String, DataInfo>,
//    pub data: *mut c_char,
//    pub version: [u8;4],
//    pub compression: meurglys3_lib::Compression,
//}

#[no_mangle]
extern "C" {
    pub unsafe fn meu3_package_dir(dir_path: *const c_char) -> *mut c_void {
        if dir_path.is_null() {
            return std::ptr::null_mut::<c_void>()
        }
        let path = CString::from_raw(dir_path).to_str().expect("failed to read dir_path string");
        let pack = meurglys3_lib::package_dir(path);
        let pack = pack.expect("failed to package directory");
        let pack = Box::new(pack);
        Box::into_raw(pack) as *mut c_void
    }
    pub fn meu3_free_package(package: *mut c_void) {
        if !package.is_null() {
            let pack = unsafe {
                Box::from_raw(package as *mut meurglys3_lib::Package)
            };
            drop(pack);
        }
    }
    pub fn meu3_load_package(dir_path: *const c_char) -> *mut c_void {

    }
}
