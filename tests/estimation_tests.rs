use wzllama::core::estimation;

#[test]
fn test_tokens_book() {
    assert_eq!(estimation::tokens_book(0), 0);
    assert_eq!(estimation::tokens_book(1), 550);
    assert_eq!(estimation::tokens_book(100), 55000);
    assert_eq!(estimation::tokens_book(50), 27500);
}

#[test]
fn test_tokens_code() {
    assert_eq!(estimation::tokens_code(0), 0);
    assert_eq!(estimation::tokens_code(1), 8);
    assert_eq!(estimation::tokens_code(1000), 8000);
    assert_eq!(estimation::tokens_code(10000), 80000);
}

#[test]
fn test_chunks() {
    // Division exacte
    assert_eq!(estimation::chunks(1000, 512), 2); // (1000 + 512 - 1) / 512 = 1511 / 512 = 2
    // Division avec remainder
    assert_eq!(estimation::chunks(100, 512), 1); // (100 + 511) / 512 = 611 / 512 = 1
    // Plus de tokens
    assert_eq!(estimation::chunks(2000, 512), 4); // (2000 + 511) / 512 = 2511 / 512 = 4
}

#[test]
fn test_chunks_edge_cases() {
    assert_eq!(estimation::chunks(1, 512), 1);
    assert_eq!(estimation::chunks(512, 512), 1);
    assert_eq!(estimation::chunks(513, 512), 2);
}

#[test]
fn test_time_minutes() {
    let (min, max) = estimation::time_minutes(12000, 30.0);
    // 12000 / 30 = 400s = 6.67 min
    // margin 0.3 -> 4.67 min to 8.67 min
    assert!(min < max);
    assert!(min > 0.0);
    // Max should be approximately 20% higher than min (1.3x)
    assert!((max - min) / min > 0.2);
}

#[test]
fn test_performance_gpu_vs_cpu() {
    // Même modèle, le GPU devrait être plus rapide
    let cpu_perf = estimation::performance(14, false);
    let gpu_perf = estimation::performance(14, true);
    assert!(gpu_perf > cpu_perf);
    
    // GPU 14B: 12 tokens/s vs CPU 14B: 2 tokens/s
    assert_eq!(gpu_perf, 12.0);
    assert_eq!(cpu_perf, 2.0);
}

#[test]
fn test_performance_all_sizes() {
    // GPU performances
    assert_eq!(estimation::performance(3, true), 30.0);
    assert_eq!(estimation::performance(7, true), 20.0);
    assert_eq!(estimation::performance(14, true), 12.0);
    assert_eq!(estimation::performance(32, true), 8.0);
    
    // CPU performances
    assert_eq!(estimation::performance(3, false), 8.0);
    assert_eq!(estimation::performance(7, false), 5.0);
    assert_eq!(estimation::performance(14, false), 2.0);
    assert_eq!(estimation::performance(32, false), 1.0);
}

#[test]
fn test_performance_unknown_size() {
    // Taille inconnue retourne 10.0 par défaut
    assert_eq!(estimation::performance(100, false), 10.0);
    assert_eq!(estimation::performance(1, false), 10.0);
}

#[test]
fn test_format_duration() {
    // Secondes (< 1 min)
    assert_eq!(estimation::format_duration(0.5), "30s");
    assert_eq!(estimation::format_duration(0.25), "15s");
    
    // Minutes (< 2h) - note: {:.0} rounds 1.5 to 2
    assert_eq!(estimation::format_duration(1.0), "1min");
    assert_eq!(estimation::format_duration(1.4), "1min");
    assert_eq!(estimation::format_duration(1.5), "2min");
    assert_eq!(estimation::format_duration(5.0), "5min");
    assert_eq!(estimation::format_duration(59.0), "59min");
    
    // Heures (>= 2h)
    assert_eq!(estimation::format_duration(120.0), "2h00min");
    assert_eq!(estimation::format_duration(180.0), "3h00min");
    assert_eq!(estimation::format_duration(150.0), "2h30min");
}

#[test]
fn test_format_duration_edge_cases() {
    // Très exactement 2h
    assert_eq!(estimation::format_duration(120.0), "2h00min");
    // Juste en dessous de 2h
    assert_eq!(estimation::format_duration(119.0), "119min");
}