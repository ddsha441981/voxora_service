# Add Windows Firewall Rule for Voxora Service
# Run as Administrator

$ErrorActionPreference = 'Stop'

Write-Host "Adding Windows Firewall rule for Voxora Service..." -ForegroundColor Cyan

# Check if running as Administrator
$currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
$principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
$isAdmin = $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Host "ERROR: Please run as Administrator!" -ForegroundColor Red
    Write-Host "Right-click PowerShell and select 'Run as Administrator'" -ForegroundColor Yellow
    exit 1
}

$RuleName = "Voxora Service - Port 8080"
$Port = 8080

# Remove existing rule if present
$existing = Get-NetFirewallRule -DisplayName $RuleName -ErrorAction SilentlyContinue
if ($existing) {
    Write-Host "Removing existing firewall rule..." -ForegroundColor Yellow
    Remove-NetFirewallRule -DisplayName $RuleName
}

# Add new inbound rule for TCP port 8080
Write-Host "Creating firewall rule: $RuleName" -ForegroundColor Cyan
New-NetFirewallRule -DisplayName $RuleName `
    -Direction Inbound `
    -Protocol TCP `
    -LocalPort $Port `
    -Action Allow `
    -Profile Domain,Private,Public `
    -Enabled True `
    -Description "Allow incoming connections to Voxora Service on port 8080"

Write-Host ""
Write-Host "✓ Firewall rule added successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Rule details:" -ForegroundColor Cyan
Write-Host "  Name: $RuleName" -ForegroundColor White
Write-Host "  Port: $Port" -ForegroundColor White
Write-Host "  Direction: Inbound" -ForegroundColor White
Write-Host "  Profiles: Domain, Private, Public" -ForegroundColor White
Write-Host ""
Write-Host "Mobile devices on the same network can now access:" -ForegroundColor Green
Write-Host "  http://YOUR_PC_IP:8080" -ForegroundColor Yellow
Write-Host ""
Write-Host "To find your PC's IP address, run: ipconfig" -ForegroundColor Cyan
Write-Host ""
