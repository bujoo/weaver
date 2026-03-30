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
	missionId: string;
	title: string;
	status: string;
	repoUrl: string | null;
	repos: { repoId: string; repoUrl: string | null; branch: string | null }[];
	phaseCount: number;
	todoCount: number;
}

export interface WorkspaceRegistry {
	workspace: string;
	missions: MissionSummary[];
}

export const registry = writable<WorkspaceRegistry | null>(null);

export async function initRegistryListener() {
	if (!isTauri()) return;
	const { listen } = await import('@tauri-apps/api/event');
	listen<WorkspaceRegistry>('mqtt-registry', (event) => {
		registry.set(event.payload);
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
