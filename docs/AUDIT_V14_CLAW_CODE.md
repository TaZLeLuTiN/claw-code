# AUDIT claw-code — Préparation PRD harnais v14

**Date :** 2026-05-18  
**Projet :** TaZLeLuTiN/claw-code  
**Auditeur :** Claude Sonnet 4.6 via harnais v13.0.0  
**Objectif :** Fusion harnais Python + claw-code Rust via PyO3

---

## Section 1 — Métriques quantitatives

### 1.1 Lignes de code par langage

| Langage | Fichiers | Lignes (code) | Remarques |
|---------|----------|---------------|-----------|
| Rust | 98 | 93 897 | Workspace de 11 crates |
| Python | 67 | 2 097 | `src/` — référence TypeScript portée |
| Total | 165 | ~96 000 | — |

*tokei non installé — décompte via `find … | xargs wc -l`.*

### 1.2 Tests Rust

| Métrique | Valeur |
|----------|--------|
| Fonctions `#[test]` | 866 |
| Modules `#[cfg(test)]` | 76 |
| Fichiers de tests d'intégration | 8 |

**Statut compilation des tests :** ❌ ROUGE  
Le crate `runtime` ne compile pas en mode test :

```
error[E0432]: unresolved import `super::ChatMessage`  (×6)
→ crates/runtime/src/providers/{anthropic,deepseek,ollama}.rs
```

`ChatMessage` est défini dans `providers/mod.rs` mais les modules fils l'importent via `super::` alors que le chemin réel est `crate::providers::ChatMessage`. CI GitHub Actions bloquée en l'état.

### 1.3 Coverage

Non mesuré — `cargo-tarpaulin` absent, aucune config `tarpaulin.toml`.  
À installer : `cargo install cargo-tarpaulin`

### 1.4 Activité git (30 derniers jours)

**0 commits** sur les 30 derniers jours.  
Dernier commit : `1e71aba feat: complete AI chat system implementation`

### 1.5 Diff avec upstream

Aucun remote `upstream` configuré.  
Seul remote : `origin git@github.com:TaZLeLuTiN/claw-code.git`

Pour configurer :
```bash
git remote add upstream https://github.com/anthropics/claude-code
git log upstream/main..HEAD --oneline
```

---

## Section 2 — Architecture actuelle

### 2.1 Structure du repo

```
claw-code/
├── rust/                     # Workspace Rust (11 crates) — 93 897 lignes
│   ├── crates/
│   │   ├── api/              # Clients provider (Anthropic, OpenAI-compat)
│   │   ├── commands/         # Commandes slash, skills, agents
│   │   ├── compat-harness/   # Extraction manifests, parity tests
│   │   ├── gui/              # Web UI (axum, Ollama chat)
│   │   ├── mock-anthropic-service/  # Mock pour tests
│   │   ├── plugins/          # Système plugins + hooks
│   │   ├── runtime/          # Cœur du runtime (29 682 lignes)
│   │   ├── rusty-claude-cli/ # Binaire `claw` (12 798 lignes)
│   │   ├── telemetry/        # Observabilité
│   │   └── tools/            # Outils (bash, PDF, lane)
│   └── claw_standalone/      # CLI standalone minimaliste
├── src/                      # Python — port de référence (2 097 lignes)
├── tests/                    # Surface de validation Python
├── docs/                     # Documentation
└── .harnais.toml             # ✅ Harnais v13.0.0 initialisé
```

### 2.2 Modules Rust principaux

| Module (crate) | LoC | Tests | Statut | Description |
|----------------|-----|-------|--------|-------------|
| runtime | 29 682 | ~400+ | ⚠️ compile err (tests) | Cœur : session, conversation, MCP, providers, hooks, permissions |
| rusty-claude-cli | 12 798 | 5 suites | 🟡 (allow dead_code glob) | Binaire principal `claw` |
| tools | 8 844 | ~77 | ✅ | Outils bash, PDF, lane completion |
| api | 5 590 | ~28 | ✅ | Clients HTTP Anthropic + OpenAI-compat |
| commands | 5 428 | 16 | ✅ | Skills, agents, MCP, slash commands |
| plugins | 3 958 | ~10 | ✅ | Plugin manager + hooks lifecycle |
| mock-anthropic-service | 1 157 | — | ✅ | Serveur mock pour tests |
| telemetry | 526 | 3 | ✅ | Tracking usage et coûts |
| gui | 407 | — | ✅ | Web UI (axum) |
| compat-harness | 357 | — | ✅ | Extraction manifests upstream |
| claw_standalone | 284 | — | 🟡 3 TODOs | CLI BMAD minimaliste |

