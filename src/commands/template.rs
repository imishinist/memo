use crate::commands::add::builtin_template;
use crate::context::MemoContext;
use crate::error::MemoResult;
use crate::utils::editor;
use std::collections::BTreeSet;
use std::fs;

const BUILTIN_TEMPLATES: &[&str] = &["1on1", "idea", "meeting", "todo"];

pub fn run_add(context: &MemoContext, name: &str) -> MemoResult<()> {
    let templates_dir = context.templates_dir();
    fs::create_dir_all(&templates_dir)?;

    let path = templates_dir.join(format!("{}.md", name));
    if path.exists() {
        eprintln!("Template '{}' already exists. Use `memo template edit {}` to modify.", name, name);
        return Ok(());
    }

    fs::write(&path, builtin_template(name))?;
    editor::open_editor(context, &path)?;
    Ok(())
}

pub fn run_edit(context: &MemoContext, name: &str) -> MemoResult<()> {
    let templates_dir = context.templates_dir();
    fs::create_dir_all(&templates_dir)?;

    let path = templates_dir.join(format!("{}.md", name));
    if !path.exists() {
        fs::write(&path, builtin_template(name))?;
    }

    editor::open_editor(context, &path)?;
    Ok(())
}

pub fn run_list(context: &MemoContext) -> MemoResult<()> {
    let mut names = BTreeSet::new();
    for &name in BUILTIN_TEMPLATES {
        names.insert(name.to_string());
    }

    let templates_dir = context.templates_dir();
    if templates_dir.exists() {
        if let Ok(entries) = fs::read_dir(&templates_dir) {
            for entry in entries.flatten() {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    names.insert(stem.to_string());
                }
            }
        }
    }

    for name in &names {
        let custom_path = templates_dir.join(format!("{}.md", name));
        let source = if custom_path.exists() {
            "custom"
        } else {
            "builtin"
        };
        println!("  {}  ({})", name, source);
    }
    Ok(())
}
