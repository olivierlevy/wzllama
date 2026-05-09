#!/usr/bin/env python3
"""
wzllama : wizard multi-outils pour Ollama + Open WebUI + Hermes + OpenClaw

- Installe dans ~/.wzllama
- Détecte / propose d’installer les outils
- Profile la machine (RAM/VRAM)
- Wizard d’usage (gros livre / gros projet code / agents rapides)
- Estime tokens, contextes, temps approximatif
"""

import os
import sys
import platform
import shutil
import subprocess
import json
from pathlib import Path

HOME = Path.home()
BASE_DIR = HOME / ".wzllama"
CONFIG_DIR = BASE_DIR / "config"
LOG_DIR = BASE_DIR / "logs"
CONFIG_FILE = CONFIG_DIR / "config.json"
BENCH_FILE = CONFIG_DIR / "bench.json"

# ---------- utilitaires de base ----------

def run(cmd, check=False):
    """Exécute une commande shell et retourne (code, stdout, stderr)."""
    try:
        proc = subprocess.run(
            cmd,
            shell=True,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=check,
        )
        return proc.returncode, proc.stdout.strip(), proc.stderr.strip()
    except Exception as e:
        return 1, "", str(e)

def ask_yes_no(msg, default=True):
    suffix = " [Y/n] " if default else " [y/N] "
    ans = input(msg + suffix).strip().lower()
    if not ans:
        return default
    return ans.startswith("y")

def ensure_dirs():
    BASE_DIR.mkdir(exist_ok=True)
    CONFIG_DIR.mkdir(exist_ok=True)
    LOG_DIR.mkdir(exist_ok=True)

def load_json(path, default):
    if path.is_file():
        try:
            return json.loads(path.read_text())
        except Exception:
            return default
    return default

def save_json(path, data):
    path.write_text(json.dumps(data, indent=2))

def is_installed(cmd):
    return shutil.which(cmd) is not None

# ---------- détection hardware ----------

def detect_ram_gb():
    """Retourne la RAM en Go (approx)."""
    system = platform.system()
    if system in ("Linux", "Darwin"):
        code, out, _ = run("free -g | awk '/Mem:/ {print $2}'")
        if code == 0 and out:
            try:
                return int(out)
            except ValueError:
                pass
    # fallback
    return None

def detect_gpus():
    """Retourne une liste de GPUs avec VRAM si dispo (via nvidia-smi)."""
    if shutil.which("nvidia-smi"):
        code, out, _ = run("nvidia-smi --query-gpu=name,memory.total --format=csv,noheader")
        gpus = []
        if code == 0 and out:
            for line in out.splitlines():
                parts = [p.strip() for p in line.split(",")]
                if len(parts) >= 2:
                    name, vram = parts[0], parts[1]
                    try:
                        vram_val = int(vram.split()[0])
                    except Exception:
                        vram_val = None
                    gpus.append({"name": name, "vram_gb": vram_val})
        if gpus:
            return gpus
    # fallback: aucun GPU détecté ou autre vendor
    return []

def summarize_hardware():
    ram = detect_ram_gb()
    gpus = detect_gpus()
    system = platform.system() + " " + platform.release()
    print("\n=== Profil matériel détecté ===")
    print(f"- OS     : {system}")
    print(f"- RAM    : {ram} Go" if ram is not None else "- RAM    : inconnue")
    if gpus:
        for g in gpus:
            print(f"- GPU    : {g['name']} ({g['vram_gb']} Go VRAM)" if g["vram_gb"] else f"- GPU    : {g['name']} (VRAM inconnue)")
    else:
        print("- GPU    : aucun GPU NVIDIA détecté (fonctionnera en CPU/RAM, plus lent)")
    return {"ram_gb": ram, "gpus": gpus, "os": system}

# ---------- estimation tokens / temps ----------

def estimate_tokens_from_pages(pages):
    """
    Estimation grossière :
    ~400–700 tokens/page → prendre 550 tokens/page comme moyenne.
    """
    return int(pages * 550)

def estimate_tokens_from_loc(loc):
    """
    Estimation pour le code : ~6-9 tokens par ligne (très variable).
    On prend 8 tokens/ligne comme moyenne.
    """
    return int(loc * 8)

