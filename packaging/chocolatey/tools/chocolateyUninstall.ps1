# Chocolatey uninstall script for echomind

$ErrorActionPreference = 'Stop'

$packageName = 'echomind'
$softwareName = 'echomind*'

Write-Host "Uninstalling echomind..."

# Remove from PATH
$binDir = "$env:APPDATA\echomind\bin"
$envPath = [Environment]::GetEnvironmentVariable('Path', 'User', 'Process')
if ($envPath -like "*$binDir*") {
    Write-Host "Removing from PATH..."
    $newPath = $envPath -replace [regex]::Escape("$binDir;?"), ''
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User', 'Process')
}

# Remove installation directory
$installDir = "$env:LOCALAPPDATA\Programs\echomind"
if (Test-Path $installDir) {
    Write-Host "Removing installation directory..."
    Remove-Item -Path $installDir -Recurse -Force
}

# Remove bin directory
if (Test-Path $binDir) {
    Write-Host "Removing bin directory..."
    Remove-Item -Path $binDir -Recurse -Force
}

Write-Host "Uninstallation complete!"
