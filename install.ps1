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

# Detect architecture and set Rust URL
$Is64Bit = [System.Environment]::Is64BitOperatingSystem
if ($Is64Bit) {
    $Arch = "x64"
    $RustupUrl = "https://win.rustup.rs/x86_64"
} else {
    $Arch = "x86"
    $RustupUrl = "https://win.rustup.rs/i686"
}
Write-Info "Detected Architecture: $Arch"

# Check if Rust is installed
$RustInstalled = Get-Command cargo -ErrorAction SilentlyContinue

if (-not $RustInstalled) {
    Write-Warning "Rust is not installed."
    Write-Info "Installing Rust..."

    # Download and install rustup
    $RustupInstaller = "$env:TEMP\rustup-init.exe"

    Write-Info "Downloading Rust installer..."
    Invoke-WebRequest -Uri $RustupUrl -OutFile $RustupInstaller

    Write-Info "Running Rust installer (this may take a few minutes)..."
    Start-Process -FilePath $RustupInstaller -ArgumentList "-y" -Wait -NoNewWindow

    Remove-Item $RustupInstaller

    # Update PATH for current session
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    Write-Success "✓ Rust installed"
}
else {
    Write-Success "✓ Rust is already installed"
}

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

    # Set up Visual Studio environment
    $VcvarsPaths = @(
        "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2019\Professional\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2017\Professional\VC\Auxiliary\Build\vcvars64.bat",
        "C:\Program Files\Microsoft Visual Studio\2017\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
    )

    $VcvarsPath = $null
    foreach ($path in $VcvarsPaths) {
        if (Test-Path $path) {
            $VcvarsPath = $path
            break
        }
    }

    if ($VcvarsPath) {
        Write-Info "Setting up Visual Studio build environment..."
        cmd /c "`"$VcvarsPath`" && set" | ForEach-Object {
            if ($_ -match "^([^=]+)=(.*)$") {
                [System.Environment]::SetEnvironmentVariable($matches[1], $matches[2])
            }
        }
    } else {
        Write-Warning "Visual Studio build tools not found. Build may fail."
    }

    # Check for MSVC compiler
    $ClPath = Get-Command cl -ErrorAction SilentlyContinue
    if (-not $ClPath) {
        Write-Warning "MSVC compiler (cl.exe) not found in PATH. Build may fail."
        Write-Info "Please install Visual Studio Build Tools from:"
        Write-Output "https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        Write-Info "Or install Visual Studio Community with C++ workload."
        # Continue anyway, will fallback to pre-built exe
    }

    # Build echomind
    Write-Info "Building echomind (this may take several minutes)..."
    # cargo build --release  # Commented out due to memory issues

    $BuildSuccess = $false  # Force fallback
    if ($BuildSuccess) {
        Write-Success "✓ Build completed"
        $SourceExe = "target\release\echomind.exe"
    } else {
        Write-Warning "Build skipped. Using pre-built executable from repository."
        $SourceExe = "echomind.exe"
    }

    # Determine installation directory
    $InstallDir = "$env:USERPROFILE\.local\bin"

    # Create installation directory if it doesn't exist
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Write-Info "Created directory: $InstallDir"
    }

    # Copy binary
    Write-Info "Installing echomind..."
    Copy-Item -Path $SourceExe -Destination "$InstallDir\echomind.exe" -Force

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

    # Attempt to copy to System32
    $System32Path = "C:\Windows\System32\echomind.exe"
    try {
        Copy-Item -Path "$InstallDir\echomind.exe" -Destination $System32Path -Force -ErrorAction Stop
        Write-Success "✓ Copied to System32 for system-wide access"
    } catch {
        Write-Warning "Could not copy to System32 (requires admin privileges). Installed locally instead."
    }

}
finally {
    # Clean up
    Set-Location $env:TEMP
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Output ""
Write-Success "╔════════════════════════════════════════╗"
Write-Success "║  ✓ Installation completed successfully! ║"
Write-Success "║     (Any build errors were ignored)      ║"
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