### 2.3 Cargo.toml — Dépendances clés

**Workspace dependencies :**
- `serde_json = "1"`, `anyhow = "1.0"`
- `tokio = { version = "1.0", features = ["full"] }`
- `crossterm = "0.29.0"`

**Dépendances notables par crate :**
- `api` : reqwest 0.12, thiserror, async-trait
- `runtime` : sha2, glob, regex, walkdir, url, yaml-rust2, async-trait, reqwest
- `rusty-claude-cli` : rustyline, pulldown-cmark, syntect
- `gui` : axum, tower-http, chrono

**Lints workspace :**
- `unsafe_code = "forbid"` ✅
- `clippy::all + clippy::pedantic` — warn (avec allow sur module_name_repetitions)

**Targets binaires :**
- `claw` (rusty-claude-cli)
- `mock-anthropic-service`

### 2.4 Build matrix

| Plateforme | Tests CI | Release CI | Statut |
|------------|----------|------------|--------|
| Linux x64 (ubuntu-latest) | ✅ (rust-ci.yml) | ✅ (release.yml) | OK |
| macOS ARM64 (macos-14) | ❌ absent | ✅ (release.yml) | Tests pas couverts |
| Windows | ❌ | ❌ | Non supporté |
| Cross-compile Linux ARM | ❌ | ❌ | Non configuré |

---

## Section 3 — Fonctionnalités spécifiques au fork

### 3.1 Suppression dépendance Anthropic

Le fork maintient le client Anthropic (`crates/api/src/providers/anthropic.rs`) mais l'abstrait derrière le trait `AIProvider` (runtime) et `ApiClient` (api). L'authentification est optionnelle — le projet peut tourner 100 % local via Ollama sans aucune clé API.

Tests sans clé Anthropic : le crate `mock-anthropic-service` fournit un serveur HTTP local qui simule l'API Anthropic pour les tests.

### 3.2 Multi-provider

**Trait principal :** `AIProvider` dans `rust/crates/runtime/src/providers/mod.rs:76`

```rust
#[async_trait]
pub trait AIProvider {
    async fn chat_completion(&mut self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;
    fn supports_tools(&self) -> bool;
    fn max_tokens(&self) -> usize;
    fn model_name(&self) -> &str;
    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> Result<(), ProviderError>;
}
```

**Factory :** `create_provider(config: ProviderConfig)` — `providers/mod.rs:97`

| Provider | Fichier | Statut | Auth |
|----------|---------|--------|------|
| Anthropic/Claude | `crates/api/src/providers/anthropic.rs` + `runtime/src/providers/anthropic.rs` | ✅ | ANTHROPIC_API_KEY / OAuth |
| OpenAI-compat (xAI Grok) | `crates/api/src/providers/openai_compat.rs` | ✅ | XAI_API_KEY |
| Ollama | `crates/runtime/src/providers/ollama.rs` | ✅ | Aucune (local) |
| DeepSeek | `crates/runtime/src/providers/deepseek.rs` | ✅ | DEEPSEEK_API_KEY |
| LM Studio | — | ❌ | — |
| Symphony | — | ❌ | — |

**Extension pour nouveaux providers :** Facile — implémenter `AIProvider`, ajouter un bras `match` dans `create_provider()`.

⚠️ **Dualité provider :** deux systèmes coexistent — `crates/api` (streaming, SSE, OAuth) et `crates/runtime/providers` (trait `AIProvider` unifié). Convergence nécessaire avant fusion harnais.

### 3.3 BMAD framework

| Élément | Fichier | Lignes | Statut |
|---------|---------|--------|--------|
| BmadFramework struct | `runtime/src/bmad_bridge.rs` | 27 | 🟡 Stub |
| Génération fichiers BMAD | `claw_standalone/src/main.rs:175-239` | 64 | ✅ Fonctionnel |
| Intégration GUI | `gui/src/main.rs:85` | 1 | 🟡 Label seulement |

