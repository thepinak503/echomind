# Echomind Installation Script for Windows
# Usage: irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex

$ErrorActionPreference = "Stop"

# Colors
function Write-ColorOutput($ForegroundColor) {
    $fc = $host.UI.RawUI.ForegroundColor
    $host.UI.RawUI.ForegroundColor = $ForegroundColor
    if ($args) {
        Write-Output $args
    }
    $host.UI.RawUI.ForegroundColor = $fc
}

function Write-Success { Write-ColorOutput Green $args }
function Write-Info { Write-ColorOutput Cyan $args }
function Write-Warning { Write-ColorOutput Yellow $args }
function Write-Error { Write-ColorOutput Red $args }

Write-Info "=== Echomind Installer for Windows ==="
Write-Output ""

# Detect architecture
if ([System.Environment]::Is64BitOperatingSystem) {
    $Arch = "x64"
} else {
    $Arch = "x86"
}
Write-Info "Detected Architecture: $Arch"

# Note: Using pre-built binary, no need for Rust

# Create temporary directory
$TempDir = Join-Path $env:TEMP "echomind_install_$(Get-Random)"
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    # Clone repository
    Write-Info "Cloning echomind repository..."

    # Check if git is installed
    $GitInstalled = Get-Command git -ErrorAction SilentlyContinue

    if ($GitInstalled) {
        git clone --depth 1 https://github.com/thepinak503/echomind.git $TempDir
    }
    else {
        Write-Warning "Git not found. Downloading source as ZIP..."
        $ZipUrl = "https://github.com/thepinak503/echomind/archive/refs/heads/master.zip"
        $ZipFile = Join-Path $env:TEMP "echomind.zip"

        Invoke-WebRequest -Uri $ZipUrl -OutFile $ZipFile
        Expand-Archive -Path $ZipFile -DestinationPath $env:TEMP -Force
        Move-Item -Path (Join-Path $env:TEMP "echomind-master\*") -Destination $TempDir -Force
        Remove-Item $ZipFile
    }

    Set-Location $TempDir

    Write-Info "Using pre-built binary..."

    # Determine installation directory
    $InstallDir = "$env:USERPROFILE\.local\bin"

    # Create installation directory if it doesn't exist
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Write-Info "Created directory: $InstallDir"
    }

    # Copy binary
    Write-Info "Installing echomind..."
    Copy-Item -Path "echomind-windows-x86_64.exe" -Destination "$InstallDir\echomind.exe" -Force

    # Create documentation directory
    $DocDir = "$env:USERPROFILE\.local\share\doc\echomind"
    if (-not (Test-Path $DocDir)) {
        New-Item -ItemType Directory -Path $DocDir -Force | Out-Null
    }

    # Copy documentation
    Copy-Item -Path "README.md" -Destination "$DocDir\README.md" -Force -ErrorAction SilentlyContinue
    Copy-Item -Path "CONTRIBUTING.md" -Destination "$DocDir\CONTRIBUTING.md" -Force -ErrorAction SilentlyContinue
    Copy-Item -Path "config.example.toml" -Destination "$DocDir\config.example.toml" -Force -ErrorAction SilentlyContinue

    Write-Success "✓ Binary installed to $InstallDir\echomind.exe"

    # Check if install directory is in PATH
    $UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")

    if ($UserPath -notlike "*$InstallDir*") {
        Write-Warning "The installation directory is not in your PATH."
        Write-Info "Adding $InstallDir to your PATH..."

        $NewPath = $UserPath + ";" + $InstallDir
        [System.Environment]::SetEnvironmentVariable("Path", $NewPath, "User")

        Write-Success "✓ Added to PATH (restart your terminal to use 'echomind' command)"
        Write-Warning "OR run this in your current terminal:"
        Write-Output "    `$env:Path += `";$InstallDir`""
    }
    else {
        Write-Success "✓ Installation directory is already in PATH"
    }

}
finally {
    # Clean up
    Set-Location $env:TEMP
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Output ""
Write-Success "╔════════════════════════════════════════╗"
Write-Success "║  ✓ echomind installed successfully!    ║"
Write-Success "╚════════════════════════════════════════╝"
Write-Output ""
Write-Info "Quick Start:"
Write-Output "  1. Initialize config:  echomind --init-config"
Write-Output "  2. Try it out:        echo 'Hello AI!' | echomind"
Write-Output "  3. Interactive mode:  echomind --interactive"
Write-Output "  4. View help:         echomind --help"
Write-Output ""
Write-Info "Configuration:"
Write-Output "  Config location: $env:USERPROFILE\.config\echomind\config.toml"
Write-Output ""
Write-Info "For more information:"
Write-Output "  https://github.com/thepinak503/echomind"
Write-Output ""

# Test installation (if PATH was already correct)
$EchomindCmd = Get-Command echomind -ErrorAction SilentlyContinue
if ($EchomindCmd) {
    Write-Info "Testing installation..."
    $Version = & echomind --version 2>&1
    Write-Success "✓ echomind $Version"
}
else {
    Write-Warning "Restart your terminal to use the 'echomind' command"
}
