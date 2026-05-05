# Echomind Installer for Windows
# One-line installation: irm https://raw.githubusercontent.com/thepinak503/echomind/master/install.ps1 | iex

param(
    [switch]$Help,
    [switch]$Verify,
    [switch]$SkipVerify,
    [string]$Version = "0.3.2",
    [string]$Repo = "thepinak503/echomind"
)

$ErrorActionPreference = "Stop"

$BinaryName = "echomind.exe"

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
}

function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

function Download-Binary {
    param([string]$OS, [string]$Arch)
    
    return "https://github.com/${Repo}/releases/download/v${Version}/echomind-${OS}-${Arch}.exe.zip"
}

function Install-ViaScoop {
    info "Checking for Scoop..."
    
    if (Test-Command scoop) {
        Write-Host ""
        Write-Host "Scoop detected. Would you like to install via Scoop?" -ForegroundColor Yellow
        Write-Host "1. Scoop (recommended for developers)" -ForegroundColor Green
        Write-Host "2. Chocolatey (recommended for updates)" -ForegroundColor Cyan
        Write-Host "3. Binary from GitHub releases" -ForegroundColor White
        Write-Host "4. Cargo (build from source)" -ForegroundColor White
        Write-Host ""
        
        $choice = Read-Host "Enter choice (1-4) [Default: 1]: "
        
        if ($choice -eq "2" -or $choice -eq "") {
            Write-Host "Installing via Chocolatey..."
            if (choco install echomind -y) {
                success "Chocolatey installation complete!"
                Write-Host "To update: choco upgrade echomind"
                Write-Host "To uninstall: choco uninstall echomind"
                return $true
            } else {
                Write-Warning "Chocolatey installation failed, trying Scoop..."
            }
        } elseif ($choice -eq "3") {
            Write-Host "Installing via Scoop..."
            if (scoop install echomind) {
                success "Scoop installation complete!"
                Write-Host "To update: scoop update echomind"
                Write-Host "To uninstall: scoop uninstall echomind"
                return $true
            } else {
                Write-Warning "Scoop installation failed, trying Chocolatey..."
                if (choco install echomind -y) {
                    success "Chocolatey installation complete!"
                    Write-Host "To update: choco upgrade echomind"
                    Write-Host "To uninstall: choco uninstall echomind"
                    return $true
                } else {
                    Write-Warning "Chocolatey installation failed, falling back to binary"
                    return $false
                }
            }
        } elseif ($choice -eq "4") {
            info "Installing via Cargo..."
            if (cargo install echomind) {
                success "Cargo installation complete!"
                return $true
            } else {
                Write-Warning "Cargo installation failed, falling back to binary"
                return $false
            }
        }
    }
    
    return $false
}

