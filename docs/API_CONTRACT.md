# GOLD-e GrowthOS Marketing API — Production API Contract

**Production Base URL:** `https://api.goldetech.com`  
**API Prefix:** `/api/v1`  
**Runtime:** Rust / Axum / SQLx / PostgreSQL  
**Verified Baseline Commit:** `8c1f28c9c9d186c467d891f5fd85e536a2e08aeb`

---

## 1. Global Standards & Conventions

### 1.1 Response Format
All standard API responses follow a uniform JSON envelope:

#### Standard Success Envelope (`ApiResponse<T>`)
```json
{
  "success": true,
  "message": "Descriptive success message",
  "data": { ... }
}
```

#### Paginated Success Envelope (`PaginatedResponse<T>`)
Returned by endpoints supporting `page` and `limit` query parameters:
```json
{
  "success": true,
  "message": "Descriptive success message",
  "data": [ ... ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 54,
    "total_pages": 3,
    "has_next": true,
    "has_prev": false
  }
}
```

#### Error Envelope
All error responses across all endpoints adhere to this strict structure:
```json
{
  "success": false,
  "error": "Error description message"
}
```

### 1.2 HTTP Status Code Mappings
| Status Code | Meaning | Typical Usage |
|---|---|---|
| `200 OK` | Request succeeded | Standard reads, updates, single actions |
| `201 Created` | Resource created | Registrations, resource creation |
| `202 Accepted` | Asynchronously accepted | Asynchronous automation webhooks |
| `400 Bad Request` | Client validation failure | Malformed input, cyclic workflow edges, missing required params |
| `401 Unauthorized` | Authentication required | Missing/invalid Bearer token, invalid login credentials, invalid webhook secret |
| `403 Forbidden` | Access denied | Insufficient workspace role, cross-tenant access attempt, compromised token family |
| `404 Not Found` | Resource not found | Resource ID does not exist in the specified workspace |
| `409 Conflict` | State conflict | Duplicate email on register, approving a step not in `WAITING_APPROVAL`, run timeout |
| `422 Unprocessable Entity` | Validation error | Semantic field validation failure |
| `500 Internal Server Error` | Server error | Database or internal error (redacted in response) |
| `502 Bad Gateway` | Upstream service error | External Meta Graph API failure |

---

## 2. Authentication & Authorization Contract

### 2.1 Overview
- **Token Type:** Stateless JWT Access Token + Stateful Rotated Refresh Token.
- **Access Token Expiration:** 900 seconds (15 minutes).
- **Refresh Token Expiration:** 604,800 seconds (7 days).
- **Security Features:** Automatic Token Family Reuse & Breach Detection. If an already revoked refresh token is replayed, the entire token family is invalidated immediately.

### 2.2 Endpoints (`/api/v1/auth`)

#### `POST /api/v1/auth/register`
Creates a new user account, provisions an initial default workspace, grants the user `OWNER` membership in that workspace, and generates auth tokens.
- **Auth:** Public
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "SecurePassword123!",
    "first_name": "Krushna",
    "last_name": "Bade",
    "phone": "+919876543210",
    "workspace_name": "My Agency"
  }
  ```
  *(Note: `phone` and `workspace_name` are optional. `workspace_name` defaults to `"{first_name}'s Workspace"`)*
- **Response (`200 OK`):**
  ```json
  {
    "success": true,
    "message": "User registered successfully",
    "data": {
      "user": {
        "id": "a0000000-0000-0000-0000-000000000001",
        "email": "user@example.com",
        "first_name": "Krushna",
        "last_name": "Bade",
        "phone": "+919876543210",
        "role": "USER",
        "is_active": true
      },
      "tokens": {
        "access_token": "eyJhbGciOi...",
        "refresh_token": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "token_type": "Bearer",
        "expires_in": 900
      },
      "workspace_id": "b0000000-0000-0000-0000-000000000001"
    }
  }
  ```

#### `POST /api/v1/auth/login`
Authenticates with email and password, resolves the user primary workspace, issues a rotated refresh token and a signed access token.
- **Auth:** Public
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "email": "user@example.com",
    "password": "SecurePassword123!"
  }
  ```
- **Response (`200 OK`):** Same schema as `/api/v1/auth/register`.

#### `POST /api/v1/auth/refresh`
Rotates the refresh token and returns a fresh access token.
- **Auth:** Public
- **Headers:** `Content-Type: application/json`
- **Request Body:**
  ```json
  {
    "refresh_token": "3fa85f64-5717-4562-b3fc-2c963f66afa6"
  }
  ```
- **Response (`200 OK`):**
  ```json
  {
    "success": true,
    "message": "Tokens refreshed",
    "data": {
      "access_token": "eyJhbGciOi...",
      "refresh_token": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "token_type": "Bearer",
      "expires_in": 900
    }
  }
  ```

