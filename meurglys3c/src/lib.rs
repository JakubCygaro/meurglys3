use std::ffi::CStr;
use libc::{c_char, c_uchar, c_ulonglong, c_void};
use std::path::PathBuf;
use std::ptr::{self, null_mut};
use meurglys3_lib::{self, Package};
pub use meurglys3_lib::Compression;

#[repr(C)]
pub enum Error {
    NoError = -1,
    StringError = 0,
    PackError,
    ParameterWasNull,
    FailedCast,
}
#[repr(C)]
pub struct PackageVersion {
    pub major: c_uchar,
    pub minor: c_uchar,
    pub tweak: c_uchar,
    pub patch: c_uchar,
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
#[no_mangle]
pub unsafe extern "C" fn meu3_package_get_version(pack: &mut PACKAGE, err: &mut Error) -> crate::PackageVersion {
    *err = Error::NoError;
    match extract_mut_ref(pack as *mut c_void as *mut Package) {
        Ok(pack) => {
            let v = pack.version().ver;
            crate::PackageVersion {
                major: v.0,
                minor: v.1,
                patch: v.2,
                tweak: v.3,
            }
        },
        Err(e) => {
            *err = e;
            crate::PackageVersion { 
                major: 0,
                minor: 0,
                patch: 0,
                tweak: 0,
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn meu3_package_get_compression(pack: &mut PACKAGE, err: &mut Error) -> Compression {
    *err = Error::NoError;
    match extract_mut_ref(pack as *mut c_void as *mut Package) {
        Ok(pack) => {
            pack.compression() 
        },
        Err(e) => {
            *err = e;
            meurglys3_lib::Compression::None 
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
