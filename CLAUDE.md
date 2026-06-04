# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Detected stack
- Languages: Rust.
- Frameworks: none detected from the supported starter markers.

## Verification
- Run Rust verification from `rust/`: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`
- `src/` and `tests/` are both present; update both surfaces together when behavior changes.

## Repository shape
- `rust/` contains the Rust workspace and active CLI/runtime implementation.
- `src/` contains source files that should stay consistent with generated guidance and tests.
- `tests/` contains validation surfaces that should be reviewed alongside code changes.

## Working agreement
- Prefer small, reviewable changes and keep generated bootstrap files aligned with actual repo workflows.
- Keep shared defaults in `.claude.json`; reserve `.claude/settings.local.json` for machine-local overrides.
- Do not overwrite existing `CLAUDE.md` content automatically; update it intentionally when repo workflows change.

## Setup paths — environnement Mike (macOS)

- Repo cloné     : `~/Documents/GitHub/claw-code/`
- Source harnais : `~/Documents/GitHub/harnais/` (Python v13, scope v14.0)
- Scope canonique v14.0 : `harnais/docs/v14_status/v14.0_SCOPE.md`

## Sessions

### Session 2026-06-03 — Merge Phase B (sub-PRD #1 Fusion)

#### Fait
- Vérification branche `feature/v14-pyo3-foundation` :
  - `cargo fmt --check` : OK
  - `harnais-ffi clippy (-D warnings)` : OK — 0 erreur dans le code Phase B
  - `cargo test --workspace` : 1 test pré-existant cassé (api crate, commit 5bcbc86)
    Phase B n'a pas touché crate api — régression antérieure documentée
- Merge `--no-ff` exécuté sur main local : `05efc4b`
  - 7 commits Phase B + 1 merge commit
  - Zéro conflit
- Tag annoté posé : `v14-fusion-complete`
- Sub-PRD #1 Fusion : **COMPLETE** (Phase B finalisée)

#### Issues quarantinées (pré-existantes, deadline à respecter)
- `api/client_integration::send_message_blocks_oversized_requests_before_the_http_call`
  → deadline **2026-06-30**
- 46 clippy errors dans `crates/runtime/`
  → deadline **2026-07-15**

#### En cours
- Push à effectuer manuellement par Mike :
  `git push origin main && git push origin v14-fusion-complete`
- Décision Mike sur conservation/suppression de `feature/v14-pyo3-foundation`
  (locale et distante)

#### Prochaine action
- Côté harnais : marquer `#1 Fusion` comme COMPLETE dans `docs/v14_status/v14.0_SCOPE.md`
- Démarrer sub-PRD #2 Standalone (SQLite CB fallback — indépendant, ~6j)

#### Blocages
- Aucun

---

### Session 2026-06-04 — Sub-PRD #8 IA Router (Lots A + B)

#### Fait
- Crate `harnais-mcp` créé : `rust/crates/harnais-mcp/`
  - `classifier.rs` : heuristiques statiques (26 CLAUDE_KW + 4 catégories Ollama)
  - `server.rs` : JSON-RPC 2.0 stdio, MCP protocol 2024-11-05
  - `tools.rs` : `ollama_generate` (HTTP Ollama + audit PG) + `ollama_route` (classify only)
  - `main.rs` : OLLAMA_HOST + PG_HARNAIS_DSN env, tracing → stderr
  - 10/10 tests verts, clippy 0 warning
- Commits locaux : `f77cfed` (Lot A), `0eade4c` (Lot B)

#### Dispatch D-PLAN-6 actif (crate harnais-mcp)

| Type | Provider | Modèle |
|---|---|---|
| architecture/design/review | Claude | Sonnet 4.6 |
| implémentation | Ollama | gemma4:31b |
| boilerplate/tests | Ollama | gemma3:4b |
| code algorithmique | Ollama | qwen2.5:32b-instruct-q6_K |
| context-fichier seul | Ollama | gemma3:12b |

