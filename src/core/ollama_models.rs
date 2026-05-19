#![allow(dead_code)]

use crate::core::hardware::HardwareInfo;
use crate::core::ollama_api::OllamaModel;
use crate::config::I18n;

#[derive(Debug, Clone)]
pub enum TaskType {
    #[allow(dead_code)]
    QuickChat, BookWriting, LargeCodeProject,
    MultiAgent { agent_count: u8 }, #[allow(dead_code)] Rag, Mixed,
}

impl TaskType {
    pub fn parse_from_str(s: &str) -> Self {
        match s {
            "agents" => TaskType::MultiAgent { agent_count: 4 },
            "book" => TaskType::BookWriting,
            "code" => TaskType::LargeCodeProject,
            _ => TaskType::Mixed,
        }
    }
    pub fn to_str(&self) -> &str {
        match self {
            TaskType::QuickChat => "chat", TaskType::BookWriting => "book",
            TaskType::LargeCodeProject => "code", TaskType::MultiAgent { .. } => "agents",
            TaskType::Rag => "rag", TaskType::Mixed => "mixed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub model_name: String,
    pub num_ctx: u32,
    pub kv_cache_type: String,
    pub flash_attention: bool,
    pub temperature: f32,
    pub system_prompt: Option<String>,
}

impl ModelConfig {
    pub fn generate_modelfile(&self) -> String {
        let mut mf = format!("FROM {}\nPARAMETER num_ctx {}\nPARAMETER temperature {:.1}\n",
            self.model_name, self.num_ctx, self.temperature);
        if let Some(ref p) = self.system_prompt {
            mf.push_str(&format!("SYSTEM \"{}\"\n", p));
        }
        mf
    }
    pub fn env_vars(&self) -> Vec<(String, String)> {
        let mut v = vec![
            ("OLLAMA_KV_CACHE_TYPE".into(), self.kv_cache_type.clone()),
            ("OLLAMA_CONTEXT_LENGTH".into(), self.num_ctx.to_string()),
        ];
        if self.flash_attention { v.push(("OLLAMA_FLASH_ATTENTION".into(), "1".into())); }
        v
    }
    pub fn env_vars_display(&self) -> String {
        self.env_vars().iter().map(|(k, v)| format!("export {}={}", k, v)).collect::<Vec<_>>().join("\n")
    }
    pub fn write_modelfile(&self, name: &str) -> anyhow::Result<String> {
        let tmp = format!("/tmp/wzllama_{}.Modelfile", name);
        std::fs::write(&tmp, self.generate_modelfile())?;
        Ok(format!("ollama create {} -f {}", name, tmp))
    }
}

pub struct FleetCapacity {
    pub max_experts_ram: u32,
    #[allow(dead_code)]
    pub max_experts_vram: u32,
    #[allow(dead_code)]
    pub max_reflexion: u32,
    pub ram_total_gb: f64,
    pub vram_total_gb: f64,
}

/// Extract approximate parameter size from model name (in billions)
/// Returns 0 if cannot determine - caller should handle this case
pub fn extract_size(name: &str) -> u32 {
    let lower = name.to_lowercase();
    
    // Known model families with their typical parameter counts
    let family_sizes: &[(&str, u32)] = &[
        ("qwen3.6", 35),
        ("qwen3", 8),  // qwen3:latest is typically 8B
        ("qwen2.5", 7),
        ("qwen2", 7),
        ("llama3.3", 8),
        ("llama3.2", 3),
        ("llama3.1", 8),
        ("llama3", 8),
        ("codellama", 7),
        ("mistral", 7),
        ("mistral-nemo", 12),
        ("deepseek", 7),
        ("devstral", 24),
    ];
    
    // Check for known family prefixes first
    for (family, size) in family_sizes {
        if lower.starts_with(family) || lower.starts_with(&format!("{}-", family)) {
            return *size;
        }
    }
    
    // Standard patterns like "3b", "7b", "70b"
    for part in name.split([':', '-', '/', '_']) {
        if let Some(size) = part.strip_suffix('b') {
            if let Ok(n) = size.parse::<f32>() {
                return (n * 10.0).round() as u32 / 10;
            }
            if let Ok(n) = size.parse::<u32>() { return n; }
        }
    }
    
    // Try to extract from model name patterns like "qwen3-30b" or "gpt-oss-120b"
    for part in name.split(['-', ':', '/', '_']) {
        let lower_part = part.to_lowercase();
        if lower_part.ends_with("b") && lower_part.len() > 1 {
            let num_part = &lower_part[..lower_part.len()-1];
            if let Ok(n) = num_part.parse::<f32>() {
                return (n * 10.0).round() as u32 / 10;
            }
        }
    }
    0
}

pub fn calculate_fleet_capacity(hw: &HardwareInfo, orchestrator: &OllamaModel) -> FleetCapacity {
    let vram_gb = hw.total_vram_mb as f64 / 1024.0;
    let orch_vram = orchestrator.size.unwrap_or(0) as f64 / 1_073_741_824.0;
    FleetCapacity {
        max_experts_ram: ((hw.ram_gb * 0.4) / 2.0) as u32,
        max_experts_vram: ((vram_gb - orch_vram).max(0.0) / 1.0) as u32,
        max_reflexion: ((vram_gb - orch_vram).max(0.0) / 4.0) as u32,
        ram_total_gb: hw.ram_gb,
        vram_total_gb: vram_gb,
    }
}

pub fn score_model(model: &OllamaModel, usage: &str, hw: &HardwareInfo) -> f32 {
    let name = model.name.to_lowercase();
    let size = extract_size(&model.name);
    
    // Cloud models are handled separately - return low score here
    if name.contains("cloud") || name.contains("remote") { return 0.0; }
    
    // Check disk space - model needs at least its size + 20% buffer
    let model_gb = model.size.unwrap_or(0) as f64 / 1_073_741_824.0;
    let has_disk_space = hw.available_disk_gb >= model_gb * 1.2;
    
    let has_gpu = hw.has_gpu();
    let vram_gb = hw.total_vram_mb as f64 / 1024.0;
    let mut score: f32 = 0.2;
    
    // VRAM/RAM fit check
    // For quantized models (Q4_K_M, etc), memory needed ≈ model file size
    // For full precision (FP16), memory needed ≈ 2x model file size
    // We use model file size as the primary metric since that's what's actually loaded
    let needs_mem = if model_gb > 0.0 {
        // Use actual file size - this is what will be loaded into memory
        model_gb
    } else if size > 0 {
        // Fallback: estimate from parameter count (Q4_K_M ≈ 0.5 GB per B params)
        size as f64 * 0.6
    } else {
        0.0
    };
    
    let (fits_memory, uses_ram) = if has_gpu {
        if needs_mem <= vram_gb {
            (true, false)  // Fits in VRAM - optimal
        } else if needs_mem <= hw.ram_gb {
            (true, true)   // Fits in RAM but not VRAM - slower but possible
        } else {
            (false, false) // Doesn't fit anywhere
        }
    } else {
        (needs_mem <= hw.ram_gb, false)
    };
    
    let size_score = if fits_memory {
        if uses_ram {
            0.15  // Fits but only in RAM - slower performance warning
        } else if needs_mem <= vram_gb * 0.3 { 
            0.4   // Small model - excellent fit
        } else if needs_mem <= vram_gb * 0.5 { 
            0.25  // Medium model - good fit
        } else { 
            0.1   // Large model - tight fit
        }
    } else { -1.0 };
    score += size_score;
    
    // Disk space check
    if !has_disk_space {
        score -= 0.5;
    }
    
    if score < 0.0 { return 0.0; }

    match usage {
        "agents" if size <= 7 => score += 0.2,
        "book" | "code" if size >= 32 => score += 0.3,
        "book" | "code" if size >= 14 => score += 0.2,
        _ => {}
    }

    let family = model.details.as_ref().and_then(|d| d.family.as_deref()).unwrap_or("");
    let keywords: &[&str] = match usage {
        "code" => &["code", "coder", "dev"],
        "book" => &["writer", "story", "large"],
        "agents" => &["small", "fast", "light"],
        _ => &[],
    };
    for kw in keywords { if name.contains(kw) || family.contains(kw) { score += 0.1; } }

    score.clamp(0.0, 1.0)
}

/// Check if a model is a cloud model
pub fn is_cloud_model(model: &OllamaModel) -> bool {
    let name = model.name.to_lowercase();
    name.contains("cloud") || name.contains("remote")
}

pub fn rank_models(models: &[OllamaModel], usage: &str, hw: &HardwareInfo, limit: usize) -> Vec<(OllamaModel, f32)> {
    let mut scored: Vec<_> = models.iter()
        .filter(|m| { let s = extract_size(&m.name); s > 0 && !m.name.to_lowercase().contains("cloud") })
        .map(|m| { let s = score_model(m, usage, hw); (m.clone(), s) })
        .filter(|(_, s)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored
}

pub fn recommend_config(hw: &HardwareInfo, task: &TaskType, model: &OllamaModel, i18n: &I18n) -> ModelConfig {
    let vram_gb = hw.total_vram_mb as f64 / 1024.0;
    let model_gb = model.size.unwrap_or(0) as f64 / 1_073_741_824.0;
    let size_b = extract_size(&model.name);
    let ctx_per_gb = 8192;
    let available = (vram_gb - model_gb).max(0.5) as u32;
    let mut max_ctx = available * ctx_per_gb;

    max_ctx = match size_b {
        0..=3 => max_ctx.min(16384), 4..=7 => max_ctx.min(32768),
        8..=14 => max_ctx.min(49152), _ => max_ctx.min(65536),
    };

    let (num_ctx, kv, flash, temp, prompt): (u32, String, bool, f32, Option<String>) = match task {
        TaskType::QuickChat => (max_ctx.clamp(2048, 8192), "f16".into(), false, 0.8, Some(i18n.t("config.prompt.quick_chat"))),
        TaskType::BookWriting => (max_ctx.clamp(8192, 65536), "q8_0".into(), true, 0.7, Some(i18n.t("config.prompt.book_writing"))),
        TaskType::LargeCodeProject => (max_ctx.clamp(8192, 32768), "q8_0".into(), true, 0.3, Some(i18n.t("config.prompt.code_project"))),
        TaskType::MultiAgent { agent_count } => {
            let ctx = (max_ctx / *agent_count as u32).min(4096);
            (ctx, "q4_0".into(), true, 0.9, Some(i18n.t_with_vars("config.prompt.multi_agent", &[("count", &agent_count.to_string())])))
        },
        TaskType::Rag => (max_ctx.clamp(4096, 16384), "q8_0".into(), true, 0.5, Some(i18n.t("config.prompt.rag"))),
        TaskType::Mixed => (max_ctx.clamp(4096, 16384), "q8_0".into(), max_ctx > 8192, 0.7, None),
    };

    let (num_ctx, kv, flash) = if num_ctx < 2048 { (2048, "q4_0".into(), true) } else { (num_ctx, kv, flash) };

    ModelConfig { model_name: model.model.clone(), num_ctx, kv_cache_type: kv, flash_attention: flash, temperature: temp, system_prompt: prompt }
}