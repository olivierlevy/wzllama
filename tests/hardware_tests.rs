use wzllama::core::hardware::{self, GpuInfo, HardwareInfo};

#[test]
fn test_hardware_info_default() {
    let hw = HardwareInfo::default();
    assert_eq!(hw.os, "");
    assert_eq!(hw.ram_gb, 0.0);
    assert_eq!(hw.total_vram_mb, 0);
    assert!(hw.gpus.is_empty());
}

#[test]
fn test_hardware_info_default_for_test() {
    let hw = HardwareInfo::default_for_test();
    assert!(hw.os.contains("linux"));
    assert_eq!(hw.ram_gb, 16.0);
    assert_eq!(hw.total_vram_mb, 0);
    assert!(hw.gpus.is_empty());
}

#[test]
fn test_hardware_info_with_gpu() {
    let hw = HardwareInfo {
        os: "linux x86_64".into(),
        ram_gb: 32.0,
        total_vram_mb: 73728, // Total VRAM should match sum of GPU VRAM
        gpus: vec![
            GpuInfo { name: "RTX 4090".into(), vram_mb: 24576 },
            GpuInfo { name: "RTX A6000".into(), vram_mb: 49152 },
        ],
    };
    
    assert!(hw.has_gpu());
    assert_eq!(hw.gpus.len(), 2);
    // Vérifie que la somme des VRAM GPU est correcte
    let gpu_vram_sum: u64 = hw.gpus.iter().map(|g| g.vram_mb).sum();
    assert_eq!(gpu_vram_sum, 73728);
}

#[test]
fn test_hardware_info_no_gpu() {
    let hw = HardwareInfo {
        os: "linux x86_64".into(),
        ram_gb: 16.0,
        total_vram_mb: 0,
        gpus: vec![],
    };
    
    assert!(!hw.has_gpu());
}

#[test]
fn test_gpu_info() {
    let gpu = GpuInfo {
        name: "RTX 3080".into(),
        vram_mb: 10240,
    };
    
    assert_eq!(gpu.name, "RTX 3080");
    assert_eq!(gpu.vram_mb, 10240);
}

#[test]
fn test_hardware_info_serialization() {
    let hw = HardwareInfo {
        os: "linux x86_64".into(),
        ram_gb: 32.0,
        total_vram_mb: 16384,
        gpus: vec![GpuInfo { name: "RTX 4090".into(), vram_mb: 24576 }],
    };
    
    let json = serde_json::to_string(&hw).unwrap();
    let deserialized: HardwareInfo = serde_json::from_str(&json).unwrap();
    
    assert_eq!(deserialized.os, hw.os);
    assert_eq!(deserialized.ram_gb, hw.ram_gb);
    assert_eq!(deserialized.total_vram_mb, hw.total_vram_mb);
    assert_eq!(deserialized.gpus.len(), 1);
}

#[test]
fn test_detect_returns_valid_hardware() {
    let hw = hardware::detect();
    
    assert!(!hw.os.is_empty());
    assert!(hw.ram_gb > 0.0);
    assert!(hw.gpus.iter().all(|g| !g.name.is_empty()));
}

#[test]
fn test_total_vram_calculation() {
    let hw = HardwareInfo {
        os: "linux".into(),
        ram_gb: 16.0,
        total_vram_mb: 0,
        gpus: vec![
            GpuInfo { name: "GPU1".into(), vram_mb: 4096 },
            GpuInfo { name: "GPU2".into(), vram_mb: 8192 },
        ],
    };
    
    let total: u64 = hw.gpus.iter().map(|g| g.vram_mb).sum();
    assert_eq!(total, 12288);
}