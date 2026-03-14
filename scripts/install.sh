#!/usr/bin/env bash
# AgenticData installer — release-first with source fallback.
# Profiles: desktop (default), terminal, server
set -euo pipefail

VERSION="${ADAT_VERSION:-latest}"
PROFILE="${1:-desktop}"
INSTALL_DIR="${HOME}/.local/bin"
REPO="agentralabs/agentic-data"

echo "╔══════════════════════════════════════════════╗"
echo "║  AgenticData Installer                       ║"
echo "║  Profile: ${PROFILE}                         ║"
echo "╚══════════════════════════════════════════════╝"

mkdir -p "${INSTALL_DIR}"

# Detect OS and architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: ${ARCH}"; exit 1 ;;
esac

# Try release artifact first
ARTIFACT="agentic-data-${VERSION}-${OS}-${ARCH}.tar.gz"
RELEASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${ARTIFACT}"

echo "→ Checking for release artifact..."
if curl -fsSL -o "/tmp/${ARTIFACT}" "${RELEASE_URL}" 2>/dev/null; then
    echo "  ✓ Downloaded release artifact"
    tar -xzf "/tmp/${ARTIFACT}" -C "${INSTALL_DIR}"
    rm -f "/tmp/${ARTIFACT}"
else
    echo "  → No release artifact found, building from source..."
    if ! command -v cargo &>/dev/null; then
        echo "  ✗ Rust/Cargo not found. Install from https://rustup.rs"
        exit 1
    fi
    TEMP_DIR="$(mktemp -d)"
    git clone --depth 1 "https://github.com/${REPO}.git" "${TEMP_DIR}/agentic-data"
    cd "${TEMP_DIR}/agentic-data"
    cargo build --release -j 1
    cp target/release/adat "${INSTALL_DIR}/" 2>/dev/null || true
    cp target/release/agentic-data-mcp "${INSTALL_DIR}/" 2>/dev/null || true
    rm -rf "${TEMP_DIR}"
    echo "  ✓ Built from source"
fi

# MCP config (merge-only, never destructive overwrite)
MCP_CONFIG="${HOME}/.config/claude/claude_desktop_config.json"
if [ "${PROFILE}" = "desktop" ] || [ "${PROFILE}" = "terminal" ]; then
    echo "→ MCP configuration"
    echo "  Add to your MCP client config:"
    echo "    \"agentic-data\": {"
    echo "      \"command\": \"${INSTALL_DIR}/agentic-data-mcp\","
    echo "      \"args\": []"
    echo "    }"
fi

echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║  Installation complete!                      ║"
echo "║                                              ║"
echo "║  Test: adat formats                          ║"
echo "║  MCP:  agentic-data-mcp                      ║"
echo "║                                              ║"
echo "║  Works with any MCP client:                  ║"
echo "║  Claude Desktop, Codex, Cursor, VS Code,     ║"
echo "║  Windsurf, Cline, or any MCP-compatible tool ║"
echo "╚══════════════════════════════════════════════╝"
