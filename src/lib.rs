use crate::err::UnpackError;
use bytes::Buf;
use std::collections::HashMap;
use std::fs::{self, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;

mod err;
mod package;
#[cfg(test)]
mod tests;
use package::*;
pub use package::{Compression, Package, PackageVersion};

const FILE_HEADER: [u8; 4] = [0xFF, 0x69, 0xFF, 0x69];
const VERSION_0_0_0_1: [u8; 4] = [0x00, 0x00, 0x00, 0x01];
const NO_COMPRESSION: [u8; 2] = [0x00, 0x00];

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

pub fn package_dir(dir_path: PathBuf) -> Result<Package, err::PackingError> {
    let dir_path = std::fs::canonicalize(dir_path)?;
    let files = collect_files(&dir_path)?
        .into_iter()
        .filter_map(|(f, p)| {
            if p.is_file() {
                return Some((f, p));
            }
            None
        })
        .map(|(_f, p)| {
            let full_path = p.canonicalize()?;
            let buf = std::fs::read(&full_path)?;

            let rel_path = full_path
                .strip_prefix(&dir_path)
                .map_err(err::PackingError::FileReadingError)?;

            #[cfg(target_os = "windows")]
            let rel_path: PathBuf = rel_path
                .to_slash()
                .expect("slash replacement in file path failed, file path must contain non-unicode characters")
                .deref()
                .into();

            Ok(FileInfo::new(
                rel_path.to_path_buf(),
                buf,
            ))
        })
        .collect::<Result<Vec<_>, err::PackingError>>()?;
    Ok(Package::from_file_info(
        files,
        PackageVersion::from((0, 0, 0, 1)),
        Compression::None,
    ))
}

pub fn write_package(mut path: PathBuf, package: &mut Package) -> std::io::Result<()> {
    let mut buf: Vec<u8> = vec![];

    //header
    buf.write_all(&FILE_HEADER)?;
    let ver: [u8; 4] = package.version.into();
    buf.write_all(&ver)?;
    let comp: [u8; 2] = package.compression.into();
    buf.write_all(&comp)?;

    for (name, data) in &package.names {
        buf.write_all(name.as_bytes())?;
        buf.write_all(&[0x00])?;
        buf.write_all(&data.clone().to_le_bytes())?;
    }
    buf.write_all(&[0x0])?;

    buf.write_all(&package.data[..])?;
    path.set_extension("m3pkg");
    let mut file = fs::File::create(path)?;

    file.write_all(&buf[..])
}

pub fn load_package(path_to_dir: PathBuf) -> Result<Package, err::UnpackError> {
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
    // read version
    bytes = remain;
    remain = bytes.split_off(4);
    let version = PackageVersion::try_from(bytes.as_ref())?;

    // read compression type
    bytes = remain;
    remain = bytes.split_off(2);
    let compression = Compression::try_from(&bytes[..2])?;

    use err::UnsupportedError;
    match (version.ver, compression) {
        ((0, 0, 0, 1), Compression::None) => {
            let (map, bytes) = read_data_table(&mut remain)?;
            let data: Vec<u8> = bytes.collect::<Result<_, _>>()?;

            Ok(Package {
                names: map,
                data: data,
                version: version,
                compression: compression,
            })
        }
        (_, Compression::None) => Err(UnpackError::UnsupportedFormat(UnsupportedError::Version)),
    }
}

#[derive(PartialEq)]
enum ParseState {
    String,
    Index,
    Size,
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

    let mut state = ParseState::String;

    let mut str = String::default();
    let mut index = 0;

    while let Some(Ok(b)) = bytes.next() {
        if b == b'\0' && state == ParseState::String {
            break;
        }
        if state == ParseState::String {
            let mut str_buf = vec![b];
            while let Some(str_byte) = bytes.next() {
                let str_byte = str_byte?;
                if str_byte != b'\0' {
                    str_buf.push(str_byte);
                } else {
                    break;
                }
            }
            state = ParseState::Index;
            str = String::from_utf8(str_buf)?;
        } else if state == ParseState::Index {
            let mut idx: [u8; 4] = [b; 4];
            for i in 1..4 {
                idx[i] = bytes.next().ok_or_else(|| err::ParseError::Index)??;
            }
            index = u32::from_le_bytes(idx);
            state = ParseState::Size;
        } else if state == ParseState::Size {
            let mut sz: [u8; 4] = [b; 4];
            for i in 1..4 {
                sz[i] = bytes.next().ok_or_else(|| err::ParseError::Size)??;
            }
            let size = u32::from_le_bytes(sz);
            state = ParseState::String;
            map.insert(str.clone(), DataInfo::new(index, size));
        }
    }
    Ok((map, bytes))
}

pub fn unpack_to_dir(dir_path: PathBuf, pack: &Package) -> std::io::Result<()> {
    DirBuilder::new().recursive(true).create(dir_path.clone())?;
    for (file_name, _info) in &pack.names {
        let bytes = pack.get_data_ref(file_name).unwrap();
        let mut path = dir_path.clone();
        path.push(file_name);
        if let Some(prefix) = path.parent() {
            fs::create_dir_all(prefix)?;
        }
        let mut file = fs::File::create(path)?;
        file.write_all(bytes)?;
    }
    Ok(())
}
