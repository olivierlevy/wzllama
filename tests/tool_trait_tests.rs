use wzllama::tools::tool_trait::{Tool, ToolStatus};
use wzllama::config::i18n::I18n;
use anyhow::Result;

// Mock tool for testing
struct MockTool {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    installed: bool,
}

impl Tool for MockTool {
    fn id(&self) -> &str {
        self.id
    }
    
    fn name(&self) -> &str {
        self.name
    }
    
    fn description(&self, _i18n: &I18n) -> String {
        self.description.to_string()
    }
    
    fn status(&self) -> ToolStatus {
        if self.installed {
            ToolStatus::Installed
        } else {
            ToolStatus::NotInstalled
        }
    }
    
    fn install(&self, _i18n: &I18n) -> Result<()> {
        if self.installed {
            anyhow::bail!("Already installed");
        }
        Ok(())
    }
    
    fn launch(&self, _i18n: &I18n, _state: &wzllama::config::WzllamaState, _model: Option<&str>) -> Result<()> {
        Ok(())
    }
    
    fn supports_fleets(&self) -> bool {
        true
    }
    
    fn requires_docker(&self) -> bool {
        self.id == "open_webui"
    }
}

#[test]
fn test_tool_status_installed() {
    let tool = MockTool {
        id: "test_tool",
        name: "Test Tool",
        description: "A test tool",
        installed: true,
    };
    
    assert_eq!(tool.status(), ToolStatus::Installed);
}

#[test]
fn test_tool_status_not_installed() {
    let tool = MockTool {
        id: "test_tool",
        name: "Test Tool",
        description: "A test tool",
        installed: false,
    };
    
    assert_eq!(tool.status(), ToolStatus::NotInstalled);
}

#[test]
fn test_tool_id() {
    let tool = MockTool {
        id: "my_unique_id",
        name: "Test",
        description: "Desc",
        installed: false,
    };
    
    assert_eq!(tool.id(), "my_unique_id");
}

#[test]
fn test_tool_name() {
    let tool = MockTool {
        id: "id",
        name: "My Tool Name",
        description: "Desc",
        installed: false,
    };
    
    assert_eq!(tool.name(), "My Tool Name");
}

#[test]
fn test_tool_description() {
    let i18n = I18n::default();
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Test description",
        installed: false,
    };
    
    assert_eq!(tool.description(&i18n), "Test description");
}

#[test]
fn test_tool_status_message_installed() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: true,
    };
    
    let msg = tool.status_message(&I18n::default());
    assert!(!msg.is_empty());
}

#[test]
fn test_tool_status_message_not_installed() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: false,
    };
    
    let msg = tool.status_message(&I18n::default());
    assert!(!msg.is_empty());
}

#[test]
fn test_tool_requires_docker() {
    let docker_tool = MockTool {
        id: "open_webui",
        name: "Open WebUI",
        description: "Web UI",
        installed: false,
    };
    
    let other_tool = MockTool {
        id: "other",
        name: "Other",
        description: "Other",
        installed: false,
    };
    
    assert!(docker_tool.requires_docker());
    assert!(!other_tool.requires_docker());
}

#[test]
fn test_tool_supports_fleets() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: false,
    };
    
    assert!(tool.supports_fleets());
}

#[test]
fn test_tool_install_already_installed() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: true,
    };
    
    let result = tool.install(&I18n::default());
    assert!(result.is_err());
}

#[test]
fn test_tool_install_not_installed() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: false,
    };
    
    let result = tool.install(&I18n::default());
    assert!(result.is_ok());
}

#[test]
fn test_tool_update_default() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: false,
    };
    
    // Par défaut, update doit échouer
    let result = tool.update(&I18n::default());
    assert!(result.is_err());
}

#[test]
fn test_tool_uninstall_default() {
    let tool = MockTool {
        id: "id",
        name: "Name",
        description: "Desc",
        installed: false,
    };
    
    // Par défaut, uninstall doit échouer
    let result = tool.uninstall(&I18n::default());
    assert!(result.is_err());
}