# Install Service Script Fix

## Problem Fixed ✅

**Before:**
- ❌ `.\install-service.ps1 -Stop` → Only stopped voxora-service.exe
- ❌ `.\install-service.ps1 -Uninstall` → Left voxora-helper.exe running
- ❌ Helper shortcut in Startup folder not removed

**After:**
- ✅ `-Stop` → Stops BOTH voxora-service.exe AND voxora-helper.exe
- ✅ `-Uninstall` → Stops helper + removes startup shortcut
- ✅ `-Status` → Shows status of both service and helper

## What Changed

### 1. **Stop Command** (Line 186-233)
Now stops both processes:
```powershell
.\install-service.ps1 -Stop
```

**Output:**
```
Stopping VoxoraService...
Service stopped successfully!

Stopping voxora-helper...
Stopped voxora-helper process (PID: 12345)
```

### 2. **Uninstall Command** (Line 147-190)
Now does complete cleanup:
```powershell
.\install-service.ps1 -Uninstall
```

**Steps performed:**
1. Stops voxora-service (Windows Service)
2. **NEW**: Kills all voxora-helper.exe processes
3. **NEW**: Removes startup shortcut (`Voxora Helper.lnk`)
4. Removes service from Windows Services
5. (Optional) Removes credential files

**Output:**
```
Uninstalling VoxoraService...

Stopping voxora-helper...
Stopped voxora-helper process (PID: 12345)

Removed helper auto-start shortcut

Service uninstalled successfully!

Remove credential files from SYSTEM profile? (y/N)
```

### 3. **Status Command** (Line 236-270)
Now shows helper status too:
```powershell
.\install-service.ps1 -Status
```

**Output:**
```
VoxoraService Status:
Status: Running
Start Type: Automatic
Service URL: http://localhost:8080

Voxora Helper Status:
Status: Running
  PID: 12345
Auto-start: Enabled
```

## Technical Details

### Helper Process Termination
```powershell
# Find all voxora-helper processes
$helperProcesses = Get-Process -Name "voxora-helper" -ErrorAction SilentlyContinue

# Kill each one
$helperProcesses | ForEach-Object {
    try {
        $_.Kill()
        Write-Host "Stopped voxora-helper process (PID: $($_.Id))"
    } catch {
        Write-Host "Failed to stop voxora-helper (PID: $($_.Id))"
    }
}
```

### Startup Shortcut Removal
```powershell
$StartupFolder = [Environment]::GetFolderPath('Startup')
$ShortcutPath = Join-Path $StartupFolder "Voxora Helper.lnk"

if (Test-Path $ShortcutPath) {
    Remove-Item -Path $ShortcutPath -Force
    Write-Host "Removed helper auto-start shortcut"
}
```

## Usage

### Stop Both Services
```powershell
.\install-service.ps1 -Stop
```
- Stops main service
- Stops helper process
- Both will restart on next login (startup shortcut remains)

### Complete Uninstall
```powershell
.\install-service.ps1 -Uninstall
```
- Stops everything
- Removes service
- Removes helper startup shortcut
- Prompts to clean credential files

### Check Status
```powershell
.\install-service.ps1 -Status
```
- Shows service status
- Shows helper status + PID
- Shows auto-start status

## Why Helper Wasn't Stopping Before

1. **Different lifecycle**: 
   - voxora-service.exe = Windows Service (managed by NSSM)
   - voxora-helper.exe = Regular process (not a service)

2. **Startup method**:
   - Service: Controlled by Windows Service Manager
   - Helper: Started via Startup folder shortcut

3. **Stop method**:
   - Service: `Stop-Service` works
   - Helper: Need `Get-Process` + `Kill()`

## Benefits of Fix

✅ **Clean uninstall**: No orphaned processes  
✅ **Proper shutdown**: Both components stop together  
✅ **Better status**: See complete system state  
✅ **No manual cleanup**: Script handles everything  

## Backward Compatibility

✅ All existing commands work the same  
✅ No breaking changes  
✅ Only adds new functionality  

## Testing

### Test Stop
```powershell
# Start both
.\install-service.ps1 -Start
# Wait a moment for helper to start
Start-Sleep -Seconds 2
# Stop both
.\install-service.ps1 -Stop
# Check status
.\install-service.ps1 -Status
# Should show both stopped
```

### Test Uninstall
```powershell
# Install
.\install-service.ps1 -Install
.\install-service.ps1 -Start

# Check running
Get-Process voxora*
# Should see both processes

# Uninstall
.\install-service.ps1 -Uninstall

# Check
Get-Process voxora*
# Should be empty

# Check startup folder
ls $env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup
# "Voxora Helper.lnk" should be gone
```

## Files Modified

- `install-service.ps1` - Added helper management to Stop and Uninstall functions

## Lines Changed

- **Line 155-177**: Added helper stop + shortcut removal to Uninstall
- **Line 219-233**: Added helper stop to Stop command  
- **Line 250-270**: Added helper status to Status command

---

**Now both voxora-service.exe and voxora-helper.exe are properly managed together!** 🎯✅