def default_tokens_per_second(model_size_tag):
    """
    Valeurs par défaut quand on n’a pas encore fait de benchmark.
    Très approximatif, pour une carte GPU correcte (RTX 3060–4070).
    """
    # modèles plus petits -> plus rapides
    if "3b" in model_size_tag.lower():
        return 60
    if "7b" in model_size_tag.lower():
        return 45
    if "14b" in model_size_tag.lower() or "13b" in model_size_tag.lower():
        return 25
    if "32b" in model_size_tag.lower() or "30b" in model_size_tag.lower():
        return 12
    if "70b" in model_size_tag.lower():
        return 5
    return 20

def estimate_time_seconds(tokens_out, tps):
    if tps <= 0:
        return None
    return int(tokens_out / tps)

def format_duration(seconds):
    if seconds is None:
        return "inconnu"
    if seconds < 60:
        return f"{seconds}s"
    minutes = seconds // 60
    if minutes < 60:
        return f"{minutes} min"
    hours = minutes // 60
    minutes = minutes % 60
    return f"{hours}h{minutes:02d}"

# ---------- installateurs ----------

def show_manual_help(tool, commands, docs_url=None):
    print(f"\n[!] L’installation de {tool} a échoué ou a été annulée.")
    print("    Tu peux lancer ces commandes manuellement :")
    for c in commands:
        print(f"    {c}")
    if docs_url:
        print(f"    Documentation : {docs_url}")

def install_ollama():
    system = platform.system()
    if system in ("Linux", "Darwin"):
        cmd = "curl -fsSL https://ollama.com/install.sh | sh"
    elif system == "Windows":
        cmd = 'powershell -Command "irm https://ollama.com/install.ps1 | iex"'
    else:
        print("OS non supporté automatiquement pour Ollama.")
        return False

    print("\nOllama n’est pas installé.")
    print(f"Commande proposée : {cmd}")
    if not ask_yes_no("Lancer cette commande maintenant ?"):
        show_manual_help("Ollama", [cmd], "https://ollama.com")
        return False

    code, out, err = run(cmd)
    if code != 0:
        print("Erreur pendant l’installation d’Ollama :")
        print(err or out)
        show_manual_help("Ollama", [cmd], "https://ollama.com")
        return False

    print("Ollama installé (vérifier éventuellement avec `ollama --version`).")
    return True

def install_open_webui():
    # stratégie simple : pip en local, ou montrer Docker en doc
    print("\nOpen WebUI n’est pas installé.")
    print("Options simples :")
    print("1) Installation locale via pip (nécessite Python + pip)")
    print("2) Installation Docker (nécessite Docker)")
    choice = input("Choix [1/2] (1 par défaut) : ").strip() or "1"

    if choice == "1":
        cmd = "pip install open-webui && open-webui serve"
        print(f"Commande proposée : {cmd}")
        if not ask_yes_no("Lancer l’installation pip de Open WebUI ?"):
            show_manual_help("Open WebUI", [cmd], "https://docs.openwebui.com")
            return False
        code, out, err = run("pip install open-webui")
        if code != 0:
            print("Erreur pendant pip install open-webui :")
            print(err or out)
            show_manual_help("Open WebUI", ["pip install open-webui", "open-webui serve"], "https://docs.openwebui.com")
            return False
        print("Open WebUI installé. Tu peux lancer `open-webui serve`.")
        return True

    if choice == "2":
        docker_cmds = [
            "docker pull ghcr.io/open-webui/open-webui:main",
            "docker run -d --network=host "
            "-v open-webui:/app/backend/data "
            "-e OLLAMA_BASE_URL=http://127.0.0.1:11434 "
            "--name open-webui --restart always "
            "--add-host=host.docker.internal:host-gateway "
            "ghcr.io/open-webui/open-webui:main"
        ]
        print("Commandes Docker proposées :")
        for c in docker_cmds:
            print("  " + c)
        if not ask_yes_no("Les lancer maintenant si Docker est disponible ?"):
            show_manual_help("Open WebUI", docker_cmds, "https://docs.openwebui.com")
            return False
        for c in docker_cmds:
            code, out, err = run(c)
            if code != 0:
                print(f"Erreur pendant `{c}` :")
                print(err or out)
                show_manual_help("Open WebUI", docker_cmds, "https://docs.openwebui.com")
                return False
        print("Open WebUI installé via Docker.")
        return True

    print("Choix invalide, annulation.")
    return False

