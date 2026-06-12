# wzllama installer - Windows PowerShell version
# Usage: 
#   Direct: curl -fsSL https://raw.githubusercontent.com/olivierlevy/wzllama/main/install.ps1 | powershell -Command -
#   Or: iwr -useb https://raw.githubusercontent.com/olivierlevy/wzllama/main/install.ps1 | iex

param(
    [string]$Branch = "main"
)

$ErrorActionPreference = "Stop"

$Repo = "olivierlevy/wzllama"

Write-Host "==============================================" 
Write-Host "  wzllama - Installation (Windows)" 
Write-Host "==============================================" 
Write-Host ""

# ─── 0. Vérifications préalables ─────────────────────
Write-Host "Checking prerequisites..." 

# Check for Rust/Cargo
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "   Rust/Cargo not found." 
    Write-Host "   Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" 
    Write-Host "   Or download from: https://rustup.rs" 
    exit 1
}
Write-Host "   Rust/Cargo found" 

# Check for git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "   Git not found. Please install Git." 
    Write-Host "   Download from: https://git-scm.com/download/win" 
    exit 1
}
Write-Host "   Git found" 

# ─── 1. Téléchargement du code ─────────────────────
Write-Host ""
Write-Host "Step 1/4: Downloading source code..." 

$InstallDir = Join-Path $env:TEMP "wzllama-install-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
Push-Location $InstallDir

try {
    git clone --depth 1 --branch $Branch "https://github.com/$Repo.git" wzllama 2>$null
    if (-not $?) {
        throw "Failed to clone repository"
    }
    Set-Location wzllama
    Write-Host "   Source code downloaded" 
} catch {
    Write-Host "   Failed to clone repository" 
    Pop-Location
    Remove-Item $InstallDir -Recurse -Force
    exit 1
}

# ─── 2. Compilation ─────────────────────────────────
Write-Host ""
Write-Host "Step 2/4: Compiling..." 
cargo build --release 2>&1 | Select-String -Last 5
Write-Host "   Compilation finished" 

# ─── 3. Installation du binaire ──────────────────────
Write-Host ""
Write-Host "Step 3/4: Installing binary..." 
$BinSrc = "target/release/wzllama.exe"

# Get user's home directory
$Home = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }

# Install to %USERPROFILE%\.wzllama\bin
$BinDir = Join-Path $Home ".wzllama\bin"

if (-not (Test-Path $BinSrc)) {
    Write-Host "   Binary not found after compilation" 
    Pop-Location
    Remove-Item $InstallDir -Recurse -Force
    exit 1
}

New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
$BinDest = Join-Path $BinDir "wzllama.exe"
Copy-Item $BinSrc $BinDest -Force
Write-Host "   wzllama installed in $BinDir" 
Write-Host "   Add to PATH if not already done:" 
Write-Host "      `$env:PATH += ';$BinDir'" 

# ─── 4. Configuration utilisateur ────────────────────
Write-Host ""
Write-Host "Step 4/4: User configuration..." 
$WzllamaDir = Join-Path $Home ".wzllama"
$I18nDir = Join-Path $WzllamaDir "i18n"

New-Item -ItemType Directory -Path $WzllamaDir, $I18nDir -Force | Out-Null

# Copy i18n files
if (Test-Path "config/i18n") {
    Get-ChildItem "config/i18n/*.json" | ForEach-Object {
        Copy-Item $_.FullName (Join-Path $I18nDir $_.Name) -Force
    }
    $LangCount = (Get-ChildItem "$I18nDir\*.json" -ErrorAction SilentlyContinue | Measure-Object).Count
    Write-Host "   Languages: $LangCount files" 
}

# Add to user PATH permanently
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$BinDir", "User")
}

# Cleanup
Pop-Location
Remove-Item $InstallDir -Recurse -Force

# ─── Vérification finale ─────────────────────────────
Write-Host ""

# Refresh PATH for current session
$env:PATH = [Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [Environment]::GetEnvironmentVariable("PATH", "User")

if (Get-Command wzllama -ErrorAction SilentlyContinue) {
    Write-Host "==============================================" 
    Write-Host "  Installation finished!" 
    Write-Host ""
    Write-Host "  Available commands:" 
    Write-Host "    wzllama              - Launch wizard" 
    Write-Host "    wzllama validate     - Validate templates" 
    Write-Host "    wzllama --dry-run    - Dry run mode" 
    Write-Host "    wzllama --help       - Help" 
    Write-Host "==============================================" 
} else {
    Write-Host "   WARNING: wzllama not found in PATH." 
    Write-Host "   Add manually to PATH: $BinDir" 
}