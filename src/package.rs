use std::collections::HashMap;
use std::io::Write;
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
    index: u32,
    size: u32,
}
impl DataInfo {
    pub fn new(index: u32, size: u32) -> Self {
        Self { index, size }
    }
    pub fn to_le_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        let idx = self.index.to_le_bytes();
        buf[..3].copy_from_slice(&idx[..3]);
        let sz = self.size.to_le_bytes();
        buf[4..(3 + 4)].copy_from_slice(&sz[..3]);
        buf
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
    pub(crate) names: HashMap<String, DataInfo>,
    pub(crate) data: Vec<u8>,
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
        let mut data: Vec<u8> = vec![];
        for file_info in value {
            let data_info = DataInfo {
                index: data.len() as u32,
                size: file_info.data.len() as u32,
            };
            map.insert(file_info.path.to_string_lossy().to_string(), data_info);
            data.write_all(&file_info.data[..]).unwrap();
        }
        Package {
            names: map,
            data,
            version,
            compression,
        }
    }
    pub fn has(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }
    pub fn get_data(&self, name: &str) -> Option<Vec<u8>> {
        let data = self.names.get(name)?;
        let mut ret = vec![0; data.size as usize];
        ret.copy_from_slice(&self.data[data.index as usize..(data.index + data.size) as usize]);
        Some(ret)
    }
    pub fn get_data_ref(&self, name: &str) -> Option<&[u8]> {
        let data = self.names.get(name)?;
        Some(&self.data[data.index as usize..(data.index + data.size) as usize])
    }
    pub fn version(&self) -> PackageVersion {
        self.version
    }
    pub fn compression(&self) -> Compression {
        self.compression
    }
    pub fn get_names(&self) -> &HashMap<String, DataInfo> {
        &self.names
    }
}

impl std::fmt::Debug for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Package")
            .field("names", &self.names)
            .field("data", &self.data)
            .finish()
    }
}
