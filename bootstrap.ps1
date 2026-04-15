# Galapagos 3.0 bootstrap for Windows
# Works without admin rights.
# Usage (run from PowerShell):
#   powershell -ExecutionPolicy Bypass -c "irm https://raw.githubusercontent.com/kleer001/galapagos3/main/bootstrap.ps1 | iex"
$ErrorActionPreference = 'Stop'
$RepoUrl    = 'https://github.com/kleer001/galapagos3.git'
$InstallDir = if ($env:GALAPAGOS_DIR) { $env:GALAPAGOS_DIR } else { "$env:USERPROFILE\galapagos3" }

Write-Host '=== Galapagos 3.0 bootstrap ===' -ForegroundColor Cyan

# ── Git ───────────────────────────────────────────────────────────────────────
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host 'Git not found. Attempting per-user install via winget...'
    if (Get-Command winget -ErrorAction SilentlyContinue) {
        winget install --id Git.Git --scope user --silent `
            --accept-package-agreements --accept-source-agreements
        # Add the per-user Git install to this session's PATH
        $gitCmd = "$env:LOCALAPPDATA\Programs\Git\cmd"
        if (Test-Path $gitCmd) { $env:PATH = "$gitCmd;$env:PATH" }
    } else {
        Write-Host ''
        Write-Host 'ERROR: git not found and winget is not available.' -ForegroundColor Red
        Write-Host 'Install git from https://git-scm.com/download/win'
        Write-Host '  -> choose "Install for current user only" (no admin needed)'
        Write-Host 'Then re-run this script.'
        exit 1
    }
}
Write-Host "Git: $(git --version)"

# ── Rust ──────────────────────────────────────────────────────────────────────
# Installs to %USERPROFILE%\.cargo — no admin required.
# Uses the GNU toolchain, which bundles its own linker (mingw-w64).
# This avoids needing Visual Studio Build Tools.
$CargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "$env:USERPROFILE\.cargo" }
$CargoBin  = "$CargoHome\bin"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue) -and
    -not (Test-Path "$CargoBin\cargo.exe")) {
    Write-Host 'Rust not found. Installing via rustup (no admin required)...'
    $tmp = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest 'https://win.rustup.rs/x86_64' -OutFile $tmp -UseBasicParsing
    & $tmp -y --default-toolchain stable-x86_64-pc-windows-gnu --no-modify-path
    Remove-Item $tmp -Force
}

# Make cargo available for the rest of this session
$env:PATH = "$CargoBin;$env:PATH"
Write-Host "Rust: $(rustc --version)"

# ── Clone or update ───────────────────────────────────────────────────────────
if (Test-Path "$InstallDir\.git") {
    Write-Host "Updating existing clone at $InstallDir..."
    git -C $InstallDir pull
} else {
    Write-Host "Cloning to $InstallDir..."
    git clone $RepoUrl $InstallDir
}

# ── Build ─────────────────────────────────────────────────────────────────────
Write-Host 'Building release binary (first build takes a few minutes)...'
cargo build --release --manifest-path "$InstallDir\Cargo.toml"

Write-Host ''
Write-Host 'Done! To run:' -ForegroundColor Green
Write-Host ''
Write-Host "    cd $InstallDir"
Write-Host '    cargo run --release'
Write-Host ''
Write-Host 'Troubleshooting:' -ForegroundColor Yellow
Write-Host '  If you see a linker error, your system may need the MSVC toolchain instead:'
Write-Host '    1. Install VS Build Tools: https://aka.ms/vs/17/release/vs_BuildTools.exe'
Write-Host '       (select "Desktop development with C++" — no admin needed for per-user install)'
Write-Host '    2. Run: rustup default stable-x86_64-pc-windows-msvc'
Write-Host '    3. Run: cargo build --release again'
