# Voxora Service - Ubuntu Installation Guide

## Prerequisites

1. **Ubuntu 20.04 or later**
2. **Rust toolchain** (for building)
3. **Required binaries:**
   - `bin/go-server-en-linux` - English Deepgram transcription server
   - `bin/go-server-hi-linux` - Hindi Deepgram transcription server
   - `bin/SpeakerCapture` - Audio capture binary for Linux

## Credential Setup

Before installation, ensure you have Deepgram API credentials:

```bash
mkdir -p ~/mytools/data/goserver
```

Create these files:

1. **credential.txt** - Your Deepgram API key
```bash
echo "YOUR_DEEPGRAM_API_KEY" > ~/mytools/data/goserver/credential.txt
```

2. **settings_en.json** - English transcription settings
```json
{
  "language": "en-US",
  "model": "nova-3",
  "smart_format": true,
  "encoding": "linear16",
  "sample_rate": 16000,
  "channels": 1
}
```

3. **settings_hi.json** - Hindi transcription settings
```json
{
  "language": "hi",
  "model": "nova-3",
  "smart_format": true,
  "encoding": "linear16",
  "sample_rate": 16000,
  "channels": 1
}
```

## Installation

### 1. Build for Linux

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build release binary
cargo build --release

# Copy binary to project root
cp target/release/voxora-service .
```

### 2. Prepare Required Binaries

Place the following in the `bin/` directory:
- `go-server-en-linux`
- `go-server-hi-linux`
- `SpeakerCapture`

Make them executable:
```bash
chmod +x bin/*
```

### 3. Run Installation Script

```bash
# Make script executable
chmod +x install-ubuntu.sh

# Run as root
sudo ./install-ubuntu.sh
```

The script will:
- ✅ Copy files to `/opt/voxora`
- ✅ Create systemd service
- ✅ Enable auto-start on boot
- ✅ Set correct permissions
- ✅ Verify credentials

## Service Management

### Start Service
```bash
sudo systemctl start voxora
```

### Stop Service
```bash
sudo systemctl stop voxora
```

### Check Status
```bash
sudo systemctl status voxora
```

### Restart Service
```bash
sudo systemctl restart voxora
```

### View Logs
```bash
# Follow live logs
sudo journalctl -u voxora -f

# View recent logs
sudo journalctl -u voxora -n 100

# View logs since last boot
sudo journalctl -u voxora -b
```

### Disable Auto-start
```bash
sudo systemctl disable voxora
```

### Enable Auto-start
```bash
sudo systemctl enable voxora
```

## Access the UI

Once the service is running, access the web UI at:

- **Local:** http://localhost:8080
- **Network:** http://YOUR_SERVER_IP:8080

### Main Pages:
- `/` - Landing page with QR code
- `/app` - Control UI (works on desktop and mobile)
- `/mobile` - QR code for mobile access

## Firewall Configuration

If using UFW firewall, allow port 8080:

```bash
sudo ufw allow 8080/tcp
sudo ufw reload
```

## Uninstallation

```bash
# Make uninstall script executable
chmod +x uninstall-ubuntu.sh

# Run as root
sudo ./uninstall-ubuntu.sh
```

This will:
- Stop the service
- Disable auto-start
- Remove systemd service file
- Optionally remove `/opt/voxora` directory
- Keep credential files intact

## Troubleshooting

### Service won't start

1. **Check logs:**
```bash
sudo journalctl -u voxora -n 50
```

2. **Check binary exists:**
```bash
ls -la /opt/voxora/voxora-service
```

3. **Check permissions:**
```bash
ls -la /opt/voxora/bin/
```

### No transcription

1. **Verify credentials:**
```bash
cat ~/mytools/data/goserver/credential.txt
```

2. **Check go-server logs:**
```bash
cat /opt/voxora/go-server-error.log
cat /opt/voxora/go-server-stdout.log
```

3. **Check go-server is running:**
```bash
ps aux | grep go-server
```

### Audio capture not working

1. **Check SpeakerCapture binary:**
```bash
/opt/voxora/bin/SpeakerCapture
```

2. **Check audio permissions:**
```bash
groups $USER | grep audio
```

If not in audio group:
```bash
sudo usermod -a -G audio $USER
# Log out and back in
```

### Port already in use

Check if port 8080 is already taken:
```bash
sudo netstat -tulpn | grep 8080
```

## Directory Structure

```
/opt/voxora/
├── voxora-service          # Main binary
├── bin/
│   ├── go-server-en-linux  # English transcription
│   ├── go-server-hi-linux  # Hindi transcription
│   └── SpeakerCapture      # Audio capture
├── data/
│   └── settings.json       # Application settings
├── static/
│   └── ...                 # Web UI assets
└── logs/                   # Application logs
    ├── pcm_en.log
    ├── pcm_hi.log
    ├── ws_en.log
    └── ws_hi.log
```

## Environment Variables

You can customize via systemd service file:

```bash
sudo nano /etc/systemd/system/voxora.service
```

Available variables:
- `RUST_LOG` - Log level (info, debug, warn, error)
- `ENGLISH_GO_SERVER_URL` - English transcription endpoint
- `HINDI_GO_SERVER_URL` - Hindi transcription endpoint
- `CAPTURE_CMD_LINUX` - Custom audio capture command

After editing:
```bash
sudo systemctl daemon-reload
sudo systemctl restart voxora
```

## Performance Tuning

### Increase file descriptor limit

Edit `/etc/systemd/system/voxora.service`:
```ini
[Service]
LimitNOFILE=65536
```

### Adjust restart behavior

```ini
[Service]
RestartSec=10
StartLimitBurst=5
StartLimitIntervalSec=60
```

## Security Notes

1. **Firewall:** Only expose port 8080 to trusted networks
2. **Credentials:** Keep `~/mytools/data/goserver/credential.txt` secure
3. **HTTPS:** Consider using nginx as reverse proxy with SSL
4. **User:** Service runs as your user account (not root)

## Support

For issues, check:
- Service logs: `sudo journalctl -u voxora -f`
- Go-server logs: `/opt/voxora/go-server-error.log`
- Debug log: `/opt/voxora/debug.log`
