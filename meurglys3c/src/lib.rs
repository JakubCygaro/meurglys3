use std::ffi::{c_char, c_uchar, c_ulonglong, c_void, CStr, CString};
use std::path::PathBuf;
use std::ptr::{self, null, null_mut};
use meurglys3_lib::{self, Package};

#[repr(C)]
pub enum Error {
    NoError = -1,
    StringError = 0,
    PackError,
    ParameterWasNull,
    FailedCast,
}

pub type PACKAGE = c_void;
pub type BYTES = *mut c_uchar;

#[no_mangle]
pub unsafe extern "C" fn meu3_package_dir(dir_path: &c_char, err: &mut Error) -> *mut PACKAGE {
    let path = CStr::from_ptr(dir_path as *const _);
    let Ok(path ) = path.to_str() else {
        *err = Error::StringError;
        return null_mut::<c_void>();
    };
    let pack = meurglys3_lib::package_dir(std::path::PathBuf::from(path));
    let Ok(pack) = pack else {
        *err = Error::PackError;
        return null_mut::<c_void>();
    };
    let pack = Box::new(pack);
    Box::into_raw(pack) as *mut c_void
}
#[no_mangle]
pub extern "C" fn meu3_free_package(package: &mut PACKAGE) {
    let pack = unsafe { 
        Box::from_raw(package as *mut c_void as *mut Package) 
    };
    drop(pack);
}
#[no_mangle]
pub unsafe extern "C" fn meu3_load_package(dir_path: &c_char, err: &mut Error) -> *mut PACKAGE {
    let path = CStr::from_ptr(dir_path as *const _);
    let Ok(path) = path.to_str() else {
        *err = Error::StringError;
        return null_mut();
    };
    let Ok(pack) = meurglys3_lib::load_package(PathBuf::from(path)) else {
        *err = Error::PackError;
        return null_mut();
    };
    let pack = Box::new(pack);
    Box::into_raw(pack) as *mut PACKAGE
}
#[no_mangle]
pub unsafe extern "C" fn meu3_package_has(pack: &PACKAGE, path: &c_char, err: &mut Error) -> bool {
    *err = Error::NoError;
    let pack = pack as *const PACKAGE as *mut Package;
    return match extract_mut_ref(pack) {
        Ok(p) => {
            let str = CStr::from_ptr(path as *const _);
            let Ok(path) = str.to_str() else {
                *err = Error::StringError;
                return false;
            };
            p.has(path)
        },
        Err(e) => {
            *err = e;
            false
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn meu3_package_get_data_ptr(pack: &mut PACKAGE, path: &c_char, len: &mut c_ulonglong, err: &mut Error) -> BYTES {
    *err = Error::NoError;
    match extract_mut_ref(pack as *mut c_void as *mut Package) {
        Ok(pack) => {
            let path = CStr::from_ptr(path as *const _);
            let Ok(path) = path.to_str() else {
                *err = Error::StringError;
                return null_mut();
            };
            if let Some(data) = pack.get_data_ref(path) {
                *len = data.len() as _;
                data.as_ptr() as BYTES
            } else {
                null_mut()
            }
        },
        Err(e) => {
            *err = e;
            ptr::null_mut()
        }
    }
}
unsafe fn extract_mut_ref<'a, T>(val: *mut T) -> Result<&'a mut T, Error> {
    if val.is_null() {
        return Err(Error::ParameterWasNull)
    }
    return if let Some(r) = val.as_mut() {
        Ok(r)
    } else {
        Err(Error::FailedCast)
    }
}
