# Guide Migration - Rust vers Nouveau Langage

## 1. Tableau de Correspondance Types

| Rust | Python | Go | Node.js |
|------|--------|-----|---------|
| `String` | `str` | `string` | `string` |
| `u32` | `int` | `int` (int32) | `number` |
| `u64` | `int` | `int64` | `number` |
| `f64` | `float` | `float64` | `number` |
| `bool` | `bool` | `bool` | `boolean` |
| `Option<T>` | `Optional[T]` | `*T` | `T \| null \| undefined` |
| `Vec<T>` | `List[T]` | `[]T` | `T[]` |
| `HashMap<K,V>` | `Dict[K,V]` | `map[K]V` | `Record<K, V>` |
| `Result<T,E>` | `Result[T, E]` | `error` return | `Promise.reject()` ou exceptions |
| `enum` | `Enum` | `iota` + const | `const enum` |
| `trait` | Protocol class | Interface | TypeScript interface |
| `&str` | `str` | `string` | `string` |

### Structures de Données
| Rust | Python | Go | Node.js |
|------|--------|-----|---------|
| `struct Foo { fields }` | `@dataclass class Foo` | `struct Foo struct{}` | `interface Foo {}` |
| `impl Foo` | `def method(self)` | `func (f *Foo)` | `Foo.prototype.method` |
| `Box<dyn Trait>` | - | - | - |

---

## 2. Correspondance Bibliothèques

| Fonctionnalité | Rust | Python | Go | Node.js |
|---------------|------|--------|-----|---------|
| CLI Args | `clap` | `click` | `cobra` | `commander` |
| TUI/Menu | `dialoguer` | `inquirer` | `promptui` | `inquirer.js` |
| HTTP Server | `axum` | `FastAPI` | `Gin` | `Express` |
| HTTP Client | `reqwest` | `requests` | `net/http` | `axios/fetch` |
| JSON | `serde_json` | `json` | `encoding/json` | `JSON` native |
| YAML | `serde_yaml` | `pyyaml` | `yaml` | `js-yaml` |
| TOML | `toml` | `toml` | `toml` | `toml` |
| Logging | `log + env_logger` | `logging` | `logrus` | `winston` |
| Colors | `colored` | `colorama` | `color` | `chalk` |
| Concurrency | `tokio` | `asyncio` | goroutines | `worker threads` |
| System Info | `sysinfo` | `psutil` | `gopsutil` | `os, process` |
| Scraping | `scraper` | `BeautifulSoup` | `goquery` | `cheerio` |

---

## 3. Syntaxe Critique

### Rust → Python
```python
# Enums
class UseCase(Enum):
    GENERAL = "general"
    CODING = "coding"

# Structs with methods  
@dataclass
class WzllamaState:
    language: Optional[str] = None
    installed: InstalledTools = field(default_factory=InstalledTools)
    
    def save(self):
        with open(state_file(), 'w') as f:
            json.dump(self.__dict__, f)

# Trait equivalent (protocol)
class Tool(Protocol):
    def id(self) -> str: ...
    def name(self) -> str: ...
    def install(self) -> Result[None]: ...
```

### Rust → Go
```go
// Enums (iota)
type UseCase int
const (
    General UseCase = iota
    Coding
    Reasoning
)

// Structs
type WzllamaState struct {
    Language    string       `json:"language"`
    Installed   InstalledTools `json:"installed"`
    LastModel   string       `json:"last_model"`
}

// Interface (trait equivalent)
type Tool interface {
    ID() string
    Name() string
    Install(i18n *I18n) error
    Launch(i18n *I18n, state *WzllamaState, model *string) error
}
```

### Rust → Node.js
```typescript
// Enums
enum UseCase {
    General = "general",
    Coding = "coding",
    Reasoning = "reasoning"
}

// Interfaces (traits)
interface Tool {
    id(): string;
    name(): string;
    description(i18n: I18n): string;
    install(i18n: I18n): Promise<void>;
    launch(i18n: I18n, state: WzllamaState, model?: string): Promise<void>;
    isInstalled(): boolean;
    requiresDocker(): boolean;
    supportsAgentic(): boolean;
}

// Types
interface WzllamaState {
    language?: string;
    installed: InstalledTools;
    lastModel?: string;
}
```

---

## 4. Structure Recommandée Nouveau Projet