def install_hermes():
    print("\nHermes Agent n’est pas installé.")
    print("Hermes fournit un script d’installation one-liner.")
    cmd = "bash <(curl -fsSL https://install.hermes-agent.ai)"  # fictif : à adapter à la doc officielle
    print(f"Commande proposée : {cmd}")
    if not ask_yes_no("Lancer le script d’installation Hermes ?"):
        show_manual_help("Hermes Agent", [cmd], "https://hermes-agent.nousresearch.com")
        return False
    code, out, err = run(cmd)
    if code != 0:
        print("Erreur pendant l’installation de Hermes :")
        print(err or out)
        show_manual_help("Hermes Agent", [cmd], "https://hermes-agent.nousresearch.com")
        return False
    print("Hermes Agent installé.")
    return True

def install_openclaw():
    print("\nOpenClaw n’est pas installé.")
    print("Installation typique (Node/npm) :")
    cmd1 = "npm install -g openclaw"
    cmd2 = "openclaw"
    print(f"Commandes proposées :\n  {cmd1}\n  {cmd2}")
    if not ask_yes_no("Lancer `npm install -g openclaw` maintenant ?"):
        show_manual_help("OpenClaw", [cmd1, cmd2], "https://openclaw.ai")
        return False
    code, out, err = run(cmd1)
    if code != 0:
        print("Erreur pendant l’installation d’OpenClaw :")
        print(err or out)
        show_manual_help("OpenClaw", [cmd1, cmd2], "https://openclaw.ai")
        return False
    print("OpenClaw installé. Tu peux lancer `openclaw` pour le wizard propre à OpenClaw.")
    return True

# ---------- wizard usages ----------

def wizard_usage(hardware):
    print("\n=== Que veux-tu faire avec ta stack IA ? ===")
    print("1) Gros projet de code (monorepo, beaucoup de fichiers)")
    print("2) Gros projet d’écriture (roman 600 pages, etc.)")
    print("3) Agents rapides (réflexion courte, peu de tokens)")
    print("4) Autre / mixte")
    choice = input("> ").strip() or "4"

    if choice == "1":
        return wizard_big_code(hardware)
    if choice == "2":
        return wizard_big_book(hardware)
    if choice == "3":
        return wizard_fast_agents(hardware)
    return wizard_mixed(hardware)

def pick_model_for_hardware(hardware, usage_hint):
    """
    Très simplifié : choisit un modèle taille 3B/7B/14B/32B selon VRAM/RAM.
    """
    ram = hardware.get("ram_gb") or 16
    gpus = hardware.get("gpus") or []
    vram = gpus[0]["vram_gb"] if gpus else 0

    # valeur par défaut
    if vram >= 24:
        size = "14b" if usage_hint == "fast" else "32b"
    elif vram >= 12:
        size = "7b" if usage_hint == "fast" else "14b"
    elif vram >= 6:
        size = "3b" if usage_hint == "fast" else "7b"
    else:
        # CPU / RAM only
        size = "3b"

    # modèles imaginaires ici, à adapter à Qwen/Hermes/etc.
    if usage_hint == "code":
        return f"qwen2.5-coder:{size}"
    if usage_hint == "book":
        return f"qwen2.5:{size}"
    if usage_hint == "fast":
        return f"qwen2.5:{size}"
    return f"qwen2.5:{size}"

