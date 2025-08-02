use crate::error::{MemoError, MemoResult};
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MemoId {
    datetime: DateTime<Local>,
}

impl MemoId {
    /// 現在時刻からMemoIDを生成
    pub fn new() -> Self {
        Self {
            datetime: Local::now(),
        }
    }

    /// 文字列からMemoIDを作成（14桁の数字）
    pub fn from_str(id: &str) -> MemoResult<Self> {
        if id.len() != 14 || !id.chars().all(|c| c.is_ascii_digit()) {
            return Err(MemoError::InvalidId(id.to_string()));
        }

        let year: i32 = id[0..4].parse().map_err(|_| MemoError::InvalidId(id.to_string()))?;
        let month: u32 = id[4..6].parse().map_err(|_| MemoError::InvalidId(id.to_string()))?;
        let day: u32 = id[6..8].parse().map_err(|_| MemoError::InvalidId(id.to_string()))?;
        let hour: u32 = id[8..10].parse().map_err(|_| MemoError::InvalidId(id.to_string()))?;
        let minute: u32 = id[10..12].parse().map_err(|_| MemoError::InvalidId(id.to_string()))?;
        let second: u32 = id[12..14].parse().map_err(|_| MemoError::InvalidId(id.to_string()))?;

        let naive_date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| MemoError::InvalidId(id.to_string()))?;
        let naive_time = chrono::NaiveTime::from_hms_opt(hour, minute, second)
            .ok_or_else(|| MemoError::InvalidId(id.to_string()))?;
        let naive_dt = NaiveDateTime::new(naive_date, naive_time);

        let datetime = Local.from_local_datetime(&naive_dt)
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

    /// 14桁の文字列形式で取得
    pub fn as_str(&self) -> String {
        self.datetime.format("%Y%m%d%H%M%S").to_string()
    }

    /// ファイルパスに変換
    pub fn to_file_path(&self, base_dir: &Path) -> PathBuf {
        let year_month = self.datetime.format("%Y-%m").to_string();
        let day = self.datetime.format("%d").to_string();
        let time = self.datetime.format("%H%M%S").to_string();

        base_dir
            .join(year_month)
            .join(day)
            .join(format!("{}.md", time))
    }

    /// 相対パス形式で取得（ディレクトリ作成用）
    pub fn to_relative_path(&self) -> String {
        let year_month = self.datetime.format("%Y-%m").to_string();
        let day = self.datetime.format("%d").to_string();
        let time = self.datetime.format("%H%M%S").to_string();

        format!("{}/{}/{}.md", year_month, day, time)
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
        let id = MemoId::from_str("20250130143022").unwrap();
        assert_eq!(id.as_str(), "20250130143022");
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
        
        assert_eq!(
            path,
            PathBuf::from("/tmp/memo/2025-01/30/143022.md")
        );
    }

    #[test]
    fn test_memo_id_to_relative_path() {
        let id = MemoId::from_str("20250130143022").unwrap();
        let relative_path = id.to_relative_path();
        
        assert_eq!(relative_path, "2025-01/30/143022.md");
    }

    #[test]
    fn test_memo_id_display() {
        let id = MemoId::from_str("20250130143022").unwrap();
        assert_eq!(format!("{}", id), "20250130143022");
    }
}
