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

## Consolidation onto feat/redis-production-integration (2026-10-02, third session)
- Checked out target branch locally from origin/feat/redis-production-integration @ 03f3653.
- Cherry-picked ALL 3 Emergent commits: 97d4d75 (lint fix — 1 conflict in App.jsx SET-node inspector, resolved by keeping THIS branch's existing UI since branch's older block has no JSON.parse/catch; no functionality removed), 1e0954d (runbook), 0b84f83 (consolidation report).
- Final SHA = 0b84f8382a05833f0e64b056c2096bd762f684db. Files vs remote target: docs/CONSOLIDATION_REPORT.md, docs/PRODUCTION_COMPLETION_RUNBOOK.md, frontend/src/App.jsx, frontend/src/Playground.jsx.
- Validated on branch (fresh disposable DB 'redisbranch', migrations 0001-0008 all ok, 29 tables): fmt ✓, cargo check ✓, clippy -D warnings ✓, cargo test 5/5 ✓, release build ✓, npm lint 0 ✓, npm build ✓. package-lock.json npm-rewrite reverted. diff --check OK. Secret scan clean.
- Redis connectivity test: BLOCKED (no Redis service in sandbox); Redis code unchanged, config-gated, production health shows redis healthy.
- PUSH BLOCKED again (fatal: could not read Username — no GitHub creds in sandbox). Branch ready locally at 0b84f83.
- NOTE: pod restart wiped PostgreSQL role/db; recreated golde role + redisbranch DB for validation.

## Recovery (2026-10-02, second session)
- Original work INTACT — nothing lost. fix/production-completion @ 8fe6bb4, release branch @ 516647f, backup ref, all present.
- Remote main verified unchanged: e0b6ed0 (fetch OK, 0/0 divergence).
- Created NEW branch `fix/production-completion-recovery` from genuine main @ e0b6ed0; cherry-picked the 2 original commits → 48a1466 (App.jsx+Playground.jsx lint fix) + 96b70b8 (runbook). Final SHA = 96b70b80d059ca121f9db1130aea43f49d63187c.
- Diff vs main: exactly 3 files (+174/−10): docs/PRODUCTION_COMPLETION_RUNBOOK.md, frontend/src/App.jsx, frontend/src/Playground.jsx. Backend/migrations/workflows/env/lockfiles untouched.
- Gates re-run on branch: fmt ✓, clippy -D warnings ✓, cargo test 5/5 ✓, release build ✓, npm install ✓, lint 0 problems ✓, build ✓.
- PUSH BLOCKED (no GitHub creds, fatal: could not read Username). Branch intact locally.

## Next
- P0: user pushes fix/production-completion-recovery (git push -u origin ... OR Save to GitHub), review diff, open PR→main.
- Separate gate: deploy only after CI green + backups + explicit approval + post-deploy smoke.
- P1: AI_API_KEY in prod. P2 deferred: WhatsApp live creds.
