//! Menu API - Dynamic menu tree management system
//!
//! This module provides a flexible, data-driven menu system that separates
//! menu structure from business logic.

mod api_first;
pub mod api_service;
mod arc_action;
mod cleanup_menu_adapter;
mod config_loader;
mod config_menu_adapter;
mod dynamic_builder;
mod dynamic_generators;
mod main_menu_adapter;
mod menu_handler;
mod menu_item;
mod menu_tree;
mod models_engine;
mod models_menu_adapter;
mod scientific_menu_adapter;
#[cfg(test)]
mod tests;
mod tool_action;
mod tools_engine;
mod tools_menu_adapter;
mod wizard_actions;
mod wizard_adapter;
mod wizard_engine;
mod wizard_helpers;
mod wizard_integration;
mod wizard_menu_adapter;
mod wizard_menu_handler;

pub use api_first::{get_menu_structure, get_models_menu, get_tools_menu};
pub use arc_action::{ArcAction, ArcActionRunner};
pub use cleanup_menu_adapter::CleanupMenuRunner;
pub use config_menu_adapter::ConfigMenuRunner;
pub use dynamic_builder::{build_main_menu, build_models_menu, build_wizard_menu};
pub use main_menu_adapter::MainMenuRunner;
pub use menu_handler::MenuHandler;
pub use menu_item::MenuItem;
pub use menu_tree::MenuConfig;
pub use menu_tree::MenuMetadata;
pub use menu_tree::MenuTree;
pub use models_engine::ModelsEngineRunner;
pub use models_menu_adapter::ModelsMenuRunner;
pub use scientific_menu_adapter::ScientificMenuRunner;
pub use tool_action::{ActionContext, ActionDispatcher, ActionResult, ClosureAction, ToolAction};
pub use tools_engine::ToolsEngineRunner;
pub use tools_menu_adapter::ToolsMenuRunner;
pub use wizard_actions::{build_cleanup_menu_tree, create_wizard_context, WizardContext};
pub use wizard_engine::WizardEngineRunner;
pub use wizard_helpers::{
    cleanup_is_installed, enter_alternate_screen, exit_alternate_screen,
    get_default_language_index, get_install_cmd, get_language_items,
    get_priority_tools_for_usecase, get_resume_label, is_cache_from_today, is_skill_installed,
    mark_uninstalled, ollama_to_localmax_model, sync_tools_state, AgenticToolInfo, LanguageInfo,
    MenuIndices, ScientificCategory, UseCase,
};
pub use wizard_menu_adapter::build_menu_tree;
pub use wizard_menu_handler::WizardMenuRunner;

// Re-export api_service types
pub use api_service::{ActionResponse, ApiService, GpuInfo, HardwareInfo, SystemStatus, ToolInfo};
pub use dynamic_generators::{
    generate_models_menu, generate_scientific_menu, generate_tools_menu, generate_usecase_menu,
};
