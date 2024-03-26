use std::fs::{self, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{collections::HashMap, ffi::OsString};

use bytes::Buf;

use crate::err::UnpackError;

mod err;
mod package;
use package::*;

const FILE_HEADER: [u8; 4] = [0xFF, 0x69, 0xFF, 0x69];
const VERSION_0_0_0_1: [u8; 4] = [0x00, 0x00, 0x00, 0x01];
const NO_COMPRESSION: [u8; 2] = [0x00, 0x00];

fn version_from_bytes(bytes: &[u8]) -> (u8, u8, u8, u8) {
    (bytes[0], bytes[1], bytes[2], bytes[3])
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

            FileInfo::new(
                rel_path.with_extension(""),
                FileExt::try_from(full_path.extension().unwrap().to_str().unwrap())
                    .expect("unsupported file extension"),
                buf,
            )
        })
        .collect::<Vec<_>>();
    Ok(Package::from(files))
}

pub fn write_package(mut path: OsString, package: &mut Package) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    //header
    buf.write(&FILE_HEADER)?;
    buf.write(&VERSION_0_0_0_1)?;
    buf.write(&NO_COMPRESSION)?;

    for (name, data) in &package.names {
        buf.write(name.as_bytes())?;
        buf.write(&[0x00])?;
        buf.write(&data.clone().to_le_bytes())?;
    }
    buf.write(&[0x0])?;

    buf.write(&package.data[..])?;
    path.push(".m3pkg");
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

            map.insert(str.clone(), DataInfo::new(index, size, ext));
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
        path.set_extension(info.ext().to_string());
        if let Some(prefix) = path.parent() {
            fs::create_dir_all(prefix)?;
        }
        let mut file = fs::File::create(path)?;
        file.write_all(bytes)?;
    }
    Ok(())
}
