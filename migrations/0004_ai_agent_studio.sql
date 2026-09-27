-- GOLD-e AI Agent Studio
-- Workspace-scoped agent definitions and durable conversations.

CREATE TABLE IF NOT EXISTS ai_agents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    template_key VARCHAR(100) NOT NULL,
    industry VARCHAR(100) NOT NULL,
    role VARCHAR(150) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'DRAFT',
    system_prompt TEXT NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    tools JSONB NOT NULL DEFAULT '[]'::jsonb,
    channels JSONB NOT NULL DEFAULT '["WHATSAPP","WEB_CHAT"]'::jsonb,
    settings JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_agents_workspace ON ai_agents(workspace_id);
CREATE INDEX IF NOT EXISTS idx_ai_agents_workspace_status ON ai_agents(workspace_id, status);
CREATE INDEX IF NOT EXISTS idx_ai_agents_workspace_template ON ai_agents(workspace_id, template_key);

CREATE TABLE IF NOT EXISTS ai_agent_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES ai_agents(id) ON DELETE CASCADE,
    contact_id UUID REFERENCES contacts(id) ON DELETE SET NULL,
    channel VARCHAR(50) NOT NULL DEFAULT 'WEB_CHAT',
    external_user_id VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'ACTIVE',
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_conversations_workspace ON ai_agent_conversations(workspace_id);
CREATE INDEX IF NOT EXISTS idx_ai_agent_conversations_agent ON ai_agent_conversations(agent_id);
CREATE INDEX IF NOT EXISTS idx_ai_agent_conversations_external ON ai_agent_conversations(agent_id, external_user_id);

CREATE TABLE IF NOT EXISTS ai_agent_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES ai_agent_conversations(id) ON DELETE CASCADE,
    role VARCHAR(30) NOT NULL,
    content TEXT NOT NULL,
    tool_name VARCHAR(150),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_ai_agent_messages_conversation ON ai_agent_messages(conversation_id, created_at);