function Download-Binary {
    param([string]$OS, [string]$Arch)
    
    Write-Info "Downloading echomind for ${OS}-${Arch}..."
    
    $DownloadUrl = Get-DownloadUrl -OS $OS -Arch $Arch
    Write-Info "Download URL: $DownloadUrl"
    
    $TempDir = $env:TEMP
    $ZipPath = Join-Path $TempDir "echomind-windows-${Arch}.exe.zip"
    $ExtractPath = Join-Path $TempDir "echomind-temp"
    
    try {
        # Download ZIP file
        Write-Info "Downloading..."
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing
        
        # Create extraction directory
        if (Test-Path $ExtractPath) {
            Remove-Item -Path $ExtractPath -Recurse -Force
        }
        New-Item -ItemType Directory -Path $ExtractPath -Force | Out-Null
        
        # Extract ZIP
        Write-Info "Extracting..."
        Expand-Archive -Path $ZipPath -DestinationPath $ExtractPath -Force
        
        # Determine installation directory
        $InstallDir = "$env:LOCALAPPDATA\Programs"
        $BinPath = "$env:USERPROFILE\bin"
        
        # Create directories
        if (!(Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
            Write-Success "Created installation directory: $InstallDir"
        }
        
        if (!(Test-Path $BinPath)) {
            New-Item -ItemType Directory -Path $BinPath -Force | Out-Null
            Write-Success "Created bin directory: $BinPath"
        }
        
        # Copy binary
        $BinarySource = Join-Path $ExtractPath $BinaryName
        if (Test-Path $BinarySource) {
            Copy-Item -Path $BinarySource -Destination $InstallDir -Force
            Write-Success "Binary installed to: $InstallDir\$BinaryName"
        } else {
            Write-Error "Binary not found in extracted files"
            Write-Info "Contents of extraction directory:"
            Get-ChildItem -Path $ExtractPath -Recurse | ForEach-Object { Write-Host "  $($_.FullName)" }
            exit 1
        }
        
        # Create wrapper script in user bin
        $WrapperScript = @"
@echo off
REM echomind wrapper script
"$InstallDir\$BinaryName" %*
"@
        $WrapperPath = Join-Path $BinPath "echomind.bat"
        $WrapperScript | Out-File -FilePath $WrapperPath -Encoding ASCII
        Write-Success "Wrapper script created: $WrapperPath"
        
        # Add to PATH if not already present
        $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($CurrentPath -notlike "*$BinPath*") {
            Write-Warning "Adding $BinPath to user PATH"
            [Environment]::SetEnvironmentVariable("Path", "$CurrentPath;$BinPath", "User")
            Write-Info "Please restart your terminal or log out/in to use the new PATH"
        }
        
        # Cleanup
        Write-Info "Cleaning up temporary files..."
        Remove-Item -Path $ZipPath -Force
        Remove-Item -Path $ExtractPath -Recurse -Force
        
        Write-Success "Installation completed successfully!"
        
        # Display instructions
        Write-Host ""
        Write-Host "==========================================" -ForegroundColor Green
        Write-Host "Installation complete!" -ForegroundColor Green
        Write-Host "==========================================" -ForegroundColor Green
        Write-Host ""
        Write-Host "Binary location: $InstallDir\$BinaryName"
        Write-Host "Wrapper: $WrapperPath"
        Write-Host ""
        Write-Host "To use echomind:" -ForegroundColor Cyan
        Write-Host "  1. Restart your terminal or log out/in" -ForegroundColor White
        Write-Host "  2. Run: echomind --help" -ForegroundColor White
        Write-Host "  3. Test: echomind 'Hello, how are you?'" -ForegroundColor White
        Write-Host ""
        
    } catch {
        Write-Error "Installation failed: $_"
        Write-Info "You can also install manually from:"
        Write-Host "  https://github.com/${Repo}/releases/latest" -ForegroundColor Cyan
        Write-Host "Or via winget: winget install echomind" -ForegroundColor Cyan
        exit 1
    }
}

function Verify-Installation {
    Write-Info "Verifying installation..."
    
    $InstallDir = "$env:LOCALAPPDATA\Programs"
    $BinaryPath = Join-Path $InstallDir $BinaryName
    
    if (Test-Path $BinaryPath) {
        try {
            $Output = & $BinaryPath --version 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Verification successful!"
                Write-Host "Version: $Output" -ForegroundColor Green
            } else {
                Write-Warning "Binary exists but may not be functioning correctly"
            }
        } catch {
            Write-Error "Binary verification failed: $_"
        }
    } else {
        Write-Error "Binary not found at expected location: $BinaryPath"
        Write-Info "Try reinstalling using: irm https://raw.githubusercontent.com/${Repo}/master/install.ps1 | iex"
    }
}

function Display-Usage {
    Write-Host ""
    Write-Host "Echomind Installation Script" -ForegroundColor Cyan
    Write-Host "Quick install: irm https://raw.githubusercontent.com/${Repo}/master/install.ps1 | iex" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor White
    Write-Host "  .\install.ps1                - Install echomind" -ForegroundColor Green
    Write-Host "  .\install.ps1 -verify        - Verify installation" -ForegroundColor Green
    Write-Host "  .\install.ps1 -help           - Show this help" -ForegroundColor Green
    Write-Host ""
    Write-Host "Parameters:" -ForegroundColor Yellow
    Write-Host "  -Version <version>     - Specific version (default: $Version)" -ForegroundColor White
    Write-Host "  -Repo <repo>         - Custom repository (default: ${Repo})" -ForegroundColor White
    Write-Host "  -SkipVerify           - Skip installation verification" -ForegroundColor White
    Write-Host ""
    Write-Host "Requirements:" -ForegroundColor Yellow
    Write-Host "  - Windows PowerShell 5.1 or later" -ForegroundColor White
    Write-Host "  - Internet connection for download" -ForegroundColor White
    Write-Host "  - (Optional) Administrator privileges" -ForegroundColor White
    Write-Host ""
}

# Main script logic
try {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host "  ECHOMIND INSTALLER v$Version" -ForegroundColor Cyan
    Write-Host "==========================================" -ForegroundColor Cyan
    Write-Host ""
    
    if ($Help) {
        Display-Usage
        exit 0
    }
    
    if ($Verify) {
        Verify-Installation
        exit 0
    }
    
    # Detect Windows architecture
    $Arch = (Get-CimInstance Win32_Processor).AddressWidth
    $ArchStr = if ($Arch -eq 64) { "amd64" } else { "386" }
    
    Write-Info "Detected architecture: $ArchStr"
    
    # Try package managers first
    if (Install-ViaScoop) {
        # Scoop installation succeeded, exit
        exit 0
    }
    
    # Fallback to binary download
    Download-Binary -OS "windows" -Arch $ArchStr
    
    if (!$SkipVerify) {
        Verify-Installation
    }
    
} catch {
    Write-Error "Script failed: $_"
    Write-Host ""
    Write-Host "Manual installation options:" -ForegroundColor Yellow
    Write-Host "  1. Download from: https://github.com/${Repo}/releases/latest" -ForegroundColor White
    Write-Host "  2. Extract to desired location" -ForegroundColor White
    Write-Host "  3. Add to PATH manually" -ForegroundColor White
    Write-Host "  4. Or use winget: winget install echomind" -ForegroundColor Cyan
    exit 1
}
