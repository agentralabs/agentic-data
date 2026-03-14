#!/usr/bin/env bash
# Guardrail: verify install commands in README match installer output.
set -euo pipefail

ERRORS=0

echo "Checking install command consistency..."

# Check install script exists
if [ ! -f "scripts/install.sh" ]; then
    echo "  ✗ scripts/install.sh missing"
    ERRORS=$((ERRORS + 1))
else
    echo "  ✓ scripts/install.sh exists"
fi

# Check MCP binary name consistency
if grep -q "agentic-data-mcp" scripts/install.sh 2>/dev/null; then
    echo "  ✓ MCP binary name consistent"
else
    echo "  ✗ MCP binary name missing from installer"
    ERRORS=$((ERRORS + 1))
fi

# Check CLI binary name
if grep -q "adat" scripts/install.sh 2>/dev/null; then
    echo "  ✓ CLI binary name consistent"
else
    echo "  ✗ CLI binary name missing from installer"
    ERRORS=$((ERRORS + 1))
fi

if [ $ERRORS -eq 0 ]; then
    echo "All install command checks passed."
else
    echo "${ERRORS} install command checks failed."
    exit 1
fi
