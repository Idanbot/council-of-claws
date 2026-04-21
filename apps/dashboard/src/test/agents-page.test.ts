import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import AgentsPage from '../routes/agents/+page.svelte';

vi.mock('$lib/api', () => ({
	getAgentsStatus: vi.fn(async () => ({
		data: {
			generated_at: new Date().toISOString(),
			heartbeat_ttl_seconds: 120,
			configured_count: 1,
			live_count: 0,
			stale_count: 0,
			agents: [
				{
					configured: {
						agent_id: 'director',
						role: 'Director',
						primary_model: 'google/gemini-3.1-pro-preview',
						fallbacks: ['github-copilot/gpt-4o'],
						priority: 'high'
					},
					live: null,
					heartbeat_age_seconds: null,
					status: 'configured'
				}
			]
		},
		error: undefined
	})),
	createWebSocket: vi.fn(() => ({
		close: vi.fn()
	}))
}));

describe('Agents page', () => {
	it('renders configured roster when no live telemetry exists', async () => {
		render(AgentsPage);

		await waitFor(() => {
			expect(screen.getByText('Refresh Roster')).toBeInTheDocument();
		});

		expect(screen.getByText('Configured Roster')).toBeInTheDocument();
		expect(screen.getByText('director')).toBeInTheDocument();
		expect(screen.getByText(/No live heartbeats yet/i)).toBeInTheDocument();
	});
});