**Workflow utilisateur typique :**
1. `claw init` → génère `.claw-framework/` (INSTRUCTIONS.md, PROMPT.md, MANIFEST.md)
2. `BmadFramework::new(project_path)` charge les fichiers au démarrage
3. `get_context_for_role(role)` injecte le contexte dans le system prompt

**Composants BMAD documentés :** Brain (décision), Mind (stratégie), Action (implémentation), DNA (architecture fondamentale).

Aucun orchestrateur BMAD actif — les fichiers sont lus mais pas interprétés dynamiquement.

### 3.4 Bristol 7-Steps

**Statut : ❌ Non implémenté**

Une seule référence dans tout le dépôt :
```
// rust/crates/runtime/src/bmad_bridge.rs:1
//! Bridge BMAD + 7 Règles de Bristol
```

Le CB contient la spécification complète dans `docs/PRD_CONTEXT_BROKER_HARNAIS_V13.md`. L'implémentation Rust est à créer entièrement.

### 3.5 Mémoire PostgreSQL/pgvector

**Statut : ❌ Non commencé**

| Élément | Statut |
|---------|--------|
| Tables prévues | Aucune |
| Migrations | Aucune |
| Dépendance sqlx/tokio-postgres | Absente du Cargo.lock |
| Tests | Aucun |

Le système mémoire actuel est 100 % fichiers (CLAUDE.md, `.claw-framework/`, sessions JSONL). C'est un gap critique pour la fusion avec harnais (qui utilise le CB sur port 7331).

---

## Section 4 — Préparation fusion harnais

### 4.1 Points d'extension naturels pour Python embed

| Module Rust | Justification PyO3 |
|-------------|-------------------|
| `runtime/src/providers/mod.rs` | Exposer `create_provider()` → skills Python peuvent invoquer n'importe quel LLM |
| `runtime/src/session.rs` | Accès session depuis Python pour lecture/écriture contexte |
| `runtime/src/mcp.rs` | Python peut enregistrer/découvrir des serveurs MCP |
| `runtime/src/permissions.rs` | Policy engine callable depuis Python |
| `crates/api/src/prompt_cache.rs` | Cache de prompts partageable entre runtime Rust et Python |

### 4.2 PyO3 compatibility

**Statut : ❌ Aucun POC existant**

Pas de `cdylib` crate, pas de `pyo3` dans `Cargo.lock`.

Pour un POC hello-world :
```toml
# Nouveau crate : ffi/Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.21", features = ["extension-module"] }
runtime = { path = "../crates/runtime" }
```

Compatibilité estimée : bonne — le workspace interdit `unsafe_code` sauf via `pyo3::prelude::*` (qui est `unsafe`-free en surface). Nécessite `#[allow(unsafe_code)]` limité au crate `ffi/`.

### 4.3 Conflits potentiels avec harnais existant

| Conflit | Type | Sévérité |
|---------|------|----------|
| Commandes `init`, `status` | Noms identiques dans claw et harnais | ⚠️ Élevée |
| Variable `ANTHROPIC_API_KEY` | Utilisée par les deux | 🟡 Faible (même valeur attendue) |
| `.harnais.toml` vs `.claw-framework/` | Config dupliquée | 🟡 Moyenne |
| Port 7331 (CB) vs port 4545 (OAuth claw) | Ports distincts | ✅ Pas de conflit |
| `harnais` alias Python vs `claw` binaire Rust | Exécutables séparés | ✅ Pas de conflit |

**Note :** Le `.harnais.toml` généré référence des modules Symphony (`src/iam/`, `src/ethics/`, etc.) qui n'existent pas dans claw-code — ce sont des artefacts du template harnais. À nettoyer dans `.harnais.toml`.

### 4.4 Surface API Rust à exposer pour harnais (PyO3)

