use crate::utils::{TestContext, assertions::*, mocks::*};
use std::fs;

#[test]
fn test_add_creates_memo_file() {
    let context = TestContext::new();
    
    let output = context.run_command(&["add"]);
    
    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");
    
    // 作成されたファイルの確認
    let memo_dir = context.memo_dir();
    let entries: Vec<_> = fs::read_dir(memo_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();
    
    assert!(!entries.is_empty(), "No memo files were created");
}

#[test]
fn test_add_with_echo_editor() {
    let context = TestContext::with_editor("echo");
    
    let output = context.run_command(&["add"]);
    
    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");
}

#[test]
fn test_add_displays_correct_id() {
    let context = TestContext::new();
    
    let output = context.run_command(&["add"]);
    
    assert_command_success(&output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // ID形式の確認（YYYY-MM/DD/HHMMSS）
    assert!(stdout.contains("Memo created:"));
    assert!(stdout.contains("2025-"));
    assert!(stdout.contains("/"));
}

#[test]
fn test_add_creates_directory_structure() {
    let context = TestContext::new();
    
    let output = context.run_command(&["add"]);
    
    assert_command_success(&output);
    
    // ディレクトリ構造の確認
    let memo_dir = context.memo_dir();
    assert!(memo_dir.exists());
    
    // 年月ディレクトリの存在確認
    let year_month_dirs: Vec<_> = fs::read_dir(memo_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect();
    
    assert!(!year_month_dirs.is_empty(), "Year-month directory was not created");
}

#[test]
fn test_add_with_custom_editor() {
    let script_path = create_mock_editor_script("Test content from custom editor");
    let context = TestContext::with_editor(script_path.to_str().unwrap());
    
    let output = context.run_command(&["add"]);
    
    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");
    
    // 作成されたファイルの内容確認
    let memo_dir = context.memo_dir();
    let mut found_content = false;
    
    for entry in fs::read_dir(memo_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            for day_entry in fs::read_dir(entry.path()).unwrap() {
                let day_entry = day_entry.unwrap();
                if day_entry.file_type().unwrap().is_dir() {
                    for file_entry in fs::read_dir(day_entry.path()).unwrap() {
                        let file_entry = file_entry.unwrap();
                        if file_entry.path().extension().map_or(false, |ext| ext == "md") {
                            let content = fs::read_to_string(file_entry.path()).unwrap();
                            if content.contains("Test content from custom editor") {
                                found_content = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    assert!(found_content, "Custom editor content was not found in created memo");
}

#[test]
fn test_add_editor_not_found() {
    let context = TestContext::with_editor(&mock_editor_nonexistent());
    
    let output = context.run_command(&["add"]);
    
    assert_command_failure(&output);
    assert_command_error(&output, "Failed to launch editor");
}

#[test]
fn test_add_editor_exits_with_error() {
    let context = TestContext::with_editor(&mock_editor_fail());
    
    let output = context.run_command(&["add"]);
    
    assert_command_failure(&output);
    assert_command_error(&output, "Editor exited with non-zero status");
}

#[test]
fn test_add_respects_xdg_data_home() {
    let context = TestContext::new();
    
    let output = context.run_command(&["add"]);
    
    assert_command_success(&output);
    
    // XDG_DATA_HOME配下にメモが作成されることを確認
    let memo_dir = context.memo_dir();
    assert!(memo_dir.starts_with(context.temp_dir.path()));
}

#[test]
fn test_add_concurrent_execution() {
    use std::thread;
    use std::sync::Arc;
    
    let context = Arc::new(TestContext::new());
    let mut handles = vec![];
    
    // 複数スレッドで同時にaddを実行
    for i in 0..3 {
        let context_clone = Arc::clone(&context);
        let handle = thread::spawn(move || {
            let output = context_clone.run_command(&["add"]);
            (i, output)
        });
        handles.push(handle);
    }
    
    // 全スレッドの完了を待つ
    let mut success_count = 0;
    for handle in handles {
        let (i, output) = handle.join().unwrap();
        if output.status.success() {
            success_count += 1;
        } else {
            println!("Thread {} failed: {}", i, String::from_utf8_lossy(&output.stderr));
        }
    }
    
    // 少なくとも1つは成功するはず
    assert!(success_count > 0, "No concurrent add operations succeeded");
}

#[test]
fn test_add_index_update_failure_does_not_fail_command() {
    let context = TestContext::new();
    
    // データディレクトリを読み取り専用にしてインデックス更新を失敗させる
    let data_dir = context.temp_dir.path();
    let original_perms = fs::metadata(data_dir).unwrap().permissions();
    
    let mut readonly_perms = original_perms.clone();
    readonly_perms.set_readonly(true);
    fs::set_permissions(data_dir, readonly_perms).unwrap();
    
    let output = context.run_command(&["add"]);
    
    // 権限を戻す
    fs::set_permissions(data_dir, original_perms).unwrap();
    
    // メモ作成は成功するはず（インデックス更新失敗は無視される）
    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");
}

#[test]
fn test_add_permission_denied() {
    let context = TestContext::new();
    
    // メモディレクトリを読み取り専用にする
    let memo_dir = context.memo_dir();
    let original_perms = fs::metadata(memo_dir).unwrap().permissions();
    
    let mut readonly_perms = original_perms.clone();
    readonly_perms.set_readonly(true);
    fs::set_permissions(memo_dir, readonly_perms).unwrap();
    
    let output = context.run_command(&["add"]);
    
    // 権限を戻す
    fs::set_permissions(memo_dir, original_perms).unwrap();
    
    // 権限エラーで失敗するはず
    assert_command_failure(&output);
}

#[test]
fn test_add_different_dates() {
    let context = TestContext::new();
    
    // 複数回addを実行（時間差で）
    let output1 = context.run_command(&["add"]);
    assert_command_success(&output1);
    
    // 少し待つ
    std::thread::sleep(std::time::Duration::from_millis(1100));
    
    let output2 = context.run_command(&["add"]);
    assert_command_success(&output2);
    
    // 異なるIDが生成されることを確認
    let stdout1 = String::from_utf8_lossy(&output1.stdout);
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    
    assert_ne!(stdout1, stdout2, "Different add commands should generate different IDs");
}

#[test]
fn test_add_default_editor_fallback() {
    // EDITOR環境変数を明示的に未設定にする
    let context = TestContext::with_editor("vi");
    
    let output = context.run_command(&["add"]);
    
    // viが利用可能な環境では成功、そうでなければ適切なエラー
    if output.status.success() {
        assert_output_contains(&output, "Memo created:");
    } else {
        // viが見つからない場合のエラー
        assert_command_error(&output, "Failed to launch editor");
    }
}

#[cfg(test)]
mod add_integration_tests {
    use super::*;
    
    #[test]
    fn test_add_then_show_workflow() {
        let script_path = create_mock_editor_script("Content for workflow test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());
        
        // メモを追加
        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);
        
        // 作成されたIDを抽出
        let stdout = String::from_utf8_lossy(&add_output.stdout);
        let id_line = stdout.lines()
            .find(|line| line.contains("Memo created:"))
            .expect("Could not find created memo ID");
        
        let id = id_line.split("Memo created: ").nth(1)
            .expect("Could not extract memo ID");
        
        // 作成したメモを表示
        let show_output = context.run_command(&["show", id]);
        assert_command_success(&show_output);
        assert_output_contains(&show_output, "Content for workflow test");
    }
    
    #[test]
    fn test_add_then_list_workflow() {
        let script_path = create_mock_editor_script("Content for list test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());
        
        // メモを追加
        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);
        
        // リストに表示されることを確認
        let list_output = context.run_command(&["list"]);
        assert_command_success(&list_output);
        assert_output_contains(&list_output, "Content for list test");
    }
}
