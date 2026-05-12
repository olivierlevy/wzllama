# wzllama

Je veux que tu conçoives et implémentes un outil CLI appelé **`wzllama`**, en Rust, destiné à simplifier au maximum l’installation et l’usage d’une stack IA locale centrée sur **Ollama**, **Open WebUI**, **Hermes Agent** et **OpenClaw**. L’objectif est qu’un utilisateur lambda ait le moins de choses possible à faire : une commande, puis un wizard interactif qui fait (ou guide) tout le reste.

### Objectif général

Créer un **wizard CLI multi‑plateforme** (Linux, macOS, Windows) qui :

1. S’installe et stocke sa configuration dans `~/.wzllama/`.
2. Détecte la machine (OS, RAM, VRAM, GPU, réseau).
3. Vérifie la présence d’Ollama, Open WebUI, Hermes Agent, OpenClaw.
4. Propose d’installer ce qui manque (avec confirmation explicite).
5. Si une étape bloque, **n’échoue jamais silencieusement** :
   - affiche clairement l’erreur,
   - montre les commandes à lancer manuellement,
   - sert de mini‑documentation exécutable.
6. Une fois la stack en place, lance un **wizard d’usages** (questionnement par arborescence, dépendant de la langue) qui :
   - demande ce que l’utilisateur veut faire (gros livre, gros code, agents rapides, mixte, etc.),
   - explique les **limites de sa machine**,
   - choisit des modèles adaptés (taille, VRAM/RAM, contexte, tokens/s),
   - estime le temps nécessaire (fourchettes larges),
   - propose éventuellement des profils de fine‑tuning LoRA adaptés (surtout pour agents rapides).

### Contraintes techniques et structure

- Langage : **Rust**
- Arborescence interne :
  - `~/.wzllama/`
    - `config/`
      - `config.json`
      - `usages.yaml`
      - `bench.json`
      - `i18n/en.json`, `i18n/fr.json`, `i18n/de.json`, `i18n/es.json`
    - `logs/`
      - `wzllama.log`
- Modules :
  - `core` : détection système, exécution de commandes, gestion des erreurs, logging.
  - `installers` : fonctions d’installation Ollama, Open WebUI, Hermes, OpenClaw.
  - `wizard` : logique d’usage (gros livre, gros code, agents, mixte), calcul tokens/contextes/temps, interaction utilisateur.
  - `config` : chargement/validation des templates (i18n, usages), gestion de la langue et des poids.

### Gestion de la langue (i18n)

1. **Langues supportées** : EN, FR, DE, ES.
2. Pas de chaînes en dur dans le code : tout texte utilisateur vient de fichiers JSON de langue, par exemple `config/i18n/fr.json`. Utilise des clés de type :
   - `app.title`
   - `menu.language.choice`
   - `menu.usage.title`
   - `usage.big_book.label`
   - `usage.big_code.label`
   - `usage.fast_agents.label`
   - `usage.mixed.label`
3. Détection de la langue par défaut via la variable d’environnement (`LANG`, etc.), normalisation en `en`, `fr`, `de`, `es`, puis possibilité de choisir la langue au premier lancement.
4. Fallback en cascade :
   - si le fichier de langue choisi est invalide → tomber sur `en`,
   - si la clé est manquante → afficher la clé elle‑même.
5. Prévoir une commande `wzllama --check-i18n` pour vérifier que toutes les langues ont les clés obligatoires.

### Wizard d’usages externalisé + pondération

1. La structure du wizard d’usage doit être externalisée dans `config/usages.yaml` (YAML), et non codée en dur.
2. Schéma proposé pour `usages.yaml` :