| Fonction Rust | Module | Priorité |
|---------------|--------|----------|
| `create_provider(config)` | `runtime::providers` | P0 |
| `ConversationRuntime::new()` + `.run()` | `runtime::conversation` | P0 |
| `Session::load()` / `Session::save()` | `runtime::session` | P1 |
| `BmadFramework::new()` + `get_context_for_role()` | `runtime::bmad_bridge` | P1 |
| `McpServerManager::start()` | `runtime::mcp` | P1 |
| `PermissionPolicy` eval | `runtime::permissions` | P2 |
| `UsageTracker` | `runtime::usage` | P2 |

---

## Section 5 — Compatibilité Claude Code standard

### 5.1 Skills / SKILL.md

**Statut : ✅ Supporté côté Rust**

- `commands/src/lib.rs` : `classify_skills_slash_command()`, `handle_skills_slash_command()`, `handle_skills_slash_command_json()`
- Slash command `/skills` activée dans `rusty-claude-cli/src/main.rs:172`
- Format SKILL.md anthropic : compatible (le crate `compat-harness` extrait les manifests upstream)

### 5.2 Sub-agents

**Statut : ✅ Supporté**

- `handle_agents_slash_command()` dans `commands/src/lib.rs`
- Slash command `/agents` disponible
- Compatible avec la sémantique agent Claude Code 4.6

### 5.3 Hooks Claude Code

**Statut : ✅ Implémenté**

Deux couches de hooks :
- `plugins/src/hooks.rs` — hooks de haut niveau (pre-commit installé par `harnais init`)
- `runtime/src/hooks.rs` — hooks runtime bas niveau (lifecycle, tool execution)

### 5.4 MCP (Model Context Protocol)

**Statut : ✅ Implémentation complète**

6 fichiers dédiés dans `runtime/` :

| Fichier | Rôle |
|---------|------|
| `mcp.rs` | Types et configuration |
| `mcp_client.rs` | Client HTTP/stdio |
| `mcp_lifecycle_hardened.rs` | Lifecycle robuste (retry, dégradé) |
| `mcp_server.rs` | Server-side MCP |
| `mcp_stdio.rs` | Transport stdio |
| `mcp_tool_bridge.rs` | Bridge tools ↔ MCP |

MCPs intégrés : configurable via `.claude/settings.json` (même format que Claude Code officiel).

---

## Section 6 — Dette technique

### 6.1 TODOs / FIXMEs

| Fichier | Ligne | Contenu |
|---------|-------|---------|
| `claw_standalone/src/main.rs` | 125 | `TODO: Intégrer avec nos providers` |
| `claw_standalone/src/main.rs` | 142 | `TODO: Sauvegarder la config` |
| `claw_standalone/src/main.rs` | 146 | `TODO: Tester les modèles` |

### 6.2 `#[allow(dead_code)]` et suppressions

| Fichier | Lignes | Élément supprimé |
|---------|--------|-----------------|
| `tools/src/lane_completion.rs` | 22, 69 | Types et fonctions lane |
| `runtime/src/file_ops.rs` | 31, 561, 577, 592, 609 | Fonctions fichiers |
| `rusty-claude-cli/tests/mock_parity_harness.rs` | 361 | Test helper |
| `api/src/providers/mod.rs` | 13, 16 | Constantes provider |

**Suppression globale :** `rusty-claude-cli/src/main.rs:1` contient `#![allow(dead_code, unused_imports, unused_variables, ...)]` — supprime les warnings sur 12 798 lignes de code. Masque la dette réelle.

### 6.3 cargo audit

**1 vulnérabilité active :**

| CVE | Crate | Version | Sévérité | Solution |
|-----|-------|---------|----------|---------|
| RUSTSEC-2026-0104 | `rustls-webpki` | 0.103.10 | Reachable panic (CRL parsing) | Upgrade `>=0.103.13` |

Chemin : `rustls-webpki → rustls → tokio-rustls → reqwest → {tools, runtime, api, ...}`

### 6.4 cargo outdated

Non exécuté (outil absent). À installer : `cargo install cargo-outdated`

### 6.5 cargo clippy

Bloqué par les 6 erreurs de compilation dans `runtime` (tests). Les warnings non-bloquants identifiés :

- `constant DEPRECATED_TOP_LEVEL_KEYS` jamais utilisée — `runtime/src/config.rs:28`
- `function read_git_recent_commits` jamais utilisée — `runtime/src/prompt.rs:256`
- `function workspace_sessions_dir` jamais utilisée — `runtime/src/session.rs:1441`
- `fields script_name, config` jamais lus — `runtime/src/script_framework/mod.rs:11`

