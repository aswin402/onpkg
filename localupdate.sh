#!/usr/bin/env bash
set -euo pipefail

GREEN='\033[0;32m'
CYAN='\033[0;36m'
MUTED='\033[38;2;100;116;139m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${BOLD}${CYAN}  onpkg Local Updater${NC}"
echo -e "${MUTED}  ────────────────────────${NC}"

echo -e "${MUTED}Rebuilding and updating binary...${NC}"
cargo install --path .

echo ""
echo -e "${GREEN}✓ onpkg updated globally successfully!${NC}"
echo ""
