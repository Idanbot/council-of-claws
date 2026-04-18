-- Batch 5: Council Votes and Fine-Grained RBAC

-- 1. Create council_votes table
CREATE TABLE IF NOT EXISTS council_votes (
    id TEXT PRIMARY KEY,
    council_id TEXT NOT NULL REFERENCES council_runs(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES agents(id),
    vote TEXT NOT NULL, -- 'approve', 'reject', 'abstain'
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. Update agents table to ensure scope_profile is used
-- (Column already exists but we ensure it's JSONB for granular control)
ALTER TABLE agents ALTER COLUMN scope_profile TYPE JSONB USING scope_profile::JSONB;

-- 3. Seed some default scope profiles for existing agents
UPDATE agents SET scope_profile = '{"allow_task_create": true, "allow_mission_close": true, "allow_council_finalize": true}' WHERE id = 'director';
UPDATE agents SET scope_profile = '{"allow_task_create": true, "allow_council_propose": true}' WHERE id IN ('architect', 'senior-engineer');
UPDATE agents SET scope_profile = '{"allow_task_claim": true}' WHERE id IN ('junior-engineer', 'intern');