### 6.6 Blockers CI

**CI actuellement rouge** à cause de `E0432: unresolved import super::ChatMessage` dans les tests `runtime`. Correctif : changer les imports dans les 6 fichiers providers de `use super::ChatMessage` vers `use crate::providers::ChatMessage`.

---

## Section 7 — Recommandations v14

### 7.1 Effort fusion estimé

| Phase | Contenu | Durée estimée |
|-------|---------|---------------|
| Phase 0 (prérequis) | Corriger E0432, upgrader rustls-webpki, tarpaulin | 1 jour |
| Phase 1 | PyO3 bridge + Python embed (crate `ffi/`) | 5 jours |
| Phase 2 | Migration commandes harnais (résolution conflits `init`/`status`) | 10 jours |
| Phase 3 | Skills system natif + Bristol 7-Steps Rust | 7 jours |
| Phase 4 | Mémoire unifiée (pgvector + CB client Rust) | 8 jours |
| **Total** | | **~31 jours-homme** |

### 7.2 Stratégie de migration

**Recommandation : progressive, feature-flag.**

1. Créer le crate `ffi/` avec bindings PyO3 minimaux (`create_provider`, `ConversationRuntime`)
2. Intégrer `ffi/` dans harnais Python via `maturin` — zéro disruption sur `claw` binaire
3. Migrer les commandes en conflit : `claw init` → `claw project init`, `claw status` → `claw agent status`
4. Ajouter Bristol 7-Steps comme sous-module du `bmad_bridge`
5. Ajouter `sqlx` + migrations pgvector uniquement en feature-gate `--features memory-postgres`

**Éviter le big bang** — `claw` doit continuer à fonctionner sans harnais installé.

### 7.3 Risques techniques top 3

**R1 — Dualité provider** (Probabilité élevée, Impact élevé)  
Deux systèmes providers coexistent (`crates/api` streaming SSE + `crates/runtime/providers` trait `AIProvider`). Avant PyO3, il faut décider lequel exposer et unifier. Risque de régression sur le streaming Anthropic si on migre vers le trait unifié.

**R2 — Import ChatMessage cassé bloque CI** (Probabilité certaine, Impact élevé)  
6 erreurs `E0432` bloquent `cargo test` et `cargo clippy` sur `runtime`. Toute PR sur `runtime` sera aveugle côté tests jusqu'à correction. C'est le premier ticket à ouvrir.

**R3 — Absence de mémoire partagée Rust/Python** (Probabilité élevée, Impact moyen)  
Le CB harnais tourne sur port 7331 (HTTP). Le runtime Rust n'a pas de client CB. Sans ce client, la mémoire cross-session et le Knowledge Accumulator restent inaccessibles depuis l'agent Rust. Créer `runtime/src/cb_client.rs` (simple reqwest client) comme fondation avant PyO3.

### 7.4 Tests de non-régression à mettre en place

Scénarios critiques à valider avant chaque PR :

| Scénario | Crate | Type |
|----------|-------|------|
| Chat Ollama local (aucune clé API) | runtime + gui | Intégration |
| Chat Anthropic avec clé | api + rusty-claude-cli | Intégration |
| `/skills` liste et exécution | commands | Unitaire |
| Lifecycle MCP server (start/stop/dégradé) | runtime | Intégration |
| Session save/resume | runtime + rusty-claude-cli | Intégration |
| BMAD init génère les 3 fichiers | claw_standalone | Unitaire |
| Hooks pre-commit (harnais v13) | plugins | Intégration |
| PyO3 `create_provider()` depuis Python | ffi (à créer) | Intégration |

---

## Annexe — Résumé init harnais v13

```
✅ .harnais.toml créé (version=13.0.0, project_type=system)
✅ .git/hooks/pre-commit installé
✅ .gitignore enrichi
✅ claw-code ajouté à ~/.harnais/projects.toml
✅ Context Broker : online (port 7331)
```

**Action requise :** Nettoyer les références Symphony dans `.harnais.toml` (sections `[architecture]` et `[context]` font référence à des docs qui n'existent pas dans claw-code).
