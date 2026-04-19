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
			providers: [{ provider: 'openai', configured: true, via: 'OPENAI_API_KEY' }],
			configured_agents: []
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
			gateway: { status: 'healthy', message: 'reachable' },
			providers: [],
			backend_log_tail: ['line-a', 'line-b'],
			notes: []
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
