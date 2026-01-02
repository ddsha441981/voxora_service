#!/usr/bin/env bash
set -Eeuo pipefail

# Voxora Service: Ubuntu setup/build/run helper
# Usage:
#   ./scripts/setup_ubuntu.sh [--rustls] [--run]
# Env (optional):
#   ENGLISH_GO_SERVER_URL=ws://127.0.0.1:8085/ws
#   HINDI_GO_SERVER_URL=ws://127.0.0.1:8086/ws
#   GROQ_KEY=... GEMINI_KEY=... OPENROUTER_KEY=...   # if provided, keys will be saved to OS keyring via HTTP
#
# Behavior:
#   - Installs system deps (build-essential, pkg-config, libssl-dev unless --rustls)
#   - Installs rustup if missing
#   - (optional) Switches reqwest to rustls in Cargo.toml when --rustls passed
#   - Ensures Linux helper binaries are executable (bin/*)
#   - Builds in release mode
#   - If --run passed, starts the service

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
cd "$ROOT_DIR"

log(){ echo -e "\033[1;36m[setup]\033[0m $*"; }
warn(){ echo -e "\033[1;33m[warn]\033[0m $*"; }
err(){ echo -e "\033[1;31m[err ]\033[0m $*"; }

RUSTLS=0
RUN_AFTER=0
for arg in "$@"; do
  case "$arg" in
    --rustls) RUSTLS=1 ;;
    --run) RUN_AFTER=1 ;;
    *) warn "Unknown arg: $arg" ;;
  esac
done

log "Updating apt index"
sudo apt-get update -y

log "Installing base packages"
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
  build-essential pkg-config curl git ca-certificates

if [[ "$RUSTLS" -eq 0 ]]; then
  log "Installing OpenSSL development libraries (native-tls)"
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y libssl-dev
else
  log "Using rustls (no OpenSSL dev libs required)"
fi

log "Installing keyring runtime"
sudo DEBIAN_FRONTEND=noninteractive apt-get install -y gnome-keyring libsecret-1-0 >/dev/null 2>&1 || true

if ! command -v cargo >/dev/null 2>&1; then
  log "Installing Rust toolchain via rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
else
  log "Rust toolchain already present"
fi

if [[ "$RUSTLS" -eq 1 ]]; then
  if grep -q '^reqwest\s*=\s*{.*rustls-tls' Cargo.toml; then
    log "Cargo.toml already configured for rustls"
  else
    log "Patching Cargo.toml to use reqwest with rustls"
    cp Cargo.toml Cargo.toml.bak
    # Replace reqwest line (simple heuristic). If no existing line, append.
    if grep -q '^reqwest\s*=\s*' Cargo.toml; then
      sed -i 's/^reqwest\s*=.*/reqwest = { version = "0.12", default-features = false, features = ["json", "gzip", "rustls-tls"] }/' Cargo.toml
    else
      printf '\nreqwest = { version = "0.12", default-features = false, features = ["json", "gzip", "rustls-tls"] }\n' >> Cargo.toml
    fi
  fi
fi

log "Ensuring Linux helper binaries are executable (if present)"
CHANGED=0
for f in bin/go-server-en-linux bin/go-server-hi-linux bin/SpeakerCapture; do
  if [[ -f "$f" ]]; then
    chmod +x "$f" || true
    CHANGED=1
  fi
done
if [[ "$CHANGED" -eq 0 ]]; then
  warn "No helper binaries found in bin/. Place Linux binaries (go-server-en-linux, go-server-hi-linux, SpeakerCapture)."
fi

log "Building voxora-service (release)"
cargo build --release

BIN="${ROOT_DIR}/target/release/voxora-service"
if [[ ! -x "$BIN" ]]; then
  err "Build did not produce executable at $BIN"
  exit 1
fi

# Optionally set API keys if provided as env vars
set_key(){
  local name="$1"; shift
  local val="$1"; shift
  if [[ -n "$val" ]]; then
    log "Saving API key for $name via HTTP"
    curl -sS -X POST http://127.0.0.1:8080/api/providers/${name}/key \
      -H 'Content-Type: application/json' \
      -d "{\"api_key\":\"${val}\"}" || warn "Failed to save key for $name (is the service running?)"
  fi
}

if [[ "$RUN_AFTER" -eq 1 ]]; then
  log "Starting voxora-service"
  ENGLISH_GO_SERVER_URL="${ENGLISH_GO_SERVER_URL:-ws://127.0.0.1:8085/ws}" \
  HINDI_GO_SERVER_URL="${HINDI_GO_SERVER_URL:-ws://127.0.0.1:8086/ws}" \
  "$BIN" &
  APP_PID=$!
  log "Service PID: $APP_PID"
  # Give it a moment to start before posting keys
  sleep 2
  set_key groq "${GROQ_KEY:-}"
  set_key gemini "${GEMINI_KEY:-}"
  set_key openrouter "${OPENROUTER_KEY:-}"
  log "Open http://127.0.0.1:8080/app"
  wait "$APP_PID"
else
  log "Done. To run:"
  echo "ENGLISH_GO_SERVER_URL=ws://127.0.0.1:8085/ws HINDI_GO_SERVER_URL=ws://127.0.0.1:8086/ws $BIN"
  echo "Then set keys via UI or HTTP:"
  echo "curl -X POST http://127.0.0.1:8080/api/providers/groq/key -H 'Content-Type: application/json' -d '{"api_key":"…"}'"
fi
