# GOLD-e GrowthOS Marketing API — PRD / Working Memory

## Problem statement
Finish the existing GOLD-e GrowthOS Marketing API + portal (do NOT replace it).
Backend: Rust/Axum/SQLx. DB: PostgreSQL. Prod API: api.goldetech.com. Portal: goldetech.com / www.goldetech.com. Deploy: AWS EC2 + GitHub Actions.
Preserve backend, DB, migrations, env, features. Don't print/request secrets. Don't deploy unverified. WhatsApp deferred.

## Repo facts
- GitHub: Aakruti7870/Marketing-API-Rust, default branch `main` @ e0b6ed0 (source of truth, confirmed).
- Scope confirmed: audit/fix `main` only; 5 unmerged feature branches left untouched.
- Delivery mode confirmed: commit to local branch `fix/production-completion`, hand owner the SHA + runbook; owner pushes & deploys (sandbox has no GitHub push / AWS / prod-DB access).

## Architecture
- `src/` Axum routes + services (auth, workspaces, contacts, campaigns, messages, agents, public_agents, analytics, webhooks, automations, custom_domains, playground). JWT HS256 access(15m)+refresh(7d) rotation. Tenant via x-workspace-id / JWT claim.
- `migrations/0001-0008` additive, idempotent (sqlx::migrate! at boot).
- `frontend/` React 19 + Vite 8, axios client with 401 refresh-rotation interceptor.
- CI `.github/workflows/ci.yml` (fmt→migrate→clippy -D warnings→test→release build). Deploy: `deploy-ec2.yml`, `deploy-portal.yml` (npm build → S3 → SSM).

## Work done (2026-10-02)
- Full audit vs live prod (API /health healthy, portal 200, redis healthy).
- Validated on real toolchain: cargo fmt ✓, migrations ✓, clippy -D warnings ✓, cargo test 5/5 ✓, cargo build --release ✓, yarn install/lint/build ✓.
- Live API e2e smoke on local server+PostgreSQL: register/login/me/refresh/logout, workspaces, dashboard, contacts create, campaigns/agents/templates/automations/playground list, unauth→401. All pass.
- **Only blocker found & fixed**: portal `yarn lint` 16 errors → 0 errors/0 warnings (unused imports/vars, empty catch blocks, react-hooks effect rules) in App.jsx + Playground.jsx. No runtime change. Backend/DB/env untouched.
- Branch `fix/production-completion`: commit 5f16b30 (lint fix) + 8fe6bb4 (runbook). HEAD = 8fe6bb4ea57df0bdabb622294228d3cb71850b05.
- Runbook: docs/PRODUCTION_COMPLETION_RUNBOOK.md (audit, validation, deploy, rollback, limitations).

## Integration honesty ledger
- Custom domain verify: REAL (Google DoH CNAME check).
- AI agent chat: REAL OpenAI-compatible call; needs AI_API_KEY (clear 400 if unset).
- WhatsApp: simulation default; real path gated on Meta creds + approval (DEFERRED).
- Redis: REAL, prod requires authenticated rediss://, PING at boot.

## Blocked verifications (no creds in sandbox)
GitHub push, AWS/EC2 deploy, prod DB connection, prod authenticated smoke. Owner executes via runbook §5.

## Backlog / next
- P0: owner push branch → green CI → merge → deploy portal; re-run authenticated prod smoke.
- P1: set AI_API_KEY in prod secret store to enable real agent completions.
- P2 (deferred): WhatsApp Cloud API live creds + WHATSAPP_SIMULATION_MODE=false.
- Consider: reconcile the 5 unmerged feature branches in a later scope.
