use crate::context::MemoContext;
use crate::error::{MemoError, MemoResult};
use crate::repository::MemoRepository;
use crate::utils::id_resolver::resolve_memo_id;

pub fn run(context: &MemoContext, targets: &[String]) -> MemoResult<()> {
    if targets.is_empty() {
        return Err(MemoError::ArchiveError(
            "At least one target (ID, file path, or directory) is required".to_string(),
        ));
    }

    let repo = MemoRepository::new(context.clone());
    let mut archived_count = 0;
    let mut errors = Vec::new();

    for target in targets {
        match archive_target(&repo, target) {
            Ok(count) => archived_count += count,
            Err(e) => errors.push(format!("Error archiving '{}': {}", target, e)),
        }
    }

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("{}", error);
        }
        if archived_count == 0 {
            return Err(MemoError::ArchiveError(
                "Failed to archive any targets".to_string(),
            ));
        }
    }

    if archived_count > 0 {
        println!("Archived {} memo(s)", archived_count);
    }

    Ok(())
}

fn archive_target(repo: &MemoRepository, target: &str) -> MemoResult<usize> {
    if let Ok(file_path) = resolve_memo_id(repo.memo_dir(), target) {
        let memo = crate::memo::MemoFile::from_path(&file_path)?;
        repo.archive_memo(&memo)?;
        return Ok(1);
    }

    let file_path = if target.ends_with(".md") {
        repo.memo_dir().join(target)
    } else {
        repo.memo_dir().join(format!("{}.md", target))
    };

    if file_path.exists() && file_path.is_file() {
        let memo = crate::memo::MemoFile::from_path(&file_path)?;
        repo.archive_memo(&memo)?;
        return Ok(1);
    }

    let file_path_no_ext = repo.memo_dir().join(target);
    if file_path_no_ext.exists() && file_path_no_ext.is_file() {
        let memo = crate::memo::MemoFile::from_path(&file_path_no_ext)?;
        repo.archive_memo(&memo)?;
        return Ok(1);
    }

    let dir_path = target.trim_end_matches('/');
    if repo.memo_dir().join(dir_path).exists() && repo.memo_dir().join(dir_path).is_dir() {
        let archived_memos = repo.archive_directory(dir_path)?;
        return Ok(archived_memos.len());
    }

    Err(MemoError::MemoNotFound(target.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::MemoContext;
    use crate::repository::MemoRepository;
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

    fn setup_test_memos(context: &MemoContext) {
        let repo = MemoRepository::new(context.clone());

        repo.create_memo("2025-01/30/143022.md", "Test memo 1".to_string())
            .unwrap();
        repo.create_memo("2025-01/30/151545.md", "Test memo 2".to_string())
            .unwrap();
        repo.create_memo("2025-01/29/120000.md", "Test memo 3".to_string())
            .unwrap();
    }

    #[test]
    fn test_archive_single_id() {
        let (_temp_dir, context) = create_test_context();
        setup_test_memos(&context);

        let targets = vec!["2025-01/30/143022".to_string()];
        let result = run(&context, &targets);

        assert!(result.is_ok());

        assert!(!context.memo_dir.join("2025-01/30/143022.md").exists());

        assert!(context.archive_dir().join("2025-01/30/143022.md").exists());
    }

    #[test]
    fn test_archive_multiple_ids() {
        let (_temp_dir, context) = create_test_context();
        setup_test_memos(&context);

        let targets = vec![
            "2025-01/30/143022.md".to_string(),
            "2025-01/30/151545.md".to_string(),
        ];
        let result = run(&context, &targets);

        assert!(result.is_ok());

        assert!(!context.memo_dir.join("2025-01/30/143022.md").exists());
        assert!(!context.memo_dir.join("2025-01/30/151545.md").exists());

        assert!(context.archive_dir().join("2025-01/30/143022.md").exists());
        assert!(context.archive_dir().join("2025-01/30/151545.md").exists());

        assert!(context.memo_dir.join("2025-01/29/120000.md").exists());
    }

    #[test]
    fn test_archive_directory() {
        let (_temp_dir, context) = create_test_context();
        setup_test_memos(&context);

        let targets = vec!["2025-01/30/".to_string()];
        let result = run(&context, &targets);

        assert!(result.is_ok());

        assert!(!context.memo_dir.join("2025-01/30/143022.md").exists());
        assert!(!context.memo_dir.join("2025-01/30/151545.md").exists());

        assert!(context.archive_dir().join("2025-01/30/143022.md").exists());
        assert!(context.archive_dir().join("2025-01/30/151545.md").exists());

        assert!(context.memo_dir.join("2025-01/29/120000.md").exists());
    }

    #[test]
    fn test_archive_nonexistent_id() {
        let (_temp_dir, context) = create_test_context();
        setup_test_memos(&context);

        let targets = vec!["999999".to_string()];
        let result = run(&context, &targets);

        assert!(result.is_err());
    }

    #[test]
    fn test_archive_no_arguments() {
        let (_temp_dir, context) = create_test_context();

        let targets = vec![];
        let result = run(&context, &targets);

        assert!(result.is_err());
        if let Err(MemoError::ArchiveError(msg)) = result {
            assert!(msg.contains("At least one target"));
        } else {
            panic!("Expected ArchiveError");
        }
    }
}
