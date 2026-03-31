use memo::{MemoContext, MemoFile};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

/// テスト用のコンテキスト管理構造体
pub struct TestContext {
    pub temp_dir: TempDir,
    pub memo_context: MemoContext,
    pub binary_path: PathBuf,
}

impl TestContext {
    /// 新しいテストコンテキストを作成
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let memo_dir = temp_dir.path().join("memo");
        fs::create_dir_all(&memo_dir).expect("Failed to create memo directory");

        let memo_context = MemoContext {
            memo_dir,
            editor: "echo".to_string(),
        };

        let binary_path = get_binary_path();

        Self {
            temp_dir,
            memo_context,
            binary_path,
        }
    }

    /// カスタムエディタでテストコンテキストを作成
    pub fn with_editor(editor: &str) -> Self {
        let mut context = Self::new();
        context.memo_context.editor = editor.to_string();
        context
    }

    /// テストメモを作成
    pub fn create_memo(&self, relative_path: &str, content: &str) -> MemoFile {
        let repo = memo::repository::MemoRepository::new(self.memo_context.clone());
        repo.create_memo(relative_path, content.to_string())
            .expect("Failed to create test memo")
    }

    /// コマンドを実行
    pub fn run_command(&self, args: &[&str]) -> Output {
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(args)
            .env("XDG_DATA_HOME", self.temp_dir.path())
            .env("EDITOR", &self.memo_context.editor)
            .current_dir(self.temp_dir.path()); // 作業ディレクトリも設定

        cmd.output().expect("Failed to execute command")
    }

    /// 複数のテストメモを一括作成
    pub fn setup_test_memos(&self) -> Vec<MemoFile> {
        vec![
            self.create_memo("2025-01/30/20250130143022.md", TestMemoTemplates::BASIC),
            self.create_memo("2025-01/30/20250130151545.md", TestMemoTemplates::WITH_FRONT_MATTER),
            self.create_memo("2025-01/29/20250129120000.md", TestMemoTemplates::MULTILINE),
            self.create_memo("2025-01/28/20250128090000.md", TestMemoTemplates::JAPANESE),
        ]
    }

    /// 検索インデックスを構築
    pub fn build_search_index(&self) -> Result<(), memo::error::MemoError> {
        memo::commands::index::run_index(&self.memo_context)
    }

    /// メモディレクトリのパスを取得
    pub fn memo_dir(&self) -> &std::path::Path {
        &self.memo_context.memo_dir
    }

    /// アーカイブディレクトリのパスを取得
    pub fn archive_dir(&self) -> PathBuf {
        self.memo_context.archive_dir()
    }

    pub fn index_dir(&self) -> PathBuf {
        self.memo_context.index_dir()
    }
}

/// バイナリパスを取得
fn get_binary_path() -> PathBuf {
    let mut path = std::env::current_dir().expect("Failed to get current directory");
    path.push("target");
    path.push("debug");
    path.push("memo");
    path
}

/// テストメモのテンプレート
pub struct TestMemoTemplates;

impl TestMemoTemplates {
    pub const BASIC: &'static str = "# Basic Memo\n\nThis is a basic test memo.\n\n@test @basic";

    pub const WITH_FRONT_MATTER: &'static str = r#"---
title: Test Memo with Frontmatter
tags: ["@test", "@frontmatter"]
priority: 1
created_at: "2025-01-30 15:15:45"
---

# Test Memo with Frontmatter

This memo has frontmatter for testing.

@test @frontmatter"#;

    pub const MULTILINE: &'static str = r#"# Multiline Test Memo

This is a multiline memo for testing purposes.

## Section 1
Content of section 1 with some details.

## Section 2
Content of section 2 with more information.

### Subsection
Even more nested content.

@test @multiline @sections"#;

    pub const JAPANESE: &'static str = r#"# 日本語テストメモ

これは日本語のテストメモです。

## セクション1
日本語の内容をテストします。

## セクション2
漢字、ひらがな、カタカナの混在テスト。

絵文字もテスト: 🚀 📝 ✅

@日本語 @テスト @絵文字"#;

    pub const WITH_SPECIAL_CHARS: &'static str = r#"# Special Characters Test

Testing special characters: !@#$%^&*()_+-=[]{}|;':\",./<>?

Unicode characters: αβγδε ñáéíóú

Emoji: 🚀 📝 ✅ 🎉 💻 🔍

@special @unicode @emoji"#;

    pub const EMPTY: &'static str = "";

    /// 指定サイズの大きなメモを生成
    pub fn large_memo(size_kb: usize) -> String {
        let line = "This is a line of text for testing large memos with sufficient content.\n";
        let lines_needed = (size_kb * 1024) / line.len();
        format!(
            "# Large Test Memo\n\nThis memo is approximately {} KB in size.\n\n{}\n@large @test",
            size_kb,
            line.repeat(lines_needed)
        )
    }