#### Notes techniques importantes
- `tracing → stderr` : intentionnel, stdout réservé JSON-RPC. Ne pas rediriger vers /dev/null.
- `target/debug/harnais-mcp` → `rust/target/debug/harnais-mcp` (workspace dans rust/)
- `cargo clippy -p harnais-mcp` pour éviter les erreurs pré-existantes de runtime/

#### Prochaine action
- Push : `git push origin main` (commits f77cfed + 0eade4c)
- Activer harnais-mcp dans Claude Code : `claude mcp list` (enregistré via harnais Lot C)
- Tester routing réel sur un prompt boilerplate ✅ (session 2026-06-04 bis)
- Démarrer sub-PRD #3 Skills ou #4 KFs

---

### Session 2026-06-04 bis — Retest ollama_generate

#### Fait
- `ollama_generate` retesté via MCP : réponse correcte, gemma3:4b (task_type boilerplate)
- Routing D-PLAN-6 compris : `task_type` explicite court-circuite le classifier → mapping direct `tools.rs:43`
- 3 commits en avance sur origin/main : `bf25331`, `a5319b7`, `7064a70`

#### Prochaine action
- Push manuel : `git push origin main`
- Démarrer sub-PRD #3 Skills ou #4 KFs (prochaine session)

---

### Session 2026-06-04 ter — Patch harnais-mcp : timeout dynamique + routing Python

#### Fait
- `compute_timeout()` : timeout dynamique basé sur modèle + longueur prompt
  - Remplace le timeout fixe 120s
  - Formule : (input_tokens + 600) / tok_per_sec * 2.5, clampé [30s, 600s]
  - Vitesses estimées (tok/s Apple M) : gemma3:4b=60, gemma3:12b=25, gemma4:31b=12, qwen2.5:32b=8
- `select_model()` : routing affiné D-PLAN-7
  - Python files → qwen2.5:32b-instruct-q6_K
  - Rust files → gemma4:31b
  - Boilerplate/Tests/Distillation → gemma3:4b
  - Défaut impl → qwen2.5:32b
  - Intégré dans `ollama_generate` (paths auto et task_type) et `ollama_route`
- 8 nouveaux tests — 18/18 verts, clippy 0 warning
- Templates `.harnais.toml` mis à jour (clé 'python', timeout_sec=dynamic)
- `v14.0_SCOPE.md` : entrée D-PLAN-7 ajoutée
- Commit claw-code : `91ee1f8`
- Commit harnais : `564e49f`

#### Prochaine action
- Push manuel : `git push origin main` (dans les deux repos)
- Relancer la session Skills #3 (15 skills en 3 lots)
  → skills.py générés via qwen2.5:32b avec timeout adaptatif
  → Lancer depuis `~/Documents/GitHub/harnais`

---

## Politiques qualité (référence harnais)

Ce projet applique les politiques qualité définies dans le repo harnais.
Avant toute modification de tests, lire :

- `~/Documents/GitHub/harnais/docs/policies/QUALITY_POLICY_v1.md`
- `~/Documents/GitHub/harnais/docs/policies/TEST_TYPOLOGIES_v1.md`
- `~/Documents/GitHub/harnais/docs/policies/SKIP_AND_FLAKY_POLICY_v1.md`

Vérification rapide : `~/Documents/GitHub/harnais/bin/policy-check.sh`

### Règles critiques

- Un test rouge est un bug. Pas de "skip parce que ça ne passe pas".
- Skip légitime **UNIQUEMENT** pour : environnement / dépendance / plateforme.
- Tests flaky → **quarantine OBLIGATOIRE** avec deadline (≤ 30j) + issue tracker.
- Pas de quarantine sans deadline.

### Issues en quarantaine (décidées session 2026-06-03)

| Test | Deadline | Issue |
|---|---|---|
| `api/client_integration::send_message_blocks_oversized_requests_before_the_http_call` | 2026-06-30 | À créer |
| 46 clippy errors dans `crates/runtime/` | 2026-07-15 | À créer |
