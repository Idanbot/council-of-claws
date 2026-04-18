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
