CREATE TABLE IF NOT EXISTS automations (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(160) NOT NULL,
    description TEXT,
    status VARCHAR(32) NOT NULL DEFAULT 'DRAFT',
    trigger_type VARCHAR(64) NOT NULL,
    trigger_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    webhook_token VARCHAR(96) UNIQUE,
    graph JSONB NOT NULL DEFAULT '{"nodes":[],"edges":[]}'::jsonb,
    version INTEGER NOT NULL DEFAULT 1,
    created_by_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_automations_workspace ON automations(workspace_id);
CREATE INDEX IF NOT EXISTS idx_automations_webhook ON automations(webhook_token);
CREATE INDEX IF NOT EXISTS idx_automations_status ON automations(workspace_id,status);

CREATE TABLE IF NOT EXISTS automation_runs (
    id UUID PRIMARY KEY,
    automation_id UUID NOT NULL REFERENCES automations(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    status VARCHAR(32) NOT NULL DEFAULT 'RUNNING',
    trigger_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    output_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_automation_runs_workspace ON automation_runs(workspace_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_automation_runs_automation ON automation_runs(automation_id, started_at DESC);

CREATE TABLE IF NOT EXISTS automation_run_steps (
    id UUID PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES automation_runs(id) ON DELETE CASCADE,
    node_id VARCHAR(128) NOT NULL,
    node_type VARCHAR(64) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'RUNNING',
    input_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    output_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_automation_run_steps_run ON automation_run_steps(run_id, started_at);
