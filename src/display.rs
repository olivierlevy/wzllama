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