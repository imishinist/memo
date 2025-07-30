use crate::context::Context;
use crate::error::MemoError;
use crate::search::SearchManager;

pub fn run_search(ctx: &Context, query: &str) -> Result<(), MemoError> {
    let search_manager = SearchManager::new(ctx.data_dir.clone());
    
    // 検索実行
    let results = search_manager.search(query)?;
    
    if results.is_empty() {
        println!("No results found for query: {}", query);
        return Ok(());
    }
    
    println!("Found {} results for query: {}", results.len(), query);
    println!();
    
    for (i, result) in results.iter().enumerate() {
        let memo = &result.memo;
        
        // ファイル名から ID を抽出
        let filename = std::path::Path::new(&memo.path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        
        // 日付フォーマット
        let date_str = memo.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
        
        // タイトルを取得（frontmatterから）
        let title = memo.frontmatter
            .as_ref()
            .and_then(|fm| fm.get("title"))
            .and_then(|v| v.as_str())
            .unwrap_or("(No title)");
        
        // スコア表示
        println!("{}. [{}] {} (score: {:.2})", 
                 i + 1, filename, title, result.score);
        println!("   Date: {}", date_str);
        
        // タグがあれば表示
        if let Some(frontmatter) = &memo.frontmatter {
            if let Some(tags) = frontmatter.get("tags").and_then(|v| v.as_array()) {
                let tag_strs: Vec<String> = tags.iter()
                    .filter_map(|t| t.as_str())
                    .map(|s| format!("@{}", s))
                    .collect();
                if !tag_strs.is_empty() {
                    println!("   Tags: {}", tag_strs.join(" "));
                }
            }
        }
        
        // 内容のプレビュー（最初の100文字）
        let preview = if memo.content.len() > 100 {
            format!("{}...", &memo.content[..100])
        } else {
            memo.content.clone()
        };
        
        // 改行を空白に置換してプレビュー表示
        let preview = preview.replace('\n', " ").replace('\r', "");
        println!("   Preview: {}", preview);
        println!();
    }
    
    Ok(())
}
