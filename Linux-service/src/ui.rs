use qrcode::QrCode;

pub fn landing_html() -> String {
    r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Voxora</title>
    <link rel="icon" href="data:">
    <style>
      :root { --bg:#000000; --fg:#e6e6e6; --muted:#a8b3cf; --blue:#1e88e5; --green:#43a047; }
      html, body { height:100%; }
      body { margin:0; background:#000000; color:var(--fg); font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial; display:grid; place-items:center; }\r
      .hero { text-align:center; padding:24px; max-width:860px; }
      .logo { display:flex; align-items:center; justify-content:center; gap:10px; margin-bottom:10px; }
      .logo .mark { width:40px; height:40px; display:inline-block; }
      .hero h1 { margin:0 0 6px; font-size:28px; font-weight:800; letter-spacing:.2px; }
      .tagline { margin:0 0 18px; color:var(--muted); font-size:14px; }
      .actions { display:flex; gap:16px; justify-content:center; flex-wrap:wrap; }
      .btn { padding:12px 18px; border-radius:10px; text-decoration:none; color:#fff; display:inline-flex; align-items:center; gap:10px; font-weight:600; box-shadow: 0 6px 18px rgba(0,0,0,.25); transition: transform .15s ease, opacity .2s ease; }
      .btn:hover { transform: translateY(-1px); opacity:.95; }
      .btn.laptop { background: var(--blue); }
      .btn.mobile { background: var(--green); }
      .ico { display:inline-flex; width:18px; height:18px; }
      .ico svg, .ico img { width:18px; height:18px; display:block; }
      .ico.float { animation: float 3s ease-in-out infinite; }
      @keyframes float { 0% { transform: translateY(0px);} 50% { transform: translateY(-2px);} 100% { transform: translateY(0px);} }
      @media (prefers-reduced-motion: reduce) { .ico.float { animation: none; } }
    </style>
  </head>
  <body>
    <div class="hero">
      <div class="logo">
        <img class="mark" src="/static/logo.svg" alt="Voxora logo">
      </div>
      <h1>Voxora</h1>
      <p class="tagline">Voice-first control and transcripts in English and Hindi</p>
      <div class="actions">
        <a href="/app" class="btn laptop">
          <picture class="ico float" aria-hidden="true">
            <source srcset="/static/laptop.png" type="image/png">
            <img src="/static/laptop.svg" alt=""/>
          </picture>
          <span>Laptop (open control UI)</span>
        </a>
        <a href="/mobile" class="btn mobile">
          <picture class="ico float" aria-hidden="true">
            <source srcset="/static/mobile.png" type="image/png">
            <img src="/static/mobile.svg" alt=""/>
          </picture>
          <span>Mobile (QR code)</span>
        </a>
      </div>
    </div>
  </body>
</html>"#.to_string()
}

pub fn app_html() -> String {
    r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Voxora Service</title><link rel="icon" href="data:">
    <style>
      :root {
        --bg: #0b0f1a; --fg: #e6e6e6; --muted:#8aa; --accent:#4caf50; --accent2:#2196f3; --danger:#e53935;
      }
      html { background:#000 !important; }
      body { margin:0; font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial; background:#000 !important; color:var(--fg); }
      .container { max-width: 960px; margin: 0 auto; padding: 24px; background:#000; }
      h1 { margin: 0 0 12px; font-size: 22px; font-weight: 600; }
      .status { margin: 4px 0 8px; color: var(--muted); }
      .toolbar { display:flex; gap:16px; align-items:center; margin:12px 0 16px; flex-wrap: wrap; }
      .btn { width:56px; height:56px; border-radius:50%; border:none; cursor:pointer; display:flex; align-items:center; justify-content:center; transition: transform .15s ease, background .2s ease; background:#1a2233; color:#cfe1ff; position:relative; }
      .btn:hover { transform: translateY(-1px); }
      .btn.active.en { background: var(--accent2); color:#fff; }
      .btn.active.hi { background: var(--accent); color:#fff; }
      .btn.active.tr { background:#7b1fa2; color:#fff; }
      .btn.active.ai { background:#9c27b0; color:#fff; }
      .btn.active.cap { background: var(--danger); color:#fff; }
      .btn.active.sess { background:#00acc1; color:#fff; }
      .btn.active.set { background:#ff9800; color:#fff; }
      .btn .label { font-size: 10px; display:block; margin-top: 4px; color: var(--muted); text-align:center; }
      .stack { display:flex; flex-direction:column; align-items:center; gap:2px; }
      .icon { width:26px; height:26px; }
      .emojiIcon { font-size:22px; line-height:1; }
      .ws-dot { position:absolute; top:6px; right:6px; width:8px; height:8px; border-radius:50%; background:#7a2730; box-shadow:0 0 0 2px rgba(0,0,0,.25); transition: all 0.3s ease; }
      .ws-dot.on { background:#3ecf5e; }
      /* Animated pulse on WebSocket dots */
      @keyframes dotPulse {
        0%, 100% { box-shadow: 0 0 0 2px rgba(0,0,0,.25), 0 0 0 0 currentColor; }
        50% { box-shadow: 0 0 0 2px rgba(0,0,0,.25), 0 0 0 4px transparent; }
      }
      .ws-dot.on {
        animation: dotPulse 2s ease-in-out infinite;
      }
      .hidden { display:none !important; }
      .chips { display:flex; gap:10px; flex-wrap:wrap; align-items:center; }
      .chip-vad { display:flex; align-items:center; gap:8px; padding:6px 10px; border-radius:999px; background:#000000; border:1px solid #223356; font-size:12px; color:#cfe1ff; }
      .chip-vad .tag { font-weight:600; opacity:.9; }
      .chip-vad .state { padding:2px 8px; border-radius:999px; font-weight:600; }
      .chip-vad .state.speech { background:#1f5228; color:#c9f7d1; }
      .chip-vad .state.silence { background:#3a3f55; color:#d0d4e0; }
      .chip-vad .state.idle { background:#444b66; color:#e2e6f2; }
      /* Status indicator animations */
      @keyframes breathe {
        0%, 100% { opacity: 1; transform: scale(1); }
        50% { opacity: 0.7; transform: scale(0.98); }
      }
      .chip-vad .state.idle {
        animation: breathe 3s ease-in-out infinite;
      }
      @keyframes statePulse {
        0%, 100% { box-shadow: 0 0 0 0 transparent; }
        50% { box-shadow: 0 0 8px 2px currentColor; }
      }
      .chip-vad .state.speech {
        animation: statePulse 1.5s ease-in-out infinite;
      }
      .chip-vad.en { border-color:#2a3e7a; }
      .chip-vad.hi { border-color:#2e6a3a; }
      .chip-vad .kv { color:#aac; }
      .chip-vad .sep { color:#3a4a6a; }

      /* Mobile Navbar */
      .mobile-navbar { display: none; position: fixed; top: 0; left: 0; right: 0; height: 56px; background: rgba(11, 15, 26, 0.98); backdrop-filter: blur(12px); border-bottom: 1px solid #223356; z-index: 200; align-items: center; justify-content: space-between; padding: 0 16px; }
      .mobile-navbar .nav-left { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--muted); }
      .mobile-navbar .nav-left .status-dot { width: 8px; height: 8px; border-radius: 50%; background: #444b66; transition: all 0.3s ease; }
      .mobile-navbar .nav-left .status-dot.active-en { background: #2196f3; animation: dotPulse 2s ease-in-out infinite; }
      .mobile-navbar .nav-left .status-dot.active-hi { background: #4caf50; animation: dotPulse 2s ease-in-out infinite; }
      .mobile-navbar .nav-center { font-size: 18px; font-weight: 600; color: var(--fg); }
      .mobile-navbar .nav-right { cursor: pointer; padding: 8px; font-size: 24px; color: var(--muted); }
      .mobile-navbar .nav-right:hover { color: var(--fg); }

      /* Bottom Sheet for Stats */
      .stats-sheet { display: none; position: fixed; bottom: 0; left: 0; right: 0; background: var(--bg); border-top: 1px solid #223356; border-radius: 16px 16px 0 0; z-index: 300; max-height: 60vh; overflow: auto; transform: translateY(100%); transition: transform 0.3s ease; box-shadow: 0 -4px 20px rgba(0,0,0,0.5); }
      .stats-sheet.open { transform: translateY(0); }
      .stats-sheet .sheet-header { padding: 16px; border-bottom: 1px solid #223356; display: flex; align-items: center; justify-content: space-between; }
      .stats-sheet .sheet-title { font-size: 16px; font-weight: 600; }
      .stats-sheet .sheet-close { cursor: pointer; font-size: 24px; color: var(--muted); padding: 4px 8px; }
      .stats-sheet .sheet-content { padding: 16px; }
      .stats-sheet .stat-section { margin-bottom: 20px; }
      .stats-sheet .stat-header { display: flex; align-items: center; gap: 8px; font-size: 15px; font-weight: 600; margin-bottom: 8px; }
      .stats-sheet .stat-header.en { color: #2196f3; }
      .stats-sheet .stat-header.hi { color: #4caf50; }
      .stats-sheet .stat-row { display: flex; justify-content: space-between; padding: 6px 12px; background: #000000; border: 1px solid #223356; border-radius: 6px; margin-bottom: 6px; font-size: 13px; }
      .stats-sheet .stat-label { color: var(--muted); }
      .stats-sheet .stat-value { color: var(--fg); font-weight: 500; }
      .stats-sheet .stat-divider { height: 1px; background: #223356; margin: 16px 0; }
      .stats-overlay { display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 250; }
      .stats-overlay.open { display: block; }
      .transcript { border:1px solid #203049; background:#000000; border-radius:8px; padding:12px; height: 50vh; overflow:auto; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace; font-size: 13px; line-height:1.5; }
.askai { border:1px solid #203049; background:#000000; border-radius:8px; padding:12px; height: 50vh; overflow:auto; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace; font-size: 13px; line-height:1.5; }
      .askout { white-space: pre-wrap; font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial; font-size: 14px; line-height: 1.6; color:#e6e6e6; }
      .askout h1 { font-size: 22px; font-weight: 800; margin: 18px 0 10px; border-bottom:1px solid #223356; padding-bottom:6px; }
      .askout h2 { font-size: 18px; font-weight: 700; margin: 16px 0 8px; }
      .askout h3 { font-size: 16px; font-weight: 600; margin: 14px 0 6px; }
      .askout p { margin: 0 0 10px; color:#d6e2ff; }
      .askout ul { list-style: disc inside; margin:10px 0; color:#cfe1ff; }
      .askout ol { list-style: decimal inside; margin:10px 0; color:#cfe1ff; }
      .askout li { margin-left: 10px; }
      .askout blockquote { border-left:4px solid #1e88e5; padding:8px 12px; background:#050505; color:#cfd8ff; margin:10px 0; }
      .askout table { border-collapse: collapse; width: 100%; margin:10px 0; }
      .askout th, .askout td { border:1px solid #2a3a5a; padding:8px 10px; }
      .askout thead { background:#0a0a0a; }
      .askout pre { position:relative; background:#000000; border:1px solid #223356; border-radius:8px; padding:12px; overflow:auto; }
      .askout :not(pre) > code { background:#0f0f0f; color:#8ad; padding:2px 6px; border-radius:6px; font-size: 13px; }
      .askout code { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, "Liberation Mono", monospace; }
      .askout .copy-btn { position:absolute; top:8px; right:8px; background:#1f2937; border:1px solid #2f3b57; color:#dfe7ff; font-size:12px; padding:2px 8px; border-radius:6px; cursor:pointer; opacity:.0; transition:opacity .15s ease; }
      .askout pre:hover .copy-btn { opacity:1; }
      .ai-meta { color:#8aa; margin-bottom:8px; font-size:12px; }
      .para { margin: 0 0 10px; color:#d6e2ff; }
      .para .seg { display:inline; }
      .para .sep { color:#52607a; margin: 0 4px; }
      .chip { display:inline-block; padding:2px 8px; border-radius:999px; font-size:11px; margin-right:6px; }
      .chip.en { background:#1a2a5a; color:#bcd4ff; }
      .chip.hi { background:#1f5228; color:#c9f7d1; }
      .lang-tag { margin: 6px 0 6px; }
      .lang-rule { height: 2px; border: none; background: #2a3e7a; margin: 6px 0 10px; }
      .lang-rule.en { background: #2196f3; }
      .lang-rule.hi { background: #4caf50; }

      /* Settings modal */
      .modal { position: fixed; inset: 0; display:none; align-items: center; justify-content: center; z-index: 1000; }
      .modal.open { display:flex; }
      .modal .overlay { position:absolute; inset:0; background: rgba(0,0,0,.55); backdrop-filter: blur(2px); }
      .modal .panel { position:relative; width: min(720px, 92vw); max-height: 90vh; overflow:auto; background:#000000; border:1px solid #333333; border-radius: 12px; padding:16px; box-shadow: 0 12px 28px rgba(0,0,0,.6); }
      .modal .titlebar { display:flex; align-items:center; justify-content: space-between; margin-bottom:10px; }
      .modal h2 { margin:0; font-size:18px; }
      .modal .close { background:transparent; border:none; color:#cfe1ff; font-size:20px; cursor:pointer; }
      .tabs { display:flex; gap:8px; margin: 6px 0 12px; }
      .tabs .tabbtn { padding:8px 12px; border-radius:8px; border:1px solid #223356; background:#000000; color:#d8e5ff; cursor:pointer; }
      .tabs .tabbtn.active { background:#0a0a0a; border-color:#2e4a88; }
      .form-grid { display:grid; grid-template-columns: 1fr 1fr; gap:12px; }
      .form-grid .full { grid-column: 1 / -1; }
      label { font-size:12px; color:#a8b3cf; display:block; margin:4px 0; }
      select, input[type="text"], input[type="password"], textarea { width:100%; background:#000000; color:#e6f0ff; border:1px solid #223356; border-radius:8px; padding:8px 10px; }
      textarea { min-height: 90px; resize: vertical; }

      /* Checkbox with tick-drawing animation */
      .check-wrap { display:inline-flex; align-items:center; gap:8px; cursor:pointer; user-select:none; }
      .check {
        position: relative;
        width: 18px; height: 18px; border-radius: 4px;
        border: 2px solid #2e4a88; background: #000000;
        display: inline-block; vertical-align: middle;
        transition: background 0.2s ease, border-color 0.2s ease, box-shadow 0.2s ease;
      }
      .check::after {
        content: '';
        position: absolute; left: 4px; top: 0px;
        width: 6px; height: 12px;
        border-right: 2px solid transparent; border-bottom: 2px solid transparent;
        transform: rotate(45deg);
      }
      input[type="checkbox"] { display:none; }
      input[type="checkbox"] + .check {
        box-shadow: inset 0 0 0 0 #2196f3;
      }
      input[type="checkbox"]:checked + .check {
        background: #1a2a5a; border-color: #2e4a88;
        box-shadow: inset 0 0 0 9px #1a2a5a;
      }
      /* Draw the tick with stroke-dash animation */
      @keyframes drawTick {
        0%   { width: 0; height: 0; border-color: #8ab4ff; }
        50%  { width: 6px; height: 0; border-color: #8ab4ff; }
        100% { width: 6px; height: 12px; border-color: #8ab4ff; }
      }
      input[type="checkbox"]:checked + .check::after {
        border-right-color: #8ab4ff; border-bottom-color: #8ab4ff;
        animation: drawTick 200ms ease-out forwards;
      }
      /* Label text */
      .check-label { color:#d8e5ff; font-size: 13px; }
      .check-wrap:hover .check { border-color:#3b63bf; }
      .check-wrap:active .check { box-shadow: 0 0 0 6px rgba(33,150,243,.15); }
      .row { display:flex; gap:10px; align-items:center; }
      .actions-bar { margin-top:10px; display:flex; gap:10px; justify-content:flex-end; }
      .btn-ghost { background:#000000; border:1px solid #223356; color:#d8e5ff; padding:8px 12px; border-radius:8px; cursor:pointer; }
      .btn-solid { background:#3f51b5; border:1px solid #32408f; color:#fff; padding:8px 12px; border-radius:8px; cursor:pointer; }
      .btn-mini { padding:4px 8px; font-size:11px; }
      .hint { font-size:11px; color:#8aa; }
      .key-badge { display:inline-block; margin-left:8px; padding:2px 8px; border-radius:999px; font-size:11px; border:1px solid #223356; background:#000000; color:#d8e5ff; vertical-align:middle; }
      .key-badge.ok { background:#1f5228; color:#c9f7d1; border-color:#2f7a3b; }
      .key-badge.missing { background:#3a3f55; color:#e2e6f2; border-color:#2a3149; }

      /* Sessions modal */
      .session-item { padding:12px; border:1px solid #223356; border-radius:8px; margin-bottom:8px; cursor:pointer; transition:background .15s ease; position:relative; overflow:hidden; }
      .session-item:hover { background:#0a0a0a; }
      .session-item.active { background:#1a2a5a; border-color:#2e4a88; }
      .session-title { font-weight:600; margin-bottom:4px; color:#e6f0ff; }
      .session-meta { font-size:11px; color:#8aa; display:flex; justify-content:space-between; margin-bottom:6px; }
      .session-actions { margin-top:8px; display:flex; gap:6px; flex-wrap:wrap; }
      .session-actions button { padding:4px 8px; font-size:11px; white-space:nowrap; }
      /* Session progress bar */
      .session-progress {
        position: absolute;
        bottom: 0;
        left: 0;
        height: 2px;
        background: linear-gradient(90deg, #2196f3, #4caf50);
        transition: width 0.8s cubic-bezier(0.4, 0, 0.2, 1);
        opacity: 0.6;
      }
      @keyframes progressShimmer {
        0% { background-position: -200% 0; }
        100% { background-position: 200% 0; }
      }
      .session-progress {
        background: linear-gradient(90deg, #2196f3 0%, #4caf50 50%, #2196f3 100%);
        background-size: 200% 100%;
        animation: progressShimmer 2s ease-in-out infinite;
      }

      /* === ANIMATIONS === */
      /* 1. Voice Activity Pulse */
      @keyframes pulse {
        0%, 100% { transform: scale(1); box-shadow: 0 0 0 0 currentColor; }
        50% { transform: scale(1.05); box-shadow: 0 0 0 8px transparent; }
      }
      @keyframes pulse-ring {
        0% { box-shadow: 0 0 0 0 currentColor; opacity: 1; }
        100% { box-shadow: 0 0 0 20px transparent; opacity: 0; }
      }
      .btn.speaking {
        animation: pulse 1.5s ease-in-out infinite;
      }
      .btn.speaking::before {
        content: '';
        position: absolute;
        inset: -4px;
        border-radius: 50%;
        border: 2px solid currentColor;
        animation: pulse-ring 1.5s ease-out infinite;
      }
      
      /* 2. AI Thinking Animation */
      @keyframes thinking-dots {
        0%, 20% { content: '●'; }
        40% { content: '●●'; }
        60% { content: '●●●'; }
        80%, 100% { content: '●'; }
      }
      .ai-thinking {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        padding: 8px 12px;
        background: #1a2a5a;
        border-radius: 8px;
        color: #2196f3;
        font-size: 13px;
        margin: 8px 0;
      }
      .ai-thinking::after {
        content: '●';
        animation: thinking-dots 1.5s steps(1) infinite;
        color: #2196f3;
      }
      @keyframes spinner {
        to { transform: rotate(360deg); }
      }
      .spinner {
        display: inline-block;
        width: 14px;
        height: 14px;
        border: 2px solid #223356;
        border-top-color: #2196f3;
        border-radius: 50%;
        animation: spinner 0.8s linear infinite;
      }
      
      /* 3. Modal Transitions */
      .modal {
        opacity: 0;
        transition: opacity 0.3s ease;
      }
      .modal.open {
        opacity: 1;
      }
      .modal .overlay {
        opacity: 0;
        transition: opacity 0.3s ease;
      }
      .modal.open .overlay {
        opacity: 1;
      }
      .modal .panel {
        transform: scale(0.9) translateY(20px);
        opacity: 0;
        transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.3s ease;
      }
      .modal.open .panel {
        transform: scale(1) translateY(0);
        opacity: 1;
      }
      
      /* Smooth transitions for everything */
      .btn, .session-item, .chip-vad, .btn-ghost, .btn-solid {
        transition: all 0.2s ease;
      }
      .btn:active {
        transform: scale(0.95);
      }
      
      /* Fade in animation for new transcript messages */
      @keyframes fadeInUp {
        from {
          opacity: 0;
          transform: translateY(10px);
        }
        to {
          opacity: 1;
          transform: translateY(0);
        }
      }
      .para {
        animation: fadeInUp 0.4s ease;
      }
      
      /* 4. Button Ripple Effect */
      @keyframes ripple {
        to {
          transform: scale(4);
          opacity: 0;
        }
      }
      .btn, .btn-solid, .btn-ghost {
        position: relative;
        overflow: hidden;
      }
      .ripple {
        position: absolute;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.4);
        width: 20px;
        height: 20px;
        pointer-events: none;
        animation: ripple 0.6s ease-out;
      }
      /* Enhanced button bounce */
      .btn:active, .btn-solid:active, .btn-ghost:active {
        transform: scale(0.95);
        transition: transform 0.1s cubic-bezier(0.34, 1.56, 0.64, 1);
      }
      
      /* 6. Icon Hover Animations */
      /* Bounce Up - Default for most buttons */
      @keyframes bounceUp {
        0%, 100% { transform: translateY(0); }
        50% { transform: translateY(-4px); }
      }
      
      /* Rotate 360 - For settings/config buttons */
      @keyframes rotate360 {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
      }
      
      /* Shake - For delete/warning buttons */
      @keyframes shake {
        0%, 100% { transform: translateX(0); }
        25% { transform: translateX(-3px); }
        75% { transform: translateX(3px); }
      }
      
      /* Pulse Glow - For primary action buttons */
      @keyframes pulseGlow {
        0%, 100% { filter: drop-shadow(0 0 0 transparent); }
        50% { filter: drop-shadow(0 0 8px currentColor); }
      }
      
      /* Apply animations on hover */
      .btn:hover .emojiIcon {
        animation: bounceUp 0.5s ease-in-out;
      }
      
      /* Microphone buttons: Scale + Glow */
      #btnEN:hover .emojiIcon,
      #btnHI:hover .emojiIcon {
        animation: pulseGlow 1s ease-in-out infinite;
        transform: scale(1.1);
        transition: transform 0.3s ease;
      }
      
      /* Settings button: Rotate */
      #btnSET:hover .emojiIcon {
        animation: rotate360 0.6s ease-in-out;
      }
      
      /* Sessions button: Bounce */
      #btnSESS:hover .emojiIcon {
        animation: bounceUp 0.5s ease-in-out;
      }
      
      /* AI & Capture buttons: Gentle float */
      #btnAI:hover .emojiIcon,
      #btnCAP:hover .emojiIcon {
        animation: bounceUp 0.6s ease-in-out infinite;
      }
      
      /* Transcript toggle: Slide effect */
      #btnTR:hover .emojiIcon {
        transform: translateX(3px);
        transition: transform 0.2s ease;
      }
      
      /* Delete buttons: Shake + Red glow */
      .btn-ghost[style*="color:#e53935"]:hover,
      .btn-ghost[style*="color: #e53935"]:hover {
        animation: shake 0.4s ease-in-out;
      }
      
      /* 7. Progress Ring Animation */
      @keyframes progressRing {
        from { stroke-dashoffset: 251.2; }
        to { stroke-dashoffset: 0; }
      }
      .btn.active::before {
        content: '';
        position: absolute;
        inset: -3px;
        border-radius: 50%;
        border: 2px solid transparent;
        border-top-color: currentColor;
        animation: progressRing 2s linear infinite;
      }

      /* Responsive adjustments */
      @media (max-width: 768px) {
        .container { padding: 20px; }
        .transcript { height: 48vh; }
      }
      @media (max-width: 640px) {
        /* Show mobile navbar, hide desktop header */
        .mobile-navbar { display: flex; }
        .stats-sheet { display: block; }
        h1 { display: none; }
        .status { display: none; }
        #vadRow { display: none; }
        
        /* Mobile: Full-width content on top, buttons bar at bottom */
        body { padding-bottom: 70px; }
        
        /* Container starts below navbar - IMPORTANT: padding-top must come AFTER other padding */
        .container { 
          padding: 12px;
          padding-top: 72px !important;
          padding-bottom: 80px;
          min-height: 100vh;
        }
        
        /* Workarea content must clear navbar */
        .workarea { 
          display: flex;
          flex-direction: column;
          gap: 0;
          margin-top: 0;
          position: relative;
        }
        
        .transcript { 
          height: calc(100vh - 240px); 
          font-size: 12px;
          margin-top: 0;
        }
        
        /* AI panel with safe top spacing */
        .askai { 
          height: calc(100vh - 280px);
          margin-top: 0;
          overflow-y: auto;
        }
        #askPanel { 
          margin-top: 0;
        }
        #capturePanel {
          margin-top: 0;
        }
        
        /* Content panels take full width */
        .workarea > #transcript,
        .workarea > #askPanel,
        .workarea > #capturePanel { width: 100%; margin: 0; }
        
        /* Bottom button bar: fixed, horizontal scroll, SINGLE ROW like YouTube */
        .toolbar {
          position: fixed;
          bottom: 0;
          left: 0;
          right: 0;
          display: flex;
          flex-direction: row;
          flex-wrap: nowrap !important;
          gap: 8px;
          padding: 8px 12px;
          background: rgba(11, 15, 26, 0.98);
          backdrop-filter: blur(12px);
          border-top: 1px solid #223356;
          overflow-x: auto;
          overflow-y: hidden;
          -webkit-overflow-scrolling: touch;
          scroll-behavior: smooth;
          z-index: 100;
        }
        
        /* Hide scrollbar but keep functionality */
        .toolbar::-webkit-scrollbar { display: none; }
        .toolbar { -ms-overflow-style: none; scrollbar-width: none; }
        
        /* Button stacks: VERTICAL with icon + label below (compact) */
        .stack { 
          display: flex;
          flex-direction: column;
          align-items: center;
          gap: 2px;
          flex-shrink: 0 !important;
          flex-grow: 0 !important;
          min-width: 52px;
          max-width: 52px;
        }
        
        /* Compact circular buttons */
        .btn { 
          width: 52px !important; 
          height: 52px !important;
          border-radius: 50%;
          flex-shrink: 0 !important;
        }
        
        .emojiIcon { font-size: 20px; }
        
        /* Compact labels below buttons */
        .label { 
          font-size: 9px;
          white-space: nowrap;
          text-align: center;
          max-width: 56px;
          overflow: hidden;
          text-overflow: ellipsis;
        }
      }
      @media (max-width: 420px) {
        .container { padding: 16px; }
        .btn { width:48px; height:48px; }
        .label { font-size: 9px; }
        .chips { gap:6px; }
        .chip { font-size:10px; }
        .transcript { height: 40vh; font-size: 12px; }
      }
    </style>
    <!-- Markdown and syntax highlight libs -->
    <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/highlight.js@11.9.0/styles/github-dark.min.css">
    <script src="https://cdn.jsdelivr.net/npm/marked/marked.min.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/dompurify@3.0.9/dist/purify.min.js"></script>
    <script src="https://cdn.jsdelivr.net/gh/highlightjs/cdn-release@11.9.0/build/highlight.min.js"></script>
  </head>
  <body>
    <!-- Mobile Navbar -->
    <div class="mobile-navbar">
      <div class="nav-left">
        <span class="status-dot" id="mobileStatusDot"></span>
        <span id="mobileStatus">Active: None</span>
      </div>
      <div class="nav-center">Voxora</div>
      <div class="nav-right" id="btnOpenStats">⋮</div>
    </div>

    <!-- Stats Bottom Sheet -->
    <div class="stats-overlay" id="statsOverlay"></div>
    <div class="stats-sheet" id="statsSheet">
      <div class="sheet-header">
        <div class="sheet-title">VAD Statistics</div>
        <div class="sheet-close" id="btnCloseStats">×</div>
      </div>
      <div class="sheet-content">
        <div class="stat-section">
          <div class="stat-header en">🇬🇧 English (EN)</div>
          <div class="stat-row">
            <span class="stat-label">State</span>
            <span class="stat-value" id="sheetStateEN">Idle</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Sent</span>
            <span class="stat-value" id="sheetSentEN">0</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Skipped</span>
            <span class="stat-value" id="sheetSkippedEN">0</span>
          </div>
        </div>
        
        <div class="stat-divider"></div>
        
        <div class="stat-section">
          <div class="stat-header hi">🇮🇳 Hindi (HI)</div>
          <div class="stat-row">
            <span class="stat-label">State</span>
            <span class="stat-value" id="sheetStateHI">Idle</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Sent</span>
            <span class="stat-value" id="sheetSentHI">0</span>
          </div>
          <div class="stat-row">
            <span class="stat-label">Skipped</span>
            <span class="stat-value" id="sheetSkippedHI">0</span>
          </div>
        </div>
      </div>
    </div>

    <div class="container">
      <h1>Voxora Service</h1>
      <div id="status" class="status">Active: None</div>
      <div id="vadRow" class="chips">
        <div class="chip-vad en" id="chipEN">
          <span class="tag">VAD EN</span><span class="sep">:</span>
          <span class="state idle" id="stateEN">Idle</span>
          <span class="sep">|</span>
          <span class="kv">Sent = <span id="sentEN">0</span></span>
          <span class="sep">|</span>
          <span class="kv">Skipped = <span id="skippedEN">0</span></span>
        </div>
        <div class="chip-vad hi" id="chipHI">
          <span class="tag">VAD HI</span><span class="sep">:</span>
          <span class="state idle" id="stateHI">Idle</span>
          <span class="sep">|</span>
          <span class="kv">Sent = <span id="sentHI">0</span></span>
          <span class="sep">|</span>
          <span class="kv">Skipped = <span id="skippedHI">0</span></span>
        </div>
      </div>
      <div class="workarea">
      <div class="toolbar">
        <div class="stack">
          <button id="btnEN" class="btn" title="Toggle English">
            <span class="emojiIcon">🇬🇧🎙️</span>
            <span id="dotEN" class="ws-dot"></span>
          </button>
          <span class="label">English</span>
        </div>
        <div class="stack">
          <button id="btnHI" class="btn" title="Toggle Hindi">
            <span class="emojiIcon">🇮🇳🎙️</span>
            <span id="dotHI" class="ws-dot"></span>
          </button>
          <span class="label">Hindi</span>
        </div>
        <div class="stack">
          <button id="btnAI" class="btn" title="Ask AI">
            <span class="emojiIcon">✨</span>
          </button>
          <span class="label">Ask AI</span>
        </div>
        <div class="stack">
          <button id="btnCAP" class="btn" title="Capture">
            <span class="emojiIcon">⏺️</span>
          </button>
          <span class="label">Capture</span>
        </div>
        <div class="stack">
          <button id="btnTR" class="btn" title="Show Transcript">
            <span id="iconTR" class="emojiIcon">🧾</span>
          </button>
          <span id="lblTR" class="label">Show</span>
        </div>
        <div class="stack">
          <button id="btnSESS" class="btn" title="Sessions">
            <span class="emojiIcon">🗂️</span>
          </button>
          <span class="label">Sessions</span>
        </div>
        <div class="stack">
          <button id="btnSET" class="btn" title="Settings">
            <span class="emojiIcon">⚙️</span>
          </button>
          <span class="label">Settings</span>
        </div>
      </div>
      <div id="transcript" class="transcript"></div>
        <div id="askPanel" class="askai hidden">
          <div style="margin-bottom:12px; padding:10px; background:#0a0a0a; border:1px solid #223356; border-radius:8px;">
            <div style="font-size:12px; color:#8aa; margin-bottom:8px;">Click ✨ Ask AI again after enabling Remote options:</div>
            <div id="remoteControls" class="row" style="gap:12px; align-items:center;">
              <label class="check-wrap">
                <input id="rcStream" type="checkbox" />
                <span class="check" aria-hidden="true"></span>
                <span class="check-label">Streaming</span>
              </label>
              <label class="check-wrap">
                <input id="rcQuery" type="checkbox" />
                <span class="check" aria-hidden="true"></span>
                <span class="check-label">Query</span>
              </label>
              <span id="rcHint" class="hint"></span>
            </div>
          </div>
        <div id="askOut" class="askout"></div>
      </div>
      <div id="capturePanel" class="askai hidden" style="margin-top:12px;">
        <div id="capOut" class="askout"></div>
      </div>
      </div>
    </div>

    <!-- Settings Modal -->
    <div id="settingsModal" class="modal" aria-hidden="true" role="dialog" aria-modal="true">
      <div class="overlay" data-close="settings"></div>
      <div class="panel">
        <div class="titlebar">
          <h2>Settings</h2>
          <button id="btnCloseSettings" class="close" title="Close">×</button>
        </div>
        <div class="tabs">
          <button id="tabEN" class="tabbtn active">English</button>
          <button id="tabHI" class="tabbtn">Hindi</button>
          <button id="tabEDIT" class="tabbtn">Edit Providers</button>
          <button id="tabSC" class="tabbtn">Screen Capture</button>
          <button id="tabREMOTE" class="tabbtn">Remote</button>
        </div>

        <div id="paneEN" class="tabpane">
          <div class="form-grid">
            <div class="full">
              <label for="enPrompt">Prompt</label>
              <textarea id="enPrompt" placeholder="Enter system prompt for English..."></textarea>
            </div>
            <div>
              <label for="enProvider">AI Provider <span id="enProvKeyBadge" class="key-badge">…</span></label>
              <select id="enProvider"></select>
            </div>
            <div id="enModelWrap">
              <label for="enModel">Model</label>
              <select id="enModel"></select>
            </div>
            <div id="enModelCustomWrap" class="hidden">
              <label for="enModelCustom">Custom Model</label>
              <input id="enModelCustom" type="text" placeholder="provider/model-name" />
            </div>
            <div class="full">
              <div class="hint">API keys are managed in the Edit Providers tab.</div>
            </div>
          </div>
          <div class="actions-bar">
            <button id="btnSaveEN" class="btn-solid">Save English</button>
          </div>
        </div>

        <div id="paneHI" class="tabpane hidden">
          <div class="form-grid">
            <div class="full">
              <label for="hiPrompt">Prompt</label>
              <textarea id="hiPrompt" placeholder="Enter system prompt for Hindi..."></textarea>
            </div>
            <div>
              <label for="hiProvider">AI Provider <span id="hiProvKeyBadge" class="key-badge">…</span></label>
              <select id="hiProvider"></select>
            </div>
            <div id="hiModelWrap">
              <label for="hiModel">Model</label>
              <select id="hiModel"></select>
            </div>
            <div id="hiModelCustomWrap" class="hidden">
              <label for="hiModelCustom">Custom Model</label>
              <input id="hiModelCustom" type="text" placeholder="provider/model-name" />
            </div>
            <div class="full">
              <div class="hint">API keys are managed in the Edit Providers tab.</div>
            </div>
          </div>
          <div class="actions-bar">
            <button id="btnSaveHI" class="btn-solid">Save Hindi</button>
          </div>
        </div>

        <div id="paneSC" class="tabpane hidden">
          <div class="form-grid">
            <div class="full">
              <label for="scPrompt">Prompt (Screen Capture)</label>
              <textarea id="scPrompt" placeholder="Enter system prompt for Screen Capture..."></textarea>
            </div>
            <div>
              <label for="scProvider">Provider <span id="scProvKeyBadge" class="key-badge">…</span></label>
              <select id="scProvider"></select>
            </div>
            <div id="scModelWrap">
              <label for="scModel">Model</label>
              <select id="scModel"></select>
            </div>
            <div id="scModelCustomWrap" class="hidden">
              <label for="scModelCustom">Custom Model</label>
              <input id="scModelCustom" type="text" placeholder="provider/model-name" />
            </div>
          </div>
          <div class="actions-bar">
            <button id="btnSaveSC" class="btn-solid">Save Screen Capture</button>
          </div>
        </div>

        <div id="paneEDIT" class="tabpane hidden">
          <div class="form-grid">
            <div class="full"><strong>Groq</strong> <span id="badgeGroq" class="key-badge">…</span> <button id="rmGroq" class="btn-ghost btn-mini" type="button" title="Remove key">Remove</button></div>
            <div class="full">
              <label for="groqKey">Groq API Key</label>
              <input id="groqKey" type="password" placeholder="Enter Groq API key" autocomplete="off" />
            </div>
            <div>
              <label for="groqDefault">Default Model</label>
              <input id="groqDefault" type="text" placeholder="llama-3.1-8b-instant" />
            </div>
            <div>
              <label for="groqExtra">Additional Models (comma-separated)</label>
              <input id="groqExtra" type="text" placeholder="model-a, model-b" />
            </div>

            <div class="full">
              <label class="check-wrap">
                <input id="groqStreaming" type="checkbox" />
                <span class="check" aria-hidden="true"></span>
                <span class="check-label">Streaming (Groq)</span>
              </label>
              <div class="hint">Enable streaming responses for Groq (unchecked by default).</div>
            </div>



            <div class="full"><strong>Gemini</strong> <span id="badgeGemini" class="key-badge">…</span> <button id="rmGemini" class="btn-ghost btn-mini" type="button" title="Remove key">Remove</button></div>
            <div class="full">
              <label for="geminiKey">Gemini API Key</label>
              <input id="geminiKey" type="password" placeholder="Enter Gemini API key" autocomplete="off" />
            </div>
            <div>
              <label for="geminiDefault">Default Model</label>
              <input id="geminiDefault" type="text" placeholder="gemini-2.5-flash" />
            </div>
            <div>
              <label for="geminiExtra">Additional Models (comma-separated)</label>
              <input id="geminiExtra" type="text" placeholder="model-a, model-b" />
            </div>

            <div class="full"><strong>OpenRouter</strong> <span id="badgeOpenRouter" class="key-badge">…</span> <button id="rmOpenRouter" class="btn-ghost btn-mini" type="button" title="Remove key">Remove</button></div>
            <div class="full">
              <label for="openrouterKey">OpenRouter API Key</label>
              <input id="openrouterKey" type="password" placeholder="Enter OpenRouter API key" autocomplete="off" />
            </div>
            <div>
              <label for="openrouterDefault">Default Model</label>
              <input id="openrouterDefault" type="text" placeholder="meta-llama/llama-3.1-70b" />
            </div>
            <div>
              <label for="openrouterExtra">Additional Models (comma-separated)</label>
              <input id="openrouterExtra" type="text" placeholder="provider/model-a, provider/model-b" />
            </div>
            <div>
              <label for="fallbackOpenRouter">Fallback OpenRouter Model Group (common) <span id="fallbackORKeyBadge" class="key-badge">…</span></label>
              <select id="fallbackOpenRouter">
                <option value="openai">OpenAI</option>
                <option value="claude">Claude</option>
              </select>
            </div>

            <div class="full"><strong>Custom Provider</strong> <span id="badgeCustom" class="key-badge">…</span> <button id="rmCustom" class="btn-ghost btn-mini" type="button" title="Remove key">Remove</button></div>
            <div class="full">
              <label for="customKey">Custom API Key</label>
              <input id="customKey" type="password" placeholder="Enter Custom API key" autocomplete="off" />
              <div class="hint">For Custom Provider, model name is set per-language in the "Custom Model" field.</div>
            </div>

            <div class="full"><hr></div>
            <div class="full"><strong>Screen Capture Providers</strong></div>

            <div class="full"><strong>Gemini (Screen Capture)</strong> <span class="key-badge">Key reused</span></div>
            <div>
              <label for="scGeminiDefault">Default Model</label>
              <input id="scGeminiDefault" type="text" placeholder="gemini-2.5-flash" />
            </div>
            <div>
              <label for="scGeminiExtra">Additional Models (comma-separated)</label>
              <input id="scGeminiExtra" type="text" placeholder="model-a, model-b" />
            </div>

            <div class="full"><strong>Groq (Screen Capture)</strong> <span class="key-badge">Key reused</span></div>
            <div>
              <label for="scGroqDefault">Default Model</label>
              <input id="scGroqDefault" type="text" placeholder="meta-llama/llama-4-scout-17b-16e-instruct" />
            </div>
            <div>
              <label for="scGroqExtra">Additional Models (comma-separated)</label>
              <input id="scGroqExtra" type="text" placeholder="model-a, model-b" />
            </div>

            <div class="full"><strong>OpenRouter (Screen Capture Fallback)</strong> <span class="key-badge">Key reused</span></div>
            <div>
              <label for="scORDefault">Default Model</label>
              <input id="scORDefault" type="text" placeholder="google/gemini-2.5-flash" />
            </div>
            <div>
              <label for="scORExtra">Additional Models (comma-separated)</label>
              <input id="scORExtra" type="text" placeholder="provider/model-a, provider/model-b" />
            </div>
            <div class="full">
              <label class="check-wrap">
                <input id="scORFallback" type="checkbox" />
                <span class="check" aria-hidden="true"></span>
                <span class="check-label">Mark OpenRouter as Fallback for Screen Capture</span>
              </label>
            </div>
          </div>
          <div class="actions-bar">
            <button id="btnSaveProviders" class="btn-solid">Save Providers</button>
          </div>
        </div>

        <div id="paneREMOTE" class="tabpane hidden">
          <div class="form-grid">
            <div class="full">
              <div class="hint">Manage AnythingLLM connection defaults. API key and workspace slug are stored in your system keyring.</div>
            </div>
            <div class="full"><strong>API Key</strong> <span id="badgeRemoteKey" class="key-badge">…</span> <button id="rmRemoteKey" class="btn-ghost btn-mini" type="button" title="Remove key">Remove</button></div>
            <div class="full">
              <label for="remoteKey">AnythingLLM API Key</label>
              <input id="remoteKey" type="password" placeholder="Enter AnythingLLM API key" autocomplete="off" />
              <button id="btnSaveRemoteKey" class="btn-ghost btn-mini" type="button">Save Key</button>
            </div>
            <div>
              <label for="remoteSlug">Workspace Slug</label>
              <input id="remoteSlug" type="text" placeholder="workspace-slug" />
            </div>
            <div>
              <label for="remoteSlugSel">Workspaces (auto-loaded)</label>
              <select id="remoteSlugSel"></select>
            </div>
            <div>
              <label for="remoteMode">Mode</label>
              <select id="remoteMode">
                <option value="chat">Chat</option>
                <option value="stream">Streaming Chat</option>
              </select>
            </div>
            <div>
              <label for="remoteChatMode">Chat Mode</label>
              <select id="remoteChatMode">
                <option value="chat">Chat</option>
                <option value="query">Query</option>
              </select>
              <div class="hint">Query: Only uses sources from vector DB and no chat history.</div>
              <div class="hint"> Chat: uses LLM general knowledge plus embeddings with rolling history.</div>
            </div>
          </div>
          <div class="actions-bar">
            <button id="btnSaveRemoteCfg" class="btn-solid">Save Remote</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Sessions Modal -->
    <div id="sessionsModal" class="modal" aria-hidden="true" role="dialog" aria-modal="true">
      <div class="overlay" data-close="sessions"></div>
      <div class="panel">
        <div class="titlebar">
          <h2>Chat Sessions</h2>
          <button id="btnCloseSessions" class="close" title="Close">×</button>
        </div>
        <div style="margin-bottom:12px; display:flex; gap:8px;">
          <button id="btnNewSession" class="btn-solid" style="flex:1;">+ New Session</button>
          <input id="searchSessions" type="text" placeholder="Search sessions..." style="flex:2;" />
        </div>
        <div id="sessionsList" style="max-height:60vh; overflow:auto;">
          <div class="hint" style="text-align:center; padding:20px;">Loading sessions...</div>
        </div>
      </div>
    </div>

    <!-- Session Viewer Modal (separate from live transcript) -->
    <div id="sessionViewerModal" class="modal" aria-hidden="true" role="dialog" aria-modal="true">
      <div class="overlay" data-close="viewer"></div>
      <div class="panel">
        <div class="titlebar">
          <h2 id="viewerTitle">Session History</h2>
          <button id="btnCloseViewer" class="close" title="Close">×</button>
        </div>
        <div id="viewerContent" style="max-height:70vh; overflow:auto; padding:12px; border:1px solid #223356; border-radius:8px; background:#000000;">
          <div class="hint" style="text-align:center; padding:20px;">Loading...</div>
        </div>
      </div>
    </div>

    <script>
      (function(){
        const statusEl = document.getElementById('status');
        const stateEN = document.getElementById('stateEN');
        const stateHI = document.getElementById('stateHI');
        const sentEN = document.getElementById('sentEN');
        const sentHI = document.getElementById('sentHI');
        const skippedEN = document.getElementById('skippedEN');
        const skippedHI = document.getElementById('skippedHI');
        const btnEN = document.getElementById('btnEN');
        const btnHI = document.getElementById('btnHI');
        const btnAI = document.getElementById('btnAI');
        const btnCAP = document.getElementById('btnCAP');
        const btnTR = document.getElementById('btnTR');
        const btnSESS = document.getElementById('btnSESS');
        const btnSET = document.getElementById('btnSET');
        // Settings modal refs
        const settingsModal = document.getElementById('settingsModal');
        const btnCloseSettings = document.getElementById('btnCloseSettings');
        const tabEN = document.getElementById('tabEN');
        const tabHI = document.getElementById('tabHI');
        const paneEN = document.getElementById('paneEN');
        const paneHI = document.getElementById('paneHI');
        const tabEDIT = document.getElementById('tabEDIT');
        const paneEDIT = document.getElementById('paneEDIT');
        const tabSC = document.getElementById('tabSC');
        const paneSC = document.getElementById('paneSC');
        const tabREMOTE = document.getElementById('tabREMOTE');
        const paneREMOTE = document.getElementById('paneREMOTE');
        // EN controls
        const enPrompt = document.getElementById('enPrompt');
        const enProvider = document.getElementById('enProvider');
        const enProvKeyBadge = document.getElementById('enProvKeyBadge');
        const enModel = document.getElementById('enModel');
        const enModelWrap = document.getElementById('enModelWrap');
        const enModelCustomWrap = document.getElementById('enModelCustomWrap');
        const enModelCustom = document.getElementById('enModelCustom');
        const btnSaveEN = document.getElementById('btnSaveEN');
        // HI controls
        const hiPrompt = document.getElementById('hiPrompt');
        // EDIT controls (provider-level)
        const groqKey = document.getElementById('groqKey');
        const groqStreaming = document.getElementById('groqStreaming');
        const groqDefault = document.getElementById('groqDefault');
        const groqExtra = document.getElementById('groqExtra');
        const geminiKey = document.getElementById('geminiKey');
        const geminiDefault = document.getElementById('geminiDefault');
        const geminiExtra = document.getElementById('geminiExtra');
        const openrouterKey = document.getElementById('openrouterKey');
        const openrouterDefault = document.getElementById('openrouterDefault');
        const openrouterExtra = document.getElementById('openrouterExtra');
        const fallbackOpenRouter = document.getElementById('fallbackOpenRouter');
        const fallbackORKeyBadge = document.getElementById('fallbackORKeyBadge');
        const customKey = document.getElementById('customKey');
        const btnSaveProviders = document.getElementById('btnSaveProviders');
        // Screen Capture controls
        const scPrompt = document.getElementById('scPrompt');
        const scProvider = document.getElementById('scProvider');
        const scProvKeyBadge = document.getElementById('scProvKeyBadge');
        const scModel = document.getElementById('scModel');
        const scModelWrap = document.getElementById('scModelWrap');
        const scModelCustomWrap = document.getElementById('scModelCustomWrap');
        const scModelCustom = document.getElementById('scModelCustom');
        const btnSaveSC = document.getElementById('btnSaveSC');
        const scGeminiDefault = document.getElementById('scGeminiDefault');
        const scGeminiExtra = document.getElementById('scGeminiExtra');
        const scGroqDefault = document.getElementById('scGroqDefault');
        const scGroqExtra = document.getElementById('scGroqExtra');
        const scORDefault = document.getElementById('scORDefault');
        const scORExtra = document.getElementById('scORExtra');
        const scORFallback = document.getElementById('scORFallback');
        // key badges
        const badgeGroq = document.getElementById('badgeGroq');
        const badgeRemoteKey = document.getElementById('badgeRemoteKey');
        const badgeGemini = document.getElementById('badgeGemini');
        const badgeOpenRouter = document.getElementById('badgeOpenRouter');
        const badgeCustom = document.getElementById('badgeCustom');
        // remove buttons
        const rmGroq = document.getElementById('rmGroq');
        const rmGemini = document.getElementById('rmGemini');
        const rmOpenRouter = document.getElementById('rmOpenRouter');
        const rmCustom = document.getElementById('rmCustom');
        // Remote controls
        const remoteKey = document.getElementById('remoteKey');
        const btnSaveRemoteKey = document.getElementById('btnSaveRemoteKey');
        const rmRemoteKey = document.getElementById('rmRemoteKey');
        const remoteSlug = document.getElementById('remoteSlug');
        const remoteName = document.getElementById('remoteName');
        const btnSaveRemoteCfg = document.getElementById('btnSaveRemoteCfg');
        const remoteSlugSel = document.getElementById('remoteSlugSel');
        const remoteMode = document.getElementById('remoteMode');
        const remoteChatMode = document.getElementById('remoteChatMode');
        const hiProvider = document.getElementById('hiProvider');
        const hiProvKeyBadge = document.getElementById('hiProvKeyBadge');
        const hiModel = document.getElementById('hiModel');
        const hiModelWrap = document.getElementById('hiModelWrap');
        const hiModelCustomWrap = document.getElementById('hiModelCustomWrap');
        const hiModelCustom = document.getElementById('hiModelCustom');
        const btnSaveHI = document.getElementById('btnSaveHI');
        const iconTR = document.getElementById('iconTR');
        const lblTR = document.getElementById('lblTR');
        const transcript = document.getElementById('transcript');
        const askPanel = document.getElementById('askPanel');
        const askOut = document.getElementById('askOut');
        const rcStream = document.getElementById('rcStream');
        const rcQuery = document.getElementById('rcQuery');
        const rcHint = document.getElementById('rcHint');
        const capturePanel = document.getElementById('capturePanel');
        const capOut = document.getElementById('capOut');
        const dotEN = document.getElementById('dotEN');
        const dotHI = document.getElementById('dotHI');
        // Mobile navbar elements
        const mobileStatus = document.getElementById('mobileStatus');
        const mobileStatusDot = document.getElementById('mobileStatusDot');
        const btnOpenStats = document.getElementById('btnOpenStats');
        const btnCloseStats = document.getElementById('btnCloseStats');
        const statsSheet = document.getElementById('statsSheet');
        const statsOverlay = document.getElementById('statsOverlay');
        const sheetStateEN = document.getElementById('sheetStateEN');
        const sheetSentEN = document.getElementById('sheetSentEN');
        const sheetSkippedEN = document.getElementById('sheetSkippedEN');
        const sheetStateHI = document.getElementById('sheetStateHI');
        const sheetSentHI = document.getElementById('sheetSentHI');
        const sheetSkippedHI = document.getElementById('sheetSkippedHI');
        let enRunning = false, hiRunning = false;
        let wsEn = null, wsHi = null;
        let transcriptVisible = false; // default hidden, persists when hidden
        let aiVisible = false; // Ask AI panel visibility
        let activeLang = null; // 'EN' | 'HI' based on last Start click
        let currentPara = null; // { lang, el, bodyEl, lineCount }
        // Remote prefs (session overrides for separate pipeline)
        let remotePrefStream = false; // false=Chat, true=Streaming
        let remotePrefQuery = false;  // false=Chat, true=Query


        //groq streaming persist checkbox
        // Auto-save Groq streaming toggle immediately when changed
        if (groqStreaming) {
          groqStreaming.addEventListener('change', function(){
            const cfg = {
              groq: {
                default_model: (providerCfgCache.groq && providerCfgCache.groq.default_model) || null,
                extra_models: (providerCfgCache.groq && providerCfgCache.groq.extra_models) || null,
              },
              gemini: {
                default_model: (providerCfgCache.gemini && providerCfgCache.gemini.default_model) || null,
                extra_models: (providerCfgCache.gemini && providerCfgCache.gemini.extra_models) || null,
              },
              openrouter: {
                default_model: (providerCfgCache.openrouter && providerCfgCache.openrouter.default_model) || null,
                extra_models: (providerCfgCache.openrouter && providerCfgCache.openrouter.extra_models) || null,
              },
              groq_streaming: !!groqStreaming.checked,
            };
            fetch('/api/settings/providers', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(cfg) })
              .then(function(){ /* optional toast */ })
              .catch(function(){ /* ignore errors to avoid UX disruption */ });
          });
        }

        // ANIMATION: Animated counter function
        const counterCache = new Map();
        function animateCounter(el, targetValue){
          if (!el) return;
          const target = parseInt(targetValue) || 0;
          const current = parseInt(el.textContent) || 0;
          if (current === target) return;
          
          const key = el.id || el;
          if (counterCache.has(key)) clearInterval(counterCache.get(key));
          
          const duration = 300; // ms
          const steps = 15;
          const increment = (target - current) / steps;
          let step = 0;
          
          const timer = setInterval(()=>{
            step++;
            if (step >= steps) {
              el.textContent = target;
              clearInterval(timer);
              counterCache.delete(key);
            } else {
              el.textContent = Math.round(current + increment * step);
            }
          }, duration / steps);
          
          counterCache.set(key, timer);
        }

        function normalizeText(t){
          // remove leading EN/HI with optional colon
          let s = t.replace(/^\s*(EN|HI)\s*:?\s*/i, '');
          // strip surrounding quotes and any remaining quotes inside
          s = s.replace(/^"+|"+$/g, '').replace(/"/g, '');
          return s.trim();
        }
        function ensureParagraph(lang){
          if (!currentPara || currentPara.lang !== lang || currentPara.lineCount >= 4) {
            // start a new paragraph as a simple <p>
            const p = document.createElement('p');
            p.className = 'para';
            transcript.appendChild(p);
            currentPara = { lang, el: p, bodyEl: p, lineCount: 0 };
          }
        }
        function addLine(kind, text){
          const lang = activeLang ? activeLang : kind; // prefer user-selected language
          const clean = normalizeText(text);
          ensureParagraph(lang);
          // append segment with separator inside the paragraph
          if (currentPara.lineCount > 0) {
            const sep = document.createElement('span'); sep.className = 'sep'; sep.textContent = '·';
            currentPara.bodyEl.appendChild(sep);
          }
          const seg = document.createElement('span');
          seg.className = 'seg';
          seg.textContent = clean;
          currentPara.bodyEl.appendChild(seg);
          currentPara.lineCount += 1;
          if (currentPara.lineCount >= 4) {
            // next call will auto-start a new paragraph
          }
          transcript.scrollTop = transcript.scrollHeight;
        }

        async function refresh(){
          try {
            const [r1, r2, r3, r4] = await Promise.all([
              fetch('/api/status'),
              fetch('/api/status-hi'),
              fetch('/api/vad-status'),
              fetch('/api/vad-status-hi'),
            ]);
            const en = await r1.json();
            const hi = await r2.json();
            const enVad = await r3.json();
            const hiVad = await r4.json();
            enRunning = !!en.running; hiRunning = !!hi.running;
            const active = enRunning ? 'English' : (hiRunning ? 'Hindi' : 'None');
            statusEl.textContent = 'Active: ' + active;
            // Update mobile navbar
            if (mobileStatus) mobileStatus.textContent = 'Active: ' + active;
            if (mobileStatusDot) {
              mobileStatusDot.classList.remove('active-en', 'active-hi');
              if (enRunning) mobileStatusDot.classList.add('active-en');
              if (hiRunning) mobileStatusDot.classList.add('active-hi');
            }
            // Update EN chip
            const mapState = (s) => s === 'speech' ? 'Speech' : (s === 'silence' ? 'Silence' : 'Idle');
            stateEN.textContent = mapState(enVad.last_state);
            stateEN.classList.remove('speech','silence','idle');
            stateEN.classList.add(enVad.last_state || 'idle');
            // ANIMATION: Animated counter for sent/skipped
            animateCounter(sentEN, enVad.sent);
            animateCounter(skippedEN, enVad.skipped);
            // ANIMATION: Voice activity pulse on EN button
            btnEN.classList.toggle('speaking', enRunning && enVad.last_state === 'speech');
            // Update HI chip
            stateHI.textContent = mapState(hiVad.last_state);
            stateHI.classList.remove('speech','silence','idle');
            stateHI.classList.add(hiVad.last_state || 'idle');
            // ANIMATION: Animated counter for sent/skipped
            animateCounter(sentHI, hiVad.sent);
            animateCounter(skippedHI, hiVad.skipped);
            // ANIMATION: Voice activity pulse on HI button
            btnHI.classList.toggle('speaking', hiRunning && hiVad.last_state === 'speech');
            // Update bottom sheet stats
            if (sheetStateEN) sheetStateEN.textContent = mapState(enVad.last_state);
            if (sheetSentEN) animateCounter(sheetSentEN, enVad.sent);
            if (sheetSkippedEN) animateCounter(sheetSkippedEN, enVad.skipped);
            if (sheetStateHI) sheetStateHI.textContent = mapState(hiVad.last_state);
            if (sheetSentHI) animateCounter(sheetSentHI, hiVad.sent);
            if (sheetSkippedHI) animateCounter(sheetSkippedHI, hiVad.skipped);
            btnEN.classList.toggle('active', enRunning);
            btnEN.classList.toggle('en', enRunning);
            btnHI.classList.toggle('active', hiRunning);
            btnHI.classList.toggle('hi', hiRunning);
          } catch(e){ console.error(e); }
        }

        function insertLanguageHeader(lang){
          // Insert one-time chip + colored rule to separate languages
          const tag = document.createElement('p');
          tag.className = 'lang-tag';
          const chip = document.createElement('span');
          chip.className = 'chip ' + (lang === 'EN' ? 'en' : 'hi');
          chip.textContent = lang;
          tag.appendChild(chip);
          const rule = document.createElement('hr');
          rule.className = 'lang-rule ' + (lang === 'EN' ? 'en' : 'hi');
          transcript.appendChild(tag);
          transcript.appendChild(rule);
        }

        function applyRemoteControls(){
          if (rcStream) rcStream.checked = !!remotePrefStream;
          if (rcQuery) rcQuery.checked = !!remotePrefQuery;
          if (rcHint) rcHint.textContent = 'Defaults: Mode = ' + (remotePrefStream? 'Streaming':'Chat') + ', Chat Mode = ' + (remotePrefQuery? 'Query':'Chat');
        }

        async function refreshRemote(){
          try {
            // 1) Read current saved slug/key presence
            let savedSlug = '';
            const r = await fetch('/api/remote/config');
            if (r.ok) {
              const s = await r.json();
              badgeRemoteKey.textContent = s.has_key ? 'Key present' : 'No key';
              badgeRemoteKey.classList.toggle('ok', !!s.has_key);
              badgeRemoteKey.classList.toggle('missing', !s.has_key);
              savedSlug = s.slug || '';
            }
            // 2) Fetch workspaces and populate dropdown
            let chosen = savedSlug;
            const list = await fetch('/api/remote/workspaces');
            if (list.ok) {
              const data = await list.json();
              const items = (data && data.items) || [];
              remoteSlugSel.innerHTML = '';
              items.forEach(function(it, i){
                const op = document.createElement('option');
                op.value = it.slug; op.textContent = (it.name? (it.name+' — '):'') + it.slug;
                if (it.slug === savedSlug || (!savedSlug && i === 0)) op.selected = true;
                remoteSlugSel.appendChild(op);
              });
              if (!savedSlug && items.length > 0) {
                // auto-save first workspace if none saved
                chosen = items[0].slug;
                await fetch('/api/remote/config', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ slug: chosen }) });
              }
            }
            // 3) Reflect chosen slug in input
            remoteSlug.value = chosen;
            // 4) Load workspace name for chosen slug
            // load defaults
            const rcfg = await fetch('/api/remote/config');
              if (rcfg.ok) {
                const c = await rcfg.json();
                if (typeof c.stream_default === 'boolean') {
                  remoteMode.value = c.stream_default ? 'stream' : 'chat';
                  remotePrefStream = !!c.stream_default;
                }
                if (typeof c.chat_mode === 'string') {
                  remoteChatMode.value = (c.chat_mode === 'query') ? 'query' : 'chat';
                  remotePrefQuery = (c.chat_mode === 'query');
                }
                applyRemoteControls();
            }
          } catch(e) { /* noop */ }
        }

        async function toggleEN(){
          const url = enRunning ? '/api/stop' : '/api/start';
          const r = await fetch(url, { method: 'POST' });
          if (r.ok || r.status === 204) {
            if (enRunning) {
              // Stop: do not add any transcript label; just reset selection
              activeLang = null;
              currentPara = null;
            } else {
              currentPara = null; // start a fresh paragraph for EN without clearing history
              activeLang = 'EN';
              insertLanguageHeader('EN');
            }
            enRunning = !enRunning; refresh();
          }
        }
        async function toggleHI(){
          const url = hiRunning ? '/api/stop-hi' : '/api/start-hi';
          const r = await fetch(url, { method: 'POST' });
          if (r.ok || r.status === 204) {
            if (hiRunning) {
              // Stop: do not add any transcript label; just reset selection
              activeLang = null;
              currentPara = null;
            } else {
              currentPara = null; // start a fresh paragraph for HI without clearing history
              activeLang = 'HI';
              insertLanguageHeader('HI');
            }
            hiRunning = !hiRunning; refresh();
          }
        }

        // Tab switching for Remote
        function showPane(p){
          paneEN.classList.add('hidden');
          paneHI.classList.add('hidden');
          paneEDIT.classList.add('hidden');
          paneSC.classList.add('hidden');
          paneREMOTE.classList.add('hidden');
          p.classList.remove('hidden');
        }
        tabREMOTE.addEventListener('click', () => { showPane(paneREMOTE); refreshRemote(); tabEN.classList.remove('active'); tabHI.classList.remove('active'); tabEDIT.classList.remove('active'); tabSC.classList.remove('active'); tabREMOTE.classList.add('active'); });
        tabEN.addEventListener('click', () => { showPane(paneEN); tabEN.classList.add('active'); tabHI.classList.remove('active'); tabEDIT.classList.remove('active'); tabSC.classList.remove('active'); tabREMOTE.classList.remove('active'); });
        tabHI.addEventListener('click', () => { showPane(paneHI); tabHI.classList.add('active'); tabEN.classList.remove('active'); tabEDIT.classList.remove('active'); tabSC.classList.remove('active'); tabREMOTE.classList.remove('active'); });
        tabEDIT.addEventListener('click', () => { showPane(paneEDIT); tabEDIT.classList.add('active'); tabEN.classList.remove('active'); tabHI.classList.remove('active'); tabSC.classList.remove('active'); tabREMOTE.classList.remove('active'); });
        tabSC.addEventListener('click', () => { showPane(paneSC); tabSC.classList.add('active'); tabEN.classList.remove('active'); tabHI.classList.remove('active'); tabEDIT.classList.remove('active'); tabREMOTE.classList.remove('active'); });

        // Remote actions
        btnSaveRemoteKey.addEventListener('click', async () => {
          try {
            const key = remoteKey.value.trim();
            if (!key) return;
            const r = await fetch('/api/remote/key', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ api_key: key }) });
            if (r.ok) { badgeRemoteKey.textContent = 'Key present'; badgeRemoteKey.classList.add('ok'); badgeRemoteKey.classList.remove('missing'); }
          } catch(e){}
        });
        rmRemoteKey.addEventListener('click', async () => {
          try {
            const r = await fetch('/api/remote/key', { method: 'DELETE' });
            if (r.ok) { badgeRemoteKey.textContent = 'No key'; badgeRemoteKey.classList.remove('ok'); badgeRemoteKey.classList.add('missing'); }
          } catch(e){}
        });
        btnSaveRemoteCfg.addEventListener('click', async () => {
          try {
            const slug = remoteSlug.value.trim();
            const mode = remoteMode.value === 'stream' ? 'stream' : 'chat';
            const payload = { slug, chat_default: mode === 'chat', stream_default: mode === 'stream', chat_mode: remoteChatMode.value };
            const r = await fetch('/api/remote/config', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(payload) });
            if (r.ok) { btnSaveRemoteCfg.textContent = 'Saved!'; setTimeout(()=>btnSaveRemoteCfg.textContent='Save Remote', 1000); await refreshRemote(); }
          } catch(e){}
        });
        remoteSlugSel.addEventListener('change', function(){ remoteSlug.value = remoteSlugSel.value; });
        if (rcStream) rcStream.addEventListener('change', function(){ remotePrefStream = !!rcStream.checked; applyRemoteControls(); });
        if (rcQuery) rcQuery.addEventListener('change', function(){ remotePrefQuery = !!rcQuery.checked; applyRemoteControls(); });

        function applyTranscriptVisibility(){
          transcript.classList.toggle('hidden', !transcriptVisible);
          btnTR.classList.toggle('active', transcriptVisible);
          btnTR.classList.toggle('tr', transcriptVisible);
          if (transcriptVisible) {
            iconTR.textContent = '🙈';
            lblTR.textContent = 'Hide';
            btnTR.title = 'Hide Transcript';
          } else {
            iconTR.textContent = '🧾';
            lblTR.textContent = 'Show';
            btnTR.title = 'Show Transcript';
          }
        }
        function applyAskVisibility(){
          askPanel.classList.toggle('hidden', !aiVisible);
          btnAI.classList.toggle('active', aiVisible);
          btnAI.classList.toggle('ai', aiVisible);
        }
        function toggleTR(){
          transcriptVisible = !transcriptVisible;
          if (transcriptVisible) { aiVisible = false; applyAskVisibility(); }
          applyTranscriptVisibility();
        }
        function askAI(){
          // Ensure Ask AI panel is visible; do not hide on repeated clicks
          if (!aiVisible) {
            aiVisible = true;
            transcriptVisible = false;
            applyTranscriptVisibility();
            applyAskVisibility();
            // Update hint to reflect defaults (without auto-checking)
            if (rcHint) rcHint.textContent = 'Defaults: Mode = ' + (remotePrefStream? 'Streaming':'Chat') + ', Chat Mode = ' + (remotePrefQuery? 'Query':'Chat');
          }
          
          // Clear previous AI response and always attempt an Ask on click
          if (askOut) { askOut.textContent = ''; }
          // If Remote pipeline toggles are on, route to AnythingLLM
          var useRemote = (rcStream && rcStream.checked) || (rcQuery && rcQuery.checked);
          if (useRemote) {
            const chunk = enRunning ? collectNew('EN') : (hiRunning ? collectNew('HI') : '');
            if (!chunk) { setAskStatus('SYS: No transcript'); return; }
            setAskStatus('AI Thinking…');
            const mode = (rcQuery && rcQuery.checked) ? 'query' : 'chat';
            fetch('/api/remote/ask', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ input: chunk, stream: !!(rcStream && rcStream.checked), mode }) })
              .then(function(r){ return r.text().then(function(t){ try{return [r.status, JSON.parse(t)];}catch(_){return [r.status, {output:t}];} }); })
              .then(function(pair){ var status = pair[0], data = pair[1]||{}; if (status>=200 && status<300){ renderAskMarkdown('Remote', data.output||'(no output)'); } else { setAskStatus('SYS: Remote failed'); } })
              .catch(function(){ setAskStatus('SYS: Remote error'); });
            return;
          }
          askFromTranscript();
        }
        function setAskStatus(text){ 
          if (!askOut) return;
          // ANIMATION: Show AI thinking indicator
          if (text && text.includes('Thinking')) {
            askOut.innerHTML = '<div class="ai-thinking"><span class="spinner"></span> AI is thinking</div>';
          } else {
            askOut.textContent = (text||''); 
          }
        }
        function renderAskMarkdown(label, text){
          if (!askOut) return;
          const labelHtml = label ? ('<div class="ai-meta">'+label+':</div>') : '';
          try {
            // Configure marked once
            if (window.marked && !renderAskMarkdown._init) {
              // Integrate highlight.js for fenced code blocks
              marked.setOptions({
                breaks: true,
                gfm: true,
                highlight: function(code, lang){
                  try {
                    if (lang && window.hljs && hljs.getLanguage(lang)) {
                      return hljs.highlight(code, { language: lang }).value;
                    } else if (window.hljs) {
                      return hljs.highlightAuto(code).value;
                    }
                  } catch(e) {}
                  return code;
                }
              });
              renderAskMarkdown._init = true;
            }
            var rawHtml = window.marked ? marked.parse(text || '') : (text || '');
            var clean = window.DOMPurify ? DOMPurify.sanitize(rawHtml) : rawHtml;
            askOut.innerHTML = labelHtml + clean;
            // Ensure highlight.js theme applies by adding hljs class and highlighting
            if (window.hljs) {
              askOut.querySelectorAll('pre code').forEach(function(el){
                el.classList.add('hljs');
                // If no explicit language, run auto-detect
                if (!/language-/.test(el.className)) {
                  try {
                    const res = hljs.highlightAuto(el.textContent || '');
                    el.innerHTML = res.value;
                  } catch(e) {}
                }
                hljs.highlightElement(el);
                // Add copy button
                try {
                  const pre = el.closest('pre');
                  if (pre && !pre.querySelector('.copy-btn')) {
                    const btn = document.createElement('button');
                    btn.className = 'copy-btn';
                    btn.type = 'button';
                    btn.textContent = 'Copy';
                    btn.addEventListener('click', function(ev){
                      ev.stopPropagation();
                      try { navigator.clipboard.writeText(el.textContent || ''); } catch(_) {}
                      btn.textContent = 'Copied';
                      setTimeout(function(){ btn.textContent = 'Copy'; }, 800);
                    });
                    pre.appendChild(btn);
                  }
                } catch(_){}
              });
            }
          } catch(e) {
            askOut.textContent = (label ? (label+': ') : '') + (text || '');
          }
        }
        function doAsk(url, text){
          const q = (text || '').trim();
          if (!q) { addAskLine('SYS', 'No transcript available'); return; }
          fetch(url, { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ input: q }) })
            .then(function(r){
              return r.text().then(function(txt){
                var data = null; try { data = JSON.parse(txt); } catch(e){}
                if (!r.ok) {
                  var msg = (data && (data.error || data.message || data.output)) || (txt && txt.trim()) || 'Request failed';
                  setAskStatus('SYS: ' + msg);
                  throw new Error('HTTP '+r.status);
                }
                return data || {};
              });
            })
            .then(function(data){
              const out = data && data.output ? data.output : '(no output)';
              const lang = url.indexOf('/hi/') > -1 ? 'HI' : 'EN';
              const prov = data && data.provider ? (lang+' ['+data.provider+ (data.model? (': '+data.model): '') +']') : lang;
              renderAskMarkdown(prov, out);
            })
            .catch(function(){ /* already reported */ });
        }
        // Maintain recent transcript history per language for Ask AI
        const EN_MAX = 100, HI_MAX = 100;
        var enHist = [];
        var hiHist = [];
        var enTotal = 0, hiTotal = 0; // total segments seen
        var lastSentEn = 0, lastSentHi = 0; // total count at last Ask
        function pushHist(lang, text){
          if (!text) return;
          if (lang === 'EN') { enHist.push(text); enTotal++; if (enHist.length > EN_MAX) enHist.shift(); }
          else if (lang === 'HI') { hiHist.push(text); hiTotal++; if (hiHist.length > HI_MAX) hiHist.shift(); }
        }
        function collectNew(lang){
          var total = (lang==='EN') ? enTotal : hiTotal;
          var last = (lang==='EN') ? lastSentEn : lastSentHi;
          var arr = (lang==='EN') ? enHist : hiHist;
          var newCount = total - last;
          if (newCount <= 0) return '';
          var start = Math.max(0, arr.length - newCount);
          var out = arr.slice(start).join('\n');
          if (lang==='EN') lastSentEn = total; else lastSentHi = total;
          return out;
        }
        function askFromTranscript(){
          if (enRunning) { const chunk = collectNew('EN'); if (!chunk) { setAskStatus('SYS: No transcript'); return; } setAskStatus('AI Thinking…'); doAsk('/api/ai/en', chunk); return; }
          if (hiRunning) { const chunk = collectNew('HI'); if (!chunk) { setAskStatus('SYS: No transcript'); return; } setAskStatus('AI Thinking…'); doAsk('/api/ai/hi', chunk); return; }
          setAskStatus('SYS: No active language (start English or Hindi)');
        }
        function setCapStatus(text){ 
          if (!capOut) return;
          // ANIMATION: Show AI thinking indicator for capture
          if (text && text.includes('Thinking')) {
            capOut.innerHTML = '<div class="ai-thinking"><span class="spinner"></span> AI is thinking</div>';
          } else {
            capOut.textContent = (text||''); 
          }
        }
        function renderCapMarkdown(label, text){
          if (!capOut) return;
          try {
            var rawHtml = window.marked ? marked.parse(text || '') : (text || '');
            var clean = window.DOMPurify ? DOMPurify.sanitize(rawHtml) : rawHtml;
            capOut.innerHTML = (label?('<div class="ai-meta">'+label+':</div>'):'') + clean;
            if (window.hljs) { capOut.querySelectorAll('pre code').forEach(function(el){ el.classList.add('hljs'); hljs.highlightElement(el); }); }
          } catch(e) { capOut.textContent = (label? (label+': '):'') + (text||''); }
        }
        function captureAction(){
          if (capturePanel) { capturePanel.classList.remove('hidden'); }
          btnCAP.classList.add('active','cap');
          setTimeout(()=>btnCAP.classList.remove('active','cap'), 300);
          setCapStatus('AI Thinking…');
          fetch('/api/capture', { method:'POST' })
            .then(function(r){ return r.text().then(function(txt){ var data=null; try{data=JSON.parse(txt);}catch(e){}; if(!r.ok){ setCapStatus('SYS: '+(data&&(data.error||data.message||data.output)||txt||'Request failed')); throw new Error('HTTP '+r.status);} return data||{}; }); })
            .then(function(data){ var out=data&&data.output?data.output:'(no output)'; var prov=data&&data.provider?('SC ['+data.provider+(data.model?(': '+data.model):'')+']'):'SC'; renderCapMarkdown(prov, out); })
            .catch(function(){ /* already reported */ });
        }
        // Expose simple helpers so other parts can render into Capture Window
        window.CAPTURE = {
          show: function(){ if (capturePanel) capturePanel.classList.remove('hidden'); },
          hide: function(){ if (capturePanel) capturePanel.classList.add('hidden'); },
          thinking: function(){ setCapStatus('AI Thinking…'); },
          error: function(msg){ setCapStatus('SYS: ' + (msg||'Error')); },
          set: function(label, text){ renderCapMarkdown(label||'SC', text||''); },
          clear: function(){ if (capOut) capOut.textContent = ''; }
        };
        function openSessions(){
          // placeholder; integrate sessions panel later
          btnSESS.classList.add('active','sess');
          setTimeout(()=>btnSESS.classList.remove('active','sess'), 300);
        }
function switchTab(which){
          var en = which === 'en';
          var hi = which === 'hi';
          var ed = which === 'edit';
          var sc = which === 'sc';
          tabEN.classList.toggle('active', en); paneEN.classList.toggle('hidden', !en);
          tabHI.classList.toggle('active', hi); paneHI.classList.toggle('hidden', !hi);
          tabEDIT.classList.toggle('active', ed); paneEDIT.classList.toggle('hidden', !ed);
          if (typeof tabSC !== 'undefined' && tabSC) { tabSC.classList.toggle('active', sc); }
          if (typeof paneSC !== 'undefined' && paneSC) { paneSC.classList.toggle('hidden', !sc); }
        }

        const PROVIDERS_EN = [
          { value: 'groq', label: 'Groq' },
          { value: 'gemini', label: 'Gemini' },
          { value: 'openrouter', label: 'OpenRouter' },
          { value: 'custom', label: 'Custom Provider' },
        ];
        const EN_VALID = new Set(PROVIDERS_EN.map(function(p){return p.value;}));
        function sanitizeEnProvider(p){
          if (!p || p === 'default') return 'groq';
          return EN_VALID.has(p) ? p : 'groq';
        }
        const PROVIDERS_HI = [
          { value: 'default', label: 'Default (Gemini)' },
          { value: 'gemini', label: 'Gemini' },
          { value: 'openrouter', label: 'OpenRouter' },
          { value: 'custom', label: 'Custom Provider' },
        ];
const MODELS = {
          groq: ['llama-3.1-8b-instant', 'llama-3.1-8b-instant'],
          gemini: ['gemini-2.5-flash'],
          openrouter: ['meta-llama/llama-3.1-70b', 'qwen/qwen2-7b-instruct', 'nousresearch/hermes-3-llama-3.1-8b-instant'],
          default_en: ['llama-3.1-8b-instant'],
          default_hi: ['gemini-2.5-flash'],
          sc_groq: ['meta-llama/llama-4-scout-17b-16e-instruct'],
          sc_gemini: ['gemini-2.5-flash'],
          sc_openrouter: ['google/gemini-2.5-flash'],
        };

        var providerCfgCache = { groq:{}, gemini:{}, openrouter:{} };
        function getProviderCfg(name){
          return providerCfgCache[name] || {};
        }
        function mergeModels(baseArr, extraStr, def){
          var set = {};
          var out = [];
          function add(x){ if (!x) return; if (!set[x]) { set[x] = 1; out.push(x); } }
          (baseArr||[]).forEach(add);
          if (extraStr) extraStr.split(',').map(function(s){return s.trim();}).forEach(add);
          if (def) {
            // move default to front if present
            var idx = out.indexOf(def);
            if (idx > 0) { out.splice(idx,1); out.unshift(def); }
            if (idx === -1) out.unshift(def);
          }
          return out;
        }
        function fillSelect(sel, opts){ sel.innerHTML = ''; opts.forEach(function(o){ var op=document.createElement('option'); op.value=o.value; op.textContent=o.label; sel.appendChild(op); }); }
        function fillModels(sel, provider, lang){
          if (provider === 'custom') { sel.innerHTML = ''; return; }
          var base = provider === 'default' ? (lang==='en'? MODELS.default_en : MODELS.default_hi) : (MODELS[provider] || []);
          var cfg = getProviderCfg(provider);
          var merged = mergeModels(base, cfg.extra_models, cfg.default_model);
          sel.innerHTML = '';
          merged.forEach(function(m){ var op=document.createElement('option'); op.value=m; op.textContent=m; sel.appendChild(op); });
        }

        function showModelField(provider, wrapSel, wrapCustomSel){
          const isCustom = provider === 'custom';
          wrapSel.classList.toggle('hidden', isCustom);
          wrapCustomSel.classList.toggle('hidden', !isCustom);
        }

        function setBadge(el, ok){
          if (!el) return;
          el.textContent = ok ? 'Key saved' : 'No key';
          el.classList.toggle('ok', !!ok);
          el.classList.toggle('missing', !ok);
        }
        var providersStateCache = null;
        function refreshProviderBadges(){
          if (!providersStateCache) return;
          function setProv(el, prov){
            if (!el) return;
            if (prov === 'default') { el.textContent = 'N/A'; el.classList.remove('ok'); el.classList.add('missing'); return; }
            var st = providersStateCache[prov];
            setBadge(el, st && st.has_key);
          }
setProv(enProvKeyBadge, enProvider.value);
          setProv(hiProvKeyBadge, hiProvider.value);
          setProv(scProvKeyBadge, scProvider ? scProvider.value : 'default');
        }
        function updateKeyBadges(){
          fetch('/api/providers/state').then(function(r){ return r.json(); }).then(function(s){
            providersStateCache = s;
            setBadge(badgeGroq, s.groq && s.groq.has_key);
            setBadge(badgeGemini, s.gemini && s.gemini.has_key);
            setBadge(badgeOpenRouter, s.openrouter && s.openrouter.has_key);
            setBadge(badgeCustom, s.custom && s.custom.has_key);
            setBadge(fallbackORKeyBadge, s.openrouter && s.openrouter.has_key);
            refreshProviderBadges();
          }).catch(function(){ /* ignore */ });
        }
        function loadSettings(){
          // Load settings from backend
          fetch('/api/settings').then(function(r){ return r.json(); }).then(function(data){
            // Provider-level
            providerCfgCache = data.providers || providerCfgCache;
            groqDefault.value = (providerCfgCache.groq && providerCfgCache.groq.default_model) || '';
            groqExtra.value = (providerCfgCache.groq && providerCfgCache.groq.extra_models) || '';
            if (groqStreaming) groqStreaming.checked = !!data.groq_streaming;
            geminiDefault.value = (providerCfgCache.gemini && providerCfgCache.gemini.default_model) || '';
            geminiExtra.value = (providerCfgCache.gemini && providerCfgCache.gemini.extra_models) || '';
            openrouterDefault.value = (providerCfgCache.openrouter && providerCfgCache.openrouter.default_model) || '';
            openrouterExtra.value = (providerCfgCache.openrouter && providerCfgCache.openrouter.extra_models) || '';
            // Fallback (common)
            if (data.fallback && data.fallback.openrouter_choice) { fallbackOpenRouter.value = (data.fallback.openrouter_choice || 'openai'); } else { fallbackOpenRouter.value = 'openai'; }
            customKey.value = '';

            // EN
            fillSelect(enProvider, PROVIDERS_EN);
            var enProv = sanitizeEnProvider(data.en && data.en.provider);
            enProvider.value = enProv;
            showModelField(enProv, enModelWrap, enModelCustomWrap);
            fillModels(enModel, enProvider.value, 'en');
            if (data.en && data.en.model) {
              enModel.value = data.en.model;
            } else if (enModel.options && enModel.options.length > 0) {
              enModel.value = enModel.options[0].value;
            }
            enModelCustom.value = (data.en && data.en.custom_model) || '';
            enPrompt.value = (data.en && data.en.prompt) || '';
            // HI
            fillSelect(hiProvider, PROVIDERS_HI);
            hiProvider.value = (data.hi && data.hi.provider) || 'gemini';
            showModelField(hiProvider.value, hiModelWrap, hiModelCustomWrap);
            fillModels(hiModel, hiProvider.value, 'hi');
            if (data.hi && data.hi.model) {
              hiModel.value = data.hi.model;
            } else if (hiModel.options && hiModel.options.length > 0) {
              hiModel.value = hiModel.options[0].value;
            }
            hiModelCustom.value = (data.hi && data.hi.custom_model) || '';
            hiPrompt.value = (data.hi && data.hi.prompt) || '';

            // Screen Capture
            if (scProvider) {
              fillSelect(scProvider, [ { value:'gemini', label:'Gemini' }, { value:'groq', label:'Groq' }, { value:'custom', label:'Custom Provider' } ]);
              scProvider.value = (data.sc && data.sc.provider) || 'gemini';
              showModelField(scProvider.value, scModelWrap, scModelCustomWrap);
              var baseSC = scProvider.value==='gemini'? MODELS.sc_gemini : (scProvider.value==='groq'? MODELS.sc_groq : MODELS.sc_openrouter);
              var cfgSC = (data.sc_providers && data.sc_providers[scProvider.value]) || {};
              var mergedSC = mergeModels(baseSC, cfgSC.extra_models, cfgSC.default_model);
              scModel.innerHTML=''; (mergedSC||[]).forEach(function(m){ var op=document.createElement('option'); op.value=m; op.textContent=m; scModel.appendChild(op); });
              if (data.sc && data.sc.model) scModel.value = data.sc.model; else if (scModel.options && scModel.options.length>0) scModel.value = scModel.options[0].value;
              scModelCustom.value = (data.sc && data.sc.custom_model) || '';
              scPrompt.value = (data.sc && data.sc.prompt) || '';

              // SC providers in Edit Providers
              if (scGeminiDefault) scGeminiDefault.value = (data.sc_providers && data.sc_providers.gemini && data.sc_providers.gemini.default_model) || 'gemini-2.5-flash';
              if (scGeminiExtra) scGeminiExtra.value = (data.sc_providers && data.sc_providers.gemini && data.sc_providers.gemini.extra_models) || '';
              if (scGroqDefault) scGroqDefault.value = (data.sc_providers && data.sc_providers.groq && data.sc_providers.groq.default_model) || 'meta-llama/llama-4-scout-17b-16e-instruct';
              if (scGroqExtra) scGroqExtra.value = (data.sc_providers && data.sc_providers.groq && data.sc_providers.groq.extra_models) || '';
              if (scORDefault) scORDefault.value = (data.sc_providers && data.sc_providers.openrouter && data.sc_providers.openrouter.default_model) || 'google/gemini-2.5-flash';
              if (scORExtra) scORExtra.value = (data.sc_providers && data.sc_providers.openrouter && data.sc_providers.openrouter.extra_models) || '';
              if (scORFallback) scORFallback.checked = !!data.sc_fallback_or;
            }

            // Apply masking per provider for API keys
            updateKeyBadges();
          });
        }

        function openSettings(){
          btnSET.classList.add('active','set');
          setTimeout(()=>btnSET.classList.remove('active','set'), 200);
          loadSettings();
          switchTab('en');
          // ANIMATION: Smooth modal transition
          settingsModal.style.display = 'flex';
          setTimeout(()=>settingsModal.classList.add('open'), 10);
          settingsModal.setAttribute('aria-hidden','false');
          updateKeyBadges();
        }
        function closeSettings(){
          // ANIMATION: Smooth modal close
          settingsModal.classList.remove('open');
          setTimeout(()=>settingsModal.style.display = 'none', 300);
          settingsModal.setAttribute('aria-hidden','true');
        }
        btnEN.addEventListener('click', toggleEN);
        btnHI.addEventListener('click', toggleHI);
        btnAI.addEventListener('click', askAI);
        btnCAP.addEventListener('click', captureAction);
        btnTR.addEventListener('click', toggleTR);
        btnSESS.addEventListener('click', openSessions);
        btnSET.addEventListener('click', openSettings);
        btnCloseSettings.addEventListener('click', closeSettings);
        settingsModal.addEventListener('click', function(e){ var t = e.target; if (t && t.dataset && t.dataset.close === 'settings') closeSettings(); });
        
        // Mobile stats sheet handlers
        if (btnOpenStats) btnOpenStats.addEventListener('click', function(){
          if (statsSheet) statsSheet.classList.add('open');
          if (statsOverlay) statsOverlay.classList.add('open');
        });
        if (btnCloseStats) btnCloseStats.addEventListener('click', function(){
          if (statsSheet) statsSheet.classList.remove('open');
          if (statsOverlay) statsOverlay.classList.remove('open');
        });
        if (statsOverlay) statsOverlay.addEventListener('click', function(){
          statsSheet.classList.remove('open');
          statsOverlay.classList.remove('open');
        });
tabEN.addEventListener('click', function(){switchTab('en');});
        tabHI.addEventListener('click', function(){switchTab('hi');});
        tabEDIT.addEventListener('click', function(){switchTab('edit');});
        if (tabSC) tabSC.addEventListener('click', function(){switchTab('sc');});
        enProvider.addEventListener('change', function(){ showModelField(enProvider.value, enModelWrap, enModelCustomWrap); fillModels(enModel, enProvider.value, 'en'); if (enModel.options && enModel.options.length>0) { enModel.value = enModel.options[0].value; } refreshProviderBadges(); });
hiProvider.addEventListener('change', function(){ showModelField(hiProvider.value, hiModelWrap, hiModelCustomWrap); fillModels(hiModel, hiProvider.value, 'hi'); if (hiModel.options && hiModel.options.length>0) { hiModel.value = hiModel.options[0].value; } refreshProviderBadges(); });
        if (scProvider) scProvider.addEventListener('change', function(){ showModelField(scProvider.value, scModelWrap, scModelCustomWrap);
          var base = scProvider.value==='gemini'? MODELS.sc_gemini : (scProvider.value==='groq'? MODELS.sc_groq : MODELS.sc_openrouter);
          scModel.innerHTML=''; (base||[]).forEach(function(m){ var op=document.createElement('option'); op.value=m; op.textContent=m; scModel.appendChild(op); });
          if (scModel.options && scModel.options.length>0) { scModel.value = scModel.options[0].value; }
          refreshProviderBadges(); });
        btnSaveEN.addEventListener('click', function(){
          const payload = { provider: enProvider.value, model: enProvider.value==='custom'? undefined: enModel.value, custom_model: enProvider.value==='custom'? enModelCustom.value: undefined, prompt: enPrompt.value };
          fetch('/api/settings/en', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(payload) })
            .then(function(){ btnSaveEN.textContent = 'Saved!'; setTimeout(function(){btnSaveEN.textContent='Save English';}, 1000); updateKeyBadges(); });
        });
btnSaveHI.addEventListener('click', function(){
          const payload = { provider: hiProvider.value, model: hiProvider.value==='custom'? undefined: hiModel.value, custom_model: hiProvider.value==='custom'? hiModelCustom.value: undefined, prompt: hiPrompt.value };
          fetch('/api/settings/hi', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(payload) })
            .then(function(){ btnSaveHI.textContent = 'Saved!'; setTimeout(function(){btnSaveHI.textContent='Save Hindi';}, 1000); updateKeyBadges(); });
        });
        if (btnSaveSC) {
          btnSaveSC.addEventListener('click', function(){
            const payload = { provider: scProvider.value, model: scProvider.value==='custom'? undefined: scModel.value, custom_model: scProvider.value==='custom'? scModelCustom.value: undefined, prompt: scPrompt.value };
            fetch('/api/settings/sc', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(payload) })
              .then(function(){ btnSaveSC.textContent = 'Saved!'; setTimeout(function(){btnSaveSC.textContent='Save Screen Capture';}, 1000); updateKeyBadges(); });
          });
        }
        function removeKey(name, btn){
          if (!confirm('Remove saved key for '+name+'?')) return;
          var old = btn.textContent; btn.textContent = 'Removing…'; btn.disabled = true;
          fetch('/api/providers/'+name+'/key', { method:'DELETE' })
            .then(function(){ updateKeyBadges(); })
            .finally(function(){ btn.textContent = old; btn.disabled = false; });
        }
        rmGroq.addEventListener('click', function(){ removeKey('groq', rmGroq); });
        rmGemini.addEventListener('click', function(){ removeKey('gemini', rmGemini); });
        rmOpenRouter.addEventListener('click', function(){ removeKey('openrouter', rmOpenRouter); });
        rmCustom.addEventListener('click', function(){ removeKey('custom', rmCustom); });

        btnSaveProviders.addEventListener('click', function(){
          // Build non-secret provider cfgs (no API keys here)
          const cfg = { groq: { default_model: groqDefault.value, extra_models: groqExtra.value }, gemini: { default_model: geminiDefault.value, extra_models: geminiExtra.value }, openrouter: { default_model: openrouterDefault.value, extra_models: openrouterExtra.value }, groq_streaming: !!(groqStreaming && groqStreaming.checked) };
          const scCfg = { groq: { default_model: scGroqDefault.value, extra_models: scGroqExtra.value }, gemini: { default_model: scGeminiDefault.value, extra_models: scGeminiExtra.value }, openrouter: { default_model: scORDefault.value, extra_models: scORExtra.value } };
          // Save provider cfgs server-side
          let tasks = [
            fetch('/api/settings/providers', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(cfg) }),
            fetch('/api/settings/providers-sc', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(scCfg) }),
            fetch('/api/settings/fallback', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ openrouter_choice: fallbackOpenRouter.value }) }),
            fetch('/api/settings/sc-fallback', { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ or_fallback: !!(scORFallback && scORFallback.checked), model: scORDefault.value }) })
          ];
          // Send secrets to backend keyring only if provided
          function postKey(name, v){ if (v && v.trim().length>0) { tasks.push(fetch(`/api/providers/${name}/key`, { method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify({ api_key: v }) })); } }
          postKey('groq', groqKey.value);
          postKey('gemini', geminiKey.value);
          postKey('openrouter', openrouterKey.value);
          postKey('custom', customKey.value);
          Promise.allSettled(tasks).then(function(){
            // Clear inputs on success to avoid lingering secrets in memory/UI
            groqKey.value = '';
            geminiKey.value = '';
            openrouterKey.value = '';
            customKey.value = '';
            btnSaveProviders.textContent = 'Saved!'; setTimeout(function(){btnSaveProviders.textContent='Save Providers';}, 1000);
            // refresh model lists in EN/HI panes based on new provider configs
            fillModels(enModel, enProvider.value, 'en');
            fillModels(hiModel, hiProvider.value, 'hi');
            updateKeyBadges();
          });
        });

        try {
          // Auto-detect WebSocket protocol based on current page protocol
          const wsProto = location.protocol === 'https:' ? 'wss:' : 'ws:';
          wsEn = new WebSocket(`${wsProto}//${location.host}/ws/transcript`);
          wsEn.onopen = () => dotEN.classList.add('on');
          wsEn.onclose = () => dotEN.classList.remove('on');
          wsEn.onmessage = e => { const t = normalizeText(e.data || ''); pushHist('EN', t); addLine('EN', e.data); };
          wsHi = new WebSocket(`${wsProto}//${location.host}/ws/transcript-hi`);
          wsHi.onopen = () => dotHI.classList.add('on');
          wsHi.onclose = () => dotHI.classList.remove('on');
          wsHi.onmessage = e => { const t = normalizeText(e.data || ''); pushHist('HI', t); addLine('HI', e.data); };
        } catch(e) { console.error(e); }

        // Sessions functionality
        const sessionsModal = document.getElementById('sessionsModal');
        const btnCloseSessions = document.getElementById('btnCloseSessions');
        const btnNewSession = document.getElementById('btnNewSession');
        const searchSessions = document.getElementById('searchSessions');
        const sessionsList = document.getElementById('sessionsList');
        const sessionViewerModal = document.getElementById('sessionViewerModal');
        const btnCloseViewer = document.getElementById('btnCloseViewer');
        const viewerTitle = document.getElementById('viewerTitle');
        const viewerContent = document.getElementById('viewerContent');
        let currentSessionId = null;
        let allSessions = [];

        function openSessions(){
          btnSESS.classList.add('active','sess');
          setTimeout(()=>btnSESS.classList.remove('active','sess'), 200);
          loadSessions();
          // ANIMATION: Smooth modal transition
          sessionsModal.style.display = 'flex';
          setTimeout(()=>sessionsModal.classList.add('open'), 10);
          sessionsModal.setAttribute('aria-hidden','false');
        }
        function closeSessions(){
          // ANIMATION: Smooth modal close
          sessionsModal.classList.remove('open');
          setTimeout(()=>sessionsModal.style.display = 'none', 300);
          sessionsModal.setAttribute('aria-hidden','true');
        }
        function loadSessions(){
          sessionsList.innerHTML = '<div class="hint" style="text-align:center; padding:20px;">Loading...</div>';
          fetch('/api/sessions?limit=50')
            .then(r=>r.json())
            .then(sessions=>{
              allSessions = sessions;
              renderSessions(sessions);
            })
            .catch(()=>{ sessionsList.innerHTML = '<div class="hint" style="text-align:center; padding:20px; color:#e53935;">Failed to load sessions</div>'; });
        }
        function renderSessions(sessions){
          if (!sessions || sessions.length === 0) {
            sessionsList.innerHTML = '<div class="hint" style="text-align:center; padding:20px;">No sessions yet. Click "+  New Session" to start.</div>';
            return;
          }
          sessionsList.innerHTML = '';
          sessions.forEach(s=>{
            const item = document.createElement('div');
            item.className = 'session-item';
            if (s.id === currentSessionId) item.classList.add('active');
            const title = document.createElement('div');
            title.className = 'session-title';
            title.textContent = s.title || 'Untitled Session';
            const meta = document.createElement('div');
            meta.className = 'session-meta';
            const date = new Date(s.updated_at * 1000);
            meta.innerHTML = `<span>${s.message_count} messages</span><span>${date.toLocaleDateString()}</span>`;
            
            // ANIMATION: Progress bar based on message count
            const progress = document.createElement('div');
            progress.className = 'session-progress';
            const maxMessages = 100; // Assume max for visualization
            const progressWidth = Math.min((s.message_count / maxMessages) * 100, 100);
            progress.style.width = progressWidth + '%';
            
            const actions = document.createElement('div');
            actions.className = 'session-actions';
            const btnView = document.createElement('button');
            btnView.className = 'btn-ghost btn-mini';
            btnView.textContent = 'View';
            btnView.onclick = (e)=>{ e.stopPropagation(); viewSession(s.id); };
            
            const btnRename = document.createElement('button');
            btnRename.className = 'btn-ghost btn-mini';
            btnRename.textContent = 'Rename';
            btnRename.onclick = (e)=>{ e.stopPropagation(); renameSession(s.id, s.title); };
            
            const btnExport = document.createElement('button');
            btnExport.className = 'btn-ghost btn-mini';
            btnExport.textContent = 'Export';
            btnExport.onclick = (e)=>{ e.stopPropagation(); exportSession(s.id, s.title); };
            
            const btnCopy = document.createElement('button');
            btnCopy.className = 'btn-ghost btn-mini';
            btnCopy.textContent = 'Copy';
            btnCopy.onclick = (e)=>{ e.stopPropagation(); copySession(s.id); };
            
            const btnDownload = document.createElement('button');
            btnDownload.className = 'btn-ghost btn-mini';
            btnDownload.textContent = 'Download';
            btnDownload.onclick = (e)=>{ e.stopPropagation(); downloadSession(s.id, s.title); };
            
            const btnDelete = document.createElement('button');
            btnDelete.className = 'btn-ghost btn-mini';
            btnDelete.textContent = 'Delete';
            btnDelete.style.color = '#e53935';
            btnDelete.onclick = (e)=>{ e.stopPropagation(); deleteSession(s.id); };
            
            actions.appendChild(btnView);
            actions.appendChild(btnRename);
            actions.appendChild(btnExport);
            actions.appendChild(btnCopy);
            actions.appendChild(btnDownload);
            actions.appendChild(btnDelete);
            item.appendChild(title);
            item.appendChild(meta);
            item.appendChild(actions);
            item.appendChild(progress); // Add progress bar
            item.onclick = ()=>{ setCurrentSession(s.id); };
            sessionsList.appendChild(item);
          });
        }
        function viewSession(id){
          viewerContent.innerHTML = '<div class="hint" style="text-align:center; padding:20px;">Loading...</div>';
          fetch(`/api/sessions/${id}?limit=1000`)
            .then(r=>r.json())
            .then(data=>{
              viewerTitle.textContent = data.title || 'Session History';
              viewerContent.innerHTML = '';
              if (!data.messages || data.messages.length === 0) {
                viewerContent.innerHTML = '<div class="hint" style="text-align:center; padding:20px;">No messages in this session yet.</div>';
              } else {
                data.messages.forEach(msg=>{
                  const msgDiv = document.createElement('div');
                  msgDiv.style.cssText = 'margin-bottom:16px; padding:12px; border-radius:6px; background:#0a0a0a;';
                  
                  const header = document.createElement('div');
                  header.style.cssText = 'font-weight:600; margin-bottom:6px; color:' + (msg.role === 'user' ? '#2196f3' : '#4caf50');
                  header.innerHTML = `${msg.role === 'user' ? '👤 User' : '🤖 Assistant'}`;
                  
                  const content = document.createElement('div');
                  content.style.cssText = 'color:#e6e6e6; line-height:1.5; white-space:pre-wrap;';
                  content.textContent = msg.content;
                  
                  const meta = document.createElement('div');
                  meta.style.cssText = 'margin-top:6px; font-size:11px; color:#8aa;';
                  const date = new Date(msg.timestamp * 1000).toLocaleString();
                  let metaText = date;
                  if (msg.language) metaText += ` • ${msg.language.toUpperCase()}`;
                  if (msg.provider) metaText += ` • ${msg.provider}`;
                  if (msg.model) metaText += ` • ${msg.model}`;
                  meta.textContent = metaText;
                  
                  msgDiv.appendChild(header);
                  msgDiv.appendChild(content);
                  msgDiv.appendChild(meta);
                  viewerContent.appendChild(msgDiv);
                });
              }
              // ANIMATION: Open viewer modal smoothly
              sessionViewerModal.style.display = 'flex';
              setTimeout(()=>sessionViewerModal.classList.add('open'), 10);
              sessionViewerModal.setAttribute('aria-hidden','false');
            });
        }
        function deleteSession(id){
          if (!confirm('Delete this session? This cannot be undone.')) return;
          fetch(`/api/sessions/${id}`, { method:'DELETE' })
            .then(()=>{ loadSessions(); if (currentSessionId === id) currentSessionId = null; });
        }
        function renameSession(id, oldTitle){
          const newTitle = prompt('Enter new session title:', oldTitle);
          if (!newTitle || newTitle === oldTitle) return;
          fetch(`/api/sessions/${id}`, {
            method:'PUT',
            headers:{'Content-Type':'application/json'},
            body: JSON.stringify({ title: newTitle })
          }).then(()=>{ loadSessions(); });
        }
        function exportSession(id, title){
          fetch(`/api/sessions/${id}?limit=1000`)
            .then(r=>r.json())
            .then(data=>{
              const blob = new Blob([JSON.stringify(data, null, 2)], {type:'application/json'});
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = `${title.replace(/[^a-z0-9]/gi, '_')}.json`;
              a.click();
              URL.revokeObjectURL(url);
            });
        }
        function copySession(id){
          fetch(`/api/sessions/${id}?limit=1000`)
            .then(r=>r.json())
            .then(data=>{
              let text = `${data.title}\n${'='.repeat(data.title.length)}\n\n`;
              data.messages.forEach(msg=>{
                const icon = msg.role === 'user' ? '👤' : '🤖';
                text += `${icon} ${msg.role.toUpperCase()}:\n${msg.content}\n\n`;
              });
              navigator.clipboard.writeText(text).then(()=>{
                alert('Session copied to clipboard!');
              });
            });
        }
        function downloadSession(id, title){
          fetch(`/api/sessions/${id}?limit=1000`)
            .then(r=>r.json())
            .then(data=>{
              let md = `# ${data.title}\n\n`;
              md += `**Created:** ${new Date(data.created_at * 1000).toLocaleString()}\n`;
              md += `**Messages:** ${data.message_count}\n\n---\n\n`;
              data.messages.forEach(msg=>{
                const icon = msg.role === 'user' ? '👤' : '🤖';
                md += `### ${icon} ${msg.role.charAt(0).toUpperCase() + msg.role.slice(1)}\n\n`;
                md += `${msg.content}\n\n`;
                if (msg.provider) md += `*Provider: ${msg.provider}${msg.model ? ', Model: ' + msg.model : ''}*\n\n`;
                md += `---\n\n`;
              });
              const blob = new Blob([md], {type:'text/markdown'});
              const url = URL.createObjectURL(blob);
              const a = document.createElement('a');
              a.href = url;
              a.download = `${title.replace(/[^a-z0-9]/gi, '_')}.md`;
              a.click();
              URL.revokeObjectURL(url);
            });
        }
        function setCurrentSession(id){
          currentSessionId = id;
          fetch('/api/sessions/current', {
            method:'POST',
            headers:{'Content-Type':'application/json'},
            body: JSON.stringify({ session_id: id })
          }).then(()=>{ renderSessions(allSessions); });
        }
        function closeViewer(){
          // ANIMATION: Smooth modal close
          sessionViewerModal.classList.remove('open');
          setTimeout(()=>sessionViewerModal.style.display = 'none', 300);
          sessionViewerModal.setAttribute('aria-hidden','true');
        }
        btnCloseSessions.addEventListener('click', closeSessions);
        sessionsModal.addEventListener('click', (e)=>{ if (e.target.dataset.close === 'sessions') closeSessions(); });
        btnCloseViewer.addEventListener('click', closeViewer);
        sessionViewerModal.addEventListener('click', (e)=>{ if (e.target.dataset.close === 'viewer') closeViewer(); });
        btnNewSession.addEventListener('click', ()=>{
          const title = prompt('Enter session title:') || 'New Session';
          fetch('/api/sessions', {
            method:'POST',
            headers:{'Content-Type':'application/json'},
            body: JSON.stringify({ title })
          }).then(()=>{ loadSessions(); });
        });
        searchSessions.addEventListener('input', (e)=>{
          const q = e.target.value.trim();
          if (!q) { renderSessions(allSessions); return; }
          fetch(`/api/sessions/search?q=${encodeURIComponent(q)}`)
            .then(r=>r.json())
            .then(renderSessions);
        });

        // ANIMATION: Add ripple effect to all buttons
        function addRipple(e){
          const btn = e.currentTarget;
          const rect = btn.getBoundingClientRect();
          const x = e.clientX - rect.left;
          const y = e.clientY - rect.top;
          const ripple = document.createElement('span');
          ripple.className = 'ripple';
          ripple.style.left = x + 'px';
          ripple.style.top = y + 'px';
          btn.appendChild(ripple);
          setTimeout(()=>ripple.remove(), 600);
        }
        document.querySelectorAll('.btn, .btn-solid, .btn-ghost').forEach(btn=>{
          btn.addEventListener('click', addRipple);
        });

        applyTranscriptVisibility();
        applyAskVisibility();
        // Optional: persist ask note per session in memory only
        refresh();
        refreshRemote();
        // Don't auto-apply checkbox states on load - let user choose manually
        // Show hint about saved defaults instead
        if (rcHint) rcHint.textContent = '';
        setInterval(refresh, 1000);
      })();
    </script>
  </body>
</html>"#.to_string()
}

pub fn mobile_html(addrs: &[String], port: u16) -> String {
    // Use first address for QR
    let primary = addrs.get(0).cloned().unwrap_or_else(|| "127.0.0.1".to_string());
    let url = format!("http://{}:{}/app", primary, port);
    let svg = match QrCode::new(url.as_bytes()) {
        Ok(code) => code.render::<qrcode::render::svg::Color>().min_dimensions(256,256).build(),
        Err(_) => "<p>Failed to generate QR</p>".to_string(),
    };
    let mut list_html = String::new();
    for ip in addrs {
        let u = format!("http://{}:{}/app", ip, port);
        list_html.push_str(&format!("<li><a href=\"{}\">{}</a></li>", u, u));
    }
    {
        let template = r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Voxora Mobile</title>
    <link rel="icon" href="data:">
    <style>
      :root { --bg:#0b0f1a; --fg:#e6e6e6; --muted:#a8b3cf; --card:#0f1629; --edge:#223356; --accent:#1e88e5; }
      html, body { height:100%; }
      body { margin:0; background:var(--bg); color:var(--fg); font-family: Inter, system-ui, -apple-system, Segoe UI, Roboto, Arial; display:grid; place-items:center; }
      .hero { text-align:center; padding:24px; max-width:900px; width:100%; }
      .logo { display:flex; align-items:center; justify-content:center; gap:10px; margin-bottom:10px; }
      .logo .mark { width:40px; height:40px; display:inline-block; }
      h1 { margin:0 0 6px; font-size:26px; }
      .tagline { margin:0 0 16px; color:var(--muted); }
      .card { margin:0 auto; display:inline-block; padding:20px; background:var(--card); border:1px solid var(--edge); border-radius:14px; box-shadow: 0 8px 24px rgba(0,0,0,.35); }
      .qr { display:flex; align-items:center; justify-content:center; }
      .hint { color:var(--muted); margin:10px 0 16px; }
      .list { margin:12px auto 0; text-align:left; max-width:480px; }
      .list a { color: var(--accent); text-decoration: none; }
      .list a:hover { text-decoration: underline; }
      .back { margin-top:18px; display:inline-block; color:var(--accent); text-decoration:none; }
    </style>
  </head>
  <body>
    <div class="hero">
      <div class="logo">
        <img class="mark" src="/static/logo.svg" alt="Voxora logo">
      </div>
      <h1>Voxora Mobile</h1>
      <p class="tagline">Open the control UI on your phone via QR</p>
      <div class="card">
        <div class="qr">{svg}</div>
      </div>
      <div class="list">
        <p>Or open one of these addresses:</p>
        <ul>{list}</ul>
      </div>
      <a class="back" href="/">← Back to Home</a>
    </div>
  </body>
</html>"#;
        template.replace("{svg}", &svg).replace("{list}", &list_html)
    }
}
