-- Combined bootstrap schema for docker-compose postgres init
-- Generated from apps/backend/migrations in order

-- Initial Schema for Council of Claws (Clean)

CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    priority TEXT NOT NULL,
    status TEXT NOT NULL,
    owner_agent TEXT NOT NULL,
    mission_id TEXT,
    created_by_agent TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    blocked_reason TEXT
);

CREATE TABLE IF NOT EXISTS missions (
    id TEXT PRIMARY KEY,
    root_task_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL,
    created_by_agent TEXT NOT NULL,
    closed_by_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    closed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_tasks_mission_id ON tasks(mission_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

CREATE TABLE IF NOT EXISTS task_events (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    summary TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_runs (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    model_name TEXT NOT NULL,
    status TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    ended_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS model_usage (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    model_name TEXT NOT NULL,
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    estimated_cost_usd DOUBLE PRECISION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS council_runs (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    phase TEXT NOT NULL,
    director_agent TEXT NOT NULL,
    ruling_summary TEXT,
    confidence DOUBLE PRECISION,
    obsidian_path TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS council_rounds (
    id TEXT PRIMARY KEY,
    council_id TEXT NOT NULL REFERENCES council_runs(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL,
    round_type TEXT NOT NULL,
    summary TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

-- Agent Identity and Audit Log (Clean)

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    role TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    scope_profile TEXT,
    secret_id TEXT,
    secret_hash TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    request_id TEXT,
    agent_id TEXT,
    operation TEXT NOT NULL,
    resource_type TEXT,
    resource_id TEXT,
    allowed BOOLEAN NOT NULL,
    result TEXT,
    reason TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Essential Real Agent Definitions (No hashes yet, using legacy fallback for smoke tests)
INSERT INTO agents (id, display_name, role, secret_hash) VALUES
('director', 'Director Agent', 'director', NULL),
('contractor', 'Contractor Agent', 'contractor', NULL),
('architect', 'Architect Agent', 'architect', NULL),
('senior-engineer', 'Senior Engineer Agent', 'senior-engineer', NULL),
('junior-engineer', 'Junior Engineer Agent', 'junior-engineer', NULL),
('intern', 'Intern Agent', 'intern', NULL)
ON CONFLICT (id) DO UPDATE SET
    display_name = EXCLUDED.display_name,
    role = EXCLUDED.role;

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

-- Clean Real Schema (No Seeds)
-- This ensures zero tokens and zero tasks on fresh start

DELETE FROM tasks;
DELETE FROM missions;
DELETE FROM model_usage;
DELETE FROM agent_runs;
DELETE FROM task_events;
DELETE FROM audit_events;
DELETE FROM council_votes;
DELETE FROM council_rounds;
DELETE FROM council_runs;

-- Only keep the core agent identities with no secrets (fallback mode)
INSERT INTO agents (id, display_name, role, enabled, scope_profile) VALUES
('director', 'Director', 'director', true, '{"allow_task_create": true, "allow_mission_close": true, "allow_council_finalize": true}'),
('contractor', 'Contractor', 'contractor', true, '{"allow_task_create": true}'),
('architect', 'Architect', 'architect', true, '{"allow_council_propose": true}'),
('senior-engineer', 'Senior Engineer', 'senior-engineer', true, '{"allow_task_claim": true}'),
('junior-engineer', 'Junior Engineer', 'junior-engineer', true, '{"allow_task_claim": true}'),
('intern', 'Intern', 'intern', true, '{"allow_task_claim": true}')
ON CONFLICT (id) DO UPDATE SET
    enabled = EXCLUDED.enabled,
    scope_profile = EXCLUDED.scope_profile;