```yaml
usages:
  big_book:
    weights:
      default: 0.7
      writer: 0.9
    i18n_key: "usage.big_book.label"
    params:
      type: "book"
      pages_per_chunk: 20
      context_tokens: 8192

  big_code:
    weights:
      default: 0.6
      dev: 0.9
    i18n_key: "usage.big_code.label"
    params:
      type: "code"
      loc_per_chunk: 500
      context_tokens: 4096

  fast_agents:
    weights:
      default: 0.9
    i18n_key: "usage.fast_agents.label"
    params:
      type: "agents"
      max_tokens_per_call: 1024
      context_tokens: 2048

  mixed:
    weights:
      default: 0.3
    i18n_key: "usage.mixed.label"
    params:
      type: "mixed"
      context_tokens: 4096
```

3. Au démarrage, le wizard :
   - charge `usages.yaml`,
   - valide sa structure (clés obligatoires, types, valeurs autorisées),
   - calcule un ordre d’affichage des usages **triés par poids décroissant** (par profil de poids, ex. `default`, `writer`, `dev`),
   - affiche les libellés en prenant `i18n_key` → fichier de langue.
4. Plus tard, possibilité de faire évoluer dynamiquement les poids (usage tracking), mais le prompt doit déjà prévoir une **structure propre et validable**.

### Validation des templates utilisateur

L’utilisateur doit pouvoir modifier les templates (`usages.yaml`, fichiers i18n) **sans casser définitivement le wizard** :

