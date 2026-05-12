use colored::*;

pub fn header(title: &str) {
    println!("\n{}\n{}", "═".repeat(50).cyan(), title.bold());
}

pub fn section(title: &str) {
    println!("\n{}\n{}", "─".repeat(40).dimmed(), title.bold());
}

pub fn success(msg: &str) {
    println!("   ✅ {}", msg.green());
}

pub fn warning(msg: &str) {
    println!("   ⚠️  {}", msg.yellow());
}

pub fn error(msg: &str) {
    println!("   ❌ {}", msg.red());
}

pub fn info(msg: &str) {
    println!("   ℹ️  {}", msg.dimmed());
}

pub fn comment(msg: &str) {
    println!("#  {}", msg.dimmed());
}

pub fn run(msg: &str) {
    println!("🚀 {}", msg.dimmed().bold());
}

pub fn resources(ram_total: f64, ram_avail: f64, vram_total: f64, vram_avail: Option<f64>, running: &[String]) {
    println!("   💾 RAM : {:.1} / {:.1} Go libres", ram_avail, ram_total);
    if let Some(vram) = vram_avail {
        println!("   🎮 VRAM : {:.1} / {:.1} Go libres", vram, vram_total);
    }
    if !running.is_empty() {
        println!("   ⚡ Modèles chargés : {}", running.join(", ").dimmed());
    }
}

pub fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 { format!("{:.1} Go", bytes as f64 / 1_073_741_824.0) }
    else if bytes >= 1_048_576 { format!("{:.1} Mo", bytes as f64 / 1_048_576.0) }
    else { format!("{} o", bytes) }
}

/// Formatte un nombre avec espaces tous les 3 chiffres
pub fn format_number(n: u64) -> String {
    let s = n.to_string();
    let len = s.len();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 { result.push(' '); }
        result.push(c);
    }
    result
}

/// Affiche les ressources système avec barres de progression
pub fn resources_with_bars(ram_total: f64, ram_avail: f64, vram_total: f64, vram_avail: Option<f64>, running: &[String]) {
    let ram_used = ram_total - ram_avail;
    let ram_pct = if ram_total > 0.0 { ram_avail / ram_total * 100.0 } else { 0.0 };
    
    println!("   {} {:.0}% {:.1}/{:.1} Go {} ",
        "💾".cyan(), ram_pct, ram_avail, ram_total,
        progress_bar(ram_used, ram_total, 20).yellow());
    
    if let Some(vram) = vram_avail {
        let vram_used = vram_total - vram;
        let vram_pct = if vram_total > 0.0 { vram / vram_total * 100.0 } else { 0.0 };
        println!("   {} {:.0}% {:.1}/{:.1} Go {} ",
            "🎮".cyan(), vram_pct, vram, vram_total,
            progress_bar(vram_used, vram_total, 20).yellow());
    }
    
    if !running.is_empty() {
        println!("   {} {}", "⚡ Loaded:".cyan(), running.join(", ").dimmed());
    }
}

/// Barre de progression visuelle
pub fn progress_bar(used: f64, total: f64, width: usize) -> String {
    let ratio = if total > 0.0 { (used / total).min(1.0) } else { 0.0 };
    let filled = ((ratio * width as f64) as usize).min(width);
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

/// Formatte un modèle pour l'affichage dans la liste
pub fn format_model(name: &str, size: u64, score: f32, installed: bool) -> String {
    let status = if installed { "✅" } else { "⬇️" };
    let size_str = format_size(size);
    format!("{} {} ({} - {:.0}%)", status, name, size_str, score * 100.0)
}

/// Affiche un titre de section avec icône
pub fn section_title(icon: &str, title: &str) {
    println!("\n{} {}{}\n", icon.cyan(), title.bold(), "─".repeat(40 - title.len() - icon.len()).dimmed());
}