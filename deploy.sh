#!/bin/bash
set -e

# Option --clean pour un build propre
CLEAN=false
if [ "$1" = "--clean" ]; then
    CLEAN=true
fi

echo "══════════════════════════════════════════════════"
echo "  📦 wzllama - Installation complète"
echo "══════════════════════════════════════════════════"
echo ""

# ─── 1. Compilation ─────────────────────────────────
echo "🔨 Étape 1/4 : Compilation..."
if command -v cargo &>/dev/null; then
    if [ "$CLEAN" = true ]; then
        echo "   🧹 Nettoyage complet..."
        cargo clean 2>/dev/null
    fi
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

if [ ! -f "$BIN_SRC" ]; then
    echo "   ❌ Binaire non trouvé à $BIN_SRC"
    exit 1
fi

# Try ~/.local/bin first, then /usr/local/bin (requires sudo)
if mkdir -p ~/.local/bin 2>/dev/null; then
    BIN_DEST="$HOME/.local/bin/wzllama"
    mkdir -p ~/.local/bin
    cp "$BIN_SRC" "$BIN_DEST"
    chmod +x "$BIN_DEST"
    echo "   ✅ wzllama installé dans ~/.local/bin/"
else
    BIN_DEST="/usr/local/bin/wzllama"
    sudo cp "$BIN_SRC" "$BIN_DEST"
    sudo chmod +x "$BIN_DEST"
    echo "   ✅ wzllama installé dans /usr/local/bin/"
fi

# ─── 3. Configuration utilisateur ────────────────────
echo ""
echo "⚙️  Étape 3/4 : Configuration utilisateur..."
mkdir -p ~/.wzllama/{config,i18n,logs}

# Configurations
if [ -f config/default_usages.yaml ]; then
    cp config/default_usages.yaml ~/.wzllama/config/usages.yaml
    echo "   ✅ usages.yaml installé"
fi
if [ -f config/default_config.yaml ]; then
    cp config/default_config.yaml ~/.wzllama/config/config.yaml
    echo "   ✅ config.yaml installé"
fi

# Fichiers i18n
if [ -d config/i18n ]; then
    cp config/i18n/*.json ~/.wzllama/i18n/ 2>/dev/null || true
    LANG_COUNT=$(ls ~/.wzllama/i18n/*.json 2>/dev/null | wc -l)
    echo "   ✅ Fichiers i18n installés ($LANG_COUNT langues)"
fi

# Vérification des fichiers requis
echo ""
echo "🔍 Vérification des fichiers..."
if [ -f ~/.wzllama/config/config.yaml ]; then
    echo "   ✅ ~/.wzllama/config/config.yaml"
else
    echo "   ⚠️  ~/.wzllama/config/config.yaml manquant"
fi
if [ -f ~/.wzllama/config/usages.yaml ]; then
    echo "   ✅ ~/.wzllama/config/usages.yaml"
else
    echo "   ⚠️  ~/.wzllama/config/usages.yaml manquant"
fi
if ls ~/.wzllama/i18n/*.json 1>/dev/null 2>&1; then
    echo "   ✅ Fichiers i18n présents"
else
    echo "   ⚠️  Aucun fichier i18n trouvé"
fi

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
    echo "    wzllama --tui        → Interface TUI (beta)"
    echo "    wzllama --dry-run    → Mode simulation"
    echo "    wzllama --help       → Aide complète"
    echo "    wzllama uninstall    → Désinstallation"
    echo "══════════════════════════════════════════════════"
    
    if [ "$BIN_DEST" = "$HOME/.local/bin/wzllama" ]; then
        echo ""
        echo "💡 Conseil : Si wzllama n'est pas trouvé, ajoutez à votre ~/.bashrc :"
        echo "   export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
else
    echo "   ⚠️  wzllama introuvable dans le PATH."
    echo "   Essayez: hash -r ou relancez votre terminal."
fi