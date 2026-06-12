# wzllama - Windows Deployment Script
# Usage: .\deploy.ps1 [-Clean]

param(
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

Write-Host "==============================================" 
Write-Host "  wzllama - Installation complete (Windows)" 
Write-Host "==============================================" 
Write-Host ""

# ─── 1. Compilation ─────────────────────────────────
Write-Host "Step 1/4: Compilation..." 
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    if ($Clean) {
        Write-Host "   Cleaning..." 
        cargo clean 2>$null
    }
    cargo build --release 2>$null
    Write-Host "   Compilation finished" 
} else {
    Write-Host "   Rust/Cargo not found. Install Rust: https://rustup.rs" 
    exit 1
}

# ─── 2. Installation du binaire ──────────────────────
Write-Host ""
Write-Host "Step 2/4: Installing binary..." 
$BinSrc = "target/release/wzllama.exe"

if (-not (Test-Path $BinSrc)) {
    Write-Host "   Binary not found at $BinSrc" 
    exit 1
}

# Get user's home directory (cross-platform compatible)
$UserHome = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }

# Install to %USERPROFILE%\.wzllama\bin
$BinDir = Join-Path $UserHome ".wzllama\bin"

if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

$BinDest = Join-Path $BinDir "wzllama.exe"
Copy-Item $BinSrc $BinDest -Force
Write-Host "   wzllama installed in $BinDir" 

# ─── 3. Configuration utilisateur ────────────────────
Write-Host ""
Write-Host "Step 3/4: User configuration..." 
$WzllamaDir = Join-Path $UserHome ".wzllama"
$ConfigDir = Join-Path $WzllamaDir "config"
$I18nDir = Join-Path $WzllamaDir "i18n"
$LogsDir = Join-Path $WzllamaDir "logs"

New-Item -ItemType Directory -Path $ConfigDir, $I18nDir, $LogsDir -Force | Out-Null

# Configurations
if (Test-Path "config/default_usages.yaml") {
    Copy-Item "config/default_usages.yaml" (Join-Path $ConfigDir "usages.yaml") -Force
    Write-Host "   usages.yaml installed" 
}
if (Test-Path "config/default_config.yaml") {
    Copy-Item "config/default_config.yaml" (Join-Path $ConfigDir "config.yaml") -Force
    Write-Host "   config.yaml installed" 
}

# Fichiers i18n
if (Test-Path "config/i18n") {
    Get-ChildItem "config/i18n/*.json" | ForEach-Object {
        Copy-Item $_.FullName (Join-Path $I18nDir $_.Name) -Force
    }
    $LangCount = (Get-ChildItem "$I18nDir\*.json" -ErrorAction SilentlyContinue | Measure-Object).Count
    Write-Host "   i18n files installed ($LangCount languages)" 
}

# Vérification des fichiers requis
Write-Host ""
Write-Host "Checking files..." 
if (Test-Path (Join-Path $ConfigDir "config.yaml")) {
    Write-Host "   OK: config.yaml" 
} else {
    Write-Host "   WARNING: config.yaml missing" 
}
if (Test-Path (Join-Path $ConfigDir "usages.yaml")) {
    Write-Host "   OK: usages.yaml" 
} else {
    Write-Host "   WARNING: usages.yaml missing" 
}
if (Get-ChildItem "$I18nDir\*.json" -ErrorAction SilentlyContinue) {
    Write-Host "   OK: i18n files present" 
} else {
    Write-Host "   WARNING: No i18n files found" 
}

# ─── 4. Vérification ────────────────────────────────
Write-Host ""
Write-Host "Step 4/4: Verification..." 

# Add to user PATH if not already there
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$BinDir", "User")
    Write-Host "   User PATH updated" 
    Write-Host "   Note: Restart your terminal or run:" 
    Write-Host "      `$env:PATH += ';$BinDir'" 
}

# Refresh PATH for current session
$env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")

if (Get-Command wzllama -ErrorAction SilentlyContinue) {
    $Version = wzllama --version 2>$null
    if (-not $Version) { $Version = "unknown" }
    Write-Host "   wzllama is installed ($Version)" 
    Write-Host ""
    Write-Host "==============================================" 
    Write-Host "  Installation finished!" 
    Write-Host ""
    Write-Host "  Available commands:" 
    Write-Host "    wzllama              - Launch wizard" 
    Write-Host "    wzllama validate     - Validate templates" 
    Write-Host "    wzllama --dry-run    - Dry run mode" 
    Write-Host "    wzllama --help       - Help" 
    Write-Host "    wzllama uninstall    - Uninstall" 
    Write-Host "==============================================" 
} else {
    Write-Host "   WARNING: wzllama not found in PATH." 
    Write-Host "   Add manually to PATH: $BinDir" 
}