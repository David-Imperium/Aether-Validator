#!/bin/bash
# Synward MCP Server Installer for Linux/macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/Synward/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

info() { echo -e "${CYAN}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }

# Default values
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
CONTRACTS_DIR="${CONTRACTS_DIR:-}"
CONFIGURE_FACTORY="${CONFIGURE_FACTORY:-false}"
CONFIGURE_GEMINI="${CONFIGURE_GEMINI:-false}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version) VERSION="$2"; shift 2 ;;
        --install-dir) INSTALL_DIR="$2"; shift 2 ;;
        --contracts-dir) CONTRACTS_DIR="$2"; shift 2 ;;
        --factory-cli) CONFIGURE_FACTORY=true; shift ;;
        --gemini-cli) CONFIGURE_GEMINI=true; shift ;;
        --help|-h)
            echo "Synward MCP Server Installer"
            echo ""
            echo "Usage: $0 [options]"
            echo ""
            echo "Options:"
            echo "    --version       Version to install (default: latest)"
            echo "    --install-dir   Installation directory (default: \$HOME/.local/bin)"
            echo "    --contracts-dir Custom contracts directory"
            echo "    --factory-cli   Configure for Factory CLI"
            echo "    --gemini-cli    Configure for Gemini CLI"
            echo "    --help          Show this help"
            echo ""
            echo "Environment variables:"
            echo "    VERSION         Version to install"
            echo "    INSTALL_DIR     Installation directory"
            echo "    CONTRACTS_DIR   Contracts directory"
            echo ""
            echo "Examples:"
            echo "    $0 --factory-cli"
            echo "    VERSION=v0.1.0 $0"
            echo "    $0 --install-dir /usr/local/bin"
            exit 0
            ;;
        *) error "Unknown option: $1" ;;
    esac
done

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux*)  TARGET="linux" ;;
    Darwin*) TARGET="macos" ;;
    *)       error "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
    x86_64|amd64)  TARGET="$TARGET-x86_64" ;;
    arm64|aarch64) TARGET="$TARGET-aarch64" ;;
    *)             error "Unsupported architecture: $ARCH" ;;
esac

info "Detected: $TARGET"

# Get latest version if not specified
if [[ "$VERSION" == "latest" ]]; then
    info "Fetching latest version..."
    VERSION=$(curl -fsSL https://api.github.com/repos/YOUR_REPO/Synward/releases/latest 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/' || echo "v0.1.0")
    warn "Using version: $VERSION"
fi

info "Installing Synward MCP $VERSION"

# Create installation directory
mkdir -p "$INSTALL_DIR"

# Download binary
BINARY_NAME="synward-mcp-server-$TARGET"
DOWNLOAD_URL="https://github.com/YOUR_REPO/Synward/releases/download/$VERSION/$BINARY_NAME"
BINARY_PATH="$INSTALL_DIR/synward-mcp-server"

info "Downloading from: $DOWNLOAD_URL"

if command -v curl &>/dev/null; then
    curl -fsSL "$DOWNLOAD_URL" -o "$BINARY_PATH" || error "Download failed"
elif command -v wget &>/dev/null; then
    wget -q "$DOWNLOAD_URL" -O "$BINARY_PATH" || error "Download failed"
else
    error "Neither curl nor wget available"
fi

chmod +x "$BINARY_PATH"
success "Downloaded binary to: $BINARY_PATH"

# Setup contracts
CONTRACTS_PATH="${CONTRACTS_DIR:-$HOME/.local/share/synward/contracts}"
mkdir -p "$CONTRACTS_PATH"/{rust,cpp,lex}
info "Created contracts directory: $CONTRACTS_PATH"

# Configure Factory CLI
if [[ "$CONFIGURE_FACTORY" == "true" ]]; then
    info "Configuring Factory CLI..."
    
    MCP_CONFIG="$HOME/.factory/mcp.json"
    mkdir -p "$(dirname "$MCP_CONFIG")"
    
    # Create or update config
    if [[ -f "$MCP_CONFIG" ]]; then
        # Use jq if available, otherwise use python
        if command -v jq &>/dev/null; then
            jq --arg bin "$BINARY_PATH" --arg contracts "$CONTRACTS_PATH" \
                '.mcpServers.synward = {type: "stdio", command: $bin, args: ["--contracts", $contracts], disabled: false}' \
                "$MCP_CONFIG" > "${MCP_CONFIG}.tmp" && mv "${MCP_CONFIG}.tmp" "$MCP_CONFIG"
        else
            warn "jq not installed, manual configuration may be needed"
        fi
    else
        cat > "$MCP_CONFIG" <<EOF
{
  "mcpServers": {
    "synward": {
      "type": "stdio",
      "command": "$BINARY_PATH",
      "args": ["--contracts", "$CONTRACTS_PATH"],
      "disabled": false
    }
  }
}
EOF
    fi
    success "Configured Factory CLI: $MCP_CONFIG"
fi

# Configure Gemini CLI
if [[ "$CONFIGURE_GEMINI" == "true" ]]; then
    info "Configuring Gemini CLI..."
    
    GEMINI_CONFIG="$HOME/.gemini/settings.json"
    mkdir -p "$(dirname "$GEMINI_CONFIG")"
    
    if [[ -f "$GEMINI_CONFIG" ]]; then
        if command -v jq &>/dev/null; then
            jq --arg bin "$BINARY_PATH" --arg contracts "$CONTRACTS_PATH" \
                '.mcpServers.synward = {command: $bin, args: ["--contracts", $contracts]}' \
                "$GEMINI_CONFIG" > "${GEMINI_CONFIG}.tmp" && mv "${GEMINI_CONFIG}.tmp" "$GEMINI_CONFIG"
        else
            warn "jq not installed, manual configuration may be needed"
        fi
    else
        cat > "$GEMINI_CONFIG" <<EOF
{
  "mcpServers": {
    "synward": {
      "command": "$BINARY_PATH",
      "args": ["--contracts", "$CONTRACTS_PATH"]
    }
  }
}
EOF
    fi
    success "Configured Gemini CLI: $GEMINI_CONFIG"
fi

# Verify installation
info "Verifying installation..."
if "$BINARY_PATH" --version &>/dev/null; then
    success "Synward MCP Server installed successfully!"
    echo ""
    echo "Binary:     $BINARY_PATH"
    echo "Contracts:  $CONTRACTS_PATH"
    echo ""
    
    if [[ "$CONFIGURE_FACTORY" != "true" && "$CONFIGURE_GEMINI" != "true" ]]; then
        echo "To configure your AI client, add to your MCP config:"
        echo ""
        cat <<EOF
{
  "mcpServers": {
    "synward": {
      "type": "stdio",
      "command": "$BINARY_PATH",
      "args": ["--contracts", "$CONTRACTS_PATH"],
      "disabled": false
    }
  }
}
EOF
    fi
else
    error "Installation verification failed"
fi
