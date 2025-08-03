use crate::utils::{TestContext, assertions::*, mocks::*};
use std::fs;

#[test]
fn test_add_creates_memo_file() {
    let context = TestContext::new();
    let output = context.run_command(&["add"]);

    assert_command_success(&output);
    assert_output_contains(&output, "Memo created:");

    // check if a memo file was created
    let memo_dir = context.memo_dir();
    let entries: Vec<_> = fs::read_dir(memo_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .collect();

    assert!(!entries.is_empty(), "No memo files were created");

    // respects_xdg_data_home
    let memo_dir = context.memo_dir();
    assert!(memo_dir.starts_with(context.temp_dir.path()));
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

#[cfg(test)]
mod add_integration_tests {
    use super::*;

    #[test]
    fn test_add_then_show_workflow() {
        let script_path = create_mock_editor_script("Content for workflow test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);

        // extract the memo ID from the output
        let stdout = String::from_utf8_lossy(&add_output.stdout);
        let id_line = stdout
            .lines()
            .find(|line| line.contains("Memo created:"))
            .expect("Could not find created memo ID");

        let id = id_line
            .split("Memo created: ")
            .nth(1)
            .expect("Could not extract memo ID");

        let show_output = context.run_command(&["show", id]);
        assert_command_success(&show_output);
        assert_output_contains(&show_output, "Content for workflow test");
    }

    #[test]
    fn test_add_then_list_workflow() {
        let script_path = create_mock_editor_script("Content for list test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);

        // retrieve the memo ID from the output
        let list_output = context.run_command(&["list"]);
        assert_command_success(&list_output);
        assert_output_contains(&list_output, "Content for list test");
    }

    #[test]
    fn test_add_then_search_workflow() {
        let script_path = create_mock_editor_script("Content for search test");
        let context = TestContext::with_editor(script_path.to_str().unwrap());

        let add_output = context.run_command(&["add"]);
        assert_command_success(&add_output);

        // search for the content
        let search_output = context.run_command(&["search", "search test"]);
        assert_command_success(&search_output);
        assert_output_contains(&search_output, "Content for search test");
    }
}
