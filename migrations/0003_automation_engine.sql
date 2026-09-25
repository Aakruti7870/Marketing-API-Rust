-- GOLD-e native automation engine
CREATE TABLE IF NOT EXISTS automations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(30) NOT NULL DEFAULT 'DRAFT',
    version INT NOT NULL DEFAULT 1,
    created_by_id UUID NOT NULL REFERENCES users(id),
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_automations_workspace ON automations(workspace_id);

CREATE TABLE IF NOT EXISTS automation_triggers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    automation_id UUID NOT NULL REFERENCES automations(id) ON DELETE CASCADE,
    trigger_type VARCHAR(40) NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_automation_triggers_automation ON automation_triggers(automation_id);
CREATE INDEX IF NOT EXISTS idx_automation_triggers_type_enabled ON automation_triggers(trigger_type, enabled);

CREATE TABLE IF NOT EXISTS automation_nodes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    automation_id UUID NOT NULL REFERENCES automations(id) ON DELETE CASCADE,
    node_key VARCHAR(100) NOT NULL,
    node_type VARCHAR(60) NOT NULL,
    name VARCHAR(255) NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    position JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_automation_node_key UNIQUE (automation_id, node_key)
);
CREATE INDEX IF NOT EXISTS idx_automation_nodes_automation ON automation_nodes(automation_id);

CREATE TABLE IF NOT EXISTS automation_edges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    automation_id UUID NOT NULL REFERENCES automations(id) ON DELETE CASCADE,
    source_node_key VARCHAR(100) NOT NULL,
    target_node_key VARCHAR(100) NOT NULL,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_automation_edge UNIQUE (automation_id, source_node_key, target_node_key)
);
CREATE INDEX IF NOT EXISTS idx_automation_edges_source ON automation_edges(automation_id, source_node_key);

CREATE TABLE IF NOT EXISTS automation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    automation_id UUID NOT NULL REFERENCES automations(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    status VARCHAR(40) NOT NULL DEFAULT 'RUNNING',
    trigger_type VARCHAR(40),
    trigger_payload JSONB NOT NULL DEFAULT '{}'::jsonb,
    variables JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_automation_runs_workspace ON automation_runs(workspace_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_automation_runs_automation ON automation_runs(automation_id, started_at DESC);

CREATE TABLE IF NOT EXISTS automation_run_steps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES automation_runs(id) ON DELETE CASCADE,
    node_key VARCHAR(100) NOT NULL,
    node_type VARCHAR(60) NOT NULL,
    status VARCHAR(40) NOT NULL DEFAULT 'RUNNING',
    input JSONB NOT NULL DEFAULT '{}'::jsonb,
    output JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT,
    attempt INT NOT NULL DEFAULT 1,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_automation_run_steps_run ON automation_run_steps(run_id, started_at);

CREATE TABLE IF NOT EXISTS automation_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    automation_id UUID NOT NULL UNIQUE REFERENCES automations(id) ON DELETE CASCADE,
    interval_seconds BIGINT NOT NULL CHECK (interval_seconds >= 15),
    next_run_at TIMESTAMPTZ,
    enabled BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_automation_schedules_due ON automation_schedules(enabled, next_run_at);

CREATE TABLE IF NOT EXISTS automation_logs (
    id BIGSERIAL PRIMARY KEY,
    run_id UUID NOT NULL REFERENCES automation_runs(id) ON DELETE CASCADE,
    node_key VARCHAR(100),
    level VARCHAR(20) NOT NULL DEFAULT 'INFO',
    message TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_automation_logs_run ON automation_logs(run_id, created_at);

CREATE TABLE IF NOT EXISTS automation_variables (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    automation_id UUID REFERENCES automations(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL DEFAULT 'null'::jsonb,
    is_secret BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_automation_variable UNIQUE (workspace_id, automation_id, key)
);
