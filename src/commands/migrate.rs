use crate::context::MemoContext;
use crate::error::MemoResult;
use std::fs;
use std::path::Path;

/// 旧形式（YYYY-MM/DD/HHMMSS.md）を新形式（YYYY-MM/DD/YYYYMMDDHHmmss.md）に移行
pub fn run(context: &MemoContext, dry_run: bool) -> MemoResult<()> {
    let mut count = 0;
    migrate_dir(&context.memo_dir, &context.memo_dir, dry_run, &mut count)?;

    let archive_dir = context.archive_dir();
    if archive_dir.exists() {
        migrate_dir(&archive_dir, &archive_dir, dry_run, &mut count)?;
    }

    if count == 0 {
        eprintln!("No files need migration.");
    } else if dry_run {
        eprintln!("{} file(s) would be renamed. Run without --dry-run to apply.", count);
    } else {
        eprintln!("{} file(s) renamed.", count);
    }

    Ok(())
}

fn migrate_dir(dir: &Path, base_dir: &Path, dry_run: bool, count: &mut usize) -> MemoResult<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // skip hidden directories other than .archive (handled separately)
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
            migrate_dir(&path, base_dir, dry_run, count)?;
            continue;
        }

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };

        // 新形式（14桁）はスキップ
        if stem.len() == 14 && stem.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // 旧形式（6桁 = HHMMSS）を検出
        if stem.len() != 6 || !stem.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        // パスからYYYY-MM/DDを取得してフルIDを組み立てる
        let parent = match path.parent() {
            Some(p) => p,
            None => continue,
        };
        let day = match parent.file_name().and_then(|n| n.to_str()) {
            Some(d) if d.len() == 2 => d,
            _ => continue,
        };
        let year_month_dir = match parent.parent() {
            Some(p) => p,
            None => continue,
        };
        let year_month = match year_month_dir.file_name().and_then(|n| n.to_str()) {
            Some(ym) if ym.len() == 7 && ym.contains('-') => ym,
            _ => continue,
        };

        let year_month_clean = year_month.replace('-', "");
        let full_id = format!("{}{}{}", year_month_clean, day, stem);
        let new_path = parent.join(format!("{}.md", full_id));

        if dry_run {
            let old_rel = path.strip_prefix(base_dir).unwrap_or(&path);
            let new_rel = new_path.strip_prefix(base_dir).unwrap_or(&new_path);
            eprintln!("  {} -> {}", old_rel.display(), new_rel.display());
        } else {
            fs::rename(&path, &new_path)?;
        }
        *count += 1;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::MemoContext;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_context() -> (TempDir, MemoContext) {
        let temp_dir = TempDir::new().unwrap();
        let memo_dir = temp_dir.path().join("memo");
        fs::create_dir_all(&memo_dir).unwrap();

        let context = MemoContext {
            memo_dir,
            editor: "echo".to_string(),
        };

        (temp_dir, context)
    }

    #[test]
    fn test_migrate_old_format() {
        let (_temp_dir, context) = create_test_context();

        // 旧形式のファイルを作成
        let dir = context.memo_dir.join("2025-01/30");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("143022.md"), "memo 1").unwrap();
        fs::write(dir.join("151545.md"), "memo 2").unwrap();

        run(&context, false).unwrap();

        assert!(!dir.join("143022.md").exists());
        assert!(!dir.join("151545.md").exists());
        assert!(dir.join("20250130143022.md").exists());
        assert!(dir.join("20250130151545.md").exists());
    }

    #[test]
    fn test_migrate_dry_run() {
        let (_temp_dir, context) = create_test_context();

        let dir = context.memo_dir.join("2025-01/30");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("143022.md"), "memo 1").unwrap();

        run(&context, true).unwrap();

        // dry-run なのでファイルは変更されない
        assert!(dir.join("143022.md").exists());
        assert!(!dir.join("20250130143022.md").exists());
    }

    #[test]
    fn test_migrate_skips_new_format() {
        let (_temp_dir, context) = create_test_context();

        let dir = context.memo_dir.join("2025-01/30");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("20250130143022.md"), "already migrated").unwrap();

        run(&context, false).unwrap();

        assert!(dir.join("20250130143022.md").exists());
    }

    #[test]
    fn test_migrate_archive() {
        let (_temp_dir, context) = create_test_context();

        let archive = context.archive_dir().join("2025-01/30");
        fs::create_dir_all(&archive).unwrap();
        fs::write(archive.join("143022.md"), "archived memo").unwrap();

        run(&context, false).unwrap();

        assert!(!archive.join("143022.md").exists());
        assert!(archive.join("20250130143022.md").exists());
    }
}
