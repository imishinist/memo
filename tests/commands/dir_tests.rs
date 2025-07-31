use crate::utils::{TestContext, assertions::*};
use std::fs;

#[test]
fn test_dir_displays_path() {
    let context = TestContext::new();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let displayed_path = stdout.trim();
    
    // 表示されたパスが実際のメモディレクトリと一致することを確認
    assert_eq!(displayed_path, context.memo_dir().to_string_lossy());
}

#[test]
fn test_dir_creates_directory_if_not_exists() {
    let context = TestContext::new();
    
    // メモディレクトリを削除
    fs::remove_dir_all(context.memo_dir()).unwrap();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    // ディレクトリが再作成されていることを確認
    assert!(context.memo_dir().exists());
    assert!(context.memo_dir().is_dir());
}

#[test]
fn test_dir_respects_xdg_data_home() {
    let context = TestContext::new();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let displayed_path = stdout.trim();
    
    // XDG_DATA_HOME配下のパスが表示されることを確認
    assert!(displayed_path.starts_with(context.temp_dir.path().to_str().unwrap()));
    assert!(displayed_path.ends_with("memo"));
}

#[test]
fn test_dir_with_existing_memos() {
    let context = TestContext::new();
    
    // テストメモを作成
    context.setup_test_memos();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let displayed_path = stdout.trim();
    
    // メモが存在してもパス表示は変わらない
    assert_eq!(displayed_path, context.memo_dir().to_string_lossy());
}

#[test]
fn test_dir_output_format() {
    let context = TestContext::new();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // 出力が1行で、改行で終わることを確認
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert!(stdout.ends_with('\n'));
}

#[test]
fn test_dir_absolute_path() {
    let context = TestContext::new();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let displayed_path = stdout.trim();
    
    // 絶対パスが表示されることを確認
    assert!(displayed_path.starts_with('/'));
}

#[test]
fn test_dir_no_arguments() {
    let context = TestContext::new();
    
    let output = context.run_command(&["dir"]);
    
    assert_command_success(&output);
    
    // 引数なしで正常に動作することを確認
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn test_dir_with_extra_arguments() {
    let context = TestContext::new();
    
    // 余分な引数を付けて実行
    let output = context.run_command(&["dir", "extra", "arguments"]);
    
    // 実装では余分な引数はエラーになる
    assert_command_failure(&output);
    assert_command_error(&output, "unexpected argument");
}

#[test]
fn test_dir_permission_denied() {
    let context = TestContext::new();
    
    // 親ディレクトリを読み取り専用にする
    let parent_dir = context.memo_dir().parent().unwrap();
    let original_perms = fs::metadata(parent_dir).unwrap().permissions();
    
    let mut readonly_perms = original_perms.clone();
    readonly_perms.set_readonly(true);
    fs::set_permissions(parent_dir, readonly_perms).unwrap();
    
    let output = context.run_command(&["dir"]);
    
    // 権限を戻す
    fs::set_permissions(parent_dir, original_perms).unwrap();
    
    // パス表示は成功するはず（ディレクトリ作成は失敗する可能性があるが）
    assert_command_success(&output);
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.trim().is_empty());
}

#[test]
fn test_dir_consistency_across_calls() {
    let context = TestContext::new();
    
    // 複数回呼び出して同じパスが返されることを確認
    let output1 = context.run_command(&["dir"]);
    let output2 = context.run_command(&["dir"]);
    let output3 = context.run_command(&["dir"]);
    
    assert_command_success(&output1);
    assert_command_success(&output2);
    assert_command_success(&output3);
    
    let path1 = String::from_utf8_lossy(&output1.stdout);
    let path2 = String::from_utf8_lossy(&output2.stdout);
    let path3 = String::from_utf8_lossy(&output3.stdout);
    
    assert_eq!(path1, path2);
    assert_eq!(path2, path3);
}

#[cfg(test)]
mod dir_integration_tests {
    use super::*;
    
    #[test]
    fn test_dir_then_add_workflow() {
        let context = TestContext::new();
        
        // dirコマンドでパスを取得
        let dir_output = context.run_command(&["dir"]);
        assert_command_success(&dir_output);
        
        let memo_dir_path = String::from_utf8_lossy(&dir_output.stdout).trim().to_string();
        
        // addコマンドでメモを作成
        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);
        
        // 指定されたディレクトリにメモが作成されていることを確認
        assert!(fs::read_dir(&memo_dir_path).unwrap().count() > 0);
    }
    
    #[test]
    fn test_dir_for_external_tools() {
        let context = TestContext::new();
        
        // テストメモを作成
        context.create_memo("2025-01/30/143022.md", "Test content for external tools");
        
        // dirコマンドでパスを取得
        let dir_output = context.run_command(&["dir"]);
        assert_command_success(&dir_output);
        
        let memo_dir_path = String::from_utf8_lossy(&dir_output.stdout).trim().to_string();
        
        // 外部ツール（find）でメモファイルを検索
        let find_output = std::process::Command::new("find")
            .arg(&memo_dir_path)
            .arg("-name")
            .arg("*.md")
            .output()
            .expect("Failed to execute find command");
        
        assert!(find_output.status.success());
        
        let find_stdout = String::from_utf8_lossy(&find_output.stdout);
        assert!(find_stdout.contains("143022.md"));
    }
    
    #[test]
    fn test_dir_with_grep_workflow() {
        let context = TestContext::new();
        
        // 検索可能なメモを作成
        context.create_memo("2025-01/30/143022.md", "This memo contains searchable keyword");
        context.create_memo("2025-01/30/151545.md", "This memo has different content");
        
        // dirコマンドでパスを取得
        let dir_output = context.run_command(&["dir"]);
        assert_command_success(&dir_output);
        
        let memo_dir_path = String::from_utf8_lossy(&dir_output.stdout).trim().to_string();
        
        // grepで検索
        let grep_output = std::process::Command::new("grep")
            .arg("-r")
            .arg("searchable")
            .arg(&memo_dir_path)
            .output()
            .expect("Failed to execute grep command");
        
        assert!(grep_output.status.success());
        
        let grep_stdout = String::from_utf8_lossy(&grep_output.stdout);
        assert!(grep_stdout.contains("143022.md"));
        assert!(grep_stdout.contains("searchable keyword"));
    }
    
    #[test]
    fn test_dir_path_usable_by_shell() {
        let context = TestContext::new();
        
        // テストメモを作成
        context.create_memo("2025-01/30/143022.md", "Shell test content");
        
        // dirコマンドでパスを取得
        let dir_output = context.run_command(&["dir"]);
        assert_command_success(&dir_output);
        
        let memo_dir_path = String::from_utf8_lossy(&dir_output.stdout).trim().to_string();
        
        // シェルコマンドでファイル数をカウント
        let ls_output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&format!("find '{}' -name '*.md' | wc -l", memo_dir_path))
            .output()
            .expect("Failed to execute shell command");
        
        assert!(ls_output.status.success());
        
        let count = String::from_utf8_lossy(&ls_output.stdout).trim().parse::<i32>().unwrap();
        assert!(count > 0);
    }
}
