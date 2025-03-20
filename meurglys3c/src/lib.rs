use std::ffi::{c_char, c_void, CString};

use meurglys3_lib;

//#[repr(C)]
//pub struct MeurglysPackage {
//    pub names: *mut c_void,//HashMap<String, DataInfo>,
//    pub data: *mut c_char,
//    pub version: [u8;4],
//    pub compression: meurglys3_lib::Compression,
//}

#[no_mangle]
extern "C" {
    pub unsafe fn package_dir(dir_path: *const c_char) -> *mut c_void {
        let path = CString::from_raw(dir_path).to_str().expect("failed to read dir_path string");
        let pack = meurglys3_lib::package_dir(path);
        let pack = pack.expect("failed to package directory");
        let pack = Box::new(pack);
        Box::into_raw(pack) as *mut c_void
    }
}
