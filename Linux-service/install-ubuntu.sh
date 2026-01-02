#!/bin/bash
# Voxora Service Installation Script for Ubuntu
# Run with: sudo ./install-ubuntu.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SERVICE_NAME="voxora"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${INSTALL_DIR:-/opt/voxora}"
CURRENT_USER="${SUDO_USER:-$USER}"

# User-mode shortcut: install as a systemd user service if --user flag is provided
if [[ "${1:-}" == "--user" || "${1:-}" == "-u" ]]; then
    SCRIPT_DIR_USER="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    if [[ -x "$SCRIPT_DIR_USER/install-ubuntu-user.sh" ]]; then
        bash "$SCRIPT_DIR_USER/install-ubuntu-user.sh"
        exit 0
    else
        echo "install-ubuntu-user.sh not found next to this script." >&2
        exit 1
    fi
fi

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Voxora Service Installer for Ubuntu  ${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}ERROR: Please run as root or with sudo${NC}"
    echo "Usage: sudo ./install-ubuntu.sh"
    exit 1
fi

# Check if binary exists
if [ ! -f "$SCRIPT_DIR/voxora-service" ]; then
    echo -e "${RED}ERROR: voxora-service binary not found${NC}"
    echo "Please build first: cargo build --release"
    echo "Then copy target/release/voxora-service here"
    exit 1
fi

# Check required binaries
echo -e "${CYAN}Checking required binaries...${NC}"
MISSING=""
if [ ! -f "$SCRIPT_DIR/bin/go-server-en-linux" ]; then
    MISSING="$MISSING\n  - bin/go-server-en-linux"
fi
if [ ! -f "$SCRIPT_DIR/bin/go-server-hi-linux" ]; then
    MISSING="$MISSING\n  - bin/go-server-hi-linux"
fi
if [ ! -f "$SCRIPT_DIR/bin/SpeakerCapture" ]; then
    MISSING="$MISSING\n  - bin/SpeakerCapture"
fi

if [ -n "$MISSING" ]; then
    echo -e "${YELLOW}WARNING: Missing binaries:${NC}"
    echo -e "${YELLOW}$MISSING${NC}"
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Create installation directory
echo -e "${CYAN}Creating installation directory...${NC}"
mkdir -p "$INSTALL_DIR"
echo -e "${GREEN}✓ Created $INSTALL_DIR${NC}"

# Copy files
echo -e "${CYAN}Copying files...${NC}"
cp "$SCRIPT_DIR/voxora-service" "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/voxora-service"
echo -e "${GREEN}✓ Copied voxora-service${NC}"

# Copy bin folder
if [ -d "$SCRIPT_DIR/bin" ]; then
    cp -r "$SCRIPT_DIR/bin" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/bin"/*
    echo -e "${GREEN}✓ Copied bin/${NC}"
fi

# Copy data folder
if [ -d "$SCRIPT_DIR/data" ]; then
    cp -r "$SCRIPT_DIR/data" "$INSTALL_DIR/"
    echo -e "${GREEN}✓ Copied data/${NC}"
fi

# Copy static folder
if [ -d "$SCRIPT_DIR/static" ]; then
    cp -r "$SCRIPT_DIR/static" "$INSTALL_DIR/"
    echo -e "${GREEN}✓ Copied static/${NC}"
fi

# Create logs directory
mkdir -p "$INSTALL_DIR/logs"
echo -e "${GREEN}✓ Created logs/${NC}"

# Set ownership
chown -R "$CURRENT_USER:$CURRENT_USER" "$INSTALL_DIR"
echo -e "${GREEN}✓ Set ownership to $CURRENT_USER${NC}"

# Check credential files
echo -e "${CYAN}Checking credential files...${NC}"
CRED_PATH="/home/$CURRENT_USER/mytools/data/goserver"
if [ -d "$CRED_PATH" ]; then
    echo -e "${GREEN}✓ Found credentials at $CRED_PATH${NC}"
else
    echo -e "${YELLOW}WARNING: Credentials not found at $CRED_PATH${NC}"
    echo -e "${YELLOW}Go-server needs:${NC}"
    echo -e "${YELLOW}  - $CRED_PATH/credential.txt${NC}"
    echo -e "${YELLOW}  - $CRED_PATH/settings_en.json${NC}"
    echo -e "${YELLOW}  - $CRED_PATH/settings_hi.json${NC}"
fi

# Create systemd service file
echo -e "${CYAN}Creating systemd service...${NC}"
cat > /etc/systemd/system/${SERVICE_NAME}.service << EOF
[Unit]
Description=Voxora Voice Transcription Service
After=network.target

[Service]
Type=simple
User=$CURRENT_USER
Group=$CURRENT_USER
WorkingDirectory=$INSTALL_DIR
ExecStart=$INSTALL_DIR/voxora-service
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

# Environment
Environment="RUST_LOG=info"
Environment="ENGLISH_GO_SERVER_URL=ws://127.0.0.1:8085/ws"
Environment="HINDI_GO_SERVER_URL=ws://127.0.0.1:8086/ws"

[Install]
WantedBy=multi-user.target
EOF

echo -e "${GREEN}✓ Created systemd service file${NC}"

# Reload systemd
echo -e "${CYAN}Reloading systemd...${NC}"
systemctl daemon-reload
echo -e "${GREEN}✓ Systemd reloaded${NC}"

# Enable service
echo -e "${CYAN}Enabling service...${NC}"
systemctl enable ${SERVICE_NAME}
echo -e "${GREEN}✓ Service enabled (will start on boot)${NC}"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${CYAN}Useful commands:${NC}"
echo -e "  ${YELLOW}sudo systemctl start $SERVICE_NAME${NC}    - Start service"
echo -e "  ${YELLOW}sudo systemctl stop $SERVICE_NAME${NC}     - Stop service"
echo -e "  ${YELLOW}sudo systemctl status $SERVICE_NAME${NC}   - Check status"
echo -e "  ${YELLOW}sudo systemctl restart $SERVICE_NAME${NC}  - Restart service"
echo -e "  ${YELLOW}sudo journalctl -u $SERVICE_NAME -f${NC}   - View logs"
echo ""
echo -e "${CYAN}Access UI at:${NC} ${GREEN}http://localhost:8080${NC}"
echo ""

read -p "Start service now? (Y/n): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    systemctl start ${SERVICE_NAME}
    sleep 2
    systemctl status ${SERVICE_NAME} --no-pager
    echo ""
    echo -e "${GREEN}✓ Service started!${NC}"
    echo -e "${CYAN}Open: ${GREEN}http://localhost:8080${NC}"
fi
