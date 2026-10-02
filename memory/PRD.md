# GOLD-e GrowthOS Marketing API — PRD / Working Memory

## Problem statement
Finish + consolidate the existing GOLD-e GrowthOS Marketing API + portal (do NOT replace).
Backend Rust/Axum/SQLx, DB PostgreSQL, Prod API api.goldetech.com, portal goldetech.com/www.
Preserve backend/DB/migrations/env/features. No secrets in git. No unverified deploy. WhatsApp deferred.

## Repo facts
- GitHub Aakruti7870/Marketing-API-Rust, default `main` @ e0b6ed0 (source of truth).
- Working clone at /app/gold-e (nested git, remote=origin HTTPS, unauthenticated → no push creds).

## Consolidation outcome (2026-10-02)
- Inventoried ALL branches via NET diff vs CURRENT main (not merge-base — critical).
- Finding: every feature branch (release-hardening, redis, business-owner-replies, ui-clean, ui-portal)
  is a snapshot of an OLDER main; net diffs are massive DELETIONS; main already contains their features
  (Redis auth, CORS hardening, custom domains, playground, owner-reply at agents.rs:40, deploy/rollback,
  automation engine). Incorporating any would REGRESS. All DEFERRED, left untouched on remote.
- playground-contact-import / production-deploy-rollback-arch / chat = 0 ahead (already merged).
- ONLY clean net-additive, current-main-based work = my `fix/production-completion` (portal lint fix + docs).

## Release branch
- `release/golde-growthos-production-completion`, final SHA = 516647f922d336ae826216257d27b862a136e570.
- = origin/main (e0b6ed0) + 3 commits: 5f16b30 (lint fix), 8fe6bb4 (runbook), 516647f (consolidation report).
- Net vs main: frontend/src/App.jsx, frontend/src/Playground.jsx, docs/PRODUCTION_COMPLETION_RUNBOOK.md, docs/CONSOLIDATION_REPORT.md.
- Backup ref: refs/backup/pre-consolidation-20261002175541.

## Validation (release branch, real toolchain) — all PASS
cargo fmt --check ✓ | clippy -D warnings ✓ | cargo test 5/5 ✓ | cargo build --release ✓
migrations 0001-0008 on disposable DB ✓ (29 tables, numbering preserved)
npm install ✓ | npm run lint ✓ (0) | npm run build ✓ (1947 modules)
Contracts: all actively-used frontend paths match Rust routes (automationsApi.pause unused, UI-guarded).
Diff scanned: no secrets/.env/keys/node_modules/dist. package-lock.json preserved (minimal CI lockfile).

## Integration ledger
Custom domain verify: REAL (Google DoH CNAME). AI chat: REAL, needs AI_API_KEY. WhatsApp: sim default (deferred). Redis: REAL authenticated rediss:// in prod.

## Blocked (no creds in sandbox)
GitHub push (fatal: could not read Username), AWS/EC2 deploy, prod DB, prod authenticated smoke.

## Next
- P0: user pushes release branch (git push -u origin ... OR Emergent Save to GitHub), open PR→main, confirm CI green.
- Separate gate: deploy only after CI green + backups + explicit approval + post-deploy smoke.
- P1: AI_API_KEY in prod. P2 deferred: WhatsApp live creds.
