use crate::utils::{TestContext, assertions::*};
use std::fs;

fn setup_test_memos(context: &TestContext) {
    context.create_memo("2025-01/30/143022.md", "# Test memo 1\nContent 1\n@tag1");
    context.create_memo("2025-01/30/151545.md", "# Test memo 2\nContent 2\n@tag2");
    context.create_memo("2025-01/30/090000.md", "# Test memo 3\nContent 3\n@tag3");
    context.create_memo("2025-01/29/120000.md", "# Test memo 4\nContent 4\n@tag4");
}

#[test]
fn test_archive_single_id() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&["archive", "2025-01/30/143022.md"]);

    assert_command_success(&output);

    // Check that original file is moved
    assert_memo_not_exists(&context, "2025-01/30/143022.md");

    // Check that file exists in archive
    assert_memo_archived(&context, "2025-01/30/143022.md");

    // Check .ignore file is created
    let ignore_path = context.memo_dir().join(".ignore");
    assert!(ignore_path.exists());
    let ignore_content = fs::read_to_string(&ignore_path).unwrap();
    assert!(ignore_content.contains(".archive"));
}

#[test]
fn test_archive_multiple_ids() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&["archive", "2025-01/30/143022.md", "2025-01/30/151545.md"]);

    assert_command_success(&output);

    // Check that original files are moved
    assert_memo_not_exists(&context, "2025-01/30/143022.md");
    assert_memo_not_exists(&context, "2025-01/30/151545.md");

    // Check that files exist in archive
    assert_memo_archived(&context, "2025-01/30/143022.md");
    assert_memo_archived(&context, "2025-01/30/151545.md");

    // Check that other file remains
    assert_memo_exists(&context, "2025-01/30/090000.md");
}

#[test]
fn test_archive_directory() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&["archive", "2025-01/30/"]);

    assert_command_success(&output);

    // Check that original directory files are moved
    assert_memo_not_exists(&context, "2025-01/30/143022.md");
    assert_memo_not_exists(&context, "2025-01/30/151545.md");
    assert_memo_not_exists(&context, "2025-01/30/090000.md");

    // Check that files exist in archive
    assert_memo_archived(&context, "2025-01/30/143022.md");
    assert_memo_archived(&context, "2025-01/30/151545.md");
    assert_memo_archived(&context, "2025-01/30/090000.md");

    // Check that other day remains
    assert_memo_exists(&context, "2025-01/29/120000.md");
}

#[test]
fn test_archive_file_path() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&["archive", "2025-01/30/143022.md"]);

    assert_command_success(&output);

    // Check that original file is moved
    assert_memo_not_exists(&context, "2025-01/30/143022.md");

    // Check that file exists in archive
    assert_memo_archived(&context, "2025-01/30/143022.md");
}

#[test]
fn test_archive_mixed_targets() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&[
        "archive",
        "2025-01/30/143022.md", // File path
        "2025-01/30/151545.md", // File path
        "2025-01/29/",          // Directory
    ]);

    assert_command_success(&output);

    // Check that files are moved
    assert_memo_not_exists(&context, "2025-01/30/143022.md");
    assert_memo_not_exists(&context, "2025-01/30/151545.md");
    assert_memo_not_exists(&context, "2025-01/29/120000.md");

    // Check that files exist in archive
    assert_memo_archived(&context, "2025-01/30/143022.md");
    assert_memo_archived(&context, "2025-01/30/151545.md");
    assert_memo_archived(&context, "2025-01/29/120000.md");

    // Check that remaining file exists
    assert_memo_exists(&context, "2025-01/30/090000.md");
}

#[test]
fn test_archive_nonexistent_id() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&["archive", "999999"]);

    assert_command_failure(&output);
    assert_command_error(&output, "not found");
}

#[test]
fn test_archive_ignore_file_already_exists() {
    let context = TestContext::new();
    setup_test_memos(&context);

    // Create existing .ignore file
    let ignore_path = context.memo_dir().join(".ignore");
    fs::write(&ignore_path, "existing_content\n").unwrap();

    let output = context.run_command(&["archive", "2025-01/30/143022.md"]);

    assert_command_success(&output);

    // Check that .ignore file contains both existing content and .archive
    let ignore_content = fs::read_to_string(&ignore_path).unwrap();
    assert!(ignore_content.contains("existing_content"));
    assert!(ignore_content.contains(".archive"));
}

#[test]
fn test_archive_no_arguments() {
    let context = TestContext::new();
    setup_test_memos(&context);

    let output = context.run_command(&["archive"]);

    assert_command_failure(&output);
    assert_command_error(&output, "At least one target");
}

#[test]
fn test_archive_empty_directory() {
    let context = TestContext::new();

    // 空のディレクトリを作成
    let empty_dir = context.memo_dir().join("2025-01/31");
    fs::create_dir_all(&empty_dir).unwrap();

    let output = context.run_command(&["archive", "2025-01/31/"]);

    // 空のディレクトリのアーカイブは成功するが、出力は実装依存
    assert_command_success(&output);
    // 具体的な出力メッセージは実装に依存するため、成功のみ確認
}

#[cfg(test)]
mod archive_integration_tests {
    use super::*;

    #[test]
    fn test_archive_then_list_workflow() {
        let context = TestContext::new();
        setup_test_memos(&context);

        // アーカイブ前のリスト
        let list_before = context.run_command(&["list"]);
        assert_command_success(&list_before);
        assert_output_contains(&list_before, "143022");

        // メモをアーカイブ
        let archive_output = context.run_command(&["archive", "2025-01/30/143022.md"]);
        assert_command_success(&archive_output);

        // アーカイブ後のリスト
        let list_after = context.run_command(&["list"]);
        assert_command_success(&list_after);

        let stdout = String::from_utf8_lossy(&list_after.stdout);
        assert!(!stdout.contains("143022"));
    }

    #[test]
    fn test_archive_multiple_workflows() {
        let context = TestContext::new();
        setup_test_memos(&context);

        // 段階的にアーカイブ
        let archive1 = context.run_command(&["archive", "2025-01/30/143022.md"]);
        assert_command_success(&archive1);

        let archive2 = context.run_command(&["archive", "2025-01/30/151545.md"]);
        assert_command_success(&archive2);

        let archive3 = context.run_command(&["archive", "2025-01/29/"]);
        assert_command_success(&archive3);

        // 最後に残ったメモのみが存在することを確認
        assert_memo_exists(&context, "2025-01/30/090000.md");
        assert_memo_archived(&context, "2025-01/30/143022.md");
        assert_memo_archived(&context, "2025-01/30/151545.md");
        assert_memo_archived(&context, "2025-01/29/120000.md");
    }
}
