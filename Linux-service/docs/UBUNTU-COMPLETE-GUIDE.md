# Voxora on Ubuntu: Complete Guide (User Service, Keyring, Remote)

This guide shows how to build, install, and run Voxora on Ubuntu without code changes, including user keyring setup and Remote (AnythingLLM) integration.

---

## 1) Build portable package

```bash
./create-portable-ubuntu.sh
```

This creates `dist/voxora-portable-linux` with:
- `voxora-service`
- `logs/`, optional `bin/`, `data/`, `static/`
- Installers: `install-ubuntu.sh` (system service) and `install-ubuntu-user.sh` (user service)

---

## 2) Run as a systemd USER service (recommended)

Advantages: uses your user keyring (secrets persist securely), simpler screen-capture later, auto-restarts and logs via systemd.

### 2.1 One-time prerequisites (only if missing)
```bash
sudo apt-get update
sudo apt-get install -y dbus-user-session gnome-keyring libsecret-tools
```
Keep your user services running after logout:
```bash
sudo loginctl enable-linger "$USER"
```

### 2.2 Install Voxora as a user service
```bash
cd dist/voxora-portable-linux
./install-ubuntu-user.sh
```
This also sets up a user Secret Service (org.freedesktop.secrets) and verifies it is running.

Status/logs:
```bash
systemctl --user status voxora --no-pager
journalctl --user -u voxora -f
```

---

## 3) Store provider API keys (one time per user/machine)

You can use the UI (Settings → Providers / Remote), or seed via HTTP. Using curl (run as the same user that runs the service):

```bash
# Replace with your real secrets (no braces in the values)
GROQ_KEY={{GROQ_KEY}}
GEMINI_KEY={{GEMINI_KEY}}
OPENROUTER_KEY={{OPENROUTER_KEY}}
ANYTHINGLLM_KEY={{ANYTHINGLLM_KEY}}

# Providers
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

Verify detection:
```bash
curl -s http://127.0.0.1:8080/api/providers/state
```

---

## 4) Remote (AnythingLLM) integration

### 4.1 Select your tunnel
Startup page works, or via API:
```bash
curl -s -X POST http://127.0.0.1:8080/api/remote/select \
  -H 'Content-Type: application/json' \
  -d '{"server":"linux","url":"https://<your_trycloudflare>.trycloudflare.com"}'

curl -s http://127.0.0.1:8080/api/remote/status | jq
```
`online: true` indicates the URL is reachable.

### 4.2 Pick the correct workspace slug
List available workspaces:
```bash
curl -s http://127.0.0.1:8080/api/remote/workspaces | jq
```
Save the exact slug (often underscores, not hyphens):
```bash
curl -s -X POST http://127.0.0.1:8080/api/remote/config \
  -H 'Content-Type: application/json' \
  -d '{"slug":"voxora_docs"}'

curl -s http://127.0.0.1:8080/api/remote/workspace | jq
```

### 4.3 Ask the remote
```bash
# Chat
curl -s -X POST http://127.0.0.1:8080/api/remote/ask \
  -H 'Content-Type: application/json' \
  -d '{"input":"hello","stream":false,"mode":"chat"}'

# Query (vector-search first)
curl -s -X POST http://127.0.0.1:8080/api/remote/ask \
  -H 'Content-Type: application/json' \
  -d '{"input":"hello","stream":false,"mode":"query"}'
```

---

## 5) Increase timeouts (if remote is slow)

Edit the settings file in the running folder (user service → `dist/voxora-portable-linux/data/settings.json`):
```json
{
  "timeouts": {
    "health_check_secs": 5,
    "workspace_secs": 15,
    "chat_secs": 60
  }
}
```
Restart:
```bash
systemctl --user restart voxora   # user service
# or
sudo systemctl restart voxora     # system service
```

---

## 6) Alternative: System service (root)

```bash
cd dist/voxora-portable-linux
sudo ./install-ubuntu.sh
sudo systemctl enable --now voxora
sudo journalctl -u voxora -f
```
Note: system services don’t automatically see the user keyring. Prefer the user service. If you keep the system service, run the user `gnome-keyring` unit and seed keys as your user.

---

## 7) Uninstall

- User service:
```bash
systemctl --user disable --now voxora
rm -f ~/.config/systemd/user/voxora.service
systemctl --user daemon-reload
```
- System service:
```bash
sudo ./uninstall-ubuntu.sh
```

---

## 8) Troubleshooting

- Scripts show “cannot execute: required file not found”
  - Likely CRLF line endings. Fix and run:
    ```bash
    sed -i 's/\r$//' install-ubuntu.sh uninstall-ubuntu.sh install-ubuntu-user.sh
    bash ./install-ubuntu.sh
    ```

- UI shows “No key”
  - Ensure user Secret Service is running:
    ```bash
    systemctl --user status gnome-keyring --no-pager
    busctl --user list | grep org.freedesktop.secrets
    ```
  - Seed keys via UI or curl; verify:
    ```bash
    curl -s http://127.0.0.1:8080/api/providers/state
    ```

- /api/remote/ask returns 502
  - Check remote status: `curl -s http://127.0.0.1:8080/api/remote/status | jq`
  - Test API key directly:
    ```bash
    BASE="https://<your_trycloudflare>.trycloudflare.com"
    KEY={{ANYTHINGLLM_KEY}}
    curl -s -H "Authorization: Bearer $KEY" "$BASE/api/v1/workspaces" | jq
    ```
  - Confirm workspace slug via `/api/remote/workspaces` and save it.
  - Increase timeouts as shown above.

- “Workspace <name> is not a valid workspace”
  - Use the exact `slug` from `/api/remote/workspaces` and save it via `/api/remote/config`.

- “No valid api key found.”
  - Ensure the Developer API key belongs to the selected AnythingLLM instance; save with `/api/remote/key` and re-test `/workspaces` directly.

- Groq 413 “Request too large / TPM limit”
  - Reduce message size/history, change model, or upgrade tier.

- Ports & firewall
  - Voxora listens on 8080. Allow inbound if needed:
    ```bash
    sudo ufw allow 8080/tcp
    sudo ufw reload
    ```

---

## 9) Security notes
- Keep the service bound to trusted interfaces.
- Manage secrets via the OS keyring (libsecret) — never commit keys to disk.
- For AnythingLLM tunnels, prefer stable Named Tunnels over quick-tunnels for production.
