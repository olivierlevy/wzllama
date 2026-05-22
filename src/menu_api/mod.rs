//! Menu API - Dynamic menu tree management system
//!
//! This module provides a flexible, data-driven menu system that separates
//! menu structure from business logic.

mod menu_tree;
mod menu_item;
mod menu_handler;
mod tool_action;
mod config_loader;
mod wizard_adapter;
mod wizard_integration;
mod wizard_engine;
mod wizard_actions;
mod wizard_menu_handler;
mod main_menu_adapter;
mod models_menu_adapter;
mod models_engine;
mod tools_menu_adapter;
mod tools_engine;
mod cleanup_menu_adapter;
mod wizard_menu_adapter;
mod scientific_menu_adapter;
mod config_menu_adapter;
mod dynamic_builder;
mod dynamic_generators;
mod api_first;
mod wizard_helpers;
mod arc_action;
pub mod api_service;
#[cfg(test)]
mod tests;

pub use menu_tree::MenuTree;
pub use menu_tree::MenuMetadata;
pub use menu_tree::MenuConfig;
pub use menu_item::MenuItem;
pub use menu_handler::MenuHandler;
pub use tool_action::{ToolAction, ActionDispatcher, ActionContext, ActionResult, ClosureAction};
pub use arc_action::{ArcAction, ArcActionRunner};
pub use dynamic_builder::{
    build_main_menu, 
    build_wizard_menu, 
    build_models_menu,
};
pub use api_first::{get_menu_structure, get_tools_menu, get_models_menu};
pub use wizard_helpers::{
    UseCase, ScientificCategory, AgenticToolInfo, LanguageInfo,
    get_priority_tools_for_usecase, is_skill_installed, get_install_cmd,
    sync_tools_state, cleanup_is_installed, mark_uninstalled,
    is_cache_from_today, enter_alternate_screen, exit_alternate_screen,
    MenuIndices, get_resume_label, ollama_to_localmax_model,
    get_default_language_index, get_language_items,
};
pub use wizard_actions::{
    WizardContext, create_wizard_context, build_cleanup_menu_tree,
};
pub use wizard_menu_adapter::build_menu_tree;
pub use main_menu_adapter::MainMenuRunner;
pub use models_menu_adapter::ModelsMenuRunner;
pub use tools_menu_adapter::ToolsMenuRunner;
pub use cleanup_menu_adapter::CleanupMenuRunner;
pub use wizard_engine::WizardEngineRunner;
pub use models_engine::ModelsEngineRunner;
pub use tools_engine::ToolsEngineRunner;
pub use scientific_menu_adapter::ScientificMenuRunner;
pub use config_menu_adapter::ConfigMenuRunner;
pub use wizard_menu_handler::WizardMenuRunner;

// Re-export api_service types
pub use api_service::{ApiService, ActionResponse, ToolInfo, HardwareInfo, SystemStatus, GpuInfo};
pub use dynamic_generators::{
    generate_models_menu, generate_tools_menu, generate_usecase_menu, generate_scientific_menu,
};