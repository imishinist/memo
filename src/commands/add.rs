use crate::context::MemoContext;
use crate::error::MemoResult;
use crate::memo::{MemoDocument, MemoFile};
use crate::memo_id::MemoId;
use crate::repository::MemoRepository;
use crate::search::SearchManager;
use crate::utils::editor;

fn template_content(context: &MemoContext, template: &str) -> String {
    // .templates/{name}.md があればそれを使う
    let template_path = context.templates_dir().join(format!("{}.md", template));
    if template_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&template_path) {
            return content;
        }
    }
    // ビルトインにフォールバック
    builtin_template(template)
}

pub fn builtin_template(template: &str) -> String {
    match template {
        "1on1" => "---\ntitle: \"\"\ntags: ['@1on1']\n---\n\n## 話したこと\n\n\n## ネクストアクション\n\n".to_string(),
        "idea" => "---\ntitle: \"\"\ntags: ['@idea']\n---\n\n## アイデア\n\n\n## 背景・動機\n\n".to_string(),
        "todo" => "---\ntitle: \"\"\ntags: ['@todo']\n---\n\n## やること\n\n- [ ] \n".to_string(),
        "meeting" => "---\ntitle: \"\"\ntags: ['@meeting']\n---\n\n## 参加者\n\n\n## 議題\n\n\n## 決定事項\n\n\n## ネクストアクション\n\n".to_string(),
        other => format!("---\ntitle: \"\"\ntags: ['@{}']\n---\n\n", other),
    }
}

pub fn run(context: &MemoContext, template: Option<&str>) -> MemoResult<()> {
    let memo_id = MemoId::new();
    let relative_path = memo_id.to_relative_path();

    let initial_content = template.map_or(String::new(), |t| template_content(context, t));

    let repo = MemoRepository::new(context.clone());
    let memo = repo.create_memo(&relative_path, initial_content)?;

    editor::open_editor(context, &memo.path)?;
    update_search_index(context, &memo.path)?;

    println!("Memo created: {}", memo_id);
    Ok(())
}

fn update_search_index(context: &MemoContext, memo_path: &std::path::Path) -> MemoResult<()> {
    let data_dir = context.memo_dir.clone();
    let index_dir = context.index_dir();
    let search_manager = SearchManager::new(data_dir, index_dir);

    if let Ok(memo_file) = MemoFile::from_path(memo_path) {
        let memo_doc = MemoDocument::from_memo_file(&memo_file);
        let _ = search_manager.add_memo(&memo_doc)?;
    }

    Ok(())
}
