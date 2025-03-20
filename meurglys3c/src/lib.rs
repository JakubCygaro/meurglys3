use std::ffi::c_char;

use meurglys3_lib;

#[no_mangle]
extern "C" {
    pub fn package_dir(dir_path: *const c_char) {

    }
}
