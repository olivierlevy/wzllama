# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-05-20

### Changed
- Major code cleanup: removed dead code and unused modules
- Removed estimator, configurator, fleet_creator, and fleet_templates modules
- Removed unused estimation module
- Cleaned up #[allow(dead_code)] attributes across the codebase

### Removed
- `estimator.rs` - unused estimation functionality
- `configurator.rs` - unused configuration wizard
- `fleet_creator.rs` and `fleet_templates.rs` - unused fleet creation
- `core/estimation.rs` module and tests
- `npm_package()` function - never called
- `startup()` docker function - never used
- `get_install_command()` and `get_launch_command()` - never used
- `supports_fleets` and `supports_agentic` fields from ToolInfo (replaced by trait methods)

## [0.2.0] - 2025-05-17

### Added
- Model selection menu restructured: installed models shown first
- Organization-based submenu for browsing models
- Hardware compatibility indicator for model recommendations
- Cache system in `~/.wzllama/cache/` for API data (7-day validity)
- Multi-language support (English/French) for i18n

### Changed
- Improved model sorting to show most popular first
- Better handling of null API fields in localmax models

### Fixed
- JSON parsing for null fields in LocalMaxModel (family, activeParams, pipelineTag)
- Cache atomic update to prevent data loss on network failure

## [0.1.0] - 2025-05-13

### Added
- Initial release
- Wizard CLI for LLM stack setup and management
- Hardware detection (RAM, GPU, disk space)
- Model recommendation system
- TUI interface with ratatui
- Integration with Ollama for local model management
- Support for multiple tools (Open WebUI, Claude Code, OpenCode, etc.)