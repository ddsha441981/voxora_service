# Cloudflare Named Tunnel + Voxora

This doc shows how to create a stable hostname for your local AnythingLLM using Cloudflare Named Tunnels, and how to use it in Voxora.

Prereqs
- Cloudflare account (Free plan is fine)
- A domain managed in Cloudflare DNS (for the stable hostname)
- AnythingLLM running on the target host at http://localhost:3001

---

## 1) Install and login
On the host that runs AnythingLLM:

Windows (PowerShell)
```powershell
winget install Cloudflare.cloudflared -s winget
cloudflared login
# Follow the browser flow to authorize your account
```

Ubuntu
```bash
curl -fsSL https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/downloads/cloudflared-install.sh | bash
cloudflared login
# Follow the browser flow to authorize your account
```

---

## 2) Create a Named Tunnel
```bash
cloudflared tunnel create voxora-llm
# This outputs a tunnel UUID and creates a credentials file
```

Find the credentials path (example):
- Windows: %USERPROFILE%\.cloudflared\<TUNNEL_ID>.json
- Linux: ~/.cloudflared/<TUNNEL_ID>.json

---

## 3) Configure the tunnel to route to AnythingLLM
Create (or edit) ~/.cloudflared/config.yml (Windows: %USERPROFILE%\.cloudflared\config.yml):
```yaml
# config.yml
tunnel: voxora-llm
credentials-file: C:\\Users\\YOU\\.cloudflared\\<TUNNEL_ID>.json  # adjust path

ingress:
  - hostname: llm.yourdomain.com
    service: http://localhost:3001
  - service: http_status:404
```
Replace hostname with your domain.

---

## 4) Create the DNS record
```bash
cloudflared tunnel route dns voxora-llm llm.yourdomain.com
```
This creates a proxied CNAME in Cloudflare DNS pointing to your tunnel.

---

## 5) Run the tunnel
Foreground (test):
```bash
cloudflared tunnel run voxora-llm
```
Service (Windows):
```powershell
cloudflared service install
# Then configure the service to run: cloudflared tunnel run voxora-llm
# Or set it in the service parameters depending on install method
```
Service (Ubuntu):
```bash
sudo cloudflared service install
# Or create a systemd unit that runs: cloudflared tunnel run voxora-llm
```

Once running, https://llm.yourdomain.com will proxy to http://localhost:3001.

---

## 6) Configure Voxora to use the Named Tunnel
1) Open Voxora → Settings → Remote.
2) Enter the Named Tunnel URL in "Named Tunnel URL" and tick "Use Named Tunnel (remember URL)".
3) Ensure your AnythingLLM API key is saved (Save Key). Workspaces will auto-load; choose one and click Save Remote.
4) On the Startup page, Voxora will prefill the tunnel URL from your saved named config. Click Test → Connect.

Notes
- If you later disable the checkbox, Voxora will revert to session-based URLs (for Quick Tunnel usage).
- Named URL is stored in settings.json; API key and workspace slug stay in keyring.

---

## Optional: Cloudflare Access (Zero Trust)
If you protect the hostname with Access, create a service token and configure Voxora to send CF-Access headers (future enhancement). For now, keep the Developer API openly reachable and rely on the API key for auth.

---

## Troubleshooting
- 525/522 errors: ensure the tunnel is running and config.yml points to http://localhost:3001.
- DNS not resolving: verify the CNAME and propagation in Cloudflare DNS.
- AnythingLLM returns 401: verify the Developer API key in Voxora → Remote.