#### `POST /api/v1/auth/logout`
Revokes the provided refresh token session.
- **Auth:** Public / Optional
- **Headers:** `Content-Type: application/json`
- **Request Body (Optional):**
  ```json
  {
    "refresh_token": "7c9e6679-7425-40de-944b-e07fc1f90ae7"
  }
  ```
- **Response (`200 OK`):**
  ```json
  {
    "success": true,
    "message": "Logged out",
    "data": {
      "logged_out": true
    }
  }
  ```

#### `GET /api/v1/auth/me`
Retrieves the profile of the currently authenticated user.
- **Auth:** `Authorization: Bearer <access_token>`
- **Response (`200 OK`):**
  ```json
  {
    "success": true,
    "message": "User profile fetched",
    "data": {
      "id": "a0000000-0000-0000-0000-000000000001",
      "email": "user@example.com",
      "first_name": "Krushna",
      "last_name": "Bade",
      "phone": "+919876543210",
      "role": "USER",
      "is_active": true
    }
  }
  ```

---

## 3. Multi-Tenancy & Workspace Header Contract

### 3.1 Workspace Resolution Hierarchy
When accessing workspace-scoped routes, the backend resolves the tenant context (`TenantContext`) according to this strict priority order:

1. **`x-workspace-id` HTTP Header:**
   The frontend SHOULD always provide this header with the target workspace UUID on all authenticated workspace requests:
   ```http
   x-workspace-id: b0000000-0000-0000-0000-000000000001
   ```
2. **`?workspaceId=<uuid>` Query Parameter:**
   Used as fallback if the header is not present.
3. **`claims.workspace_id` in JWT:**
   Used if encoded in the JWT claims payload.
4. **Oldest Workspace Membership:**
   If none of the above are provided, the backend falls back to the user earliest joined workspace in `workspace_members`.
5. **No Workspace Found:**
   If the user does not belong to any workspace, the API returns:
   ```json
   {
     "success": false,
     "error": "No workspace found for user. Provide x-workspace-id header."
   }
   ```
   *(Status: `400 Bad Request`)*

### 3.2 Tenant Security & RBAC Enforcement
- The backend verifies that the authenticated user is an active member of the resolved `workspace_id`.
- If the user is NOT a member of that workspace, the API immediately rejects the request:
  ```json
  {
    "success": false,
    "error": "You do not belong to this workspace"
  }
  ```
  *(Status: `403 Forbidden`)*
- Global `SYSTEM_ADMIN` users are automatically elevated to `OWNER` for any workspace.
- Standard workspace roles are: `OWNER`, `ADMIN`, `MEMBER`.
- `OWNER` has full privileges across all workspace operations.

---

## 4. Workspace Management (`/api/v1/workspaces`)

| Endpoint | Method | Required Role | Description |
|---|---|---|---|
| `/api/v1/workspaces/` | `GET` | Authenticated | List all workspaces the user belongs to |
| `/api/v1/workspaces/` | `POST` | Authenticated | Create a new workspace (user becomes `OWNER`) |
| `/api/v1/workspaces/current` | `GET` | Any Member | Get details of the active workspace |
| `/api/v1/workspaces/current` | `PUT` | `OWNER`, `ADMIN` | Update name and/or description of current workspace |
| `/api/v1/workspaces/current` | `DELETE` | `OWNER` | Delete the current workspace and all associated data |
| `/api/v1/workspaces/current/members` | `GET` | Any Member | List members and their roles in current workspace |
| `/api/v1/workspaces/current/members` | `POST` | `OWNER`, `ADMIN` | Invite/add a member by email |
| `/api/v1/workspaces/current/members/:id` | `DELETE` | `OWNER`, `ADMIN` | Remove a member (Cannot remove workspace `OWNER`) |

---

## 5. Contacts Management (`/api/v1/contacts`)

| Endpoint | Method | Required Role | Description |
|---|---|---|---|
| `/api/v1/contacts/` | `GET` | Any Member | List paginated contacts (`page`, `limit`, `search`) |
| `/api/v1/contacts/` | `POST` | `OWNER`, `ADMIN`, `MEMBER` | Create a new contact |
| `/api/v1/contacts/:id` | `GET` | Any Member | Retrieve contact by ID |
| `/api/v1/contacts/:id` | `PUT` | `OWNER`, `ADMIN`, `MEMBER` | Update contact |
| `/api/v1/contacts/:id` | `DELETE` | `OWNER`, `ADMIN` | Delete contact |

