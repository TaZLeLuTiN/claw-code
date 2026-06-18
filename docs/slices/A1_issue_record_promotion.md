# Slice A1 — Promotion fidèle IssueRecord (JSONL → PG_SYMPHONY)

> Conception figée — session 2026-06-18. Impl cible : repo **symphony/** (pas claw-code).
> Ce document est l'artefact de conception ; le prompt en fin de fichier est destiné à Claude Code.

## Décision d'architecture (tranchée, vérifiée sur le code)

- **Home = `PG_SYMPHONY`** (`:5432`, DSN `SYMPHONY_DB_URL`), schéma neuf `learning`,
  migration `symphony/scripts/migration_v38_issue_record.sql` (suit v37). Code ETL :
  `symphony/src/learning/promote.py`.
- **Deux Postgres distincts, aucun pont DB** : `pg_harnais` (`:5433`, mono-tenant, edge)
  vs `PG_SYMPHONY` (`:5432`, multi-tenant, tier). Lien HTTP/JWT, pas de réplication.
  Promouvoir `issue_record` ne splitte aucun SSOT (rien n'est en PG aujourd'hui).
- **Frontière A1/A2** : A1 = promotion fidèle, **zéro IAM**, tenancy en string brut.
  A2 = mapping `tenant_id`/résolution `linked_user_id`/RLS. **Premier slice = A1 seul.**
- **Couplage par contrat de fil, pas par code** : `src/learning/` n'importe PAS le paquet
  harnais (harnais n'est de toute façon pas packagé : ni pyproject ni setup).

## Faits source vérifiés (ne pas re-dériver — lus, pas déduits)

- Contrat : `harnais/tools/issue_record.py::IssueRecord` (Pydantic, `extra="ignore"`,
  `to_envelope()==model_dump()`). `SCHEMA_VERSION="1"` ; `_KNOWN_SCHEMA_VERSIONS={"1"}`.
- `item_id = sha1(f"{spec}\x00{test_path}")[:16]` — **clé LOGIQUE, NON unique.**
- **GRAIN = PAR-ESSAI, PAS PAR-ITEM** (vérifié `bench.py:189-243`) : `run_cell` produit
  `n_trials` records pour un même `(case, config)` ; `item_id` ne dépend PAS du seed →
  **les N essais partagent le même `item_id`**. Les N sont tous émis au ledger ET tous
  passés à `compute_kpis`, qui calcule ses taux SUR LES ESSAIS (acceptance, attempts
  mean/variance, sovereignty sur N). `compute_kpis` n'a AUCUNE logique `item_id`/dédup.
  ⟹ **Collapser sur `item_id` détruirait les KPI. A1 préserve le grain-ligne, zéro collapse.**
- Ledger **append-only** (`IssueLedger._append_line`) → ligne immuable une fois écrite.
- `_error_record` (`bench.py:175`) : `recorded_at` FIXE `"1970-01-01T..."` + item_id
  `f"{config.name}:{case.name}:err"` → des lignes d'erreur peuvent être byte-identiques.
  ⟹ idempotence par digest de contenu serait LOSSY ; on idempote par `(source_file, line_no)`.
- Allow-list colonnes squelette (16) : `schema_version, recorded_at, item_id, spec,
  classification, outcome, reject_reason, attempts, first_attempt_passed, claude_touched,
  cost_usd, wall_clock_ms, cache_hit, tier, user_id, project_id, program_id`.
- `sovereign` : présent dans l'enveloppe mais DÉRIVÉ → ÉCARTÉ à l'ingest (colonne générée).
- Payload JSONB (catch-all = enveloppe − colonnes − `sovereign`) : `engine_path,
  tokens_in, tokens_out, mutants_escalated, mutation_score_base, mutation_score_hardened,
  retained_added` + futurs inconnus. ⚠ `tokens_in/out` sont des `dict[str,int]`, pas scalaires.

---

## Prompt Claude Code — slice A1

```markdown
# Slice A1 — Promotion fidèle IssueRecord (JSONL → PG_SYMPHONY), zéro IAM, GRAIN-LIGNE

## Contexte (vérifié sur le code — ne pas re-dériver)
- Source : ledger append-only `.harnais/issue_ledger.jsonl`, une enveloppe par ligne.
  Contrat = harnais/tools/issue_record.py::IssueRecord (schema_version="1",
  to_envelope()=model_dump() — plat au top-level MAIS tokens_in/out sont des dict/provider).
  NE PAS importer le paquet harnais depuis src/learning/ (couplage par enveloppe JSONL).
- GRAIN PAR-ESSAI : item_id = sha1(spec\x00test_path)[:16] NON unique ; n_trials records
  partagent le même item_id ; compute_kpis agrège SUR LES ESSAIS (vérifié bench.py:189-243).
  ⟹ A1 préserve CHAQUE ligne. AUCUN collapse/dédup sur item_id. PK ≠ item_id.
- Cible : PG_SYMPHONY (port 5432, DSN SYMPHONY_DB_URL), schéma neuf `learning`,
  migration symphony/scripts/migration_v38_issue_record.sql (style v32/v35).
  ETL : symphony/src/learning/promote.py, asyncpg. AUCUN read src/iam, AUCUNE RLS — A2.

## Méthode (non négociable)
- TDD strict : RED → GREEN → REFACTOR. Tests AVANT le code.
- from __future__ import annotations ; types + retours annotés ; zéro stub.
- Blocs > 80 lignes balisés [SECTION:NNNN].
- Action irréversible → bloc Action / Raison / Impact / Récupération avant exécution.

## DDL — migration v38 (idempotente, rejouable)
CREATE SCHEMA IF NOT EXISTS learning;
CREATE TABLE IF NOT EXISTS learning.issue_record (
    id                   BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,  -- surrogate, grain=ligne
    source_file          TEXT NOT NULL,          -- label LOGIQUE du ledger, unique par lignée
    line_no              INTEGER NOT NULL,        -- n° physique 1-based (compte TOUTES les lignes)
    line_digest          TEXT NOT NULL,           -- sha256(ligne brute) — GARDE anti-réembobinage
    item_id              TEXT NOT NULL,           -- clé logique, NON unique (n_trials la partagent)
    schema_version       TEXT NOT NULL,
    recorded_at          TIMESTAMPTZ NOT NULL,
    tier                 TEXT NOT NULL CHECK (tier IN ('harnais','claw-code')),
    user_id TEXT, project_id TEXT, program_id TEXT,    -- string brut, NULL honnête
    spec                 TEXT NOT NULL,
    classification       TEXT NOT NULL CHECK (classification IN ('simple','complex')),
    outcome              TEXT NOT NULL CHECK (outcome IN ('ACCEPTED','REJECTED')),
    reject_reason        TEXT,
    attempts             INTEGER NOT NULL,
    first_attempt_passed BOOLEAN NOT NULL,
    claude_touched       BOOLEAN NOT NULL,         -- autoritatif
    sovereign            BOOLEAN GENERATED ALWAYS AS (NOT claude_touched) STORED,
    cost_usd             DOUBLE PRECISION,
    wall_clock_ms        BIGINT NOT NULL,
    cache_hit            BOOLEAN NOT NULL DEFAULT FALSE,
    payload              JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (source_file, line_no)                 -- clé positionnelle ; line_digest = garde au conflit
);
CREATE INDEX IF NOT EXISTS idx_issue_record_item_id     ON learning.issue_record (item_id);
CREATE INDEX IF NOT EXISTS idx_issue_record_outcome     ON learning.issue_record (outcome);
CREATE INDEX IF NOT EXISTS idx_issue_record_recorded_at ON learning.issue_record (recorded_at);
CREATE INDEX IF NOT EXISTS idx_issue_record_tier        ON learning.issue_record (tier);
-- PAS d'index tenancy en A1 (pattern d'accès conçu par A2).
-- Vue "état courant par item" (DÉRIVÉE, non load-bearing) — pour dashboards A2+, optionnelle :
--   CREATE VIEW learning.issue_record_latest AS
--   SELECT DISTINCT ON (item_id) * FROM learning.issue_record ORDER BY item_id, recorded_at DESC;
--   (le collapse vit dans une VUE, jamais dans la table de base.)

### Invariants load-bearing
- GRAIN-LIGNE : une ligne ledger = une ligne PG. Aucun collapse sur item_id (détruirait les KPI).
- sovereign = GENERATED ... STORED. L'ETL ÉCARTE la clé `sovereign` de l'enveloppe avant insert.
- tier LU de l'enveloppe, JAMAIS déduit d'une tenancy NULL.
- Mapping ALLOW-LIST des 16 clés → colonnes ; payload = enveloppe − colonnes − {sovereign}.
- PRÉCONDITION clé positionnelle : `source_label` STABLE ET UNIQUE par lignée de ledger,
  au-dessus d'un espace de lignes append-only JAMAIS ré-embobiné (rotation/troncature) ni
  partagé entre deux edges. La garde line_digest rend toute violation BRUYANTE (IntegrityError),
  jamais un drop silencieux. En multi-edge (déféré A2/ops), `source_label` DOIT encoder
  l'identité d'instance.

## ETL : promote_ledger(jsonl_path: Path, source_label: str, pool) -> PromotionStats
- line_no = n° physique 1-based, comptant TOUTES les lignes (y compris vides/skippées :
  une ligne skippée ne produit pas de row mais consomme son line_no → mapping déterministe).
- Tolérance MIROIR de IssueLedger.all() : ligne vide / JSON cassé / schema_version ∉ {"1"}
  → skipped, VENTILÉ (skipped_empty / skipped_badjson / skipped_unknown_version). On continue.
- INTÉGRITÉ (≠ tolérance) : JSON-valide + version connue mais champ squelette NOT NULL manquant
  → bucket `rejected:[(item_id, line_no, missing_field)]`. On continue ; EN FIN DE RUN si
  rejected non vide → raise PromotionIntegrityError(stats) APRÈS commit des lignes valides.
  PromotionIntegrityError DOIT porter le manifeste complet (item_ids + line_no + raison).
  (Un renommage de champ squelette retombe ici — bruyant, voulu.)
- Idempotence + GARDE anti-réembobinage : line_digest = sha256(ligne brute). Sur
  INSERT ... ON CONFLICT (source_file, line_no) :
    • aucune ligne insérée + digest existant == digest entrant → already_present (idempotent) ;
    • aucune ligne insérée + digest existant != digest entrant → rejected/IntegrityError
      (« line_no réutilisé pour un contenu différent » = ledger tourné/tronqué ou source_label
      collisionné entre deux edges). Convertit le drop SILENCIEUX en échec bruyant.
  (line_digest est une GARDE, pas la clé : deux lignes d'erreur identiques à line_no différents
  restent distinctes. PAS de DO UPDATE, PAS de logique recorded_at — aucun collapse.)
  Mécanique SQL libre (ex. INSERT ... DO NOTHING RETURNING, sinon SELECT du digest existant).
- PromotionStats = dataclass : inserted, already_present (conflit, digest identique),
  skipped_empty, skipped_badjson, skipped_unknown_version,
  rejected:list[(item_id,line_no,reason)]. PAS d'`updated` (la sémantique n'existe plus).
  Observabilité NON porteuse.

## Tests (TDD — 8 tests, écrire EN PREMIER, behaviorally-coupled)
> Les tests-oracle (load-bearing) sont désignés PAR NOM, pas par numéro — un numéro casse
> au moindre renumérotage (cf. l'ancien « 4/6/7a/7b »). Oracle = **garde anti-réembobinage**,
> **invariant souveraineté**, **intégrité fail-loud**, **fidélité du grain**.

1. **round-trip squelette** : enveloppe connue → 16 colonnes identiques ; payload contient
   tokens_in/out (dicts), engine_path, mutation_score_*, mutants_escalated, retained_added.
2. **idempotence** : promote ×2 (même fichier) → même nb de lignes ; au 2e run inserted=0,
   already_present == nb de lignes valides.
3. **garde anti-réembobinage** [oracle] : même (source_file, line_no), contenu DIFFÉRENT
   (digest ≠) → PromotionIntegrityError (ledger tourné / source_label collisionné). Même
   position, même contenu → already_present (pas d'erreur).
4. **tier préservé** : enveloppe tier="claw-code" + tenancy NULL reste "claw-code".
5. **invariant souveraineté** [oracle] : claude_touched=true ⇒ sovereign=false ; l'ETL n'écrit
   jamais sovereign (clé écartée) ; la colonne générée recalcule.
6. **tolérance ventilée & forward-compat** : ligne vide / JSON cassé / schema_version="2"
   → skipped_* ventilé, pas d'insert ; champ payload inconnu → JSONB, pas de crash.
7. **intégrité fail-loud** [oracle] : version "1" + spec absent → rejected (manifeste porté) ;
   autres lignes valides committées ; PromotionIntegrityError levée en fin de run.
8. **fidélité du grain** [oracle] — trois sous-checks, tous obligatoires :
   8a. MULTIPLICITÉ : N lignes même item_id (n_trials), recorded_at distincts ET un cas
       d'erreurs byte-identiques → les N présentes en PG, aucune fusionnée ; re-promote → N.
       Prouve l'absence de collapse.
   8b. ÉGALITÉ PROFONDE (harnais-free) : pour chaque ligne, égalité des 16 colonnes ET du
       payload JSONB COMPLET (dict-à-dict) source vs PG. NE PAS collapser l'attendu (sinon
       tautologie). Couvre la fidélité colonnes + JSONB.
   8c. CROSS-CHECK compute_kpis (OBLIGATOIRE EN CI, env-gated en local) : reconstruire la liste
       COMPLÈTE depuis PG ; compute_kpis(records_PG, routing_rows) ==
       compute_kpis(records_source, routing_rows) sur les 5 têtes. Local : skip-on-ImportError
       légitime. CI (env CI=1) : import harnais manquant = ÉCHEC, pas skip (harnais sur
       PYTHONPATH). C'est l'unique garde du choix de grain.

## Hors périmètre (déféré — ne PAS implémenter)
- Mapping tenancy → tenant_id, résolution linked_user_id, RLS, index tenancy → A2.
- Vue issue_record_latest (collapse par item) → optionnelle, A2+/reporting.
- Transport JSONL inter-hôtes → A1 prend chemin de fichier local + source_label en entrée.
- Re-câblage PRODUCTION de la chaîne KPI sur PG → A1 = loaders de test (8b/8c) uniquement.
- KPI escalation_rescue/misroute_* : dépendent de routing_rows (ledger séparé), hors A1.

## Vérification finale
- ruff / mypy propres ; migration v38 rejouable ; tests 1–8b verts ; 8c (cross-check KPI) vert
  en CI (skip toléré seulement hors CI) ; AUCUN import du paquet harnais dans src/learning/ (grep garde).
```

---

## Session-typing (impl)
1. Les 8 tests d'abord, **Claude-authored** (ils encodent le contrat).
2. Les tests-oracle **par nom** — garde anti-réembobinage, invariant souveraineté, intégrité
   fail-loud, fidélité du grain (8a/8b/8c) — non négociablement Claude-authored ET Claude-reviewed.
3. DDL + squelette ETL → Ollama local autorisé.
4. Revue finale → Claude. Ne pas inverser.

## Résidu connu (côté harnais, hors A1)
- Ajout d'un champ squelette SANS bump de `schema_version` → routé silencieusement en JSONB
  (jamais en colonne). Acceptable seulement si le bump sur changement de squelette est
  discipliné côté harnais. Un *renommage* est bruyant (NOT NULL → rejected) ; un *ajout* ne
  l'est pas. À confirmer/enforcer côté harnais ; A1 reste robuste (catch-all JSONB).
