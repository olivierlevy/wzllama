#!/bin/bash

echo "📦 Installation de la configuration wzllama..."
echo "🌍 Support multilingue dynamique activé"

# Créer les répertoires
mkdir -p ~/.wzllama/config
mkdir -p ~/.wzllama/i18n
mkdir -p ~/.wzllama/logs

# Copier les fichiers de configuration
if [ -f config/default_usages.yaml ]; then
    cp config/default_usages.yaml ~/.wzllama/config/usages.yaml
    echo "  ✓ usages.yaml"
fi

if [ -f config/default_config.yaml ]; then
    cp config/default_config.yaml ~/.wzllama/config/config.yaml
    echo "  ✓ config.yaml"
fi

# Copier TOUS les fichiers i18n disponibles
if [ -d config/i18n ]; then
    for file in config/i18n/*.json; do
        if [ -f "$file" ]; then
            cp "$file" ~/.wzllama/i18n/
            echo "  ✓ $(basename $file)"
        fi
    done
fi

echo ""
echo "✅ Configuration installée dans ~/.wzllama/"
echo "🌍 Langues disponibles : $(ls ~/.wzllama/i18n/*.json 2>/dev/null | wc -l)"
echo "🚀 Lancez : cargo run"