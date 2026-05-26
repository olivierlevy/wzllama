# Concurrence et Performance

## 1. Architecture Asynchrone

### Runtime Tokio
- **Runtime**: Tokio (v1.0) avec feature "full"
- **Mode**: multi-thread par défaut
- **Usage**: Serveur API uniquement

### Threads vs Async
```
CLI Principal: Synchrone (main thread)
├── Dialoguer: Blocking I/O terminal
├── Shell commands: spawn_blocking
└── Ollama API calls: reqwest blocking

API Server: Asynchrone (tokio::spawn)
├── HTTP handlers: async
├── Graceful shutdown: signal channel
└── State shared: Arc<AtomicBool>
```

---

## 2. Gestion des Threads

### Thread blocking pour Shell
```rust
// Exécution shell bloquante
shell::run_live("ollama pull {model}")  // Bloque jusqu'à complétion

// Pour futures non bloquantes
tokio::task::spawn_blocking(|| {
    // Opération lourde
})
```

### Thread pour API Serveur
```rust
// Démarrage serveur dans runtime tokio
tokio::runtime::Runtime::new()?.block_on(start_server(addr));
```

---

## 3. Synchronisation

### AtomicBool pour Shutdown
```rust
static API_SHUTDOWN: OnceLock<Arc<AtomicBool>> = OnceLock::new();

// Thread principal peut signaler
request_shutdown() {
    flag.store(true, Ordering::SeqCst);
}

// Thread serveur vérifie périodiquement
while !flag.load(Ordering::SeqCst) {
    sleep(100ms).await;
}
```

### Partage State
- State chargé une seule fois au démarrage
- Pas de lock explicite (state immuable pendant exécution)

---

## 4. Optimisations Implémentées

### 1. Cache Modèles Locaux
```rust
// Pas de requête API à chaque affichage
get_models() {
    // Appel API /api/tags une seule fois
    // Résultat en mémoire jusqu'à modification
}
```

### 2. Fallback Intelligent
```
LLMFit API → Si timeout/échec → LocalMax scraping
```

### 3. Génération Lazy de Config
```rust
EnvConfig::load() {
    // Créé fichier config seulement si inexistant
    // Évite I/O inutile
}
```

---

## 5. Goulots d'Étranglement

### Shell Commands
- **Problème**: `ollama pull` peut prendre plusieurs minutes
- **Solution**: Messages de progression, pas d'UI pendant l'opération

### API LLMFit
- **Problème**: Service externe peut être lent
- **Solution**: Timeout court (5-10s), fallback immédiat

### Scraping Ollama
- **Problème**: requête HTML bloquante
- **Solution**: Timeout 15s, liste hardcoded de secours

---

## 6. Stratégies de Caching

### Cache Menu
- Menu reconstruit dynamiquement à chaque navigation
- Pas de cache persistant (structure simple)

### Cache Modèles
- `ollama_api::get_models()` appelle `/api/tags`
- Pas de cache en mémoire (fresh data à chaque appel)

### Cache Hardware
- `hardware::detect()` appelé une fois au démarrage
- Résultat partagé dans menu_handler

---

## 7. Performance Disk I/O

### File Writes
```
Sauvegarde state: write atomique (read-modify-write)
Fichier log: create/overwrite à chaque session
Config shells: append si existant
```

### Read Patterns
```
Chargement i18n: lazy load au besoin
Lecture state: une fois au démarrage
Liste dossiers: scandir pour découverte dynamique
```

---

## 8. Mémoire

### Allocation Stratégique
- `Vec<MenuItem>` pour sous-menus (stack allocated pour petits menus)
- `HashMap<String, String>` pour traductions (lookup O(1))
- `Option<String>` pour valeurs optionnelles (pas de chaîne vide)

### Pas de Memory Pool
- Pas de jemalloc personnalisé
- Rust default allocator (jemalloc sur Linux)