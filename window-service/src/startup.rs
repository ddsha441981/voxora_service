use serde::{Deserialize, Serialize};

pub fn startup_html() -> String {
    r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Voxora • Startup</title>
    <link rel="icon" href="data:">
    <style>
      :root { --bg:#0b0f1a; --fg:#e6e6e6; --muted:#8aa; --blue:#1e88e5; --green:#43a047; --border:#223356; }
      html, body { height:100%; }
      body { margin:0; background:#000; color:var(--fg); font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial; display:grid; place-items:center; }
      .wrap { width:min(720px, 92vw); background:#000; border:1px solid var(--border); border-radius:12px; padding:16px; box-shadow:0 12px 24px rgba(0,0,0,.5); }
      h1 { margin:0 0 8px; font-size:20px; font-weight:700; }
      p { margin: 6px 0 12px; color: var(--muted); }
      label { font-size:12px; color:#a8b3cf; display:block; margin:6px 0; }
      select, input[type="text"] { width:100%; background:#000000; color:#e6f0ff; border:1px solid var(--border); border-radius:8px; padding:8px 10px; }
      .row { display:grid; grid-template-columns: 1fr; gap:10px; }
      .btns { display:flex; gap:10px; justify-content:flex-end; margin-top:10px; }
      .btn { background:#1a2233; border:1px solid #2a3d66; color:#cfe1ff; padding:8px 12px; border-radius:8px; cursor:pointer; }
      .btn.primary { background:#1e88e5; color:#fff; border-color:#1a66b8; }
      .status { margin-top:10px; font-size:13px; color:#aac; }
      .ok { color:#8bc34a; }
      .bad { color:#ef5350; }
      .nav { margin-top:14px; display:flex; gap:10px; }
      .link { color:#8ab4ff; text-decoration:none; }
      .hint { font-size:12px; color:#8aa; }
    </style>
  </head>
  <body>
    <div class="wrap">
      <h1>Startup</h1>
      <p>Select your server and tunnel URL (cloudflared). This is independent of the EN/HI pipeline.</p>
      <div class="row">
        <div>
          <label for="server">Select Your Server</label>
          <select id="server">
            <option value="linux">Linux Server (AnythingLLM in Docker)</option>
            <option value="windows">Windows Server (AnythingLLM in Docker)</option>
          </select>
        </div>
        <div>
          <label for="tunnel">Tunnel URL (from cloudflared)</label>
          <input id="tunnel" type="text" placeholder="https://xxxxx.trycloudflare.com" />
          <div class="hint">Run cloudflared on the same host as AnythingLLM and paste the public URL here.</div>
        </div>
        <div>
          <label for="apikey">AnythingLLM API Key</label>
          <input id="apikey" type="password" placeholder="Enter AnythingLLM API key" autocomplete="off" />
        </div>
      </div>
      <div class="btns">
        <button id="test" class="btn" type="button">Test</button>
        <button id="disconnect" class="btn" type="button">Disconnect</button>
        <button id="connect" class="btn primary" type="button" disabled>Connect</button>
      </div>
      <div id="status" class="status">Status: Unknown</div>
      <div class="nav">
        <a class="link" href="/app">Open Control UI</a>
        <a class="link" href="/mobile">Mobile</a>
      </div>
    </div>

    <script>
      (function(){
        const server = document.getElementById('server');
        const tunnel = document.getElementById('tunnel');
        const status = document.getElementById('status');
        const btnConnect = document.getElementById('connect');
        const btnDisconnect = document.getElementById('disconnect');
        const btnTest = document.getElementById('test');
        const apikey = document.getElementById('apikey');

        function setStatus(text, ok){
          status.textContent = 'Status: ' + text;
          status.classList.toggle('ok', !!ok);
          status.classList.toggle('bad', ok === false);
        }

        async function refresh(){
          try {
            const r = await fetch('/api/remote/status');
            if (!r.ok) throw new Error('status http ' + r.status);
            const s = await r.json();
            if (s && s.selected) {
              server.value = s.selected.server || 'linux';
              tunnel.value = s.selected.url || '';
              setStatus(s.online ? 'Connected' : 'Selected (offline)', !!s.online);
            } else {
              setStatus('Idle', null);
            }
          } catch(e) {
            setStatus('Error fetching status', false);
          }
        }

        btnTest.addEventListener('click', async () => {
          const url = tunnel.value.trim();
          const key = apikey.value.trim();
          if (!url) { setStatus('Please enter tunnel URL', false); return; }
          if (!key) { setStatus('Please enter API key', false); return; }
          try {
            const r = await fetch('/api/remote/test-auth', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ url, api_key: key }) });
            if (r.ok) { setStatus('Auth OK', true); btnConnect.disabled = false; }
            else if (r.status === 401) { setStatus('Invalid API key', false); btnConnect.disabled = true; }
            else { const t = await r.text(); setStatus('Test failed: ' + t, false); btnConnect.disabled = true; }
          } catch(e){ setStatus('Test error', false); btnConnect.disabled = true; }
        });

        btnConnect.addEventListener('click', async () => {
          const body = { server: server.value, url: tunnel.value.trim() };
          if (!body.url) { setStatus('Please enter tunnel URL', false); return; }
          try {
            const r = await fetch('/api/remote/select', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) });
            if (r.ok) { setStatus('Connected', true); } else { const t = await r.text(); setStatus('Connect failed: ' + t, false); }
          } catch(e){ setStatus('Connect error', false); }
        });
        btnDisconnect.addEventListener('click', async () => {
          try {
            const r = await fetch('/api/remote/select', { method: 'DELETE' });
            if (r.ok) { setStatus('Disconnected', null); tunnel.value=''; btnConnect.disabled = true; }
          } catch(e){ setStatus('Disconnect error', false); }
        });

        refresh();
      })();
    </script>
  </body>
</html>"#.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RemoteSelection { pub server: String, pub url: String }