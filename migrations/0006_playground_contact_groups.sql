-- GOLD-e Playground, generated assets, and imported contact groups

CREATE TABLE IF NOT EXISTS contact_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_by_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, name)
);

CREATE INDEX IF NOT EXISTS idx_contact_groups_workspace
    ON contact_groups(workspace_id);

CREATE TABLE IF NOT EXISTS contact_group_members (
    group_id UUID NOT NULL REFERENCES contact_groups(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(group_id, contact_id)
);

CREATE INDEX IF NOT EXISTS idx_contact_group_members_contact
    ON contact_group_members(contact_id);

CREATE TABLE IF NOT EXISTS channel_contact_groups (
    channel_id UUID NOT NULL REFERENCES ai_agent_channel_connections(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES contact_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(channel_id, group_id)
);

CREATE TABLE IF NOT EXISTS playground_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    kind VARCHAR(50) NOT NULL,
    title VARCHAR(255) NOT NULL,
    prompt TEXT,
    content TEXT,
    media_data BYTEA,
    mime_type VARCHAR(100),
    public_key VARCHAR(80) UNIQUE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_by_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_playground_assets_workspace
    ON playground_assets(workspace_id, created_at DESC);
