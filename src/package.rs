use std::collections::HashMap;
use std::path::PathBuf;

use super::err;

#[derive(Clone, Copy)]
pub struct PackageVersion {
    pub ver: (u8, u8, u8, u8),
}

impl Into<[u8; 4]> for PackageVersion {
    fn into(self) -> [u8; 4] {
        let ver = self.ver;
        [ver.0, ver.1, ver.2, ver.3]
    }
}
impl From<(u8, u8, u8, u8)> for PackageVersion {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self { ver: value }
    }
}
impl TryFrom<&[u8]> for PackageVersion {
    type Error = err::ParseError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 4 {
            Err(err::ParseError::Version)
        } else {
            Ok(Self {
                ver: (value[0], value[1], value[2], value[3]),
            })
        }
    }
}

pub struct FileInfo {
    path: PathBuf,
    data: Vec<u8>,
}
impl FileInfo {
    pub fn new(path: PathBuf, data: Vec<u8>) -> Self {
        Self {
            data,
            path,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataInfo {
    pub(crate) index: u32,
    pub(crate) size: u32,
}
impl DataInfo {
    pub fn new(index: u32, size: u32) -> Self {
        Self { index, size }
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub enum Compression {
    None,
}

impl TryFrom<&[u8]> for Compression {
    type Error = err::ParseError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            Err(err::ParseError::Compression)
        } else {
            match value {
                [0x00, 0x00] => Ok(Self::None),
                _ => Err(err::ParseError::Compression),
            }
        }
    }
}
impl Into<[u8; 2]> for Compression {
    fn into(self) -> [u8; 2] {
        match self {
            Compression::None => [0x00, 0x00],
        }
    }
}

pub struct Package {
    pub(crate) names: HashMap<String, Vec<u8>>,
    pub(crate) version: PackageVersion,
    pub(crate) compression: Compression,
}

impl Package {
    pub(crate) fn from_file_info(
        value: Vec<FileInfo>,
        version: PackageVersion,
        compression: Compression,
    ) -> Self {
        let mut map = HashMap::new();
        for file_info in value {
            map.insert(file_info.path.to_string_lossy().to_string(), file_info.data);
        }
        Package {
            names: map,
            version,
            compression,
        }
    }
    pub fn has(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }
    pub fn get_data(&self, name: &str) -> Option<Vec<u8>> {
        self.names.get(name).cloned()
    }
    pub fn get_data_ref(&self, name: &str) -> Option<&[u8]> {
        self.names.get(name).map(|v| v.as_slice())
    }
    pub fn version(&self) -> PackageVersion {
        self.version
    }
    pub fn compression(&self) -> Compression {
        self.compression
    }
    pub fn get_names(&self) -> &HashMap<String, Vec<u8>> {
        &self.names
    }
    pub fn insert_data(&mut self, name: String, data: Vec<u8>) {
        self.names.insert(name, data);
    }
    pub fn remove_data(&mut self, name: &str) {
        self.names.remove(name);
    }
}

impl std::fmt::Debug for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Package")
            .field("names", &self.names)
            .finish()
    }
}
