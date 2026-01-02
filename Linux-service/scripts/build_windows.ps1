param(
  [switch]$Clean,
  [switch]$NoStop
)
$ErrorActionPreference = 'Stop'

function Info($m){ Write-Host "[info] $m" -ForegroundColor Cyan }
function Warn($m){ Write-Host "[warn] $m" -ForegroundColor Yellow }
function Fail($m){ Write-Host "[fail] $m" -ForegroundColor Red; exit 1 }

# Resolve repo root = this script's parent directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Root = Resolve-Path (Join-Path $ScriptDir '..')
Set-Location $Root

$Target = Join-Path $Root 'target/release/voxora-service.exe'
$Dist   = Join-Path $Root 'dist/voxora-service'

if(-not $NoStop){
  Info "Stopping any running voxora-service.exe"
  Get-Process 'voxora-service' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}

if($Clean){ Info 'cargo clean'; cargo clean }

Info 'Building release'
$env:RUSTFLAGS=''
$build = & cargo build --release 2>&1
if($LASTEXITCODE -ne 0){ $build | Write-Host; Fail 'cargo build failed' }

if(-not (Test-Path $Target)){ Fail "Build output not found: $Target" }

Info "Preparing dist at $Dist"
if(Test-Path $Dist){ Remove-Item $Dist -Force -Recurse }
New-Item $Dist -ItemType Directory | Out-Null

# Copy binary
Copy-Item $Target $Dist

# Copy assets if present
foreach($p in 'static','data','bin'){
  $src = Join-Path $Root $p
  if(Test-Path $src){ Info "Copy $p/"; Copy-Item $src $Dist -Recurse }
}

# Ensure logs folder exists at runtime root level next to exe
New-Item (Join-Path $Dist 'logs') -ItemType Directory -Force | Out-Null

# Create a clickable runner
$RunCmd = @"
@echo off
setlocal
REM Optional: override WS URLs
REM set ENGLISH_GO_SERVER_URL=ws://127.0.0.1:8085/ws
REM set HINDI_GO_SERVER_URL=ws://127.0.0.1:8086/ws
cd /d "%~dp0"
"voxora-service.exe"
endlocal
"@
Set-Content -Path (Join-Path $Dist 'run_service.cmd') -Value $RunCmd -Encoding ASCII

Info "Build complete"
Info "Dist folder: $Dist"
Info "Double-click run_service.cmd to start the server."
