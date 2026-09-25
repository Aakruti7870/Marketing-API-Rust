# GOLD-e GrowthOS — Frontend API Integration Guide

**Production API URL:** `https://api.goldetech.com`  
**API Prefix:** `/api/v1`  
**Backend Baseline:** `8c1f28c9c9d186c467d891f5fd85e536a2e08aeb`  
**API Contract:** `docs/API_CONTRACT.md` (`f8261773b2f56196de24d6e1d6977111b117cdd3`)  
**Frontend Repository Branch:** `ui/golde-growthos-portal` (`frontend/`)

---

## 1. Frontend Architecture & Locations

The GOLD-e platform consists of two primary frontend clients:
1. **Web Admin / Command Center (React 19 + Vite):**
   - **Location:** `Marketing-API-Rust` on branch `ui/golde-growthos-portal` under `frontend/`.
   - **Build Target:** Served statically by Nginx at `https://goldetech.com` and `https://www.goldetech.com` via `/opt/golde-portal`.
   - **Central API Client:** `frontend/src/services/api.js`.
2. **Mobile Companion Application (Expo / React Native):**
   - **Location:** `Aakruti7870/Tracking-project` (`TrackMyRMC`).
   - **Build Target:** Android/iOS native applications and Expo Web.
   - **Central API Client:** `frontend/src/api/client.ts`.

---

## 2. Environment Configuration

### 2.1 Web Portal (Vite)
Environment variables are defined in `.env.production` or passed at build time:
```bash
# Production API endpoint
VITE_API_BASE_URL=https://api.goldetech.com/api/v1
```

### 2.2 Mobile Client (Expo)
Environment variables are prefixed with `EXPO_PUBLIC_`:
```bash
# Production backend base URL
EXPO_PUBLIC_BACKEND_URL=https://api.goldetech.com
```

---

## 3. Centralized API Client Implementation

The production API client (`frontend/src/services/api.js`) provides:
- **Base URL management:** defaults to `https://api.goldetech.com/api/v1`.
- **Automatic header injection:** `Authorization: Bearer <token>` and `x-workspace-id: <uuid>`.
- **Automatic 401 token refresh rotation:** intercepts expired tokens, issues a refresh request, updates storage, and transparently retries queued failed requests.
- **Request timeouts:** 30-second bounded timeout.
- **Standardized error extraction:** unwrap helper for backend `{ success, message, data }` envelopes.

---

## 4. Feature Mapping Matrix (Frontend to API)

| Frontend Feature | API Endpoint | Implemented? | Correct? | Missing / Notes |
|---|---|---|---|---|
| **AUTH: Register** | `POST /api/v1/auth/register` | Yes | Yes | Stores tokens & user in `localStorage` |
| **AUTH: Login** | `POST /api/v1/auth/login` | Yes | Yes | Stores tokens, user, & initial workspace |
| **AUTH: Refresh** | `POST /api/v1/auth/refresh` | Yes | Yes | Auto-triggered on 401 response |
| **AUTH: Logout** | `POST /api/v1/auth/logout` | Yes | Yes | Revokes refresh token session |
| **AUTH: Profile (Me)** | `GET /api/v1/auth/me` | Yes | Yes | Hydrates current session on app boot |
| **WORKSPACE: List** | `GET /api/v1/workspaces/` | Yes | Yes | Lists user joined workspaces |
| **WORKSPACE: Current** | `GET /api/v1/workspaces/current` | Yes | Yes | Requires `x-workspace-id` header |
| **WORKSPACE: Members** | `GET /api/v1/workspaces/current/members` | Yes | Yes | Returns member roster with roles |
| **CONTACTS: List** | `GET /api/v1/contacts/` | Yes | Yes | Supports `page`, `limit`, `search` |
| **CONTACTS: Create** | `POST /api/v1/contacts/` | Yes | Yes | Full custom fields and tag support |
| **CONTACTS: Update** | `PUT /api/v1/contacts/:id` | Yes | Yes | Fully supported in `api.js` |
| **CONTACTS: Delete** | `DELETE /api/v1/contacts/:id` | Yes | Yes | Requires `OWNER` or `ADMIN` role |
| **CONTACTS: Get** | `GET /api/v1/contacts/:id` | Yes | Yes | Single contact fetch |
| **CAMPAIGNS: List** | `GET /api/v1/campaigns/` | Yes | Yes | Paginated campaign listing |
| **CAMPAIGNS: Create** | `POST /api/v1/campaigns/` | Yes | Yes | Creates campaign in `DRAFT` status |
| **CAMPAIGNS: Update** | `PUT /api/v1/campaigns/:id` | Yes | Yes | Edit target tags and templates |
| **CAMPAIGNS: Get** | `GET /api/v1/campaigns/:id` | Yes | Yes | Single campaign details |
| **CAMPAIGNS: Launch** | `POST /api/v1/campaigns/:id/launch` | Yes | Yes | Dispatches messages; sets `RUNNING` |
| **CAMPAIGNS: Pause** | `POST /api/v1/campaigns/:id/pause` | Yes | Yes | Suspends campaign; sets `PAUSED` |
| **MESSAGES: List** | `GET /api/v1/messages/` | Yes | Yes | Paginated message logs |
| **MESSAGES: Send** | `POST /api/v1/messages/` | Yes | Yes | Dispatches outbound WhatsApp message |
| **MESSAGES: Get** | `GET /api/v1/messages/:id` | Yes | Yes | Single message status and payload |
| **AGENTS: Runs** | `GET /api/v1/agents/runs` | Yes | Yes | Paginated runs with step arrays |
| **AGENTS: Get Run** | `GET /api/v1/agents/runs/:id` | Yes | Yes | Step status and approval metadata |
| **AGENTS: Approve** | `POST /api/v1/agents/runs/:id/steps/:stepId/approve` | Yes | Yes | Resumes and completes pipeline |
| **AGENTS: Reject** | `POST /api/v1/agents/runs/:id/steps/:stepId/reject` | Yes | Yes | Cancels run with logged reason |
| **ANALYTICS: Dashboard** | `GET /api/v1/analytics/dashboard` | Yes | Yes | Live metrics, activity stream, channels |
| **AUTOMATIONS: List** | `GET /api/v1/automations` | Yes | Yes | Lists workflow definitions |
| **AUTOMATIONS: Create** | `POST /api/v1/automations` | Yes | Yes | Submits DAG nodes and edges |
| **AUTOMATIONS: Publish** | `POST /api/v1/automations/:id/publish` | Yes | Yes | Activates workflow |
| **AUTOMATIONS: Run** | `POST /api/v1/automations/:id/run` | Yes | Yes | Triggers manual run |
| **AUTOMATIONS: Get Run** | `GET /api/v1/automations/runs/:run_id` | Yes | Yes | Returns steps with persistent UUIDs |
| **AUTOMATIONS: Approve Resume** | `POST /api/v1/automations/runs/:run_id/steps/:step_id/approve` | Yes | Yes | Resumes paused DAG from real step UUID |
| **WEBHOOKS: WhatsApp** | `GET/POST /api/v1/webhooks/whatsapp` | Yes | Yes | Meta handshake and status updates |
| **WEBHOOKS: Automation** | `POST /api/v1/automation-webhooks/:id` | Yes | Yes | Public webhook with optional secret |