1. Au chargement de `usages.yaml` :
   - vérifier :
     - présence de la clé racine `usages`,
     - que chaque entrée est un dict avec : `i18n_key` (string), `weights` (dict avec au moins `default` numérique), `params` (dict avec au minimum `params.type` ∈ {`book`,`code`,`agents`,`mixed`}),
   - en cas de problème, **ne pas planter** :
     - afficher un rapport lisible :
       - `Usage 'big_book' : 'params.type' manquant.`
       - `usages.yaml : 'weights.default' doit être numérique.`
     - revenir sur une configuration interne par défaut,
     - éventuellement proposer une commande `wzllama --reset-templates` qui régénère des templates propres (en sauvegardant l’ancien fichier en `.bak`).  [solaris.readthedocs](https://solaris.readthedocs.io/en/latest/tutorials/notebooks/creating_the_yaml_config_file.html)
2. Même logique pour les fichiers i18n :
   - liste de clés obligatoires (minimum vital pour le wizard),
   - validation type/présence,
   - rapport d’erreurs + fallback sur `en.json`.

### Détection matériel & profil de la machine

Au lancement du wizard, il doit faire un **profil matériel** :

1. Détection :
   - OS : `platform.system()` + version.
   - RAM totale (Go).
   - GPU (NVIDIA via `nvidia-smi` : nom + VRAM par carte), fallback “aucun GPU détecté / VRAM inconnue”.  [localllm](https://localllm.in/blog/ollama-vram-requirements-for-local-llms)
2. Affichage à l’utilisateur :
   - OS,
   - RAM,
   - GPU/VRAM ou CPU‑only.
3. Utiliser des règles simples issues de la littérature Ollama pour déterminer ce qui est raisonnable :
   - 3B : confortable même en CPU/RAM,
   - 7B : ~5–8 Go de mémoire pour rester fluide,
   - 13–14B : ~10–12 Go,
   - 30–32B : ~24 Go,
   - 70B+ : 40–64 Go+, plutôt station de travail.  [localllm](https://localllm.in/blog/ollama-vram-requirements-for-local-llms)

### Estimation tokens / contextes / temps

Le wizard doit transformer un besoin utilisateur (“gros projet de code”, “roman 600 pages”) en :

- estimation du volume en tokens,
- proposition de découpage en **plusieurs contextes**,
- estimation de temps (fourchette) selon la machine.

Règles :

1. Conversion :
   - 1 page ≈ 400–700 tokens (prendre ~550 tokens/page comme moyenne).  [news.ycombinator](https://news.ycombinator.com/item?id=35841781)
   - 1 ligne de code ≈ 6–9 tokens (prendre ~8 tokens/LOC comme moyenne).
2. Exemples :
   - Livre 600 pages → ~240 000–320 000 tokens → impossible à mettre dans un seul contexte, d’où découpage en chapitres.
   - Monorepo 100k LOC → ~800 000 tokens → idem, besoin de travailler par modules + RAG.
3. Proposition de contextes :
   - Gros livre : chapitres de ~20 pages (≈ 11 000 tokens), contextes 8k–16k tokens, RAG ou résumés pour cohérence globale.
   - Gros code : chunks de ~500 LOC (~4 000 tokens), contextes 4k–8k, index/RAG sur repo.
4. Temps :
   - utiliser des valeurs par défaut de tokens/s par taille de modèle (3B, 7B, 14B, 32B, 70B) basées sur des benchmarks typiques GPU grand public.  [hardware-corner](https://www.hardware-corner.net/gpu-for-llm-in-march-2025-20250326/)
   - plus tard, prévoir une commande `wzllama bench` qui :
     - appelle l’API Ollama avec un prompt standard,
     - récupère les tokens/s réels,
     - enregistre tout dans `bench.json` et utilise ça pour les estimations.
   - afficher le temps estimé avec une **fourchette large** :
     - ex. “Chapitre (~10 000 tokens) → entre 2 et 5 minutes sur ta machine”.

### Choix des modèles, VRAM vs RAM, agents rapides

Le wizard doit décider, en fonction de la machine + usage, quels modèles sont réalistes et où ils tourneront :

1. Familles de modèles (exemples, à paramétrer) :
   - Code : `qwen2.5-coder:{3b,7b,14b,32b}`.
   - Écriture : `qwen2.5:{3b,7b,14b,32b}` ou autres modèles texte adaptés.
   - Agents rapides : petits modèles (3B–7B) pour rester en VRAM avec latence minimale.
2. Décision VRAM vs RAM :
   - si VRAM suffisante (≥ débordement + marge) → modèle GPU (rapide),
   - sinon : alerter l’utilisateur que le modèle débordera en RAM (plus lent, acceptable pour tâches ponctuelles mais pas pour usage interactif intensif).  [localllm](https://localllm.in/blog/ollama-vram-requirements-for-local-llms)
3. Pour **agents de réflexion courte** :
   - proposer un profil “agent-fast” :
     - modèle 3B–7B,
     - contexte 2k–4k,
     - max tokens par réponse ~1k,
     - tournant idéalement 100 % en VRAM,
     - option de fine‑tuning LoRA (non implémentée, mais préparer l’interface pour ça).

### Installateurs : Ollama, Open WebUI, Hermes, OpenClaw

Le wizard doit :

1. **Détecter** si chaque outil est déjà installé (`which`/`shutil.which` ou équivalent).
2. Pour chaque outil manquant, afficher :
   - la ou les commandes officielles d’installation (par exemple :  
     - Ollama Linux/macOS : `curl -fsSL https://ollama.com/install.sh | sh`  
     - Ollama Windows : script PowerShell officiel,  
     - Open WebUI : `pip install open-webui` + `open-webui serve` ou Docker,  
     - Hermes Agent : one‑liner d’installation fourni par le projet,  
     - OpenClaw : `npm install -g openclaw` + `openclaw`),  [youtube](https://www.youtube.com/watch?v=tVTwZRhxw9w)
   - **demander une confirmation explicite** avant de lancer un `curl | sh` ou `npm install -g`.
3. Si l’installation échoue :
   - afficher l’erreur (stderr),
   - **montrer les commandes à relancer manuellement**,
   - garder trace de l’état dans `config.json` (par ex. `hermes: "failed"`),
   - ne pas interrompre le wizard brutalement (continuer en mode dégradé / doc).

### Validation, logging, modes CLI

1. Validation :
   - validation systématique des templates (usages, i18n),
   - en cas d’erreur → rapport + fallback.
2. Logging :
   - tout log technique détaillé dans `logs/wzllama.log`,
   - affichage dans le terminal limité à des messages lisibles.
3. Modes CLI :
   - `wzllama` : wizard interactif normal,
   - `wzllama --dry-run` : ne fait qu’afficher ce qui serait exécuté (install, commandes) sans rien lancer,
   - `wzllama --validate` : vérifie `usages.yaml` + i18n et affiche les erreurs,
   - `wzllama --reset-templates` : régénère les templates par défaut (avec sauvegarde `.bak`),
   - éventuellement `wzllama bench` : mini-benchmark Ollama (si Ollama est présent).

### Style de code attendu

- Code Rust clair, découpé en fonctions/méthodes lisibles.
- Commentaires concis mais suffisants pour comprendre la logique.
- Tester la robustesse : gérer les erreurs réseau, les timeouts, les fichiers manquants, les mauvaises permissions, etc.

***

## Architecture wzllama (Rust)

### Dossiers et fichiers

- Binaire : `wzllama`  
- Config utilisateur :  
  - `~/.wzllama/config.yaml` (chemin principal)  
  - `~/.wzllama/usages.yaml` (arbre de wizard + poids)  
  - `~/.wzllama/i18n/en.json`, `fr.json`, `de.json`, `es.json`
- Config “par défaut” livrée **à côté du binaire** (pour initialisation) :  
  - `./config/default_config.yaml`  
  - `./config/default_usages.yaml`  
  - `./config/i18n/en.json`, etc.

Au premier lancement, wzllama :

1. cherche `~/.wzllama/...`  
2. si absent, copie les templates depuis `./config` vers `~/.wzllama`  
3. charge **uniquement** depuis `~/.wzllama` ensuite.

***

## Crates Rust utiles

- **clap** pour le CLI (`wzllama`, `wzllama --validate`, `wzllama bench`, etc.).  [oneuptime](https://oneuptime.com/blog/post/2026-02-03-rust-clap-cli-applications/view)  
- **serde + serde_yaml + serde_json** pour les configs YAML/JSON.  [reddit](https://www.reddit.com/r/rust/comments/1ewazih/which_config_format_should_i_choose_in_rust/)  
- Optionnel : **config-rs** si tu veux un système de config plus riche (multi‑formats, layering), mais tu peux commencer sans.  [github](https://github.com/rust-cli/config-rs)  
- Pour l’i18n, tu peux rester simple : JSON + `HashMap<String, String>` chargés à l’exécution, plutôt que `rust-i18n` (qui est plutôt orienté compile‑time).  [github](https://github.com/longbridge/rust-i18n)

***

## Schéma minimal de config en Rust

### 1. Structures Rust (usages)

```rust
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct UsageParams {
    pub r#type: String,          // "book" | "code" | "agents" | "mixed"
    #[serde(default)]
    pub pages_per_chunk: Option<u32>,
    #[serde(default)]
    pub loc_per_chunk: Option<u32>,
    #[serde(default)]
    pub context_tokens: Option<u32>,
    #[serde(default)]
    pub max_tokens_per_call: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UsageSpec {
    pub i18n_key: String,
    pub weights: HashMap<String, f32>, // "default", "writer", "dev", ...
    pub params: UsageParams,
}

#[derive(Debug, Deserialize)]
pub struct UsagesConfig {
    pub usages: HashMap<String, UsageSpec>,
}
```

### 2. Chargement des fichiers (binaire + `~/.wzllama`)

```rust
use std::path::{PathBuf, Path};
use std::fs;
use anyhow::{Result, Context};

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

fn wzllama_dir() -> PathBuf {
    home_dir().join(".wzllama")
}

fn ensure_user_templates() -> Result<()> {
    let user_dir = wzllama_dir();
    let config_dir = user_dir.join("config");
    let i18n_dir = user_dir.join("i18n");

    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&i18n_dir)?;

    // Chemin du dossier config à côté du binaire
    let exe = std::env::current_exe()?;
    let exe_dir = exe.parent().unwrap_or(Path::new("."));
    let default_cfg_dir = exe_dir.join("config");

    // Copier usages.yaml si manquant
    let user_usages = config_dir.join("usages.yaml");
    if !user_usages.exists() {
        let src = default_cfg_dir.join("default_usages.yaml");
        if src.exists() {
            fs::copy(src, &user_usages)
                .with_context(|| format!("Impossible de copier default_usages.yaml vers {}", user_usages.display()))?;
        }
    }

    // Copier i18n/*.json si manquants
    for lang in ["en", "fr", "de", "es"] {
        let user_lang = i18n_dir.join(format!("{lang}.json"));
        if !user_lang.exists() {
            let src = default_cfg_dir.join("i18n").join(format!("{lang}.json"));
            if src.exists() {
                fs::copy(src, &user_lang)
                    .with_context(|| format!("Impossible de copier i18n {lang}.json vers {}", user_lang.display()))?;
            }
        }
    }

    Ok(())
}
```

***

## i18n simple (EN/FR/DE/ES)

```rust
use serde::Deserialize;
use std::collections::HashMap;
use anyhow::{Result, Context};
use std::fs;

#[derive(Debug, Deserialize)]
pub struct LangMap(pub HashMap<String, String>);

pub struct I18n {
    map: HashMap<String, String>,
}

impl I18n {
    pub fn load(lang: &str) -> Result<Self> {
        let base = wzllama_dir().join("i18n");
        let path = base.join(format!("{lang}.json"));

        let data = if path.exists() {
            fs::read_to_string(&path)
                .with_context(|| format!("Lecture i18n {lang} impossible ({}).", path.display()))?
        } else {
            // fallback en
            let fallback = base.join("en.json");
            fs::read_to_string(&fallback)
                .with_context(|| format!("Lecture i18n en impossible ({}).", fallback.display()))?
        };

        let map: HashMap<String, String> = serde_json::from_str(&data)
            .context("Parsing JSON i18n échoué")?;

        Ok(Self { map })
    }

    pub fn t(&self, key: &str) -> &str {
        self.map.get(key).map(String::as_str).unwrap_or(key)
    }
}
```

Usage :

```rust
let lang = std::env::var("LANG").unwrap_or_else(|_| "en".into());
let lang_code = lang.split('.').next().unwrap_or("en").split('_').next().unwrap_or("en");
let i18n = I18n::load(lang_code).unwrap_or_else(|_| I18n { map: HashMap::new() });

println!("{}", i18n.t("app.title"));
```

***

## Validation des templates (best practice)

```rust
fn validate_usages(cfg: &UsagesConfig) -> Vec<String> {
    let mut errors = Vec::new();

    if cfg.usages.is_empty() {
        errors.push("La section 'usages' est vide.".into());
        return errors;
    }

    for (key, spec) in &cfg.usages {
        if spec.i18n_key.trim().is_empty() {
            errors.push(format!("Usage '{key}': i18n_key manquant ou vide."));
        }
        if !spec.weights.contains_key("default") {
            errors.push(format!("Usage '{key}': weights.default manquant."));
        }
        match spec.params.r#type.as_str() {
            "book" | "code" | "agents" | "mixed" => {}
            other => errors.push(format!(
                "Usage '{key}': params.type invalide '{other}' (attendu: book|code|agents|mixed)."
            )),
        }
    }

    errors
}

fn load_usages_config() -> Result<UsagesConfig> {
    let path = wzllama_dir().join("config").join("usages.yaml");
    let s = fs::read_to_string(&path)
        .with_context(|| format!("Lecture de {}", path.display()))?;
    let cfg: UsagesConfig = serde_yaml::from_str(&s)
        .context("Parsing YAML usages.yaml échoué")?;
    Ok(cfg)
}
```

Au démarrage :

```rust
let cfg = match load_usages_config() {
    Ok(cfg) => {
        let errs = validate_usages(&cfg);
        if !errs.is_empty() {
            eprintln!("[!] Problèmes dans usages.yaml :");
            for e in errs {
                eprintln!("  - {e}");
            }
            eprintln!("Utilisation de la config interne par défaut.");
            default_usages_config() // une fonction qui renvoie un UsagesConfig hardcodé
        } else {
            cfg
        }
    }
    Err(e) => {
        eprintln!("[!] Impossible de charger usages.yaml : {e}");
        eprintln!("Utilisation de la config interne par défaut.");
        default_usages_config()
    }
};
```

***

## CLI wzllama avec clap

Avec `clap` tu peux définir des sous‑commandes type :

- `wzllama` → wizard interactif
- `wzllama validate` → vérifie templates i18n + usages
- `wzllama bench` → mini‑benchmark Ollama
- `wzllama reset-templates` → recopie les templates par défaut vers `~/.wzllama` (en sauvegardant les anciens)  [oneuptime](https://oneuptime.com/blog/post/2026-02-03-rust-clap-cli-applications/view)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wzllama", about = "Wizard pour stack Ollama locale")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Lancer le wizard interactif
    Wizard,
    /// Valider les templates (usages, i18n)
    Validate,
    /// Mini-benchmark Ollama
    Bench,
    /// Réinitialiser les templates utilisateur
    ResetTemplates,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    ensure_user_templates()?; // copie des templates par défaut si besoin

    match cli.command.unwrap_or(Command::Wizard) {
        Command::Wizard => run_wizard()?,
        Command::Validate => run_validate()?,
        Command::Bench => run_bench()?,
        Command::ResetTemplates => run_reset_templates()?,
    }

    Ok(())
}
```

***

En résumé : construis un outil CLI `wzllama` qui, avec une seule entrée utilisateur, installe/configure la stack IA locale (Ollama, Open WebUI, Hermes, OpenClaw) si besoin, profile la machine, propose un wizard d’usage multilingue basé sur des templates externalisés et pondérés, vérifie et valide les templates modifiés par l’utilisateur, explique les limites matérielles (RAM/VRAM), choisit des modèles adaptés (VRAM vs RAM), estime les temps de génération pour de gros projets (code, livres) en termes de tokens et de secondes, et ne laisse jamais l’utilisateur bloqué sans explication ni commandes à lancer manuellement.

***
Menus actuel:

Que voulez-vous faire ?:
> 🤖 Choisir un modèle IA
  🛠️  Lancer un outil
  🧹 Nettoyage
  ⚙️  Configuration
  🌍 Changer de langue
  ❌ Quitter


🤖 Choisir un modèle IA
────────────────────────────────────────
Choisissez votre usage ::
> ⚡ Agents rapides
  📚 Gros livre
  💻 Grand codebase
  🎯 Usage général
  ↩️  Retour


  Lancer un outil
> ✅ Ollama - Chat avec un modèle IA local
  📦 Open WebUI - Interface web pour vos modèles IA
  📦 OpenClaw - Assistant IA personnel avec 100+ skills
  📦 Claude Code - Outil de codage d'Anthropic avec sous-agents
  📦 Hermes Agent - Agent IA auto-améliorant de Nous Research
  📦 OpenCode - Agent de codage open-source d'Anomaly
  📦 Codex - Agent de codage open-source d'OpenAI
  📦 Copilot CLI - Agent de codage IA de GitHub pour le terminal
  📦 Droid - Agent de codage de Factory (terminal + IDE)
  ✅ Pi - Agent IA minimal avec support plugins
  ✅ Pool - Agent de codage de Poolside (https://github.com/poolsideai/pool)
  ↩️  Retour


  🧹 Nettoyage
> 🗑️  Désinstaller des outils
  📂 Supprimer des flottes
  🤖 Supprimer des modèles
  ↩️  Retour


  🗑️  Désinstaller des outils
> 🗑️  Ollama
  🗑️  Pi
  🗑️  Pool
  ↩️  Retour

  ⚙️  Configuration
   🔧 127.0.0.1:11434 | keep=-1 | cloud=❌ | ctx=16384
   🤖 Code: qwen2.5-coder:14b | Livre: qwen2.5:14b | Agent: qwen2.5:3b | Chat: qwen2.5:7b

> 🔄 Modèles par usage
  ⚡ Performance
  📂 Shells
  📄 Regénérer ~/.wzllama/env
  🗑️  Désinstaller wzllama
  ↩️  Retour