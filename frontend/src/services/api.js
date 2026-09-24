import axios from "axios";

const api = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || "/api/v1",
  headers: { "Content-Type": "application/json" },
});

api.interceptors.request.use((config) => {
  const token = localStorage.getItem("golde_access_token");
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

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
  current: () => api.get("/workspaces/current"),
  members: () => api.get("/workspaces/current/members"),
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
};

export const messagesApi = {
  list: (params = {}) => api.get("/messages", { params }),
  send: (payload) => api.post("/messages", payload),
};

export default api;


export const automationsApi = {
  list: () => api.get("/automations"),
  get: (id) => api.get(`/automations/${id}`),
  create: (payload) => api.post("/automations", payload),
  update: (id, payload) => api.put(`/automations/${id}`, payload),
  activate: (id) => api.post(`/automations/${id}/activate`),
  pause: (id) => api.post(`/automations/${id}/pause`),
  run: (id, payload = {}) => api.post(`/automations/${id}/run`, payload),
  runs: (id) => api.get(`/automations/${id}/runs`),
  runSteps: (id, runId) => api.get(`/automations/${id}/runs/${runId}/steps`),
};
