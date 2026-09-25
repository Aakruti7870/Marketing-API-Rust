ALTER TABLE automations
  ADD COLUMN IF NOT EXISTS next_run_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_automations_due_schedule
  ON automations(status, trigger_type, next_run_at)
  WHERE status='ACTIVE' AND trigger_type='SCHEDULE';

UPDATE automations
SET next_run_at = NOW()
WHERE trigger_type='SCHEDULE' AND status='ACTIVE' AND next_run_at IS NULL;
