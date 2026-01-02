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

---

# Advanced Ubuntu Guide (User Service, Keyring, Remote)

This section adds end-to-end steps to run Voxora as a user service (recommended), configure the Linux keyring for API keys, and set up the Remote (AnythingLLM) integration.

## Portable build and layout

1. Build portable package
   ```bash
   ./create-portable-ubuntu.sh
   ```
2. The folder `dist/voxora-portable-linux` will contain:
   - `voxora-service`, `logs/`, optional `bin/`, `data/`, `static/`
   - installers: `install-ubuntu.sh` (system), `install-ubuntu-user.sh` (user)

## Option A: Run as a systemd USER service (recommended)

Advantages: uses your user keyring (no sudo), easy screen-capture later, restarts and logging via systemd.

1) Optional prerequisites (one-time)

- Install helpers (only if missing):
  ```bash
  sudo apt-get update
  sudo apt-get install -y dbus-user-session gnome-keyring libsecret-tools
  ```
- Keep your user services alive after logout:
  ```bash
  sudo loginctl enable-linger "$USER"
  ```

2) Install Voxora as a user service

```bash
cd dist/voxora-portable-linux
./install-ubuntu-user.sh
```
- This script also sets up a user Secret Service (org.freedesktop.secrets) via a user unit and verifies it is running.
- Status/logs:
  ```bash
  systemctl --user status voxora --no-pager
  journalctl --user -u voxora -f
  ```

3) Store provider API keys (one time per user/machine)

Use UI (Settings → Providers) or curl. Using curl:

```bash
# Replace placeholders with your real secrets (do not include braces in the values)
GROQ_KEY={{GROQ_KEY}}
GEMINI_KEY={{GEMINI_KEY}}
OPENROUTER_KEY={{OPENROUTER_KEY}}
ANYTHINGLLM_KEY={{ANYTHINGLLM_KEY}}

curl -s -X POST http://127.0.0.1:8080/api/providers/groq/key \
  -H 'Content-Type: application/json' -d "{\"api_key\":\"$GROQ_KEY\"}"

curl -s -X POST http://127.0.0.1:8080/api/providers/gemini/key \
  -H 'Content-Type: application/json' -d "{\"api_key\":\"$GEMINI_KEY\"}"

curl -s -X POST http://127.0.0.1:8080/api/providers/openrouter/key \
  -H 'Content-Type: application/json' -d "{\"api_key\":\"$OPENROUTER_KEY\"}"

# Remote (AnythingLLM) API key
curl -s -X POST http://127.0.0.1:8080/api/remote/key \
  -H 'Content-Type: application/json' -d "{\"api_key\":\"$ANYTHINGLLM_KEY\"}"
```

Check detection:
```bash
curl -s http://127.0.0.1:8080/api/providers/state
```

## Option B: Run as a systemd SYSTEM service (root)

Use this if you prefer a root-managed service; it will still run as your user by default.

```bash
cd dist/voxora-portable-linux
sudo ./install-ubuntu.sh
sudo systemctl enable --now voxora
sudo journalctl -u voxora -f
```

Note: System services do not automatically see the user keyring. It’s simpler to use the USER service. If you keep the system service, still run the user `gnome-keyring` unit and seed keys as your user.

## Remote (AnythingLLM) setup

1) Select your tunnel (Startup page or API)

- Startup page → enter tunnel URL from `cloudflared` and test/connect.
- Or via API:
  ```bash
  curl -s -X POST http://127.0.0.1:8080/api/remote/select \
    -H 'Content-Type: application/json' \
    -d '{"server":"linux","url":"https://<your_trycloudflare>.trycloudflare.com"}'

  curl -s http://127.0.0.1:8080/api/remote/status | jq
  ```

2) Save the AnythingLLM API key

- UI: Settings → Remote → Save API key.
- Or API (see above, `/api/remote/key`).

3) Pick the correct workspace slug

- List workspaces from the selected remote:
  ```bash
  curl -s http://127.0.0.1:8080/api/remote/workspaces | jq
  ```
