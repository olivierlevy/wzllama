use crate::config::I18n;
use crate::display;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

const SOURCE_LINE: &str = "source ~/.wzllama/env  # wzllama";

pub struct ShellConfig {
    pub name: String,
    pub path: PathBuf,
}

impl ShellConfig {
    fn all() -> Vec<Self> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        vec![
            Self {
                name: "bash".into(),
                path: home.join(".bashrc"),
            },
            Self {
                name: "zsh".into(),
                path: home.join(".zshrc"),
            },
            Self {
                name: "fish".into(),
                path: home.join(".config/fish/config.fish"),
            },
        ]
    }

    fn exists(&self) -> bool {
        self.path.exists()
    }

    fn has_source(&self) -> bool {
        if !self.exists() {
            return false;
        }
        fs::read_to_string(&self.path)
            .map(|c| c.lines().any(|l| l.trim() == SOURCE_LINE))
            .unwrap_or(false)
    }

    fn install(&self) -> Result<()> {
        if self.has_source() {
            return Ok(());
        }
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut content = if self.path.exists() {
            fs::read_to_string(&self.path)?
        } else {
            String::new()
        };
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push_str(SOURCE_LINE);
        content.push('\n');
        fs::write(&self.path, content)?;
        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        if !self.exists() || !self.has_source() {
            return Ok(());
        }
        let content = fs::read_to_string(&self.path)?;
        let new_content: String = content
            .lines()
            .filter(|l| l.trim() != SOURCE_LINE)
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&self.path, new_content)?;
        Ok(())
    }
}

pub fn install_all_shells(i18n: &I18n) -> Result<()> {
    for shell in ShellConfig::all() {
        if shell.exists() {
            shell.install()?;
            display::success(
                &i18n.t_with_vars("config.shell.installed", &[("shell", &shell.name)]),
            );
        }
    }
    Ok(())
}

pub fn uninstall_all_shells(i18n: &I18n) -> Result<()> {
    for shell in ShellConfig::all() {
        if shell.has_source() {
            shell.uninstall()?;
            display::success(
                &i18n.t_with_vars("config.shell.uninstalled", &[("shell", &shell.name)]),
            );
        }
    }
    Ok(())
}

pub fn uninstall_all_shells_cli() -> Result<()> {
    for shell in ShellConfig::all() {
        if shell.has_source() {
            shell.uninstall()?;
            println!("   ✅ {} cleaned", shell.name);
        }
    }
    Ok(())
}

pub fn get_shells_status(_i18n: &I18n) -> Vec<String> {
    ShellConfig::all()
        .iter()
        .map(|s| {
            let status = if s.has_source() { "✅" } else { "❌" };
            format!("{} {}", status, s.name)
        })
        .collect()
}

pub fn install_all_shells_cli() -> Result<()> {
    for shell in ShellConfig::all() {
        if shell.exists() {
            shell.install()?;
            println!("   ✅ {} configured", shell.name);
        }
    }
    Ok(())
}
