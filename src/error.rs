/// Errors that can occur while reading a `.musx` file or its EnigmaXML.
#[derive(thiserror::Error, Debug)]
pub enum FinaleError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("neither a .musx archive nor EnigmaXML")]
    NotMusx,

    #[error("not a .musx archive: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error(".musx archive has no {entry}: {source}")]
    MissingEntry {
        entry: &'static str,
        source: zip::result::ZipError,
    },

    #[error("{entry} is not valid EnigmaXML: {source}")]
    Compression {
        entry: &'static str,
        source: std::io::Error,
    },

    #[error("invalid EnigmaXML: {0}")]
    Xml(String),
}

pub type Result<T> = std::result::Result<T, FinaleError>;