```
wzllama/
├── src/
│   ├── cli.{ext}           # Point d'entrée + arguments
│   ├── config/
│   │   ├── state.{ext}     # État persistant
│   │   ├── i18n.{ext}      # Traductions
│   │   ├── env.{ext}       # Configuration env
│   │   └── paths.{ext}     # Chemins fichiers
│   ├── core/
│   │   ├── hardware.{ext}  # Détection système
│   │   ├── ollama_api.{ext} # Interface Ollama
│   │   └── shell.{ext}     # Commands système
│   ├── tools/
│   │   ├── tool_trait.{ext} # Interface outils
│   │   ├── ollama.{ext}    # Impl Ollama
│   │   ├── open_webui.{ext}
│   │   └── *.rs → outils
│   ├── wizard/
│   │   ├── menu_main.{ext}
│   │   ├── menu_wizard.{ext}
│   │   └── *.rs → menus
│   └── menu_api/
│       ├── menu_tree.{ext}
│       ├── menu_item.{ext}
│       ├── menu_handler.{ext}
│       └── tool_action.{ext}
├── config/
│   ├── i18n/
│   │   ├── fr.json
│   │   └── en.json
│   └── mcp/
├── tests/
└── README.md
```

---

## 5. Checklist Migration Étape par Étape

### Phase 1: Core (2-3 jours)
- [ ] Implémenter paths.rs → paths.py/go/js
- [ ] Implémenter state.rs avec sérialisation JSON
- [ ] Implémenter hardware.rs (RAM/VRAM detection)
- [ ] Implémenter shell.rs (command execution)

### Phase 2: Configuration (1-2 jours)
- [ ] Implémenter i18n.rs avec chargement fichiers JSON
- [ ] Implémenter env.rs avec génération script shell
- [ ] Test: state_tests équivalent

### Phase 3: Ollama API (2 jours)
- [ ] get_models() via HTTP GET /api/tags
- [ ] pull_model() via ollama CLI
- [ ] detect_url() pour vérifier service
- [ ] Tests avec serveur mock

### Phase 4: Tools Trait (2-3 jours)
- [ ] Définir interface Tool
- [ ] Implémenter OllamaTool
- [ ] Implémenter DockerTool
- [ ] Tests unitaires tools

### Phase 5: Menu API (2-3 jours)
- [ ] MenuItem avec label/action_id/submenus
- [ ] MenuTree avec find_by_path
- [ ] MenuHandler avec navigation
- [ ] ToolAction trait

### Phase 6: Wizard (2 jours)
- [ ] UseCase enum
- [ ] Menu wizard avec UseCases
- [ ] Integration LLMFit API
- [ ] Tests wizard

### Phase 7: Serveur API (2-3 jours)
- [ ] Routes GET /api/v1/menu
- [ ] Routes GET /api/v1/tools
- [ ] Routes GET /api/v1/models
- [ ] CORS + HTML UI

---

## 6. Pièges Courants à Éviter

### Error Handling
```python
# ❌ Pas de try/catch partout
# ✅ Utiliser Result/Option équivalent
from typing import Optional, Result

def get_models() -> Optional[list[OllamaModel]]:
    ...
```

### Async/Await
```python
# ❌ Mélanger sync/async
# ✅ Cohérence dans les appels HTTP
async def fetch_models(): ...
```

```go
// ❌ Goroutines non synchronisées
// ✅ Context avec timeout
ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
```

### State Management
```python
# ❌ State mutable partout
# ✅ Pattern charge une fois, modifie via fonctions pures
```

### Path Handling
```python
# ❌ Paths hardcodés
# ✅ Utiliser os.path.expanduser, pathlib.Path
Path.home() / ".wzllama" / "state.json"
```

### Format Serialization
```python
# ❌ Indentation inconséquente
# ✅ JSON indenté 2 espaces comme Rust serde_json::to_string_pretty
```

### Null Safety
```python
# ❌ Supposer que l'outil est installé
# ✅ Toujours vérifier ToolStatus avant opération
```

### CLI Integration
```python
# ❌ print() partout
# ✅ Utiliser logging module + affichage couleur
```

### Docker Integration
```python
# ❌ Blocking shell call
# ✅ Timeout + check statut + fallback messages
```

---

## 7. Points d'Attention Spécifiques

### LLMFit Service
- Dépendance externe git: `llmfit-core`
- Doit être lancé en parallèle (service TCP)
- API locale sur port différent

### Ollama Service
- Vérifier `/api/tags` disponible
- `ollama` command dans PATH
- systemd service management (Linux uniquement)

### Shell Commands
- `run_live()` vs `run_quiet()` vs `run()`
- Gestion `sudo` automatique
- Timeout commands longues

### Internationalisation
- Fichiers JSON dans `~/.wzllama/i18n/`
- Fallback vers `config/i18n/` embarqué
- Variables `{name}` remplacées par `.t_with_vars()`