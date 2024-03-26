use std::fs::{self, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, ffi::OsString};

use bytes::Buf;

use crate::err::UnpackError;

mod err;

const FILE_HEADER: [u8; 4] = [0xFF, 0x69, 0xFF, 0x69];
const VERSION_0_0_0_1: [u8; 4] = [0x00, 0x00, 0x00, 0x01];
const NO_COMPRESSION: [u8; 2] = [0x00, 0x00];
pub struct Package {
    pub(crate) names: HashMap<String, DataInfo>,
    data: Vec<u8>,
    version: (u8, u8, u8, u8),
    compression: Compression,
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

fn version_from_bytes(bytes: &[u8]) -> (u8, u8, u8, u8) {
    (bytes[0], bytes[1], bytes[2], bytes[3])
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

#[derive(Clone, Debug)]
pub struct DataInfo {
    index: u32,
    size: u32,
    ext: FileExt,
}
impl DataInfo {
    fn to_le_bytes(self) -> [u8; 9] {
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

#[derive(Clone, Debug)]
enum FileExt {
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

struct FileInfo {
    path: PathBuf,
    ext: FileExt,
    data: Vec<u8>,
}

fn collect_files(dir: &std::path::Path) -> std::io::Result<Vec<(fs::DirEntry, PathBuf)>> {
    let mut ret = vec![];
    for entry in fs::read_dir(dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                let mut inner_files = collect_files(&path)?;
                ret.append(&mut inner_files);
            }
            ret.push((entry, path));
        }
    }
    Ok(ret)
}

pub fn package_dir(dir_path: PathBuf) -> std::io::Result<Package> {
    let dir_path = std::fs::canonicalize(dir_path)?;
    //println!("{dir_path:?}");
    let files = collect_files(&dir_path)?
        .into_iter()
        .filter_map(|(f, p)| {
            if p.is_file() {
                return Some((f, p));
            }
            None
        })
        .filter(|(_f, p)| {
            let ext = p.extension();
            ext.is_some_and(|s| FileExt::try_from(s.to_str().unwrap()).is_ok())
        })
        .map(|(_f, p)| {
            let full_path = p.canonicalize().unwrap();
            let buf = std::fs::read(&full_path).unwrap();

            //println!("path: {:?}, buf {:?}", f.path(), buf);

            let rel_path = full_path
                .strip_prefix(&dir_path)
                .expect("strip prefix failed");

            FileInfo {
                path: rel_path.with_extension(""),
                ext: FileExt::try_from(full_path.extension().unwrap().to_str().unwrap())
                    .expect("unsupported file extension"),
                data: buf,
            }
        })
        .collect::<Vec<_>>();
    Ok(Package::from(files))
}

pub fn write_package(path: OsString, package: &mut Package) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    //header
    buf.write(&FILE_HEADER)
        .expect("could not write header data");
    buf.write(&VERSION_0_0_0_1)
        .expect("could not write version data");
    buf.write(&NO_COMPRESSION)
        .expect("could not write compression format data");

    for (name, data) in &package.names {
        buf.write(name.as_bytes()).expect("could not write path");
        buf.write(&[0x00]).expect("could not write terminator");
        buf.write(&data.clone().to_le_bytes())
            .expect("could not write data info");
    }
    buf.write(&[0x0]).expect("could not terminate name table");

    buf.write(&package.data[..])
        .expect("failed to write byte data");
    path.clone().push(".m3pkg");
    let mut file = fs::File::create(path)?;

    file.write_all(&buf[..])
}

pub fn load_package(path_to_dir: OsString) -> Result<Package, err::UnpackError> {
    let file = fs::read(path_to_dir)?;
    let file_len = file.len();
    let mut bytes = bytes::Bytes::from(file);

    if file_len < FILE_HEADER.len() + VERSION_0_0_0_1.len() + NO_COMPRESSION.len() + 1 {
        return Err(UnpackError::InvalidFile);
    }

    let mut remain = bytes.split_off(4);
    if bytes.as_ref() != FILE_HEADER {
        return Err(UnpackError::InvalidFile);
    }
    bytes = remain;
    remain = bytes.split_off(4);
    if bytes.as_ref() != VERSION_0_0_0_1 {
        return Err(UnpackError::InvalidVersion);
    }
    let version = version_from_bytes(bytes.as_ref());
    bytes = remain;
    remain = bytes.split_off(2);
    if bytes.as_ref() != NO_COMPRESSION {
        return Err(UnpackError::InvalidFile);
    }
    let compression = Compression::try_from(bytes[0])?;
    //println!("bytes: {:?}", bytes.as_ref());

    let (map, bytes) = read_data_table(&mut remain)?;
    let data: Vec<u8> = bytes.collect::<Result<_, _>>()?;

    Ok(Package {
        names: map,
        data: data,
        version: version,
        compression: compression,
    })
}

#[derive(PartialEq)]
enum ParseState {
    ReadingString,
    ReadingIndex,
    ReadingSize,
    ReadingExt,
}

fn read_data_table(
    bytes: &mut bytes::Bytes,
) -> Result<
    (
        HashMap<String, DataInfo>,
        std::io::Bytes<bytes::buf::Reader<&mut bytes::Bytes>>,
    ),
    err::UnpackError,
> {
    let mut map = HashMap::new();
    let reader = bytes.reader();
    let mut bytes = reader.bytes();

    let mut state = ParseState::ReadingString;

    let mut str = String::default();
    let mut index = 0;
    let mut size = 0;
    let mut ext: FileExt;
    while let Some(Ok(b)) = bytes.next() {
        if b == b'\0' && state == ParseState::ReadingString {
            break;
        }
        if state == ParseState::ReadingString {
            let mut str_buf = vec![b];
            while let Some(str_byte) = bytes.next() {
                let str_byte = str_byte.expect("failed to read string data");
                if str_byte != b'\0' {
                    str_buf.push(str_byte);
                } else {
                    break;
                }
            }
            state = ParseState::ReadingIndex;
            str = String::from_utf8(str_buf).unwrap();
        } else if state == ParseState::ReadingIndex {
            let mut idx: [u8; 4] = [b; 4];
            for i in 1..4 {
                idx[i] = bytes.next().unwrap().unwrap();
            }
            index = u32::from_le_bytes(idx);
            state = ParseState::ReadingSize;
        } else if state == ParseState::ReadingSize {
            let mut sz: [u8; 4] = [b; 4];
            for i in 1..4 {
                sz[i] = bytes.next().unwrap().unwrap();
            }
            size = u32::from_le_bytes(sz);
            state = ParseState::ReadingExt;
        } else {
            let e: [u8; 1] = [b];
            ext = FileExt::try_from(u8::from_le_bytes(e))?;
            state = ParseState::ReadingString;

            map.insert(
                str.clone(),
                DataInfo {
                    size: size,
                    index: index,
                    ext: ext,
                },
            );
        }
    }
    Ok((map, bytes))
}

pub fn unpack_to_dir(dir_path: String, pack: &Package) -> std::io::Result<()> {
    DirBuilder::new().recursive(true).create(dir_path.clone())?;
    for (file_name, info) in &pack.names {
        let bytes = pack.get_data_ref(&file_name).unwrap();
        let mut path = PathBuf::from(dir_path.clone());
        path.push(file_name);
        match info.ext {
            FileExt::MP3 => path.set_extension("mp3"),
            FileExt::PNG => path.set_extension("png"),
            FileExt::WAV => path.set_extension("wav"),
        };
        if let Some(prefix) = path.parent() {
            fs::create_dir_all(prefix)?;
        }
        let mut file = fs::File::create(path)?;
        file.write_all(bytes)?;
    }
    Ok(())
}
