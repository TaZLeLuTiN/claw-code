#!/usr/bin/env bash
# Wrapper-mince CONSOMMATEUR ENGAGEMENT (RAF-36) — posé par le merge engagement chez un consommateur
# (ex. aegis) qui porte le CÂBLAGE engagement mais résout `harnais` par PATH (portabilité F-B, patron
# éprouvé `kallos_consumer_hook.sh`).
#
# DISPATCH par mode (argv) — sémantiques HÉTÉROGÈNES, dans le WRAPPER (pas le Rust, qui n'append que) :
#   session-up / session-down → TOLÉRANTS (lifecycle : démarrer/arrêter le daemon partagé ne doit
#              JAMAIS bloquer un tour ; un `harnais` absent est signalé loud mais exit 0).
#   stop-gate  → FAIL-CLOSED : rc≠0 → exit 2 ; rc127 (`harnais` hors PATH) PROPAGE loud, jamais un
#              silence (backstop d'ENFORCEMENT ; même classe que « détecteur KALLOS cassé = rc LOUD »).
#              C'est la DENT RAF-36 : retirer le résolveur `harnais` → stop-gate erre LOUD (exit 2),
#              jamais un exit 0 qui perdrait l'enforcement en silence.
#
# `harnais` NU résolu par PATH (wrapper `/usr/local/bin/harnais`, install.sh:206). PAS de `set -e`
# (masquerait le rc de stop-gate) ; PAS de `|| true` sur stop-gate (rc127 doit propager).
set -uo pipefail

MODE="${1:-}"

# ── FIL 109 D4 — ANCRAGE : le verdict est celui du PROJET, jamais celui du `cwd` ───────────
# Mesuré par `cc-orion` le 2026-08-27 : ce wrapper appelait `harnais` SANS aucun `cd`, donc il
# héritait du répertoire courant de qui le lançait — alors que `${CLAUDE_PROJECT_DIR}` est
# disponible ici (les commandes de `settings.json` s'en servent déjà pour TROUVER ce script).
#
#     harnais gate depuis orion/                → ✅ vert
#     harnais gate depuis orion/trading-expert/ → 🔴 « aucun oracle projet dérivable »
#     → deux `gate_status.json` distincts ; le hook lisait CELUI DU cwd
#
# Le shell de `cc-orion` s'était arrêté dans le sous-dépôt en fin de tour : le hook a BLOQUÉ sa
# session pendant que le gate du projet était vert et l'est resté. **Un backstop d'enforcement
# fail-closed dont le verdict dépend du `cwd` n'est pas déterministe**, et son mode de
# défaillance est le pire à diagnostiquer — un blocage inexplicable, pendant que la commande
# relancée à la main rend vert. Personne ne cherche un bug dans le répertoire courant.
#
# ⚠️ LA SÉMANTIQUE DU DISPATCH EST PRÉSERVÉE, et c'est délibéré :
#   · `stop-gate` (fail-closed) → sans ancre, on REFUSE et on le DIT. Un verdict d'enforcement
#     non ancré est pire qu'un refus nommé : il tranche sur un objet qu'on n'a pas choisi.
#   · `session-up`/`session-down` (lifecycle) → on ancre SI on peut, on ne bloque JAMAIS.
#     Empêcher un tour de démarrer parce qu'une variable manque serait la cure pire que le mal.
if [ -n "${CLAUDE_PROJECT_DIR:-}" ] && [ -d "${CLAUDE_PROJECT_DIR}" ]; then
  cd "${CLAUDE_PROJECT_DIR}" || {
    echo "ENGAGEMENT ($MODE) : \`cd\` vers CLAUDE_PROJECT_DIR=${CLAUDE_PROJECT_DIR} a ÉCHOUÉ." >&2
    [ "$MODE" = "stop-gate" ] && exit 2
  }
elif [ "$MODE" = "stop-gate" ]; then
  echo "ENGAGEMENT (stop-gate) : projet INDÉTERMINABLE — CLAUDE_PROJECT_DIR absent ou non" \
       "répertoire (valeur : '${CLAUDE_PROJECT_DIR:-<vide>}'). Un verdict rendu depuis un" \
       "répertoire quelconque ne dit rien du projet : REFUS (exit 2), jamais un vert ni un" \
       "rouge fabriqué." >&2
  exit 2
fi

case "$MODE" in
  session-up)
    harnais session-up --json
    rc=$?
    [ "$rc" -ne 0 ] && echo "ENGAGEMENT (session-up) : harnais rc=$rc (toléré, lifecycle) — exit 0." >&2
    exit 0
    ;;
  session-down)
    harnais session-down --json
    rc=$?
    [ "$rc" -ne 0 ] && echo "ENGAGEMENT (session-down) : harnais rc=$rc (toléré, lifecycle) — exit 0." >&2
    exit 0
    ;;
  stop-gate)
    harnais stop-gate
    rc=$?
    if [ "$rc" -eq 0 ]; then
      exit 0
    else
      echo "ENGAGEMENT (stop-gate) : harnais stop-gate rc=$rc ≠ 0 → fail-closed (exit 2)." >&2
      exit 2
    fi
    ;;
  *)
    echo "engagement_consumer_hook.sh : mode inconnu '$MODE' (attendu : session-up|session-down|stop-gate)" >&2
    exit 2
    ;;
esac