#### Contact Schema
```json
{
  "id": "d0000000-0000-0000-0000-000000000001",
  "workspace_id": "b0000000-0000-0000-0000-000000000001",
  "first_name": "Vikram",
  "last_name": "Singhania",
  "email": "vikram@concreteinfratech.in",
  "phone": "+919822011223",
  "company": "Singhania Infra RMC",
  "title": "Managing Director",
  "tags": ["vip", "high-intent", "rmc-buyer"],
  "status": "ACTIVE",
  "custom_fields": {
    "monthlyDemandCuM": 3000
  },
  "last_contacted_at": "2026-09-25T14:30:00Z",
  "created_at": "2026-09-20T10:00:00Z",
  "updated_at": "2026-09-25T14:30:00Z"
}
```

---

## 6. Campaigns Management (`/api/v1/campaigns`)

| Endpoint | Method | Required Role | Description |
|---|---|---|---|
| `/api/v1/campaigns/` | `GET` | Any Member | List paginated campaigns (`page`, `limit`) |
| `/api/v1/campaigns/` | `POST` | `OWNER`, `ADMIN`, `MEMBER` | Create campaign (`DRAFT` status) |
| `/api/v1/campaigns/:id` | `GET` | Any Member | Retrieve campaign |
| `/api/v1/campaigns/:id` | `PUT` | `OWNER`, `ADMIN` | Update campaign metadata/parameters |
| `/api/v1/campaigns/:id/launch` | `POST` | `OWNER`, `ADMIN` | Launch campaign: sends messages and updates to `RUNNING` |
| `/api/v1/campaigns/:id/pause` | `POST` | `OWNER`, `ADMIN` | Pause campaign (`PAUSED` status) |

---

## 7. Messages Management (`/api/v1/messages`)

| Endpoint | Method | Required Role | Description |
|---|---|---|---|
| `/api/v1/messages/` | `GET` | Any Member | List paginated messages (`page`, `limit`, `status`) |
| `/api/v1/messages/` | `POST` | `OWNER`, `ADMIN`, `MEMBER` | Send single outbound message |
| `/api/v1/messages/:id` | `GET` | Any Member | Retrieve message by ID |

#### Send Message Request (`POST /api/v1/messages/`)
```json
{
  "contact_id": "d0000000-0000-0000-0000-000000000001",
  "content": "Special pricing update on concrete grades.",
  "template_name": "rmc_offer_v1",
  "template_data": { "discount": "10%" }
}
```

---

## 8. Agent Command Center (`/api/v1/agents`)

| Endpoint | Method | Required Role | Description |
|---|---|---|---|
| `/api/v1/agents/runs` | `GET` | Any Member | List agent runs with step breakdown |
| `/api/v1/agents/runs` | `POST` | `OWNER`, `ADMIN`, `MEMBER` | Trigger a new autonomous agent run |
| `/api/v1/agents/runs/:id` | `GET` | Any Member | Get single run with all step states |
| `/api/v1/agents/runs/:id/steps/:stepId/approve` | `POST` | `OWNER`, `ADMIN` | Approve a gatekeeper step and resume workflow |
| `/api/v1/agents/runs/:id/steps/:stepId/reject` | `POST` | `OWNER`, `ADMIN` | Reject a gatekeeper step and cancel run |

---

## 9. Analytics & Dashboard (`/api/v1/analytics`)

#### `GET /api/v1/analytics/dashboard`
Returns aggregated real-time metrics for the Command Center.
- **Auth:** Bearer + `x-workspace-id`
- **Required Role:** Any Member
- **Response Schema:**
  ```json
  {
    "success": true,
    "message": "Command Center dashboard data retrieved",
    "data": {
      "summary": {
        "total_contacts": 250,
        "active_campaigns": 3,
        "messages_sent": 120,
        "messages_delivered": 115,
        "messages_read": 80,
        "delivery_rate_percent": 96.0,
        "read_rate_percent": 70.0,
        "agent_runs_total": 14,
        "pending_approvals_count": 2
      },
      "channel_breakdown": [
        { "channel": "WHATSAPP", "sent": 120, "delivered": 115, "read": 80, "failed": 5 },
        { "channel": "EMAIL", "sent": 0, "delivered": 0, "read": 0, "failed": 0 },
        { "channel": "SMS", "sent": 0, "delivered": 0, "read": 0, "failed": 0 }
      ],
      "recent_agent_runs": [ ... ],
      "recent_activities": [ ... ]
    }
  }
  ```

---

## 10. Native Automation Engine (`/api/v1/automations`)

