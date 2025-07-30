use std::fmt;

#[derive(Debug)]
pub enum MemoError {
    IoError(std::io::Error),
    YamlError(serde_yaml::Error),
    MemoNotFound(String),
    InvalidId(String),
    EditorError(String),
    ArchiveError(String),
}

impl fmt::Display for MemoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoError::IoError(err) => write!(f, "IO error: {}", err),
            MemoError::YamlError(err) => write!(f, "YAML error: {}", err),
            MemoError::MemoNotFound(id) => write!(f, "Memo with ID '{}' not found", id),
            MemoError::InvalidId(id) => write!(f, "Invalid memo ID: '{}'", id),
            MemoError::EditorError(msg) => write!(f, "Editor error: {}", msg),
            MemoError::ArchiveError(msg) => write!(f, "Archive error: {}", msg),
        }
    }
}

impl std::error::Error for MemoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MemoError::IoError(err) => Some(err),
            MemoError::YamlError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MemoError {
    fn from(err: std::io::Error) -> Self {
        MemoError::IoError(err)
    }
}

impl From<serde_yaml::Error> for MemoError {
    fn from(err: serde_yaml::Error) -> Self {
        MemoError::YamlError(err)
    }
}

pub type MemoResult<T> = Result<T, MemoError>;