def wizard_big_book(hardware):
    print("\n=== Gros projet d’écriture ===")
    pages_str = input("Nombre de pages cible (ex: 600) : ").strip() or "600"
    try:
        pages = int(pages_str)
    except ValueError:
        pages = 600
    total_tokens = estimate_tokens_from_pages(pages)

    # découpe par chapitres ~20 pages
    pages_per_chunk = 20
    tokens_per_chunk = estimate_tokens_from_pages(pages_per_chunk)
    chunks = max(1, pages // pages_per_chunk)

    model = pick_model_for_hardware(hardware, "book")
    tps = default_tokens_per_second(model)
    time_per_chunk_sec = estimate_time_seconds(tokens_per_chunk, tps)

    print(f"\nProjet : ~{pages} pages → ~{total_tokens} tokens (estimation large).")
    print(f"Découpage proposé : {chunks} chapitres de ~{pages_per_chunk} pages (~{tokens_per_chunk} tokens).")
    print(f"Modèle conseillé : {model} (profil écriture longue).")
    print(f"Temps estimé pour générer un chapitre complet : {format_duration(time_per_chunk_sec)} "
          f"(à {tps} tokens/s approx).")
    print("Recommandation :")
    print("- Utiliser un contexte de 8k–16k tokens pour chaque chapitre.")
    print("- Utiliser un système de RAG ou de résumés pour garder la cohérence globale.")

def wizard_big_code(hardware):
    print("\n=== Gros projet de code ===")
    loc_str = input("Nombre approximatif de lignes de code (LOC) (ex: 100000) : ").strip() or "100000"
    try:
        loc = int(loc_str)
    except ValueError:
        loc = 100000
    total_tokens = estimate_tokens_from_loc(loc)

    # chunks par fichier/module ~500 LOC
    loc_per_chunk = 500
    tokens_per_chunk = estimate_tokens_from_loc(loc_per_chunk)
    chunks = max(1, loc // loc_per_chunk)

    model = pick_model_for_hardware(hardware, "code")
    tps = default_tokens_per_second(model)
    time_per_chunk_sec = estimate_time_seconds(tokens_per_chunk, tps)

    print(f"\nProjet code : ~{loc} LOC → ~{total_tokens} tokens (estimation large).")
    print(f"Découpage proposé : {chunks} chunks de ~{loc_per_chunk} LOC (~{tokens_per_chunk} tokens).")
    print(f"Modèle conseillé : {model} (profil code).")
    print(f"Temps estimé pour un chunk (analyse/génération) : {format_duration(time_per_chunk_sec)} "
          f"(à {tps} tokens/s approx).")
    print("Recommandation :")
    print("- Travailler fichier/module par module (4k–8k tokens de contexte).")
    print("- Mettre en place un RAG sur le repo pour les questions globales.")

def wizard_fast_agents(hardware):
    print("\n=== Agents rapides (peu de tokens, réflexion courte) ===")
    model = pick_model_for_hardware(hardware, "fast")
    tps = default_tokens_per_second(model)
    tokens_per_call = 1024
    time_per_call = estimate_time_seconds(tokens_per_call, tps)

    print(f"Modèle conseillé : {model} (petite taille pour VRAM et latence minimales).")
    print(f"Profil : agents de réflexion courte, réponses ~{tokens_per_call} tokens.")
    print(f"Temps estimé par appel : {format_duration(time_per_call)} (à {tps} tokens/s).")
    print("Recommandation :")
    print("- Contexte court (2k–4k tokens).")
    print("- À fine-tuner en LoRA pour des tâches d’agent spécifiques (tool use, planification simple).")

def wizard_mixed(hardware):
    print("\n=== Mode mixte ===")
    print("On va utiliser un modèle moyen pour tout faire, et ajuster plus tard.")
    model = pick_model_for_hardware(hardware, "mixed")
    tps = default_tokens_per_second(model)
    print(f"Modèle par défaut : {model}, vitesse attendue ~{tps} tokens/s.")
    print("Tu pourras ensuite créer des profils séparés pour code, écriture et agents.")

# ---------- main ----------

def main():
    ensure_dirs()
    config = load_json(CONFIG_FILE, {})

    print("=== wzllama v0.1 ===")
    hardware = summarize_hardware()
    config["hardware"] = hardware

    # Détection outils
    have_ollama = is_installed("ollama")
    have_open_webui = is_installed("open-webui") or is_installed("openwebui")
    have_hermes = is_installed("hermes")
    have_openclaw = is_installed("openclaw")

    print("\n=== État des outils ===")
    print(f"- Ollama     : {'OK' if have_ollama else 'MANQUANT'}")
    print(f"- Open WebUI : {'OK' if have_open_webui else 'MANQUANT'}")
    print(f"- Hermes     : {'OK' if have_hermes else 'MANQUANT'}")
    print(f"- OpenClaw   : {'OK' if have_openclaw else 'MANQUANT'}")

    # Installations éventuelles
    if not have_ollama:
        if install_ollama():
            have_ollama = True

    if not have_open_webui:
        if ask_yes_no("Installer Open WebUI ?"):
            if install_open_webui():
                have_open_webui = True

    if not have_hermes:
        if ask_yes_no("Installer Hermes Agent ?"):
            if install_hermes():
                have_hermes = True

    if not have_openclaw:
        if ask_yes_no("Installer OpenClaw ?"):
            if install_openclaw():
                have_openclaw = True

    config["tools"] = {
        "ollama": have_ollama,
        "open_webui": have_open_webui,
        "hermes": have_hermes,
        "openclaw": have_openclaw,
    }
    save_json(CONFIG_FILE, config)

    # Wizard usages
    wizard_usage(hardware)

    print("\n=== Fin du wizard v0.1 ===")
    print("Les recommandations affichées servent de base. Tu pourras affiner les profils et scripts LoRA ensuite.")

if __name__ == "__main__":
    main()