#!/usr/bin/env bash
# Install Voxora as a systemd USER service (no root), using the current folder as WorkingDirectory
# - Ensures a user Secret Service (org.freedesktop.secrets) is running
# - Installs/updates the user unit
# - Gives clear guidance to seed API keys
# Usage: ./install-ubuntu-user.sh

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UNIT_DIR="$HOME/.config/systemd/user"
UNIT_PATH="$UNIT_DIR/voxora.service"
KEYRING_UNIT="$UNIT_DIR/gnome-keyring.service"

mkdir -p "$UNIT_DIR"

# 1) Ensure user secret service is available (gnome-keyring-daemon secrets component)
need_pkg_note=false
command -v busctl >/dev/null 2>&1 || need_pkg_note=true
command -v secret-tool >/dev/null 2>&1 || need_pkg_note=true
command -v gnome-keyring-daemon >/dev/null 2>&1 || need_pkg_note=true
if [[ "$need_pkg_note" == true ]]; then
  echo "Note: You may need to install packages: sudo apt-get install -y dbus-user-session gnome-keyring libsecret-tools"
fi

# Create user keyring unit if missing
if [[ ! -f "$KEYRING_UNIT" ]]; then
  cat > "$KEYRING_UNIT" <<'EOF_KR'
[Unit]
Description=User Secret Service (gnome-keyring-daemon)
After=default.target

[Service]
Type=simple
ExecStart=/usr/bin/gnome-keyring-daemon --foreground --components=secrets
Restart=on-failure

[Install]
WantedBy=default.target
EOF_KR
fi

# Start/enable the keyring unit
systemctl --user daemon-reload
systemctl --user enable --now gnome-keyring.service || true

# Verify org.freedesktop.secrets is present on the user bus
if ! busctl --user list 2>/dev/null | grep -q "org.freedesktop.secrets"; then
  echo "ERROR: Secret Service (org.freedesktop.secrets) is not available on the user bus." >&2
  echo "- Ensure packages installed: sudo apt-get install -y dbus-user-session gnome-keyring libsecret-tools" >&2
  echo "- Then run: systemctl --user enable --now gnome-keyring.service" >&2
  exit 1
fi

# 2) Write/update Voxora user service unit
cat > "$UNIT_PATH" <<'EOF'
[Unit]
Description=Voxora Service (user)
After=network.target

[Service]
Type=simple
WorkingDirectory=REPLACE_WORKDIR
ExecStart=REPLACE_WORKDIR/voxora-service
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=ENGLISH_GO_SERVER_URL=ws://127.0.0.1:8085/ws
Environment=HINDI_GO_SERVER_URL=ws://127.0.0.1:8086/ws

[Install]
WantedBy=default.target
EOF

# Inject absolute WorkingDirectory
sed -i "s|REPLACE_WORKDIR|$ROOT_DIR|g" "$UNIT_PATH"

# 3) Encourage linger so it survives logout
if ! loginctl show-user "$USER" 2>/dev/null | grep -q "Linger=yes"; then
  echo "Tip: to keep running after logout, enable linger: sudo loginctl enable-linger $USER"
fi

# 4) Start Voxora
systemctl --user daemon-reload
systemctl --user enable --now voxora

# 5) Post-install guidance: seed/verify keys
echo "Installed user service: $UNIT_PATH"
echo "Status: systemctl --user status voxora --no-pager"
echo "Logs:   journalctl --user -u voxora -f"

# Quick provider-state check if curl exists
if command -v curl >/dev/null 2>&1; then
  echo "Checking provider key state..."
  curl -s http://127.0.0.1:8080/api/providers/state || true
  echo
fi

echo "If providers show has_key: false, store keys on this machine (same user):"
echo "- Via API (recommended):"
echo "  curl -s -X POST http://127.0.0.1:8080/api/providers/groq/key -H 'Content-Type: application/json' -d '{"api_key":"<GROQ_KEY>"}'"
echo "  curl -s -X POST http://127.0.0.1:8080/api/providers/gemini/key -H 'Content-Type: application/json' -d '{"api_key":"<GEMINI_KEY>"}'"
echo "  curl -s -X POST http://127.0.0.1:8080/api/providers/openrouter/key -H 'Content-Type: application/json' -d '{"api_key":"<OPENROUTER_KEY>"}'"
echo "  curl -s -X POST http://127.0.0.1:8080/api/remote/key -H 'Content-Type: application/json' -d '{"api_key":"<ANYTHINGLLM_KEY>"}'"
echo "- Or via secret-tool (non-interactive API not required):"
echo "  secret-tool store --label='Voxora Groq' service voxora-service username groq"
echo "  secret-tool store --label='Voxora Gemini' service voxora-service username gemini"
echo "  secret-tool store --label='Voxora OpenRouter' service voxora-service username openrouter"
echo "  secret-tool store --label='Voxora AnythingLLM' service voxora-service username anythingllm"
echo "  secret-tool store --label='Voxora Workspace' service voxora-service username anythingllm_workspace"

# Final hint to re-check after seeding
echo "After seeding keys: systemctl --user restart voxora && curl -s http://127.0.0.1:8080/api/providers/state"
