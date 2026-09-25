import axios from "axios";

// Standard production base URL fallback
const DEFAULT_BASE_URL = "https://api.goldetech.com/api/v1";

const api = axios.create({
  baseURL: import.meta.env?.VITE_API_BASE_URL || DEFAULT_BASE_URL,
  headers: { "Content-Type": "application/json" },
  timeout: 30000,
});

// Request interceptor: injects Authorization and x-workspace-id
api.interceptors.request.use((config) => {
  const token = localStorage.getItem("golde_access_token");
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }

  const workspaceId = localStorage.getItem("golde_workspace_id");
  if (workspaceId) {
    config.headers["x-workspace-id"] = workspaceId;
  }

  return config;
});

// Response interceptor: handles automatic 401 token refresh rotation
let isRefreshing = false;
let failedQueue = [];

const processQueue = (error, token = null) => {
  failedQueue.forEach((prom) => {
    if (error) {
      prom.reject(error);
    } else {
      prom.resolve(token);
    }
  });
  failedQueue = [];
};

api.interceptors.response.use(
  (response) => response,
  async (error) => {
    const originalRequest = error.config;

    // Do not attempt refresh on auth endpoints (login, register, refresh)
    if (
      error.response?.status === 401 &&
      !originalRequest._retry &&
      !originalRequest.url.includes("/auth/login") &&
      !originalRequest.url.includes("/auth/register") &&
      !originalRequest.url.includes("/auth/refresh")
    ) {
      if (isRefreshing) {
        return new Promise((resolve, reject) => {
          failedQueue.push({ resolve, reject });
        })
          .then((token) => {
            originalRequest.headers.Authorization = `Bearer ${token}`;
            return api(originalRequest);
          })
          .catch((err) => Promise.reject(err));
      }

      originalRequest._retry = true;
      isRefreshing = true;

      const refreshToken = localStorage.getItem("golde_refresh_token");
      if (!refreshToken) {
        isRefreshing = false;
        clearAuth();
        return Promise.reject(error);
      }

      try {
        const refreshResponse = await axios.post(
          `${api.defaults.baseURL}/auth/refresh`,
          { refresh_token: refreshToken },
          { headers: { "Content-Type": "application/json" } }
        );

        const newTokens = refreshResponse.data?.data;
        if (newTokens?.access_token) {
          localStorage.setItem("golde_access_token", newTokens.access_token);
          if (newTokens.refresh_token) {
            localStorage.setItem("golde_refresh_token", newTokens.refresh_token);
          }

          api.defaults.headers.common["Authorization"] = `Bearer ${newTokens.access_token}`;
          originalRequest.headers["Authorization"] = `Bearer ${newTokens.access_token}`;

          processQueue(null, newTokens.access_token);
          return api(originalRequest);
        } else {
          throw new Error("No access token returned from refresh");
        }
      } catch (refreshErr) {
        processQueue(refreshErr, null);
        clearAuth();
        return Promise.reject(refreshErr);
      } finally {
        isRefreshing = false;
      }
    }

    return Promise.reject(error);
  }
);

function clearAuth() {
  localStorage.removeItem("golde_access_token");
  localStorage.removeItem("golde_refresh_token");
  localStorage.removeItem("golde_user");
  localStorage.removeItem("golde_workspace_id");
}

export const unwrap = (response) => response?.data?.data ?? response?.data;

export const authApi = {
  login: (payload) => api.post("/auth/login", payload),
  register: (payload) => api.post("/auth/register", payload),
  me: () => api.get("/auth/me"),
  refresh: (refreshToken) => api.post("/auth/refresh", { refresh_token: refreshToken }),
  logout: (refreshToken) => api.post("/auth/logout", refreshToken ? { refresh_token: refreshToken } : {}),
};

export const workspaceApi = {
  list: () => api.get("/workspaces"),
  create: (payload) => api.post("/workspaces", payload),
  current: () => api.get("/workspaces/current"),
  updateCurrent: (payload) => api.put("/workspaces/current", payload),
  deleteCurrent: () => api.delete("/workspaces/current"),
  members: () => api.get("/workspaces/current/members"),
  addMember: (payload) => api.post("/workspaces/current/members", payload),
  removeMember: (memberId) => api.delete(`/workspaces/current/members/${memberId}`),
};

export const dashboardApi = {
  get: () => api.get("/analytics/dashboard"),
};

export const agentsApi = {
  runs: (params = {}) => api.get("/agents/runs", { params }),
  run: (id) => api.get(`/agents/runs/${id}`),
  create: (payload) => api.post("/agents/runs", payload),
  approve: (runId, stepId, payload = {}) => api.post(`/agents/runs/${runId}/steps/${stepId}/approve`, payload),
  reject: (runId, stepId, payload) => api.post(`/agents/runs/${runId}/steps/${stepId}/reject`, payload),
};

export const campaignsApi = {
  list: (params = {}) => api.get("/campaigns", { params }),
  create: (payload) => api.post("/campaigns", payload),
  get: (id) => api.get(`/campaigns/${id}`),
  update: (id, payload) => api.put(`/campaigns/${id}`, payload),
  launch: (id) => api.post(`/campaigns/${id}/launch`),
  pause: (id) => api.post(`/campaigns/${id}/pause`),
};

export const contactsApi = {
  list: (params = {}) => api.get("/contacts", { params }),
  create: (payload) => api.post("/contacts", payload),
  get: (id) => api.get(`/contacts/${id}`),
  update: (id, payload) => api.put(`/contacts/${id}`, payload),
  delete: (id) => api.delete(`/contacts/${id}`),
};

export const messagesApi = {
  list: (params = {}) => api.get("/messages", { params }),
  send: (payload) => api.post("/messages", payload),
  get: (id) => api.get(`/messages/${id}`),
};

export const automationsApi = {
  list: () => api.get("/automations"),
  create: (payload) => api.post("/automations", payload),
  publish: (id) => api.post(`/automations/${id}/publish`),
  run: (id, payload = {}) => api.post(`/automations/${id}/run`, payload),
  getRun: (runId) => api.get(`/automations/runs/${runId}`),
  approveStep: (runId, stepId) => api.post(`/automations/runs/${runId}/steps/${stepId}/approve`),
};

export default api;
