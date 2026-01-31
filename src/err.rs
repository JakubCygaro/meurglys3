use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnpackError {
    #[error("file is not a valid .m3pgk file")]
    InvalidFile,

    #[error("file is of invalid version")]
    InvalidVersion,

    #[error("unknown compression format")]
    InvalidCompression,

    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error("unsupported file format")]
    FileExtension,

    #[error("error when reading file name string")]
    StringError(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    UnsupportedFormat(#[from] UnsupportedError),
}

#[derive(Error, Debug)]
pub enum UnsupportedError {
    #[error("unsupported file version")]
    Version,

    #[error("unsupported file compression")]
    Compression,
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to parse version")]
    Version,
    #[error("failed to parse compression format")]
    Compression,
    #[error("failed to parse extension format")]
    Extension,
    #[error("failed to parse index data")]
    Index,
    #[error("failed to parse size data")]
    Size,
}

#[derive(Error, Debug)]
pub enum PackingError {
    #[error(transparent)]
    FileReadingError(#[from] std::path::StripPrefixError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum InsertError {
    #[error("not a file path")]
    NotAFilePath,
    #[error("prohibited path")]
    ProhibitedPath,
}
