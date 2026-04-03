import { writable, derived } from 'svelte/store';
import { isTauri } from '$lib/ws';

export interface RepoStatus {
	path: string;
	name: string;
	branch: string;
	clean: boolean;
	worktrees: WorktreeInfo[];
}

export interface WorktreeInfo {
	path: string;
	branch: string;
}

export interface ToolInfo {
	name: string;
	installed: boolean;
	version: string | null;
}

export interface WorkspaceStatus {
	mountPath: string;
	repos: RepoStatus[];
	tools: ToolInfo[];
}

export const workspaceStatus = writable<WorkspaceStatus | null>(null);
export const loading = writable(false);

// MQTT registry: missions and their repos discovered via retained message
export interface MissionSummary {
	mission_id: string;
	title: string;
	status: string;
	repo_url: string | null;
	repos: { repo_id: string; repo_url: string | null; branch: string | null }[];
	phase_count: number;
	todo_count: number;
}

export interface WorkspaceRegistry {
	workspace: string;
	missions: MissionSummary[];
}

export const registry = writable<WorkspaceRegistry | null>(null);

export async function initRegistryListener() {
	if (!isTauri()) return;

	// Fetch cached registry from Rust (may already have it from auto-connect)
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		const cached = await invoke<WorkspaceRegistry | null>('get_registry');
		if (cached) registry.set(cached);
	} catch {}

	// Also listen for live updates
	const { listen } = await import('@tauri-apps/api/event');
	listen<WorkspaceRegistry>('mqtt-registry', (event) => {
		registry.set(event.payload);
	});

	// Auto-refresh workspace when autopilot finishes setup
	listen('autopilot-workspace-ready', () => {
		refreshWorkspace();
	});
}

export async function refreshWorkspace() {
	if (!isTauri()) return;
	loading.set(true);
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		const status = await invoke<WorkspaceStatus>('get_workspace_status');
		workspaceStatus.set(status);
	} catch (e) {
		console.error('Failed to load workspace status:', e);
	}
	loading.set(false);
}
