#!/usr/bin/env bash
# Wrapper-mince CONSOMMATEUR KALLOS (brique 1, C-2) — posé par le writer chez un consommateur
# (ex. aegis) qui porte le CÂBLAGE mais PAS le détecteur.
#
# MINCE PAR CONSTRUCTION : il invoque le contrat CLI C-1 `harnais kallos --verify <path>`
# (détecteur résolu target-first + fallback install — ZÉRO vendoring) et se contente de mapper
# rc→exit. AUCUNE logique de détection, AUCUNE re-dérivation de mode : le mode (posttool|stop)
# est un argv, pas une branche métier.
#
# TOTAL-PAR-CONSTRUCTION « sûr sinon bruyant » (honore le fix rc4-loud) — le mapping énumère les
# cas SÛRS, jamais les cas LOUD (une liste blanche de codes LOUD se périmerait au prochain rc) :
#   posttool → exit 0 SEULEMENT si rc ∈ {RC_OK=0, RC_BITES=1} (propre / violations surfacées, non
#              bloquant) ; TOUT LE RESTE → exit 2. ⟹ rc3 (RC_NOT_ARMED), rc4 (RC_DETECTOR),
#              rc127 (harnais hors PATH), tout rc futur = incident LOUD, jamais un silence.
#   stop     → exit 0 SEULEMENT si rc == RC_OK=0 ; tout rc≠0 → exit 2 (fail-closed).
# La SSOT des rc = tools/kallos/verify.py (RC_OK/RC_BITES/RC_NOT_ARMED/RC_DETECTOR) ; le test de
# totalité (tests/test_kallos_consumer_wrapper.py) itère ces SYMBOLES et mord ce mapping.
#
# PAS de `set -e` : le rc de `harnais` est capturé et mappé EXPLICITEMENT (un abort masquerait le
# rc). PAS de `|| true` sur l'invocation : rc127 (harnais hors PATH) doit PROPAGER loud.
set -uo pipefail

MODE="${1:-posttool}"
TARGET="${CLAUDE_PROJECT_DIR:-$(pwd)}"

harnais kallos --verify "$TARGET"
rc=$?

case "$MODE" in
  posttool)
    if [ "$rc" -eq 0 ] || [ "$rc" -eq 1 ]; then
      exit 0
    else
      echo "KALLOS (posttool) : harnais kallos --verify rc=$rc → incident LOUD (exit 2)." >&2
      exit 2
    fi
    ;;
  stop)
    if [ "$rc" -eq 0 ]; then
      exit 0
    else
      echo "KALLOS (stop) : harnais kallos --verify rc=$rc ≠ 0 → fail-closed (exit 2)." >&2
      exit 2
    fi
    ;;
  *)
    echo "kallos_consumer_hook.sh : mode inconnu '$MODE' (attendu : posttool|stop)" >&2
    exit 2
    ;;
esac
