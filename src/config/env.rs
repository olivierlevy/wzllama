use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::config::paths;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnvConfig {
    pub ollama: OllamaEnv,
    pub providers: ProvidersEnv,
    pub openclaw: OpenClawEnv,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaEnv {
    #[serde(default = "default_ollama_host")]
    pub host: String,
    #[serde(default = "default_ollama_origins")]
    pub origins: String,
    #[serde(default = "default_keep_alive")]
    pub keep_alive: i32,
    #[serde(default = "default_true")]
    pub no_cloud: bool,
    #[serde(default = "default_num_parallel")]
    pub num_parallel: u32,
    #[serde(default = "default_max_loaded")]
    pub max_loaded_models: u32,
    #[serde(default = "default_true")]
    pub flash_attention: bool,
    #[serde(default = "default_kv_cache")]
    pub kv_cache_type: String,
    #[serde(default = "default_context")]
    pub context_length: u32,
    // GPU/VRAM settings
    #[serde(default = "default_max_vram")]
    pub max_vram: u64,
    #[serde(default = "default_cuda_device")]
    pub cuda_visible_devices: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProvidersEnv {
    pub openai: ProviderEnv,
    pub anthropic: ProviderEnv,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProviderEnv {
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenClawEnv {
    pub api_key: String,
}

fn default_ollama_host() -> String { "127.0.0.1:11434".into() }
fn default_ollama_origins() -> String { "http://localhost:*".into() }
fn default_keep_alive() -> i32 { -1 }
fn default_true() -> bool { true }
fn default_num_parallel() -> u32 { 4 }
fn default_max_loaded() -> u32 { 3 }
fn default_kv_cache() -> String { "q8_0".into() }
fn default_context() -> u32 { 16384 }
fn default_max_vram() -> u64 { 0 }  // 0 = auto
fn default_cuda_device() -> String { "".into() }  // empty = all GPUs

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            ollama: OllamaEnv {
                host: default_ollama_host(),
                origins: default_ollama_origins(),
                keep_alive: default_keep_alive(),
                no_cloud: true,
                num_parallel: default_num_parallel(),
                max_loaded_models: default_max_loaded(),
                flash_attention: true,
                kv_cache_type: default_kv_cache(),
                context_length: default_context(),
                max_vram: default_max_vram(),
                cuda_visible_devices: default_cuda_device(),
            },
            providers: ProvidersEnv {
                openai: ProviderEnv {
                    api_key: "ollama".into(),
                    base_url: "http://localhost:11434/v1".into(),
                },
                anthropic: ProviderEnv {
                    api_key: "ollama".into(),
                    base_url: "http://localhost:11434/v1".into(),
                },
            },
            openclaw: OpenClawEnv {
                api_key: "ollama-local".into(),
            },
        }
    }
}

impl EnvConfig {
    pub fn config_path() -> std::path::PathBuf {
        paths::config_dir().join("config.yaml")
    }

    pub fn env_path() -> std::path::PathBuf {
        paths::wzllama_dir().join("env")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_yaml::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            let config = EnvConfig::default();
            let _ = config.save();
            config
        }
    }

    pub fn save(&self) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        fs::write(Self::config_path(), yaml)?;
        self.generate_env_file()?;
        Ok(())
    }

    pub fn generate_env_file(&self) -> Result<()> {
        let mut content = String::new();
        content.push_str("# wzllama - Environnement IA 100% locale\n");
        content.push_str(&format!("# Généré le {}\n\n", chrono::Local::now().format("%Y-%m-%d")));
        
        content.push_str("# ═══ Ollama ═══════════════════════════════\n");
        
        content.push_str(&format!("export OLLAMA_HOST='{}'\n", self.ollama.host));
        content.push_str(&format!("export OLLAMA_ORIGINS='{}'\n", self.ollama.origins));
        content.push_str(&format!("export OLLAMA_KEEP_ALIVE={}\n", self.ollama.keep_alive));
        content.push_str(&format!("export OLLAMA_NO_CLOUD={}\n", if self.ollama.no_cloud { 1 } else { 0 }));
        content.push_str(&format!("export OLLAMA_NUM_PARALLEL={}\n", self.ollama.num_parallel));
        content.push_str(&format!("export OLLAMA_MAX_LOADED_MODELS={}\n", self.ollama.max_loaded_models));
        content.push_str(&format!("export OLLAMA_FLASH_ATTENTION={}\n", if self.ollama.flash_attention { 1 } else { 0 }));
        content.push_str(&format!("export OLLAMA_KV_CACHE_TYPE={}\n", self.ollama.kv_cache_type));
        content.push_str(&format!("export OLLAMA_CONTEXT_LENGTH={}\n", self.ollama.context_length));
        // GPU/VRAM settings
        if self.ollama.max_vram > 0 {
            content.push_str(&format!("export OLLAMA_MAX_VRAM={}\n", self.ollama.max_vram));
        }
        if !self.ollama.cuda_visible_devices.is_empty() {
            content.push_str(&format!("export CUDA_VISIBLE_DEVICES='{}'\n", self.ollama.cuda_visible_devices));
        }
        content.push('\n');
        
        content.push_str("# ═══ Providers ════════════════════════════\n");
        content.push_str(&format!("export OPENAI_API_KEY='{}'\n", self.providers.openai.api_key));
        content.push_str(&format!("export OPENAI_BASE_URL='{}'\n", self.providers.openai.base_url));
        content.push_str(&format!("export ANTHROPIC_API_KEY='{}'\n", self.providers.anthropic.api_key));
        content.push_str(&format!("export ANTHROPIC_BASE_URL='{}'\n\n", self.providers.anthropic.base_url));
        
        content.push_str("# ═══ OpenClaw ═════════════════════════════\n");
        content.push_str(&format!("export OLLAMA_API_KEY='{}'\n\n", self.openclaw.api_key));
        
        content.push_str("# ═══ wzllama ══════════════════════════════\n");
        content.push_str(&format!("export WZLLAMA_HOME='{}'\n", paths::wzllama_dir().display()));
        // Ajouter la langue si disponible dans le state
        let state = crate::config::WzllamaState::load();
        if let Some(ref lang) = state.language {
            content.push_str(&format!("export WZLLAMA_LANG='{}'\n", lang));
        }
        
        fs::write(Self::env_path(), content)?;
        Ok(())
    }
}