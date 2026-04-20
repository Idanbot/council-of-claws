CREATE TABLE IF NOT EXISTS openclaw_snapshots (
    id TEXT PRIMARY KEY,
    schema_version INTEGER NOT NULL,
    snapshot_fingerprint TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    source_path TEXT NOT NULL,
    config_path TEXT NOT NULL,
    source_mtime TIMESTAMPTZ,
    generated_at TIMESTAMPTZ NOT NULL,
    last_success_at TIMESTAMPTZ NOT NULL,
    provider_count INTEGER NOT NULL,
    configured_agent_count INTEGER NOT NULL,
    available_model_ref_count INTEGER NOT NULL,
    invalid_model_ref_count INTEGER NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_openclaw_snapshots_generated_at
    ON openclaw_snapshots (generated_at DESC);
