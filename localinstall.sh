#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
CYAN='\033[0;36m'
MUTED='\033[38;2;100;116;139m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}${CYAN}  onpkg Local Installer${NC}"
echo -e "${MUTED}  ────────────────────────${NC}"

# Check for cargo
if ! command -v cargo &>/dev/null; then
    echo -e "${MUTED}Error: Cargo/Rust is not installed. Please install Rust first.${NC}"
    exit 1
fi

echo -e "${MUTED}Building and installing globally...${NC}"
cargo install --path .

echo ""
echo -e "${GREEN}✓ onpkg compiled and installed globally successfully!${NC}"
echo -e "${MUTED}You can run 'onpkg doctor' or 'onpkg stack list' to start.${NC}"
echo ""
