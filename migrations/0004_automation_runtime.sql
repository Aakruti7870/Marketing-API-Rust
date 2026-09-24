ALTER TABLE automations
    ADD COLUMN IF NOT EXISTS schedule_interval_seconds BIGINT,
    ADD COLUMN IF NOT EXISTS next_run_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_run_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_automations_schedule_due
    ON automations(status, trigger_type, next_run_at)
    WHERE status = 'ACTIVE' AND trigger_type = 'SCHEDULE';

CREATE TABLE IF NOT EXISTS automation_templates (
    id UUID PRIMARY KEY,
    workspace_id UUID REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(160) NOT NULL,
    description TEXT,
    category VARCHAR(80) NOT NULL DEFAULT 'general',
    graph JSONB NOT NULL,
    trigger_type VARCHAR(64) NOT NULL DEFAULT 'MANUAL',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_automation_templates_workspace
    ON automation_templates(workspace_id, created_at DESC);