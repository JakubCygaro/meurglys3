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
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("failed to parse version")]
    Version,
    #[error("failed to parse compression format")]
    Compression,
    #[error("failed to parse extension format")]
    Extension,
}