use std::fmt;

#[derive(Debug)]
pub enum MemoError {
    Io(std::io::Error),
    IoError(std::io::Error), // 後方互換性のため残す
    YamlError(serde_yaml::Error),
    MemoNotFound(String),
    InvalidId(String),
    EditorError(String),
    ArchiveError(String),
    Search(String),
}

impl fmt::Display for MemoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoError::Io(err) => write!(f, "IO error: {}", err),
            MemoError::IoError(err) => write!(f, "IO error: {}", err),
            MemoError::YamlError(err) => write!(f, "YAML error: {}", err),
            MemoError::MemoNotFound(id) => write!(f, "Memo with ID '{}' not found", id),
            MemoError::InvalidId(id) => write!(f, "Invalid memo ID: '{}'", id),
            MemoError::EditorError(msg) => write!(f, "Editor error: {}", msg),
            MemoError::ArchiveError(msg) => write!(f, "Archive error: {}", msg),
            MemoError::Search(msg) => write!(f, "Search error: {}", msg),
        }
    }
}

impl std::error::Error for MemoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MemoError::Io(err) => Some(err),
            MemoError::IoError(err) => Some(err),
            MemoError::YamlError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for MemoError {
    fn from(err: std::io::Error) -> Self {
        MemoError::Io(err)
    }
}

impl From<serde_yaml::Error> for MemoError {
    fn from(err: serde_yaml::Error) -> Self {
        MemoError::YamlError(err)
    }
}

impl From<tantivy::TantivyError> for MemoError {
    fn from(err: tantivy::TantivyError) -> Self {
        MemoError::Search(err.to_string())
    }
}

impl From<tantivy::query::QueryParserError> for MemoError {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        MemoError::Search(err.to_string())
    }
}

pub type MemoResult<T> = Result<T, MemoError>;
