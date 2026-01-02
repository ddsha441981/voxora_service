#!/bin/bash
# Voxora Service Uninstallation Script for Ubuntu
# Run with: sudo ./uninstall-ubuntu.sh

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

SERVICE_NAME="voxora"
INSTALL_DIR="/opt/voxora"

echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}  Voxora Service Uninstaller${NC}"
echo -e "${CYAN}======================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}ERROR: Please run as root or with sudo${NC}"
    echo "Usage: sudo ./uninstall-ubuntu.sh"
    exit 1
fi

# Stop service if running
echo -e "${CYAN}Stopping service...${NC}"
if systemctl is-active --quiet ${SERVICE_NAME}; then
    systemctl stop ${SERVICE_NAME}
    echo -e "${GREEN}✓ Service stopped${NC}"
else
    echo -e "${YELLOW}Service not running${NC}"
fi

# Disable service
echo -e "${CYAN}Disabling service...${NC}"
if systemctl is-enabled --quiet ${SERVICE_NAME}; then
    systemctl disable ${SERVICE_NAME}
    echo -e "${GREEN}✓ Service disabled${NC}"
else
    echo -e "${YELLOW}Service not enabled${NC}"
fi

# Remove systemd service file
if [ -f "/etc/systemd/system/${SERVICE_NAME}.service" ]; then
    rm "/etc/systemd/system/${SERVICE_NAME}.service"
    echo -e "${GREEN}✓ Removed systemd service file${NC}"
fi

# Reload systemd
systemctl daemon-reload
echo -e "${GREEN}✓ Systemd reloaded${NC}"

# Remove installation directory
if [ -d "$INSTALL_DIR" ]; then
    echo ""
    echo -e "${YELLOW}Remove installation directory?${NC}"
    echo -e "${YELLOW}Path: $INSTALL_DIR${NC}"
    read -p "Remove? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$INSTALL_DIR"
        echo -e "${GREEN}✓ Removed $INSTALL_DIR${NC}"
    else
        echo -e "${YELLOW}Kept $INSTALL_DIR${NC}"
    fi
fi

echo ""
echo -e "${GREEN}======================================${NC}"
echo -e "${GREEN}  Uninstallation Complete!${NC}"
echo -e "${GREEN}======================================${NC}"
echo ""
echo -e "${CYAN}Note: Credential files at ~/mytools/data/goserver/ were not removed${NC}"
echo ""
