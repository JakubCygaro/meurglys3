use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use super::err;

#[derive(Clone, Copy, Debug)]
pub enum FileExt {
    PNG,
    WAV,
    MP3,
}

impl FileExt {
    fn to_le_byte(self) -> [u8; 1] {
        match self {
            FileExt::WAV => [0x01],
            FileExt::MP3 => [0x02],
            FileExt::PNG => [0x03],
        }
    }
}
impl ToString for FileExt {
    fn to_string(&self) -> String {
        match self {
            Self::MP3 => String::from("mp3"),
            Self::WAV => String::from("wav"),
            Self::PNG => String::from("png"),
        }
    }
}
impl TryFrom<u8> for FileExt {
    type Error = err::ParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::WAV),
            0x02 => Ok(Self::MP3),
            0x03 => Ok(Self::PNG),
            _ => Err(err::ParseError::Extension),
        }
    }
}
impl TryFrom<&str> for FileExt {
    type Error = err::UnpackError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            ".wav" | "wav" => Ok(Self::WAV),
            ".mp3" | "mp3" => Ok(Self::MP3),
            ".png" | "png" => Ok(Self::PNG),
            _ => Err(err::UnpackError::FileExtension),
        }
    }
}

pub struct FileInfo {
    path: PathBuf,
    ext: FileExt,
    data: Vec<u8>,
}
impl FileInfo {
    pub fn new(path: PathBuf, ext: FileExt, data: Vec<u8>) -> Self {
        Self {
            data: data,
            ext: ext,
            path: path,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DataInfo {
    index: u32,
    size: u32,
    ext: FileExt,
}
impl DataInfo {
    pub fn new(index: u32, size: u32, ext: FileExt) -> Self {
        Self {
            ext: ext,
            index: index,
            size: size,
        }
    }
    pub fn ext(&self) -> FileExt {
        self.ext
    }
    pub fn to_le_bytes(self) -> [u8; 9] {
        let mut buf = [0u8; 9];
        //println!("self {:?}", self);
        let idx = self.index.to_le_bytes();
        for i in 0..3 {
            buf[i] = idx[i];
        }
        let sz = self.size.to_le_bytes();
        for i in 0..3 {
            buf[i + 4] = sz[i];
        }
        let e = self.ext.to_le_byte();
        buf[8] = e[0];
        buf
    }
}

impl From<Vec<FileInfo>> for Package {
    fn from(value: Vec<FileInfo>) -> Self {
        let mut map = HashMap::new();
        let mut data: Vec<u8> = vec![];
        for file_info in value {
            let data_info = DataInfo {
                index: data.len() as u32,
                size: file_info.data.len() as u32,
                ext: file_info.ext,
            };
            //println!("DataInfo: {:?}", data_info);
            map.insert(file_info.path.to_string_lossy().to_string(), data_info);
            data.write(&file_info.data[..]).unwrap();
        }
        Package {
            names: map,
            data: data,
            version: (0, 0, 0, 1),
            compression: Compression::None,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Compression {
    None,
}

impl TryFrom<u8> for Compression {
    type Error = err::ParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::None),
            _ => Err(err::ParseError::Compression),
        }
    }
}

pub struct Package {
    pub(crate) names: HashMap<String, DataInfo>,
    pub(crate) data: Vec<u8>,
    pub(crate) version: (u8, u8, u8, u8),
    pub(crate) compression: Compression,
}

impl Package {
    pub fn has(&self, name: &str) -> bool {
        self.names.contains_key(name)
    }
    pub fn get_data(&self, name: &str) -> Option<Vec<u8>> {
        let Some(data) = self.names.get(name) else {
            return None;
        };
        let mut ret = vec![];
        ret.copy_from_slice(&self.data[data.index as usize..(data.index + data.size) as usize]);
        Some(ret)
    }
    pub fn get_data_ref(&self, name: &str) -> Option<&[u8]> {
        let Some(data) = self.names.get(name) else {
            return None;
        };
        Some(&self.data[data.index as usize..(data.index + data.size) as usize])
    }
    pub fn version(&self) -> (u8, u8, u8, u8) {
        self.version
    }
    pub fn compression(&self) -> Compression {
        self.compression
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
