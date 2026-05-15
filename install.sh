#!/bin/bash
# wzllama installer - https://github.com/olivierlevy/wzllama
# Usage: curl -fsSL https://github.com/olivierlevy/wzllama/install.sh | sh

set -e

REPO="olivierlevy/wzllama"
BRANCH="main"

echo "══════════════════════════════════════════════════"
echo "  📦 wzllama - Installation"
echo "══════════════════════════════════════════════════"
echo ""

# ─── 0. Vérifications préalables ─────────────────────
echo "🔍 Vérification des prérequis..."

# Check for Rust/Cargo
if ! command -v cargo &>/dev/null; then
    echo "   ❌ Rust/Cargo non trouvé."
    echo "   Installez Rust : curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo "   ✅ Rust/Cargo trouvé"

# Check for git
if ! command -v git &>/dev/null; then
    echo "   ❌ Git non trouvé. Veuillez installer Git."
    exit 1
fi
echo "   ✅ Git trouvé"

# ─── 1. Téléchargement du code ─────────────────────
echo ""
echo "📥 Étape 1/4 : Téléchargement du code source..."

INSTALL_DIR="${TMPDIR:-/tmp}/wzllama-install-$$"
mkdir -p "$INSTALL_DIR"
cd "$INSTALL_DIR"

git clone --depth 1 --branch "$BRANCH" "https://github.com/$REPO.git" . 2>/dev/null || {
    echo "   ❌ Échec du clonage du dépôt"
    rm -rf "$INSTALL_DIR"
    exit 1
}
echo "   ✅ Code source téléchargé"

# ─── 2. Compilation ─────────────────────────────────
echo ""
echo "🔨 Étape 2/4 : Compilation..."
cargo build --release 2>&1 | tail -5
echo "   ✅ Compilation terminée"

# ─── 3. Installation du binaire ──────────────────────
echo ""
echo "📂 Étape 3/4 : Installation du binaire..."
BIN_SRC="target/release/wzllama"
BIN_DEST="/usr/local/bin/wzllama"

if [ -f "$BIN_SRC" ]; then
    sudo cp "$BIN_SRC" "$BIN_DEST"
    sudo chmod +x "$BIN_DEST"
    echo "   ✅ wzllama installé dans /usr/local/bin/"
else
    echo "   ❌ Binaire non trouvé après compilation"
    rm -rf "$INSTALL_DIR"
    exit 1
fi

# ─── 4. Configuration utilisateur ────────────────────
echo ""
echo "⚙️  Étape 4/4 : Configuration utilisateur..."
mkdir -p ~/.wzllama/{config,i18n,logs}

# Copy i18n files
if [ -d "config/i18n" ]; then
    cp config/i18n/*.json ~/.wzllama/i18n/ 2>/dev/null || true
    LANG_COUNT=$(ls ~/.wzllama/i18n/*.json 2>/dev/null | wc -l)
    echo "   🌍 Langues : $LANG_COUNT fichiers"
fi

# Cleanup
rm -rf "$INSTALL_DIR"

# ─── Vérification finale ─────────────────────────────
echo ""
if command -v wzllama &>/dev/null; then
    echo "══════════════════════════════════════════════════"
    echo "  🚀 Installation terminée !"
    echo ""
    echo "  Commandes disponibles :"
    echo "    wzllama              → Lancer le wizard"
    echo "    wzllama validate     → Valider les templates"
    echo "    wzllama --tui        → Interface TUI (beta)"
    echo "    wzllama --dry-run    → Mode simulation"
    echo "    wzllama --help       → Aide complète"
    echo "══════════════════════════════════════════════════"
else
    echo "   ⚠️  wzllama introuvable dans le PATH."
fi