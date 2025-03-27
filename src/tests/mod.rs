use lazy_static::lazy_static;
use std::fs::{DirBuilder, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::io;
use tempdir::{self, TempDir};

enum FileType {
    File {
        path: PathBuf,
        content: Option<String>,
    },
    Directory {
        path: PathBuf,
    },
}
type DirModel = Vec<FileType>;

fn collect_dir(dir_path: &std::path::Path) -> io::Result<Vec<(PathBuf, Option<String>)>> {
    let files = std::fs::read_dir(dir_path)?
        .map(|e| {
            assert!(e.is_ok(), "directory entry failed to load ");
            let entry = e.unwrap();
            let path = entry.path();
            if path.is_dir() {
                return Ok((
                    path.strip_prefix(dir_path)
                        .expect("could not strip prefix")
                        .to_path_buf(),
                    None,
                ));
            }
            let mut buf = String::new();
            let mut file = File::open(&path)?;
            let _read = File::read_to_string(&mut file, &mut buf)?;
            Ok((
                path.strip_prefix(dir_path)
                    .expect("could not strip prefix")
                    .to_path_buf(),
                Some(buf),
            ))
        })
        .collect::<Result<Vec<_>, io::Error>>()?;
    Ok(files)
}
fn create_test_directory(dir_model: &DirModel) -> io::Result<TempDir> {
    let tmp = tempdir::TempDir::new("test_dir")?;
    for file in dir_model {
        let file_path = match file {
            FileType::File { path, .. } => path,
            FileType::Directory { path } => path,
        };
        let path = tmp.path().join(file_path);
        match file {
            FileType::File { content, .. } => {
                let mut tmp_file = File::create(path)?;
                if let Some(content) = content {
                    tmp_file.write_all(content.as_bytes())?;
                }
            }
            FileType::Directory { .. } => {
                DirBuilder::new().create(path)?;
            }
        };
    }
    Ok(tmp)
}

//files need to be placed after their parent directory, otherwise all will fuck up
lazy_static! {
    static ref PACKING_TEST_MODEL: DirModel = vec![
        FileType::File {
            path: PathBuf::from_str("text_file.txt").unwrap(),
            content: Some("text".to_string())
        },
        FileType::Directory {
            path: PathBuf::from_str("directory").unwrap()
        },
        FileType::File {
            path: PathBuf::from_str("directory/text_file.txt").unwrap(),
            content: None
        },
    ];
}

#[test]
fn test_packing() -> Result<(), super::err::PackingError> {
    let tmp = create_test_directory(&PACKING_TEST_MODEL)?;
    let pack = super::package_dir(tmp.path().to_path_buf())?;
    //let tmp_path = tmp.path().to_path_buf();
    for (file, content) in PACKING_TEST_MODEL.iter().filter_map(|i| match i {
        FileType::File { path, content } => Some((path, content)),
        FileType::Directory { .. } => None,
    }) {
        let path_as_str = file.to_str();
        assert!(path_as_str.is_some(), "failed to convert file path to str");
        let path_as_str = path_as_str.unwrap();
        assert!(
            pack.has(path_as_str),
            "the package did not contain a required file: {}",
            path_as_str
        );
        if let Some(content) = content {
            let data = pack.get_data_ref(path_as_str);
            assert!(
                data.is_some(),
                "data was not present for file {}",
                path_as_str
            );
            let data = data.unwrap();
            let content_as_bytes = content.as_bytes();
            assert_eq!(data, content_as_bytes);
        }
    }
    drop(tmp);
    Ok(())
}
#[test]
fn test_packing_and_unpacking() -> Result<(), super::err::PackingError> {
    let src_tmp = create_test_directory(&PACKING_TEST_MODEL)?;
    let pack = super::package_dir(src_tmp.path().to_path_buf())?;
    let dest_tmp = tempdir::TempDir::new("dest_tmp")?;
    super::unpack_to_dir(dest_tmp.path().to_path_buf(), &pack)?;
    let dest_path = dest_tmp.path();
    let src_files = collect_dir(src_tmp.path())?;
    let dest_files = collect_dir(dest_path)?;
    assert_eq!(src_files.len(), dest_files.len());
    let _ = src_files
        .iter()
        .zip(dest_files.iter())
        .map(|(src, dst)| {
            assert!(
                src.0 == dst.0,
                "paths not equal. {} != {}",
                src.0.to_string_lossy(),
                dst.0.to_string_lossy()
            );
            if let Some(src_c) = &src.1 {
                assert!(dst.1.is_some());
                dst.1.clone().inspect(|dst_c| {
                    assert_eq!(src_c, dst_c);
                });
            }
        })
        .collect::<Vec<_>>();
    drop(src_tmp);
    drop(dest_tmp);
    Ok(())
}
#[test]
fn test_pack_and_save() -> Result<(), super::err::PackingError> {
    let src_tmp = create_test_directory(&PACKING_TEST_MODEL)?;
    let mut pack = super::package_dir(src_tmp.path().to_path_buf())?;
    let dest_tmp = tempdir::TempDir::new("dest_tmp")?;
    let mut out_file = dest_tmp.path().join("pack");
    super::write_package(out_file.clone(), &mut pack)?;
    out_file.set_extension("m3pkg");
    let pack = super::load_package(out_file);
    assert!(pack.is_ok());
    drop(src_tmp);
    drop(dest_tmp);
    Ok(())
}
