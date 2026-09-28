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

Historique des sessions : `docs/historique/SESSIONS_CLAUDE_md_2026-06.md` (déplacé le 2026-09-28).

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

### Issues en quarantaine

⚠️ La liste décidée le 2026-06-03 est archivée avec les sessions — ses deux échéances (2026-06-30, 2026-07-15) sont
DÉPASSÉES : à re-mesurer et re-décider, pas à reconduire.
