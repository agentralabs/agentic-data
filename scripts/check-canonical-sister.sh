#!/usr/bin/env bash
# Guardrail: verify canonical sister kit compliance.
set -euo pipefail

ERRORS=0
WARNINGS=0

echo "Checking Canonical Sister Kit compliance..."

# Section 1: docs folder
REQUIRED_DOCS=(
    "docs/public/quickstart.md"
    "docs/public/architecture.md"
    "docs/public/cli-reference.md"
    "docs/public/configuration.md"
    "docs/public/ffi-reference.md"
    "docs/public/mcp-tools.md"
    "docs/public/mcp-resources.md"
    "docs/public/mcp-prompts.md"
    "docs/public/troubleshooting.md"
)
for doc in "${REQUIRED_DOCS[@]}"; do
    if [ -f "$doc" ]; then
        echo "  ✓ $doc"
    else
        echo "  ✗ $doc MISSING"
        ERRORS=$((ERRORS + 1))
    fi
done

# Section 2: Canonical Sister Kit copy
if [ -f "docs/ecosystem/CANONICAL_SISTER_KIT.md" ]; then
    echo "  ✓ CANONICAL_SISTER_KIT.md present"
else
    echo "  ✗ CANONICAL_SISTER_KIT.md MISSING"
    ERRORS=$((ERRORS + 1))
fi

# Section 3: Scripts
for script in "scripts/install.sh" "scripts/check-install-commands.sh" "scripts/check-canonical-sister.sh"; do
    if [ -f "$script" ]; then
        echo "  ✓ $script"
    else
        echo "  ✗ $script MISSING"
        ERRORS=$((ERRORS + 1))
    fi
done

# Section 4: Cargo workspace
if [ -f "Cargo.toml" ]; then
    echo "  ✓ Workspace Cargo.toml"
else
    echo "  ✗ Workspace Cargo.toml MISSING"
    ERRORS=$((ERRORS + 1))
fi

# Section 5: Crate structure
for crate in "crates/agentic-data" "crates/agentic-data-mcp" "crates/agentic-data-cli" "crates/agentic-data-ffi"; do
    if [ -d "$crate" ]; then
        echo "  ✓ $crate/"
    else
        echo "  ✗ $crate/ MISSING"
        ERRORS=$((ERRORS + 1))
    fi
done

echo ""
if [ $ERRORS -eq 0 ]; then
    echo "✓ All canonical sister checks passed ($WARNINGS warnings)"
else
    echo "✗ $ERRORS canonical sister checks failed"
    exit 1
fi