- Save the slug (must match exactly what the list returns—often underscores, not hyphens):
  ```bash
  curl -s -X POST http://127.0.0.1:8080/api/remote/config \
    -H 'Content-Type: application/json' \
    -d '{"slug":"voxora_docs"}'

  curl -s http://127.0.0.1:8080/api/remote/workspace | jq
  ```

4) Ask the remote

```bash
# Chat
curl -s -X POST http://127.0.0.1:8080/api/remote/ask \
  -H 'Content-Type: application/json' \
  -d '{"input":"hello","stream":false,"mode":"chat"}'

# Query mode (runs vector-search first)
curl -s -X POST http://127.0.0.1:8080/api/remote/ask \
  -H 'Content-Type: application/json' \
  -d '{"input":"hello","stream":false,"mode":"query"}'
```

## Increase timeouts for remote calls (if needed)

Voxora’s defaults may be aggressive for streaming/query. Edit `data/settings.json` in the running folder (for user service, your `dist/voxora-portable-linux/data/settings.json`):

```json
{
  "timeouts": {
    "health_check_secs": 5,
    "workspace_secs": 15,
    "chat_secs": 60
  }
}
```

Then restart:
```bash
systemctl --user restart voxora   # if user service
# or
sudo systemctl restart voxora     # if system service
```

## Uninstall

- USER service:
  ```bash
  systemctl --user disable --now voxora
  rm -f ~/.config/systemd/user/voxora.service
  systemctl --user daemon-reload
  ```
- SYSTEM service:
  ```bash
  sudo ./uninstall-ubuntu.sh
  ```

## Troubleshooting

- Script says “cannot execute: required file not found”
  - Likely CRLF endings. Fix and run:
    ```bash
    sed -i 's/\r$//' install-ubuntu.sh uninstall-ubuntu.sh install-ubuntu-user.sh
    bash ./install-ubuntu.sh
    ```

- UI shows “No key” on Linux
  - Ensure the user Secret Service is running (installer sets it up):
    ```bash
    systemctl --user status gnome-keyring --no-pager
    busctl --user list | grep org.freedesktop.secrets
    ```
  - Seed keys via UI or curl as shown above. Verify with:
    ```bash
    curl -s http://127.0.0.1:8080/api/providers/state
    ```

- `/api/remote/ask` returns 502 Bad Gateway
  - Usually the upstream tunnel/key/slug or timeout:
    - Confirm remote status: `curl -s http://127.0.0.1:8080/api/remote/status | jq` (online should be true)
    - Confirm API key directly against the tunnel:
      ```bash
      BASE="https://<your_trycloudflare>.trycloudflare.com"
      KEY={{ANYTHINGLLM_KEY}}
      curl -s -H "Authorization: Bearer $KEY" "$BASE/api/v1/workspaces" | jq
      ```
    - Confirm workspace slug exists via `/api/remote/workspaces` and save it.
    - Increase `timeouts.chat_secs` to e.g. 60 and retry.

- Remote error: “Workspace <name> is not a valid workspace”
  - Use the exact `slug` returned by `/api/remote/workspaces` (often underscores, not hyphens), then save with `/api/remote/config`.

- Remote error: “No valid api key found.”
  - Ensure you saved the correct Developer API key for that AnythingLLM instance (keys are per instance). Save with `/api/remote/key` and test direct:
    ```bash
    curl -s -H "Authorization: Bearer $KEY" "$BASE/api/v1/workspaces"
    ```

- Groq 413 “Request too large / TPM limit”
  - Reduce message size, avoid sending long prompts/history, or switch models/tier.

- Ports and firewall
  - Voxora listens on 8080. To allow inbound:
    ```bash
    sudo ufw allow 8080/tcp
    sudo ufw reload
    ```

---

This workflow mirrors Windows behavior: for Linux, run as a user service to access the user keyring, seed provider keys once, select your AnythingLLM tunnel and slug, adjust timeouts if needed, and you’re good to go.
2. **Credentials:** Keep `~/mytools/data/goserver/credential.txt` secure
3. **HTTPS:** Consider using nginx as reverse proxy with SSL
4. **User:** Service runs as your user account (not root)

## Support

For issues, check:
- Service logs: `sudo journalctl -u voxora -f`
- Go-server logs: `/opt/voxora/go-server-error.log`
- Debug log: `/opt/voxora/debug.log`
