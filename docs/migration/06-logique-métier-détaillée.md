# Logique Métier Détailée

## 1. Algorithme du Wizard - Flow Principal

### Étape 1: Détection et Initialisation
```
FONCTION main():
    state = WzllamaState::load()
    i18n = select_language(state)      // Détection ou chargement langue
    hardware = HardwareInfo::detect()    // RAM, GPU, disque
    wizard::run(i18n, state, hardware) // Menu principal
```

### Étape 2: Navigation Menu Principal
```
FONCTION run(i18n, state, hw) [menu_main.rs]:
    OllamaTool::ensure_running(i18n)   // Vérifie/installe Ollama
    LLMFitTool::ensure_running(i18n)   // Vérifie LLMFit
    setup_models::ensure_first_models(i18n, hw, state)
    
    MainMenuRunner::new(i18n, state, hw)
        .run()  // Boucle menu interactif
```

### Étape 3: Workflow Wizard UseCase
```
FONCTION models_wizard(i18n, state, hw) [menu_wizard.rs]:
    
    ÉCHAUFFEMENT:
        - Vérifier ollama_api::detect_url() disponible
        - Message d'erreur si non disponible
    
    BOUCLE MENU:
        1. Afficher header: RAM, VRAM, modèle par défaut
        2. Présenter UseCases: General, Coding, Reasoning, Chat, Multimodal, Embedding
        3. Si sélection == 0 → Retour menu principal
        4. Sinon:
           use_case = UseCase::all()[selection - 1]
           handle_usecase_selection(use_case)
```

### Étape 4: Sélection Modèle et Outil
```
FONCTION handle_usecase_selection(i18n, state, hw, use_case):
    
    DONNÉES:
        local_models = ollama_api::get_models()
        local_names = {m.name for m in local_models}
        api_models = get_models_from_llmfit(use_case)
        
        SI api_models vide:
            api_models = localmax_models.fetch_models_by_search(use_case.as_str(), 50)
                .map(m → OllamaModel)
        
        available = [m for m in api_ollama_models si m.name NOT IN local_names]
        
    MENU CHOIX:
        options = []
        options.append("↩️ Retour")              // Position 0
        options.extend([m.name for m in local_models])  // Modèles installés
        options.extend(["📥 {m.name} (download)" for m in available])
        options.append("Use current model")
        
    SÉLECTION:
        SI selection == 0:
            RETOURNER false
            
        SI selection <= len(local_models):
            selected_model = local_models[selection - 1].name
            state.last_model = selected_model
            state.save()
            launch_tool_for_usecase(use_case, selected_model)
            RETOURNER true
            
        SI selection <= len(local_models) + len(available) + 1:
            chosen = available[selection - len(local_models) - 1]
            ollama_api::pull_model(chosen.name)
            state.last_model = chosen.name
            state.save()
            launch_tool_for_usecase(use_case, chosen.name)
            RETOURNER true
            
        SINON:
            launch_tool_for_usecase(use_case, state.last_model)
            RETOURNER false
```

### Étape 5: Lancement Outil
```
FONCTION launch_tool_for_usecase(i18n, state, use_case, model):
    
    PRIORITÉ OUTILS par UseCase:
        Coding:
            - claude_code (si installé)
            - opencode (si installé)
            - droid (si installé)
            - codex (si installé)
            - ollama (toujours)
            
        Reasoning:
            - openclaw (si installé)
            - hermes_agent (si installé)
            - ollama (toujours)
            
        Chat:
            - goose (si installé)
            - pi (si installé)
            - pool (si installé)
            - ollama (toujours)
            
        Multimodal/General/Embedding:
            - openclaw/goose (si installé)
            - ollama (toujours)
    
    installed_tools = [t.id for t in get_priority_tools_for_usecase 
                      SI t.status() == Installed]
    
    SI len(installed_tools) == 1:
        tool = get_tool(installed_tools[0])
        state.last_tool = tool.id()
        tool.launch(model)
        RETOURNER
        
    MENU SÉLECTION:
        options = []
        options.append("↩️ Retour")
        options.extend([format("🔧 {} - {}", t.name(), t.description(i18n)) 
                       for t in installed_tools])
        
        SI selection != 0 ET selection <= len(installed_tools):
            tool_id = installed_tools[selection - 1]
            state.last_tool = tool_id
            state.save()
            tool = get_tool(tool_id)
            tool.launch(model)
```

---

## 2. Règles Métier des Outils

