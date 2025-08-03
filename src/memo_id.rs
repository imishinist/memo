use crate::error::{MemoError, MemoResult};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoId {
    datetime: DateTime<Local>,
}

impl MemoId {
    pub fn new() -> Self {
        Self {
            datetime: Local::now(),
        }
    }

    pub fn from_str(id: &str) -> MemoResult<Self> {
        if id.len() != 14 {
            return Err(MemoError::InvalidId(id.to_string()));
        }

        let datetime = NaiveDateTime::parse_from_str(id, "%Y%m%d%H%M%S")
            .map_err(|_| MemoError::InvalidId(id.to_string()))?;
        let datetime = Local
            .from_local_datetime(&datetime)
            .single()
            .ok_or_else(|| MemoError::InvalidId(id.to_string()))?;
        Ok(Self { datetime })
    }

    /// ファイルパスからMemoIDを作成
    pub fn from_path(path: &Path) -> MemoResult<Self> {
        let components: Vec<_> = path
            .components()
            .rev()
            .take(3)
            .map(|c| c.as_os_str().to_string_lossy())
            .collect();
        if components.len() < 3 {
            return Err(MemoError::InvalidId(path.to_string_lossy().to_string()));
        }

        let filename = &components[0];
        let day = &components[1];
        let year_month = &components[2];

        let stem = Path::new(filename.as_ref())
            .file_stem()
            .ok_or_else(|| MemoError::InvalidId(filename.to_string()))?
            .to_string_lossy();

        // "2025-01/30/143022" の形式から "20250130143022" を構築
        let year_month_clean = year_month.replace("-", "");
        let id_str = format!("{}{}{}", year_month_clean, day, stem);
        Self::from_str(&id_str)
    }

    pub fn as_str(&self) -> String {
        self.datetime.format("%Y%m%d%H%M%S").to_string()
    }

    pub fn to_file_path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.to_relative_path())
    }

    pub fn to_relative_path(&self) -> PathBuf {
        let year_month = self.datetime.format("%Y-%m").to_string();
        let day = self.datetime.format("%d").to_string();
        let time = self.datetime.format("%H%M%S").to_string();

        PathBuf::new()
            .join(year_month)
            .join(day)
            .join(format!("{}.md", time))
    }

    pub fn get_datetime(&self) -> DateTime<Local> {
        self.datetime
    }
}

impl std::fmt::Display for MemoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_memo_id_from_str() {
        let cases = vec!["20250130143022"];
        for case in cases {
            let id = MemoId::from_str(case);
            assert!(id.is_ok());
            assert_eq!(id.unwrap().as_str(), case);
        }
    }

    #[test]
    fn test_memo_id_from_str_invalid() {
        assert!(MemoId::from_str("invalid").is_err());
        assert!(MemoId::from_str("2025013014302").is_err()); // 13桁
        assert!(MemoId::from_str("202501301430222").is_err()); // 15桁
        assert!(MemoId::from_str("20250130143a22").is_err()); // 非数字
    }

    #[test]
    fn test_memo_id_from_path() {
        let path = PathBuf::from("/tmp/memo/2025-01/30/143022.md");
        let id = MemoId::from_path(&path).unwrap();
        assert_eq!(id.as_str(), "20250130143022");
    }

    #[test]
    fn test_memo_id_to_file_path() {
        let id = MemoId::from_str("20250130143022").unwrap();
        let base_dir = PathBuf::from("/tmp/memo");
        let path = id.to_file_path(&base_dir);

        assert_eq!(path, PathBuf::from("/tmp/memo/2025-01/30/143022.md"));
    }

    #[test]
    fn test_memo_id_to_relative_path() {
        let id = MemoId::from_str("20250130143022").unwrap();
        let relative_path = id.to_relative_path().to_string_lossy().to_string();

        assert_eq!(relative_path, "2025-01/30/143022.md");
    }

    #[test]
    fn test_memo_id_display() {
        let id = MemoId::from_str("20250130143022").unwrap();
        assert_eq!(format!("{}", id), "20250130143022");
    }
}
