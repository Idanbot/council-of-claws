import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import SystemPage from '../routes/system/+page.svelte';

vi.mock('$lib/api', () => ({
	getDiagnosticsReport: vi.fn(async () => ({
		data: {
			generated_at: new Date().toISOString(),
			overall_status: 'healthy',
			checks: [{ name: 'backend', status: 'ok', info: 'Backend router active', duration_ms: 3 }]
		}
	})),
	getModelsStatus: vi.fn(async () => ({
		data: {
			generated_at: new Date().toISOString(),
			snapshot: {
				schema_version: 1,
				snapshot_fingerprint: 'abc123def456',
				status: 'healthy',
				generated_at: new Date().toISOString(),
				last_success_at: new Date().toISOString(),
				source_mtime: new Date().toISOString(),
				snapshot_age_seconds: 4
			},
			providers: [{
				provider: 'openai',
				enabled: true,
				configured: true,
				discovered: false,
				status: 'healthy',
				via: 'config-plugin',
				base_url: null,
				model_count: 0,
				available_models: [],
				configured_model_refs: ['openai/gpt-5.4'],
				auth_profiles: [],
				issues: []
			}],
			configured_agents: [],
			available_model_refs: ['codex/gpt-5.4'],
			invalid_model_refs: [],
			issues: []
		}
	})),
	getAgentsStatus: vi.fn(async () => ({
		data: {
			generated_at: new Date().toISOString(),
			heartbeat_ttl_seconds: 120,
			configured_count: 6,
			live_count: 0,
			stale_count: 0,
			agents: []
		}
	})),
	getAdminRuntimeStatus: vi.fn(async () => ({
		data: {
			generated_at: new Date().toISOString(),
			snapshot: {
				schema_version: 1,
				snapshot_fingerprint: 'abc123def456',
				status: 'healthy',
				generated_at: new Date().toISOString(),
				last_success_at: new Date().toISOString(),
				source_mtime: new Date().toISOString(),
				snapshot_age_seconds: 4
			},
			history: {
				snapshot_count: 2,
				latest_generated_at: new Date().toISOString(),
				latest_persisted_at: new Date().toISOString(),
				latest_snapshot_fingerprint: 'abc123def456'
			},
			gateway: { status: 'healthy', message: 'reachable' },
			providers: [],
			backend_log_tail: ['line-a', 'line-b'],
			notes: [],
			openclaw_source_path: '/app/data/openclaw',
			runtime_state_available: true,
			issues: []
		}
	}))
}));

describe('System page', () => {
	it('shows diagnostics log and runtime sections', async () => {
		render(SystemPage);

		await waitFor(() => {
		expect(screen.getByText('Diagnostics Run Log')).toBeInTheDocument();
		});

		expect(screen.getByText('Providers & Models')).toBeInTheDocument();
		expect(screen.getByText('Runtime Logs')).toBeInTheDocument();
		expect(screen.getByText(/line-a/)).toBeInTheDocument();
	});
});