### Ollama - Moteur LLM Local
```
RÈGLE 1: Installation Obligatoire
- Si ollama non installé → demander installation au lancement
- Ne pas permettre l'utilisation sans ollama

RÈGLE 2: Configuration Service
- Créer utilisateur système ollama
- Créer répertoire /home/ollama pour modèles
- Générer override.conf systemd avec variables env

RÈGLE 3: Variables Environment Ollama
- OLLAMA_MODELS=/home/ollama
- OLLAMA_ORIGINS=*
- OLLAMA_KEEP_ALIVE=5m
- OLLAMA_NUM_PARALLEL=4
- OLLAMA_MAX_LOADED_MODELS=4
- OLLAMA_FLASH_ATTENTION=1 (si true)
- OLLAMA_KV_CACHE_TYPE=q8_0
- OLLAMA_CONTEXT_LENGTH=4096
- OLLAMA_MAX_VRAM={vram} (si > 0)
- CUDA_VISIBLE_DEVICES={devices} (si défini)
```

### Tools avec Docker
```
RÈGLE: open_webui REQUIRES_DOCKER
- Vérifier installation docker avant
- Si refus → instructions d'installation
- Commande: docker run --network=host ...
```

---

## 3. Priorisation Tools par Use Case

### Matrice Use Case → Tools prioritaires
| Use Case | Priorité 1 | Priorité 2 | Priorité 3 | Priorité 4 | Fallback |
|----------|------------|------------|------------|------------|----------|
| Coding | claude_code | opencode | droid | codex | ollama |
| Reasoning | openclaw | hermes_agent | - | - | ollama |
| Chat | goose | pi | pool | - | ollama |
| Multimodal | openclaw | goose | - | - | ollama |
| General | openclaw | goose | - | - | ollama |
| Embedding | openclaw | goose | - | - | ollama |

---

## 4. Logique de Détection Matériau

### Algorithme RAM
```
fn detect_ram():
    sys = sysinfo::System::new_all()
    sys.refresh_memory()
    RETURN sys.total_memory() / (1024^3) GB
```

### Algorithme GPU
```
fn detect_gpus():
    essayer "nvidia-smi --query-gpu=name,memory.total --format=csv,noheader,nounits"
    SI succès:
        PARSE lignes CSV
        EXTRAIRE name, memory.total (MB)
    SINON:
        RETURN []
```

### Algorithme Disque
```
fn get_available_disk_space_gb(path):
    SI Unix:
        statvfs(path)
        RETURN f_bavail * f_frsize / (1024^3)
    SINON:
        RETURN 100.0 GB (défaut)
```

---

## 5. Machine à États - Navigation Menu

### États NavigationState
```
État Initial:
    history = [0]
    current_index = 0

Transition "Entrer dans sous-menu":
    history.push(adjusted_index)
    current_index = 0

Transition "Retour":
    SI history.len() > 1:
        history.pop()
        current_index = history.last()
        rebuild_current_menu()

Transition "Quitter":
    history = [0]
    current_menu = root.clone()
```

---

## 6. Logique d'Installation Ciblée

### Pattern Installation Tool
```
FONCTION install_tool(tool_id, i18n):
    tool = get_tool(tool_id)
    
    SELON tool_id:
        "ollama":
            curl -fsSL https://ollama.com/install.sh | sh
            setup_ollama_user_dir()
            générer systemd override
            
        "docker":
            Vérifier curl, wget présents
            curl -fsSL https://get.docker.com | sh
            
        "open_webui":
            docker run avec volumes et env
            
        TOOLS_NPM (claude_code, opencode, etc):
            npm install -g {tool_package}
            
        TOOLS_PIP (goose, pool, etc):
            pip install {tool_package}
            
        TOOLS_CARGO (hermes, obsidian):
            cargo install --locked {tool_crate}
```

---

## 7. Logique Benchmark

```
FONCTION run_benchmark():
    models = ollama_api::get_models()
    
    POUR chaque modèle:
        temps_début = now()
        shell.run("ollama run {model} 'Why is the sky blue?'")
        temps_fin = now()
        
        durée = temps_fin - temps_début
        tokens = compter_tokens(response)
        perf = tokens / durée
        
        AFFICHER "{model}: {perf} tokens/s"
    
    TRIER par performance décroissante
```

---

## 8. Workflow Langue (i18n)

```
FONCTION select_language(state):
    SI state.language IS SOME(lang):
        RETOURNER i18n.load(lang)
    
    languages = get_available_languages()
    system_lang = detect_system_language()
    selected = position de system_lang dans languages OU 0
    
    i18n = load(languages[selected].code)
    set_language(languages[selected].code, state)
    RETOURNER i18n
```

---

## 9. Logique LLMFit API

```
FONCTION get_models_from_llmfit(use_case):
    client = LLMFitClient::new()
    
    SI PAS client.is_running():
        RETOURNER []
        
    RETOURNER client.get_top_models(
        limit = Some(20),
        min_memory = None,
        use_case = Some(use_case.as_str())
    )
```

Variables hardware pour LLMFit:
- RAM (GB)
- VRAM (GB)
- CPU cores