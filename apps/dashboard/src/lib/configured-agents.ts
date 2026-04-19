import type { Agent } from './models';

export const configuredAgents: Agent[] = [
	{
		agent_id: 'contractor',
		state: 'idle',
		current_task_id: null,
		priority: 'normal',
		model: 'github-copilot/gpt-4o',
		last_heartbeat_ts: 0,
		elapsed_seconds: 0
	},
	{
		agent_id: 'director',
		state: 'idle',
		current_task_id: null,
		priority: 'high',
		model: 'google/gemini-3.1-pro-preview',
		last_heartbeat_ts: 0,
		elapsed_seconds: 0
	},
	{
		agent_id: 'architect',
		state: 'idle',
		current_task_id: null,
		priority: 'high',
		model: 'google/gemini-3.1-pro-preview',
		last_heartbeat_ts: 0,
		elapsed_seconds: 0
	},
	{
		agent_id: 'senior-engineer',
		state: 'idle',
		current_task_id: null,
		priority: 'high',
		model: 'github-copilot/gpt-4o',
		last_heartbeat_ts: 0,
		elapsed_seconds: 0
	},
	{
		agent_id: 'junior-engineer',
		state: 'idle',
		current_task_id: null,
		priority: 'normal',
		model: 'groq/llama-3.3-70b-versatile',
		last_heartbeat_ts: 0,
		elapsed_seconds: 0
	},
	{
		agent_id: 'intern',
		state: 'idle',
		current_task_id: null,
		priority: 'low',
		model: 'groq/llama-3.1-8b-instant',
		last_heartbeat_ts: 0,
		elapsed_seconds: 0
	}
];

export const configuredAgentLabels: Record<string, string> = {
	contractor: 'Front Door',
	director: 'Planner',
	architect: 'Design Review',
	'senior-engineer': 'Implementation',
	'junior-engineer': 'QA Review',
	intern: 'Notifications'
};
