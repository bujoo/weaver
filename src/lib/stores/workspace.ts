import { writable } from 'svelte/store';
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
