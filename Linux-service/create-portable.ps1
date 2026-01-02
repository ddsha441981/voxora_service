# Simple script to create portable VoxoraService package

$ErrorActionPreference = 'Stop'

Write-Host "Creating portable package..." -ForegroundColor Green

$Root = $PSScriptRoot
$DistPath = Join-Path $Root "dist\voxora-portable"
$ExePath = Join-Path $Root "target\release\voxora-service.exe"
$HelperExePath = Join-Path $Root "target\release\voxora-helper.exe"

# Build if needed
if (-not (Test-Path $ExePath)) {
    Write-Host "Building release..." -ForegroundColor Yellow
    cargo build --release
}

# Clean and create dist folder
if (Test-Path $DistPath) {
    Remove-Item $DistPath -Recurse -Force
}
New-Item $DistPath -ItemType Directory -Force | Out-Null

# Copy executables
Copy-Item $ExePath $DistPath
Write-Host "Copied voxora-service.exe" -ForegroundColor Green

Copy-Item $HelperExePath $DistPath
Write-Host "Copied voxora-helper.exe" -ForegroundColor Green

# Copy required folders
$Folders = @("data", "bin", "static")
foreach ($folder in $Folders) {
    $srcPath = Join-Path $Root $folder
    if (Test-Path $srcPath) {
        Copy-Item $srcPath $DistPath -Recurse
        Write-Host "Copied $folder/" -ForegroundColor Green
    }
}

# Create logs directory
New-Item (Join-Path $DistPath "logs") -ItemType Directory -Force | Out-Null
Write-Host "Created logs/ directory" -ForegroundColor Green

# Copy install-service.ps1 script
Copy-Item (Join-Path $Root "install-service.ps1") $DistPath
Write-Host "Copied install-service.ps1" -ForegroundColor Green

# Copy add-firewall-rule.ps1 script
if (Test-Path (Join-Path $Root "add-firewall-rule.ps1")) {
    Copy-Item (Join-Path $Root "add-firewall-rule.ps1") $DistPath
    Write-Host "Copied add-firewall-rule.ps1" -ForegroundColor Green
}

Write-Host "Package created at: $DistPath" -ForegroundColor Cyan
Write-Host "You can now copy this folder anywhere!" -ForegroundColor Yellow
Write-Host "" -ForegroundColor White
Write-Host "To install as Windows Service:" -ForegroundColor Cyan
Write-Host "1. Run PowerShell as Administrator" -ForegroundColor White
Write-Host "2. cd to dist\voxora-portable" -ForegroundColor White
Write-Host "3. .\install-service.ps1 -Install" -ForegroundColor White
Write-Host "4. .\install-service.ps1 -Start" -ForegroundColor White
Write-Host "" -ForegroundColor White
Write-Host "NOTE: voxora-helper.exe will be auto-started on user login" -ForegroundColor Yellow
Write-Host "(Required for screen capture functionality)" -ForegroundColor Yellow