---

## 5. Multi-Tenancy & Workspace State Management

All workspace-dependent requests require tenant context:
```http
x-workspace-id: <workspace UUID>
```
1. **On Login / Workspace Switch:**
   - The active workspace UUID is stored in `localStorage.getItem("golde_workspace_id")` (or `AsyncStorage`/SecureStore in React Native).
   - The Axios request interceptor automatically pulls this value and attaches the header to every outgoing request.
2. **Multi-Workspace Navigation:**
   - When a user selects a different workspace in the UI header dropdown, update `localStorage.setItem("golde_workspace_id", newWorkspaceId)`.
   - All subsequent API calls immediately reflect the new workspace context.
   - If `x-workspace-id` is omitted, the API defaults to the user's oldest joined workspace or returns `400 Bad Request` if no membership exists.

---

## 6. Automation Engine Approval Resume Flow

The frontend strictly adheres to the following sequence for approval nodes:

```text
[ Trigger Automation ]
        ↓
[ Poll GET /api/v1/automations/runs/{run_id} ]
        ↓
[ Detect run.status == "WAITING_APPROVAL" ]
        ↓
[ Locate step where node_type == "APPROVAL" and status == "WAITING_APPROVAL" ]
        ↓
[ Extract real step UUID: step.id ]
        ↓
[ POST /api/v1/automations/runs/{run_id}/steps/{step.id}/approve ]
        ↓
[ Verify response: { success: true, data: { run_id, approved: true } } ]
        ↓
[ Poll GET /api/v1/automations/runs/{run_id} until status == "COMPLETED" ]
```

**Critical Rule:** The frontend must NEVER fabricate, guess, or use `node_key` strings (e.g., `"approval"`) as the `step_id`. It must always use the persistent UUID returned in `step.id` from `GET /api/v1/automations/runs/:run_id`.

---

## 7. Error Handling & Status Code Mapping

The backend returns a uniform JSON error envelope:
```json
{
  "success": false,
  "error": "Descriptive message"
}
```

The frontend error interceptor maps status codes as follows:
- `400 Bad Request`: Display form/validation error banner (e.g., "Invalid input parameters").
- `401 Unauthorized`: Handled automatically by token refresh interceptor. If refresh fails, redirect to `/login`.
- `403 Forbidden`: Display permission alert (e.g., "Insufficient workspace permissions or non-member access").
- `404 Not Found`: Display "Resource not found" empty state.
- `409 Conflict`: Display workflow state alert (e.g., "Workflow step is no longer pending approval").
- `422 Unprocessable Entity`: Field-level validation error highlights.
- `500 Internal Server Error`: Display user-friendly notification: "Server error occurred. Please try again later." (Never display raw backend stack traces or database errors).
- `502 Bad Gateway`: Display integration alert: "External messaging service unavailable. Please retry."

---

## 8. End-to-End Production Verification Flow

The production API was verified with the following end-to-end integration lifecycle:
1. `POST /api/v1/auth/login` → Authenticated and retrieved tokens and default workspace.
2. `GET /api/v1/auth/me` → Verified user profile.
3. `GET /api/v1/workspaces/current` (with `x-workspace-id`) → Confirmed active workspace context.
4. `GET /api/v1/analytics/dashboard` → Loaded Command Center metrics.
5. `GET /api/v1/contacts/` & `GET /api/v1/campaigns/` → Populated workspace data tables.
6. `POST /api/v1/automations` → Created workflow definition with `APPROVAL` and `SET` nodes.
7. `POST /api/v1/automations/:id/publish` → Published automation.
8. `POST /api/v1/automations/:id/run` → Triggered execution.
9. `GET /api/v1/automations/runs/:run_id` → Verified `status: WAITING_APPROVAL` and extracted `steps[1].id` UUID.
10. `POST /api/v1/automations/runs/:run_id/steps/:step_id/approve` → Dispatched approval with real UUID.
11. `GET /api/v1/automations/runs/:run_id` → Confirmed final status `COMPLETED`.
