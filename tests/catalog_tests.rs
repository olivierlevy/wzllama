use wzllama::tools::catalog::ToolCatalog;
use wzllama::tools::get_all_tools;

#[test]
fn test_catalog_loads_without_panic() {
    let catalog = ToolCatalog::load();
    assert!(!catalog.tools.is_empty(), "Catalog must have at least one tool");
}

#[test]
fn test_catalog_has_new_tools() {
    let catalog = ToolCatalog::load();
    let has_cline = catalog.tools.iter().any(|t| t.id == "cline");
    assert!(has_cline, "Catalog must contain cline");
}

#[test]
fn test_get_all_tools_contains_catalog_tools() {
    let tools = get_all_tools();
    let has_cline = tools.iter().any(|t| t.id() == "cline");
    assert!(has_cline, "get_all_tools() must include catalog tool 'cline'");
}

#[test]
fn test_no_duplicate_tool_ids() {
    let tools = get_all_tools();
    let mut ids = std::collections::HashSet::new();
    for tool in &tools {
        assert!(
            ids.insert(tool.id().to_string()),
            "Duplicate tool id: {}",
            tool.id()
        );
    }
}

#[test]
fn test_static_tools_not_overridden_by_catalog() {
    let tools = get_all_tools();
    // Claude Code is a static tool AND in catalog; should appear exactly once
    let count = tools.iter().filter(|t| t.id() == "claude_code").count();
    assert_eq!(count, 1, "claude_code must appear exactly once (static priority)");
}

#[test]
fn test_tool_updater_update_needed_when_no_timestamp() {
    let home = dirs::home_dir().unwrap_or_default();
    let ts_file = home.join(".wzllama").join("last_update.txt");
    let _ = std::fs::remove_file(&ts_file);
    assert!(wzllama::core::tool_updater::ToolUpdater::is_update_needed());
}
