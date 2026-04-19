import { render, screen, waitFor } from '@testing-library/svelte';
import { describe, expect, it, vi } from 'vitest';
import AgentsPage from '../routes/agents/+page.svelte';

vi.mock('$lib/api', () => ({
	getAgents: vi.fn(async () => ({
		data: [],
		error: undefined
	})),
	getConfiguredAgents: vi.fn(async () => ({
		data: [
			{
				agent_id: 'director',
				role: 'planner',
				primary_model: 'google/gemini-3.1-pro-preview',
				fallbacks: ['github-copilot/gpt-4o'],
				priority: 'high'
			}
		]
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
