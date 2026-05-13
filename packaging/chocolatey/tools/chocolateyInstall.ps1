# Chocolatey install script for echomind

$ErrorActionPreference = 'Stop'

$packageName = 'echomind'
$packageVersion = '0.3.2'
$packageArgs = ''
$version = $null

$softwareName = 'echomind*'

function Get-ChocolateyInstallPackageParams {
    param()
    $packageArgs = $args -join ' '
}

function Get-ChocolateyUninstallPackageParams {
    param()
    $packageArgs = $args -join ' '
}

function Get-CurrentVersion {
    $version = (Get-Command echomind).Version
    if ($version -eq $null) {
        # If echomind is not installed yet, get version from release
        $releaseInfo = Invoke-RestMethod -Uri 'https://api.github.com/repos/thepinak503/echomind/releases/latest' -Method Get
        $version = $releaseInfo.tag_name -replace 'v', ''
    }
    return $version
}

# Check for installed version
$currentVersion = Get-CurrentVersion

Write-Host "Installing echomind v$packageVersion..."

if ($currentVersion) {
    Write-Host "Currently installed version: $currentVersion"
    Write-Host "New version to install: $packageVersion"
}

# Download and install
$releaseUrl = "https://github.com/thepinak503/echomind/releases/download/v$packageVersion/echomind-windows-amd64.zip"
$downloadPath = Join-Path $env:TEMP "echomind-windows-amd64.zip"

Write-Host "Downloading from: $releaseUrl"
Invoke-WebRequest -Uri $releaseUrl -OutFile $downloadPath -UseBasicParsing

$extractPath = Join-Path $env:TEMP "echomind-temp"
if (Test-Path $extractPath) {
    Remove-Item -Path $extractPath -Recurse -Force
}
New-Item -ItemType Directory -Path $extractPath -Force | Out-Null

Write-Host "Extracting..."
Expand-Archive -Path $downloadPath -DestinationPath $extractPath -Force

$binaryPath = Join-Path $extractPath "echomind.exe"

# Remove old installation
$installDir = "$env:LOCALAPPDATA\Programs\echomind"
if (Test-Path $installDir) {
    Write-Host "Removing old installation..."
    Remove-Item -Path $installDir -Recurse -Force
}

# Create installation directory
if (!(Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Copy binary
Write-Host "Installing to: $installDir"
Copy-Item -Path $binaryPath -Destination $installDir -Force

# Create wrapper script in PATH
$binDir = "$env:APPDATA\echomind\bin"
if (!(Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir -Force | Out-Null
}

$wrapperPath = Join-Path $binDir "echomind.bat"
$wrapperScript = @"
@echo off
REM echomind wrapper script
"$installDir\echomind.exe" %*
"@
$wrapperScript | Out-File -FilePath $wrapperPath -Encoding ASCII

# Add to PATH
$envPath = [Environment]::GetEnvironmentVariable('Path', 'User', 'Process')
if ($envPath -notlike "*$binDir*") {
    Write-Host "Adding to PATH..."
    [Environment]::SetEnvironmentVariable('Path', "$envPath;$binDir", 'User', 'Process')
} else {
    Write-Host "Already in PATH"
}

# Cleanup
Remove-Item -Path $downloadPath -Force
Remove-Item -Path $extractPath -Recurse -Force

Write-Host "Installation complete!"
Write-Host ""
Write-Host "Binary location: $installDir\echomind.exe"
Write-Host "Wrapper script: $wrapperPath"
Write-Host ""
Write-Host "Configuration files: $env:LOCALAPPDATA\echomind\"
Write-Host ""
Write-Host "To get started:"
Write-Host "  1. Restart your terminal"
Write-Host "  2. Run: echomind --help"
Write-Host "  3. Test: echomind 'Hello, how are you?'"
