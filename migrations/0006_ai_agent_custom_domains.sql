-- GOLD-e custom domain / DNS deployment layer
CREATE TABLE IF NOT EXISTS ai_agent_custom_domains (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    hostname VARCHAR(255) NOT NULL UNIQUE,
    status VARCHAR(32) NOT NULL DEFAULT 'PENDING_DNS',
    dns_target VARCHAR(255) NOT NULL,
    cloudflare_hostname_id VARCHAR(255),
    last_dns_status VARCHAR(64),
    last_ssl_status VARCHAR(64),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_custom_domains_agent
    ON ai_agent_custom_domains(agent_id, status);

CREATE INDEX IF NOT EXISTS idx_ai_agent_custom_domains_workspace
    ON ai_agent_custom_domains(workspace_id, status);
