#[allow(dead_code)]
pub fn tokens_book(pages: u32) -> u64 { (pages as u64) * 550 }
#[allow(dead_code)]
pub fn tokens_code(loc: u32) -> u64 { (loc as u64) * 8 }
#[allow(dead_code)]
pub fn chunks(tokens: u64, chunk_size: u64) -> u64 { (tokens + chunk_size - 1) / chunk_size }

#[allow(dead_code)]
pub fn time_minutes(tokens: u64, tokens_per_second: f64) -> (f64, f64) {
    let seconds = tokens as f64 / tokens_per_second;
    let minutes = seconds / 60.0;
    let margin = 0.3;
    (minutes * (1.0 - margin), minutes * (1.0 + margin))
}

#[allow(dead_code)]
pub fn performance(model_size: u32, use_gpu: bool) -> f64 {
    match (model_size, use_gpu) {
        (3, true) => 30.0, (3, false) => 8.0,
        (7, true) => 20.0, (7, false) => 5.0,
        (14, true) => 12.0, (14, false) => 2.0,
        (32, true) => 8.0, (32, false) => 1.0,
        _ => 10.0,
    }
}

#[allow(dead_code)]
pub fn format_duration(minutes: f64) -> String {
    if minutes >= 120.0 { format!("{:.0}h{:02.0}min", minutes / 60.0, minutes % 60.0) }
    else if minutes >= 1.0 { format!("{:.0}min", minutes) }
    else { format!("{:.0}s", minutes * 60.0) }
}