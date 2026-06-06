#!/usr/bin/env bash
set -euo pipefail

# onpkg installer
# Usage: curl -fsSL https://raw.githubusercontent.com/USER/onpkg/main/install.sh | bash

GREEN='\033[0;32m'
CYAN='\033[0;36m'
MUTED='\033[38;2;100;116;139m'
BOLD='\033[1m'
NC='\033[0m'

echo ""
echo -e "${BOLD}${CYAN}  ╔═╗╔╗ ╔═╗╦ ╦╔═╗${NC}"
echo -e "${BOLD}${CYAN}  ╠═╣╠╩╗╠═╣║ ║║ ║${NC}"
echo -e "${BOLD}${CYAN}  ╩ ╩╚═╝╩ ╩╚═╝╚═╝${NC}"
echo -e "${MUTED}  onpkg installer${NC}"
echo ""

# Check for Rust
if ! command -v cargo &>/dev/null; then
    echo -e "${MUTED}  Rust/Cargo not found. Installing via rustup...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

REPO_URL="${REPO_URL:-https://github.com/USER/onpkg.git}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.onpkg}"

echo -e "${MUTED}  Installing onpkg...${NC}"

# Clone or update
if [ -d "$INSTALL_DIR/repo" ]; then
    echo -e "${MUTED}  Updating existing installation...${NC}"
    cd "$INSTALL_DIR/repo" && git pull
else
    mkdir -p "$INSTALL_DIR"
    git clone "$REPO_URL" "$INSTALL_DIR/repo"
    cd "$INSTALL_DIR/repo"
fi

# Build
echo -e "${MUTED}  Building (this may take a minute)...${NC}"
cargo build --release 2>&1 | while IFS= read -r line; do
    if [[ "$line" == *"Compiling"* ]]; then
        echo -e "  ${MUTED}$line${NC}"
    fi
done

# Install the binary
cp target/release/onpkg "$HOME/.cargo/bin/onpkg"

echo ""
echo -e "${GREEN}  ✓ onpkg installed!${NC}"
echo ""
echo -e "  ${MUTED}Run 'onpkg doctor' to verify installation.${NC}"
echo -e "  ${MUTED}Run 'onpkg template list' to see available templates.${NC}"
echo ""
