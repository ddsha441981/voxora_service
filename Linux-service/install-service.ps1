# VoxoraService Installation Script using NSSM
# Run as Administrator

param(
    [switch]$Install,
    [switch]$Uninstall,
    [switch]$Start,
    [switch]$Stop,
    [switch]$Status
)

$ErrorActionPreference = 'Stop'

# Configuration
$ServiceName = "VoxoraService"
$NSSMPath = "C:\tools\nssm\win64\nssm.exe"
$ExePath = Join-Path $PSScriptRoot "voxora-service.exe"
$WorkingDir = $PSScriptRoot

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Install-VoxoraService {
    Write-Host "Installing VoxoraService..." -ForegroundColor Green
    
    # Check if NSSM exists
    if (-not (Test-Path $NSSMPath)) {
        Write-Host "ERROR: NSSM not found at $NSSMPath" -ForegroundColor Red
        Write-Host "Please download NSSM from https://nssm.cc/download and extract to C:\tools\nssm\" -ForegroundColor Yellow
        exit 1
    }
    
    # Check if executable exists
    if (-not (Test-Path $ExePath)) {
        Write-Host "ERROR: voxora-service.exe not found at $ExePath" -ForegroundColor Red
        Write-Host "Please build the project first using: cargo build --release" -ForegroundColor Yellow
        exit 1
    }
    
    # Copy credential files to SYSTEM user profile
    Write-Host "Setting up credential files for SYSTEM account..." -ForegroundColor Cyan
    $SourceCredPath = "$env:USERPROFILE\mytools\data\goserver"
    $SystemProfilePath = "C:\Windows\system32\config\systemprofile\mytools\data\goserver"
    
    if (Test-Path $SourceCredPath) {
        # Create directory structure
        if (-not (Test-Path $SystemProfilePath)) {
            New-Item -ItemType Directory -Path $SystemProfilePath -Force | Out-Null
            Write-Host "Created directory: $SystemProfilePath" -ForegroundColor Green
        }
        
        # Copy all files from source to system profile
        Copy-Item -Path "$SourceCredPath\*" -Destination $SystemProfilePath -Recurse -Force
        Write-Host "Copied credential files to SYSTEM profile" -ForegroundColor Green
    } else {
        Write-Host "WARNING: Credential files not found at $SourceCredPath" -ForegroundColor Yellow
        Write-Host "Go-server may not work without credential files!" -ForegroundColor Yellow
    }
    
    # Remove existing service if it exists
    try {
        & $NSSMPath remove $ServiceName confirm 2>$null
        Write-Host "Removed existing service" -ForegroundColor Yellow
    } catch {
        # Service doesn't exist, continue
    }
    
    # Install service
    Write-Host "Installing service with NSSM..." -ForegroundColor Cyan
    & $NSSMPath install $ServiceName "`"$ExePath`""
    if ($LASTEXITCODE -ne 0) { 
        Write-Host "Failed to install service" -ForegroundColor Red
        exit 1 
    }
    
    # Set working directory
    Write-Host "Setting working directory..." -ForegroundColor Cyan
    & $NSSMPath set $ServiceName AppDirectory "`"$WorkingDir`""
    
    # Set to auto-start
    Write-Host "Setting auto-start..." -ForegroundColor Cyan
    & $NSSMPath set $ServiceName Start SERVICE_AUTO_START
    
    # Set display name and description
    & $NSSMPath set $ServiceName DisplayName "Voxora Service"
    & $NSSMPath set $ServiceName Description "AI-powered voice and screen capture service"
    
    # Configure restart on failure
    & $NSSMPath set $ServiceName AppRestartDelay 5000
    & $NSSMPath set $ServiceName AppThrottle 1500
    
    # Add Windows Firewall rule for port 8080
    Write-Host "Configuring Windows Firewall..." -ForegroundColor Cyan
    $FirewallRuleName = "Voxora Service - Port 8080"
    $existing = Get-NetFirewallRule -DisplayName $FirewallRuleName -ErrorAction SilentlyContinue
    if ($existing) {
        Write-Host "Firewall rule already exists" -ForegroundColor Yellow
    } else {
        New-NetFirewallRule -DisplayName $FirewallRuleName `
            -Direction Inbound `
            -Protocol TCP `
            -LocalPort 8080 `
            -Action Allow `
            -Profile Domain,Private,Public `
            -Enabled True `
            -Description "Allow incoming connections to Voxora Service" | Out-Null
        Write-Host "Firewall rule added (Port 8080)" -ForegroundColor Green
    }
    
    # Install helper to auto-start on user login
    Write-Host "Installing helper auto-start..." -ForegroundColor Cyan
    $HelperExePath = Join-Path $PSScriptRoot "voxora-helper.exe"
    if (Test-Path $HelperExePath) {
        $StartupFolder = [Environment]::GetFolderPath('Startup')
        $ShortcutPath = Join-Path $StartupFolder "Voxora Helper.lnk"
        
        # Create shortcut
        $WScriptShell = New-Object -ComObject WScript.Shell
        $Shortcut = $WScriptShell.CreateShortcut($ShortcutPath)
        $Shortcut.TargetPath = $HelperExePath
        $Shortcut.WorkingDirectory = $PSScriptRoot
        $Shortcut.WindowStyle = 7  # Minimized
        $Shortcut.Description = "Voxora Screen Capture Helper"
        $Shortcut.Save()
        
        Write-Host "Helper auto-start configured" -ForegroundColor Green
        Write-Host "Helper will start automatically on user login" -ForegroundColor Yellow
        
        # Start helper now if not running
        $helperRunning = Get-Process -Name "voxora-helper" -ErrorAction SilentlyContinue
        if (-not $helperRunning) {
            Start-Process $HelperExePath -WindowStyle Hidden
            Write-Host "Helper started" -ForegroundColor Green
        }
    } else {
        Write-Host "WARNING: voxora-helper.exe not found" -ForegroundColor Yellow
        Write-Host "Screen capture will not work!" -ForegroundColor Yellow
    }
    
    Write-Host "Service installed successfully!" -ForegroundColor Green
    Write-Host "Use -Start to start the service" -ForegroundColor Yellow
}

function Uninstall-VoxoraService {
    Write-Host "Uninstalling VoxoraService..." -ForegroundColor Yellow
    
    # Stop service first
    try {
        Stop-Service $ServiceName -Force -ErrorAction SilentlyContinue
    } catch {}
    
    # Stop voxora-helper process
    Write-Host "Stopping voxora-helper..." -ForegroundColor Cyan
    $helperProcesses = Get-Process -Name "voxora-helper" -ErrorAction SilentlyContinue
    if ($helperProcesses) {
        $helperProcesses | ForEach-Object {
            try {
                $_.Kill()
                Write-Host "Stopped voxora-helper process (PID: $($_.Id))" -ForegroundColor Green
            } catch {
                Write-Host "Failed to stop voxora-helper (PID: $($_.Id))" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "voxora-helper not running" -ForegroundColor Gray
    }
    
    # Remove helper startup shortcut
    $StartupFolder = [Environment]::GetFolderPath('Startup')
    $ShortcutPath = Join-Path $StartupFolder "Voxora Helper.lnk"
    if (Test-Path $ShortcutPath) {
        Remove-Item -Path $ShortcutPath -Force
        Write-Host "Removed helper auto-start shortcut" -ForegroundColor Green
    }
    
    # Remove service
    & $NSSMPath remove $ServiceName confirm
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Service uninstalled successfully!" -ForegroundColor Green
    } else {
        Write-Host "Failed to uninstall service or service doesn't exist" -ForegroundColor Red
    }
    
    # Optional: Clean up SYSTEM profile credential files
    $SystemProfilePath = "C:\Windows\system32\config\systemprofile\mytools"
    if (Test-Path $SystemProfilePath) {
        $response = Read-Host "Remove credential files from SYSTEM profile? (y/N)"
        if ($response -eq 'y' -or $response -eq 'Y') {
            Remove-Item -Path $SystemProfilePath -Recurse -Force
            Write-Host "Removed credential files from SYSTEM profile" -ForegroundColor Green
        }
    }
}

function Start-VoxoraService {
    Write-Host "Starting VoxoraService..." -ForegroundColor Green
    try {
        Start-Service $ServiceName
        Write-Host "Service started successfully!" -ForegroundColor Green
        Write-Host "Service is running at: http://localhost:8080" -ForegroundColor Cyan
    } catch {
        Write-Host "Failed to start service: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "Check Windows Event Viewer for details" -ForegroundColor Yellow
    }
}

function Stop-VoxoraService {
    Write-Host "Stopping VoxoraService..." -ForegroundColor Yellow
    try {
        Stop-Service $ServiceName -Force
        Write-Host "Service stopped successfully!" -ForegroundColor Green
    } catch {
        Write-Host "Failed to stop service: $($_.Exception.Message)" -ForegroundColor Red
    }
    
    # Also stop voxora-helper
    Write-Host "Stopping voxora-helper..." -ForegroundColor Cyan
    $helperProcesses = Get-Process -Name "voxora-helper" -ErrorAction SilentlyContinue
    if ($helperProcesses) {
        $helperProcesses | ForEach-Object {
            try {
                $_.Kill()
                Write-Host "Stopped voxora-helper process (PID: $($_.Id))" -ForegroundColor Green
            } catch {
                Write-Host "Failed to stop voxora-helper (PID: $($_.Id))" -ForegroundColor Yellow
            }
        }
    } else {
        Write-Host "voxora-helper not running" -ForegroundColor Gray
    }
}

function Get-VoxoraServiceStatus {
    Write-Host "VoxoraService Status:" -ForegroundColor Cyan
    try {
        $service = Get-Service $ServiceName -ErrorAction Stop
        Write-Host "Status: $($service.Status)" -ForegroundColor $(if($service.Status -eq 'Running'){'Green'}else{'Yellow'})
        Write-Host "Start Type: $($service.StartType)" -ForegroundColor White
        
        if ($service.Status -eq 'Running') {
            Write-Host "Service URL: http://localhost:8080" -ForegroundColor Cyan
        }
    } catch {
        Write-Host "Service not found or not installed" -ForegroundColor Red
    }
    
    # Check helper status
    Write-Host "" 
    Write-Host "Voxora Helper Status:" -ForegroundColor Cyan
    $helperProcesses = Get-Process -Name "voxora-helper" -ErrorAction SilentlyContinue
    if ($helperProcesses) {
        Write-Host "Status: Running" -ForegroundColor Green
        $helperProcesses | ForEach-Object {
            Write-Host "  PID: $($_.Id)" -ForegroundColor White
        }
    } else {
        Write-Host "Status: Not Running" -ForegroundColor Yellow
    }
    
    # Check startup shortcut
    $StartupFolder = [Environment]::GetFolderPath('Startup')
    $ShortcutPath = Join-Path $StartupFolder "Voxora Helper.lnk"
    if (Test-Path $ShortcutPath) {
        Write-Host "Auto-start: Enabled" -ForegroundColor Green
    } else {
        Write-Host "Auto-start: Disabled" -ForegroundColor Yellow
    }
}

# Check if running as Administrator
if (-not (Test-Administrator)) {
    Write-Host "ERROR: This script must be run as Administrator!" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

# Execute based on parameters
switch ($true) {
    $Install { Install-VoxoraService }
    $Uninstall { Uninstall-VoxoraService }
    $Start { Start-VoxoraService }
    $Stop { Stop-VoxoraService }
    $Status { Get-VoxoraServiceStatus }
    default {
        Write-Host "VoxoraService Management Script" -ForegroundColor Cyan
        Write-Host "==============================" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "Usage (run as Administrator):" -ForegroundColor White
        Write-Host "  .\install-service.ps1 -Install    # Install service" -ForegroundColor Green
        Write-Host "  .\install-service.ps1 -Start      # Start service" -ForegroundColor Green
        Write-Host "  .\install-service.ps1 -Stop       # Stop service" -ForegroundColor Yellow
        Write-Host "  .\install-service.ps1 -Status     # Check status" -ForegroundColor Cyan
        Write-Host "  .\install-service.ps1 -Uninstall  # Remove service" -ForegroundColor Red
        Write-Host ""
        Write-Host "Prerequisites:" -ForegroundColor White
        Write-Host "- NSSM installed at: C:\tools\nssm\win64\nssm.exe" -ForegroundColor Yellow
        Write-Host "- Built executable at: voxora-service.exe" -ForegroundColor Yellow
    }
}