    /// フロントマター付きメモを生成
    pub fn with_custom_frontmatter(title: &str, tags: &[&str], content: &str) -> String {
        let tags_json = tags
            .iter()
            .map(|tag| format!("\"{}\"", tag))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            r#"---
title: {}
tags: [{}]
created_at: "2025-01-30 12:00:00"
---

{}"#,
            title, tags_json, content
        )
    }
}

/// アサーションヘルパー関数
pub mod assertions {
    use super::*;
    use std::process::Output;

    /// コマンドが成功したことをアサート
    pub fn assert_command_success(output: &Output) {
        assert!(
            output.status.success(),
            "Command failed with status: {}\n stdout: {}\n stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// コマンドが失敗したことをアサート
    pub fn assert_command_failure(output: &Output) {
        assert!(
            !output.status.success(),
            "Command unexpectedly succeeded\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    /// コマンドが特定のエラーメッセージで失敗したことをアサート
    pub fn assert_command_error(output: &Output, expected_error: &str) {
        assert_command_failure(output);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_error),
            "Expected error message '{}' not found in stderr: {}",
            expected_error,
            stderr
        );
    }

    /// メモファイルが存在することをアサート
    pub fn assert_memo_exists(context: &TestContext, relative_path: &str) {
        let file_path = context.memo_dir().join(relative_path);
        assert!(
            file_path.exists(),
            "Memo file does not exist: {}",
            file_path.display()
        );
    }

    /// メモファイルが存在しないことをアサート
    pub fn assert_memo_not_exists(context: &TestContext, relative_path: &str) {
        let file_path = context.memo_dir().join(relative_path);
        assert!(
            !file_path.exists(),
            "Memo file unexpectedly exists: {}",
            file_path.display()
        );
    }

    /// メモがアーカイブされていることをアサート
    pub fn assert_memo_archived(context: &TestContext, relative_path: &str) {
        let archived_path = context.archive_dir().join(relative_path);
        assert!(
            archived_path.exists(),
            "Memo not found in archive: {}",
            archived_path.display()
        );

        let original_path = context.memo_dir().join(relative_path);
        assert!(
            !original_path.exists(),
            "Memo still exists in original location: {}",
            original_path.display()
        );
    }

    /// JSON出力が有効であることをアサート
    pub fn assert_valid_json(output: &str) -> serde_json::Value {
        serde_json::from_str(output).expect("Invalid JSON output")
    }

    /// 出力に特定のテキストが含まれることをアサート
    pub fn assert_output_contains(output: &Output, expected_text: &str) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(expected_text),
            "Expected text '{}' not found in output: {}",
            expected_text,
            stdout
        );
    }
}

/// モックヘルパー関数
pub mod mocks {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    /// エコーエディタ（引数をファイルに書き込む）
    pub fn mock_editor_echo() -> String {
        "echo".to_string()
    }

    /// 失敗するエディタ
    pub fn mock_editor_fail() -> String {
        "false".to_string()
    }

    /// 特定の内容を書き込むエディタスクリプトを作成
    pub fn create_mock_editor_script(content: &str) -> std::path::PathBuf {
        use std::io::Write;

        let script_content = format!(
            r#"#!/bin/bash
echo '{}' > "$1"
"#,
            content.replace('\'', "'\"'\"'")
        );

        // 一意なファイル名を生成
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let script_path = std::env::temp_dir().join(format!("mock_editor_{}.sh", timestamp));

        // スクリプトファイルを作成
        let mut file = std::fs::File::create(&script_path).expect("Failed to create script file");
        file.write_all(script_content.as_bytes())
            .expect("Failed to write script");
        file.sync_all().expect("Failed to sync file");
        drop(file);

        // 実行権限を設定
        let mut perms = fs::metadata(&script_path)
            .expect("Failed to get metadata")
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).expect("Failed to set permissions");

        script_path
    }

    /// 存在しないエディタ
    pub fn mock_editor_nonexistent() -> String {
        "nonexistent_editor_12345".to_string()
    }
}

/// 検索テスト用ヘルパー
pub mod search_helpers {
    use super::*;

    /// 検索を実行して結果を確認
    pub fn search_and_assert_results(context: &TestContext, query: &str, expected_ids: &[&str]) {
        let output = context.run_command(&["search", query]);
        assertions::assert_command_success(&output);

        let stdout = String::from_utf8_lossy(&output.stdout);

        for expected_id in expected_ids {
            assert!(
                stdout.contains(expected_id),
                "Expected ID '{}' not found in search results: {}",
                expected_id,
                stdout
            );
        }
    }

    /// 検索結果が空であることを確認
    pub fn assert_no_search_results(context: &TestContext, query: &str) {
        let output = context.run_command(&["search", query]);
        assertions::assert_command_success(&output);

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("No results found"),
            "Expected no results message not found: {}",
            stdout
        );
    }
}
