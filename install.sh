#!/bin/bash
set -e

echo "══════════════════════════════════════════════════"
echo "  📦 wzllama - Installation complète"
echo "══════════════════════════════════════════════════"
echo ""

# ─── 1. Compilation ─────────────────────────────────
echo "🔨 Étape 1/4 : Compilation..."
if command -v cargo &>/dev/null; then
    cargo build --release 2>/dev/null
    echo "   ✅ Compilation terminée"
else
    echo "   ❌ Rust/Cargo non trouvé. Installez Rust : https://rustup.rs"
    exit 1
fi

# ─── 2. Installation du binaire ──────────────────────
echo ""
echo "📂 Étape 2/4 : Installation du binaire..."
BIN_SRC="target/release/wzllama"
BIN_DEST="/usr/local/bin/wzllama"

if [ -f "$BIN_SRC" ]; then
    sudo cp "$BIN_SRC" "$BIN_DEST"
    sudo chmod +x "$BIN_DEST"
    echo "   ✅ wzllama installé dans /usr/local/bin/"
else
    echo "   ❌ Binaire non trouvé à $BIN_SRC"
    exit 1
fi

# ─── 3. Configuration utilisateur ────────────────────
echo ""
echo "⚙️  Étape 3/4 : Configuration utilisateur..."
mkdir -p ~/.wzllama/{config,i18n,logs}

# Configurations
[ -f config/default_usages.yaml ] && cp config/default_usages.yaml ~/.wzllama/config/usages.yaml
[ -f config/default_config.yaml ] && cp config/default_config.yaml ~/.wzllama/config/config.yaml

# Fichiers i18n
if [ -d config/i18n ]; then
    cp config/i18n/*.json ~/.wzllama/i18n/ 2>/dev/null || true
fi

echo "   ✅ Configuration dans ~/.wzllama/"
echo "   🌍 Langues : $(ls ~/.wzllama/i18n/*.json 2>/dev/null | wc -l) fichiers"

# ─── 4. Vérification ────────────────────────────────
echo ""
echo "🔍 Étape 4/4 : Vérification..."

if command -v wzllama &>/dev/null; then
    VERSION=$(wzllama --version 2>/dev/null || echo "inconnue")
    echo "   ✅ wzllama est installé (version $VERSION)"
    echo ""
    echo "══════════════════════════════════════════════════"
    echo "  🚀 Installation terminée !"
    echo ""
    echo "  Commandes disponibles :"
    echo "    wzllama              → Lancer le wizard"
    echo "    wzllama validate     → Valider les templates"
    echo "    wzllama --dry-run    → Mode simulation"
    echo "    wzllama --help       → Aide complète"
    echo "    wzllama uninstall    → Désinstallation"
    echo "══════════════════════════════════════════════════"
else
    echo "   ⚠️  wzllama introuvable dans le PATH."
    echo "   Ajoutez /usr/local/bin à votre PATH ou relancez votre shell."
fi