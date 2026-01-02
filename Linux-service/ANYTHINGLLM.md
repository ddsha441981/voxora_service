# AnythingLLM + Voxora: Quick Tunnel Setup

This guide shows how to expose your local AnythingLLM over Cloudflare Tunnel and connect it from Voxora.
It keeps your existing EN/HI pipeline untouched; the Remote tab and Startup page are independent.

---

## Overview
- AnythingLLM runs locally on each machine (Windows or Ubuntu) on `http://localhost:3001`.
- `cloudflared` runs on the same machine and creates a public HTTPS tunnel URL.
- Voxora uses that tunnel URL, plus your AnythingLLM API key, to call the Developer API.
- You select the workspace and the default chat mode (Chat or Streaming) in Settings → Remote.

Notes
- Quick Tunnel (trycloudflare) is free and requires no domain, but its URL changes each time you start it. Keep `cloudflared` running or restart+reconnect when needed.
- Later, you can switch to a Named Tunnel for a stable hostname.

---

## 1) Start AnythingLLM locally
Run it however you prefer. Two examples:

### Windows (Docker Desktop)
```powershell
# Example: AnythingLLM listening on localhost:3001
# Replace volumes to your paths
docker run -d --name anythingllm -p 3001:3001 \
  -v C:\\data\\anythingllm:/app/server/storage \
  mintplexlabs/anythingllm:latest
```

### Ubuntu (Docker)
```bash
# Example: AnythingLLM listening on localhost:3001
# Replace volumes to your paths
sudo docker run -d --name anythingllm -p 3001:3001 \
  -v /opt/anythingllm:/app/server/storage \
  mintplexlabs/anythingllm:latest
```

---

## 2) Start a Cloudflare Quick Tunnel on the same host
Install `cloudflared` and forward local port 3001.

### Windows (PowerShell)
```powershell
# Install (winget) – or download from Cloudflare if winget unavailable
winget install Cloudflare.cloudflared -s winget

# Run a Quick Tunnel to http://localhost:3001
cloudflared tunnel --url http://localhost:3001
# Copy the printed https://xxxxx.trycloudflare.com URL
```

### Ubuntu
```bash
# Install
curl -fsSL https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/cloudflared-install.sh | bash

# Run a Quick Tunnel to http://localhost:3001
cloudflared tunnel --url http://localhost:3001
# Copy the printed https://xxxxx.trycloudflare.com URL
```

### Docker sidecar (optional, same machine)
```yaml
version: "3.8"
services:
  anythingllm:
    image: mintplexlabs/anythingllm:latest
    ports: ["3001:3001"]
    volumes:
      - ./storage:/app/server/storage
  cloudflared:
    image: cloudflare/cloudflared:latest
    command: tunnel --url http://host.docker.internal:3001
    network_mode: host  # or bridge with proper target hostname
```

---

## 3) Get the AnythingLLM API key
In the AnythingLLM UI, find the Developer API key (typically in Settings → API or Developer).
Keep it handy; Voxora stores it in your OS keyring.

---

## 4) Connect from Voxora (Startup page)
1) Launch Voxora and open the Startup page.
2) Paste the tunnel URL (e.g., `https://abc123.trycloudflare.com`).
3) Paste the AnythingLLM API key.
4) Click Test (should show Auth OK), then Connect.

This stores the URL for the session and the key in the keyring.

---

## 5) Select workspace and mode (Settings → Remote)
- Open Settings → Remote.
- Workspaces auto-load; if none saved yet, the first one is saved by default.
- Pick your desired workspace from the dropdown.
- Choose Mode: Chat or Streaming Chat.
- Click “Save Remote” (you’ll see “Saved!” briefly).

What is saved
- API key: keyring item `anythingllm`.
- Workspace slug: keyring item `anythingllm_workspace`.
- Defaults (mode): `settings.json` → `remote.stream_default` (true for Streaming; false = Chat).

---

## 6) Use from Voxora
- Ask AI panel (future wiring) will respect the selected mode.
- Remote calls go through Voxora’s backend → tunnel URL, keeping your key server-side (no CORS leaks).

---

## Troubleshooting
- 400 on `/api/remote/workspace` in Voxora: either no remote selected (Startup → Connect) or no slug saved (Settings → Remote). With auto-load, opening Remote should fix by saving the first workspace.
- 401 from remote: invalid API key. Re-save the key in Settings → Remote.
- Tunnel URL not reachable: ensure `cloudflared` is running on the AnythingLLM host and points to `http://localhost:3001`.
- UI login prompts when visiting the tunnel directly: the Developer API still works with the API key; Voxora uses Bearer auth, not the UI login.

---

## Security notes
- Keep Voxora bound to localhost; only the tunnel URL is public.
- API key is stored in the OS keyring; the workspace slug is stored in keyring as well.
- For stronger access controls, consider Cloudflare Access with service tokens; Voxora can be extended to send the required headers.

---

## Later: Named Tunnels (stable hostname)
When ready to use a stable hostname, create a Cloudflare Named Tunnel and DNS record, then point it to `http://localhost:3001`. Voxora configuration is the same—use the Named Tunnel URL instead of trycloudflare.
