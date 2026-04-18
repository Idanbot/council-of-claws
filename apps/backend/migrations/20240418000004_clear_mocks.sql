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
