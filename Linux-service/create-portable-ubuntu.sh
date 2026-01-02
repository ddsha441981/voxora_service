#!/usr/bin/env bash
# Create a portable Voxora package for Ubuntu/Linux
# Usage: ./create-portable-ubuntu.sh

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$ROOT/dist/voxora-portable-linux"
SERVICE_BIN="$ROOT/target/release/voxora-service"

printf "${CYAN}Building release (if needed)...${NC}\n"
if [[ ! -f "$SERVICE_BIN" ]]; then
  cargo build --release --bin voxora-service
fi

printf "${CYAN}Preparing dist folder...${NC}\n"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

printf "${CYAN}Copying service binary...${NC}\n"
cp "$SERVICE_BIN" "$DIST_DIR/"
chmod +x "$DIST_DIR/voxora-service"
printf "${GREEN}✓ voxora-service copied${NC}\n"

# Optional folders
copy_dir_if_exists() {
  local src="$1"
  local dst="$2"
  if [[ -d "$src" ]]; then
    cp -r "$src" "$dst/"
    printf "${GREEN}✓ Copied %s/${NC}\n" "$(basename "$src")"
  fi
}

# Copy bin/ (capture + go servers)
if [[ -d "$ROOT/bin" ]]; then
  cp -r "$ROOT/bin" "$DIST_DIR/"
  chmod +x "$DIST_DIR/bin"/* || true
  printf "${GREEN}✓ Copied bin/ (made executables)${NC}\n"
fi

copy_dir_if_exists "$ROOT/data"   "$DIST_DIR"
copy_dir_if_exists "$ROOT/static" "$DIST_DIR"

# Create logs directory
mkdir -p "$DIST_DIR/logs"
printf "${GREEN}✓ Created logs/ directory${NC}\n"

# Copy helper scripts/docs for Ubuntu (include user-mode installer)
for f in install-ubuntu.sh install-ubuntu-user.sh uninstall-ubuntu.sh README-UBUNTU.md; do
  if [[ -f "$ROOT/$f" ]]; then
    cp "$ROOT/$f" "$DIST_DIR/"
    [[ "$f" == *.sh ]] && chmod +x "$DIST_DIR/$f" || true
    printf "${GREEN}✓ Copied %s${NC}\n" "$f"
  fi
done

# Final message
printf "\n${CYAN}Package created at:${NC} ${YELLOW}%s${NC}\n" "$DIST_DIR"
printf "${CYAN}To install as a system service:${NC}\n"
printf "  1) cd \"%s\"\n" "$DIST_DIR"
printf "  2) sudo ./install-ubuntu.sh\n"
printf "  3) sudo systemctl enable --now voxora\n\n"
