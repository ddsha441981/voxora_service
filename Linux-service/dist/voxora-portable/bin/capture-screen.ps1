# Screen capture script that works from SYSTEM account
# Uses Windows.Graphics.Capture API via PowerShell

Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms

try {
    # Get all screens
    $screens = [System.Windows.Forms.Screen]::AllScreens
    $primaryScreen = [System.Windows.Forms.Screen]::PrimaryScreen
    
    # Capture primary screen
    $bounds = $primaryScreen.Bounds
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    
    # Save to temp file as PNG
    $tempFile = [System.IO.Path]::GetTempFileName() + ".png"
    $bitmap.Save($tempFile, [System.Drawing.Imaging.ImageFormat]::Png)
    
    # Read and output as base64
    $bytes = [System.IO.File]::ReadAllBytes($tempFile)
    $base64 = [Convert]::ToBase64String($bytes)
    Write-Output $base64
    
    # Cleanup
    $graphics.Dispose()
    $bitmap.Dispose()
    Remove-Item $tempFile -Force
    
    exit 0
} catch {
    Write-Error $_.Exception.Message
    exit 1
}