### 10.1 Automation Endpoints
| Endpoint | Method | Required Role | Description |
|---|---|---|---|
| `/api/v1/automations` | `GET` | Any Member | List automations in current workspace |
| `/api/v1/automations` | `POST` | `OWNER`, `ADMIN` | Create a new automation definition |
| `/api/v1/automations/:id/publish` | `POST` | `OWNER`, `ADMIN` | Publish automation (`DRAFT` -> `PUBLISHED`) |
| `/api/v1/automations/:id/run` | `POST` | `OWNER`, `ADMIN`, `MEMBER` | Trigger manual execution of published automation |
| `/api/v1/automations/runs/:run_id` | `GET` | Any Member | Retrieve run status and full step history |
| `/api/v1/automations/runs/:run_id/steps/:step_id/approve` | `POST` | `OWNER`, `ADMIN` | Approve paused step and resume DAG |

### 10.2 Workflow Definition Schema (`POST /api/v1/automations`)
```json
{
  "name": "Lead Approval & WhatsApp Outreach",
  "description": "Evaluates lead and pauses for manual approval before dispatch",
  "triggers": [
    {
      "trigger_type": "MANUAL",
      "config": {},
      "enabled": true
    }
  ],
  "nodes": [
    {
      "node_key": "start",
      "node_type": "MANUAL_TRIGGER",
      "name": "Manual Start",
      "config": {},
      "position": { "x": 100, "y": 100 }
    },
    {
      "node_key": "gatekeeper",
      "node_type": "APPROVAL",
      "name": "Executive Approval Gate",
      "config": {},
      "position": { "x": 300, "y": 100 }
    },
    {
      "node_key": "outreach",
      "node_type": "SET",
      "name": "Set Dispatch Flags",
      "config": {
        "values": { "dispatched": true }
      },
      "position": { "x": 500, "y": 100 }
    }
  ],
  "edges": [
    {
      "source_node_key": "start",
      "target_node_key": "gatekeeper",
      "config": {}
    },
    {
      "source_node_key": "gatekeeper",
      "target_node_key": "outreach",
      "config": {}
    }
  ],
  "schedule": null
}
```

### 10.3 Run Step UUID & Approval Contract
When calling `GET /api/v1/automations/runs/:run_id`, the API returns each step with its persistent database `id` (UUID):
```json
{
  "success": true,
  "message": "Automation run retrieved",
  "data": {
    "id": "e6040854-47b8-4d51-a9f2-ca6ce61665a3",
    "automation_id": "7870a000-0000-0000-0000-000000000001",
    "status": "WAITING_APPROVAL",
    "trigger_type": "MANUAL",
    "trigger_payload": { ... },
    "variables": { ... },
    "steps": [
      {
        "id": "456e4567-e89b-12d3-a456-426614174000",
        "node_key": "start",
        "node_type": "MANUAL_TRIGGER",
        "status": "COMPLETED",
        "input": {},
        "output": {},
        "error": null
      },
      {
        "id": "890e4567-e89b-12d3-a456-426614174111",
        "node_key": "gatekeeper",
        "node_type": "APPROVAL",
        "status": "WAITING_APPROVAL",
        "input": {},
        "output": {},
        "error": null
      }
    ]
  }
}
```

#### Approving an Approval Step
To approve and resume execution, the frontend **MUST** use the step `id` (UUID) returned from `steps[]`:
```http
POST /api/v1/automations/runs/e6040854-47b8-4d51-a9f2-ca6ce61665a3/steps/890e4567-e89b-12d3-a456-426614174111/approve
Authorization: Bearer <token>
x-workspace-id: <workspace_id>
```
**Response (`200 OK`):**
```json
{
  "success": true,
  "message": "Automation approval accepted and execution resumed",
  "data": {
    "run_id": "e6040854-47b8-4d51-a9f2-ca6ce61665a3",
    "approved": true
  }
}
```

---

## 11. Public Webhooks (`/api/v1/automation-webhooks` & `/api/v1/webhooks`)

### 11.1 Automation Inbound Webhook (`POST /api/v1/automation-webhooks/:id`)
Allows external systems (CRMs, payment gateways, custom forms) to trigger a published automation.
- **Path Param:** `:id` (Automation UUID)
- **Auth:** Public. If the automation trigger configuration contains a `"secret": "my-secret-key"`, the caller MUST provide it in the `x-automation-secret` header.
- **Headers:**
  - `Content-Type: application/json`
  - `x-automation-secret: <secret_value>` *(conditional)*
- **Request Body:** Any valid JSON payload.
- **Response (`202 Accepted`):**
  ```json
  {
    "accepted": true,
    "run_id": "902d5b67-e89b-12d3-a456-426614174222"
  }
  ```

### 11.2 WhatsApp Cloud API Webhook (`/api/v1/webhooks/whatsapp`)
- **Verification (`GET`):** Handles Meta webhook subscription handshake using `hub.mode`, `hub.verify_token`, and returns `hub.challenge`.
- **Event Ingestion (`POST`):** Receives delivery receipts and status updates (`SENT`, `DELIVERED`, `READ`, `FAILED`) and updates corresponding `messages` records.
