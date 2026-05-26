# Tests et Qualité

## 1. Types de Tests

### Tests Unitaires
| Fichier | Tests | Description |
|-------|-------|-------------|
| `state_tests.rs` | 6 | Tests sérialisation state.json |
| `tool_trait_tests.rs` | 11 | Tests du trait Tool |
| `i18n_tests.rs` | - | Tests traductions |
| `templates_tests.rs` | - | Tests templates config |
| `wizard_tests.rs` | 3 | Tests enum UseCase |

### Tests d'Intégration
- Tests via CLI avec `--dry-run`
- Tests serveur API (non implémenté)

---

## 2. Tests Implémentés

### state_tests.rs - Tests d'État
```rust
// Test Default values
test_installed_tools_default() {
    // Vérifie que tous les flags installés sont false par défaut
}

test_wzllama_state_default() {
    // Vérifie que language, last_model, etc sont None
}

// Test mutation state
test_mark_installed_docker() {
    // mark_installed("docker", state) → state.installed.docker = true
}

test_mark_installed_ollama() {
    // mark_installed("ollama", state) → state.installed.ollama = true
}

test_mark_installed_unknown() {
    // mark_installed("unknown", state) → ne plante pas, pas de changement
}

// Test sérialisation
test_state_serialization() {
    // serde_json roundtrip
}
```

### tool_trait_tests.rs - Tests d'Outils
```rust
// Mock Tool pour tests
struct MockTool {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    installed: bool,
}

// Tests Tool trait
test_tool_status_installed()
test_tool_status_not_installed()
test_tool_id()
test_tool_name()
test_tool_description()
test_tool_status_message_installed()
test_tool_status_message_not_installed()
test_tool_requires_docker()  // open_webui uniquement
test_tool_supports_agentic()  // false par défaut
test_tool_install_already_installed()  // Erreur si déjà installé
test_tool_install_not_installed()    // OK si non installé
test_tool_update_default()           // Erreur par défaut
test_tool_uninstall_default()        // Erreur par défaut
```

### wizard_tests.rs - Tests UseCase
```rust
test_usecase_all() {
    // Vérifie 6 use cases: General, Coding, Reasoning, Chat, Multimodal, Embedding
}

test_usecase_as_str() {
    // Coding.as_str() == "coding"
}

test_usecase_equality() {
    // Vérifie eq/ne pour UseCase::Coding
}
```

---

## 3. Couverture de Code

### Modules testés
- ✅ `config::state` (complets)
- ✅ `tools::tool_trait` (complets)
- ✅ `wizard::menu_wizard::UseCase` (partiels)

### Modules non testés
- ❌ `core::ollama_api` (nécessite serveur Ollama)
- ❌ `menu_api::*` (nécessite I/O terminal)
- ❌ `tools::*` individuels
- ❌ `api_server` (nécessite réseau)

---

## 4. Données de Test

### Fixtures utilisées
```rust
// État par défaut
WzllamaState::default()

// État personnalisé
WzllamaState {
    language: Some("en".to_string()),
    installed: InstalledTools { ollama: true, ..Default::default() },
    last_model: Some("qwen2.5:7b".to_string()),
    ..Default::default()
}
```

### Mock implementations
- `MockTool` pour tests Tool trait
- `I18n::default()` pour tests sans fichiers i18n

---

## 5. Critères de Qualité

### Standards suivis
- Rustfmt pour formatage
- Clippy pour lint (warnings allowed)
- Anyhow pour gestion erreurs sans boilerplate

### Patterns testés
- Sérialisation/désérialisation JSON
- Mutations d'état
- Comportement des traits
- Enum matching