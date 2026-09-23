# GOLD-e GrowthOS Marketing API (Rust)

A production-grade, high-throughput Marketing API built entirely from scratch in **Rust** using **Axum**, **Tokio**, and **SQLx**, backed by **PostgreSQL**.

Built specifically to power the **GOLD-e GrowthOS Command Center** with multi-tenant workspace isolation, role-based access control (RBAC), autonomous AI agent execution with manual approval checkpoints, and Meta WhatsApp Business Cloud API integration with automatic simulation mode fallback.

---

## 🏗️ Architecture & Technology Stack

| Layer | Component | Description |
|---|---|---|
| **Language** | Rust 2021 Edition | Zero-cost abstractions, memory safety without GC |
| **HTTP Framework** | Axum 0.7 | Fast, modular web framework built on Tokio & Tower |
| **Async Runtime** | Tokio 1.38 | Multi-threaded work-stealing event loop |
| **Database & ORM** | SQLx 0.7 + PostgreSQL | Pure async driver, compile-time verified SQL queries |
| **Authentication** | JWT (HS256) | Access token (15m) + Refresh token (7d) family rotation |
| **Multi-Tenancy** | TenantContext Extractor | Header (`x-workspace-id`) or query param scoped isolation |
| **Messaging** | WhatsApp Cloud API | Meta Graph API v20.0 with automatic simulated fallback |
| **Deployment** | Docker & AWS EC2 | Multi-stage slim Docker image, systemd unit, Nginx proxy |

---

## 🚀 Quickstart

### Prerequisites
- [Rust & Cargo](https://rustup.rs/) (v1.75+)
- [PostgreSQL](https://www.postgresql.org/) (v14+)
- [SQLx CLI](https://crates.io/crates/sqlx-cli): `cargo install sqlx-cli --no-default-features --features rustls,postgres`

### 1. Environment Setup
```bash
cp .env.example .env
# Edit .env with your PostgreSQL credentials
```

### 2. Database Migrations & Seeding
```bash
# Run migrations
sqlx migrate run --source ./migrations
```

### 3. Run the Development Server
```bash
cargo run
```
The server will boot on `http://0.0.0.0:4000`.

---

## 🎯 Command Center Frontend Contract Endpoints

The API is strictly compliant with the GOLD-e GrowthOS Command Center frontend contracts:

### 1. Dashboard Overview
- **`GET /api/v1/analytics/dashboard`**
  - Headers: `Authorization: Bearer <token>`, `x-workspace-id: <workspace_id>`
  - Returns total contacts, active campaigns, sent/delivered/read counts, delivery rate %, pending approvals count, channel breakdowns, and recent agent activity.

### 2. Agent Runs
- **`GET /api/v1/agents/runs`**
  - Headers: `Authorization: Bearer <token>`, `x-workspace-id: <workspace_id>`
  - Query Params: `?page=1&limit=20&status=WAITING_APPROVAL`
  - Returns array of agent runs with their complete step sequences and metadata.

### 3. Approval Workflow Checkpoint
- **`POST /api/v1/agents/runs/:id/steps/:stepId/approve`**
  - Headers: `Authorization: Bearer <token>`, `x-workspace-id: <workspace_id>`
  - Body: `{"comment": "Approved custom 14% rebate for contractors"}`
  - Approves gatekeeper step, advances autonomous agent execution, and returns updated run.

---

## 🧪 Testing

Execute the test suite:
```bash
cargo test
```

Run static analysis & checks:
```bash
cargo check
cargo clippy
```

Compile for release:
```bash
cargo build --release
```

---

## 📦 Docker Deployment

```bash
docker compose up -d --build
```
