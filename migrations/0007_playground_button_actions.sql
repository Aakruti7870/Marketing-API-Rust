-- Interactive Smart WhatsApp button actions

CREATE TABLE IF NOT EXISTS playground_button_actions (
    button_id VARCHAR(120) PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    asset_id UUID NOT NULL REFERENCES playground_assets(id) ON DELETE CASCADE,
    agent_id UUID REFERENCES ai_agents(id) ON DELETE SET NULL,
    title VARCHAR(80) NOT NULL,
    action_type VARCHAR(30) NOT NULL DEFAULT 'BOT_REPLY',
    action_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_playground_button_actions_asset
    ON playground_button_actions(asset_id);

CREATE INDEX IF NOT EXISTS idx_playground_button_actions_workspace
    ON playground_button_actions(workspace_id);
