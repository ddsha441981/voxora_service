# Download Silero VAD model (PowerShell)
# Usage: run from voxora-service directory: pwsh -File scripts\download_vad_model.ps1

$ErrorActionPreference = "Stop"
$ModelDir = Join-Path (Get-Location) "models"
$ModelFile = "silero_vad.onnx"
$Url = "https://github.com/snakers4/silero-vad/raw/master/files/silero_vad.onnx"

Write-Host "🎤 Setting up Silero VAD model..."
if (-not (Test-Path $ModelDir)) {
  Write-Host "📁 Creating models directory: $ModelDir"
  New-Item -ItemType Directory -Path $ModelDir | Out-Null
}

$Dest = Join-Path $ModelDir $ModelFile
if (Test-Path $Dest) {
  Write-Host "✅ Silero VAD model already exists at $Dest"
  $fi = Get-Item $Dest
  Write-Host ("📊 Model size: {0:N0} bytes" -f $fi.Length)
  exit 0
}

Write-Host "📥 Downloading Silero VAD model from: $Url"
try {
  Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing
} catch {
  Write-Error "❌ Error downloading model: $($_.Exception.Message)"
  Write-Host "📋 Please download manually: $Url -> $Dest"
  exit 1
}

if (Test-Path $Dest) {
  $fi = Get-Item $Dest
  if ($fi.Length -le 0) {
    Write-Error "❌ Downloaded model file is empty!"
    Remove-Item $Dest -Force
    exit 1
  }
  Write-Host "✅ Successfully downloaded Silero VAD model!"
  Write-Host ("📊 Model size: {0:N0} bytes" -f $fi.Length)
  Write-Host "📍 Model location: $Dest"
  Write-Host "🎯 VAD Setup Complete!"
  Write-Host "   • Silero model ready (optional)"
  Write-Host "   • Energy/Adaptive VAD active by default"
} else {
  Write-Error "❌ Failed to download the model file"
  exit 1
}
