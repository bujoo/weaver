import { writable, derived } from 'svelte/store';
import { isTauri } from '$lib/ws';

export interface TaskQueueEntry {
	missionId: string;
	phaseId: string;
	phaseName: string;
	todos: string[];
	status: string;
	receivedAt: string;
	contextBundles: unknown[];
}

export interface AvailablePhase {
	missionId: string;
	missionTitle: string;
	phaseId: string;
	phaseName: string;
	todoCount: number;
	status: string;
}

export interface TodoCompletion {
	todoId: string;
	status: string;
}

export interface HumanPhaseAlert {
	missionId: string;
	missionTitle: string;
	phaseId: string;
	phaseName: string;
	workspaceDir: string;
}

// My tasks: assigned to this device
export const tasks = writable<TaskQueueEntry[]>([]);

// Available phases: waiting for someone to accept
export const availablePhases = writable<AvailablePhase[]>([]);

export const activeTask = derived(tasks, ($tasks) =>
	$tasks.find((t) => t.status === 'executing' || t.status === 'preparing')
);

export const taskCounts = derived(tasks, ($tasks) => ({
	queued: $tasks.filter((t) => t.status === 'queued').length,
	executing: $tasks.filter((t) => t.status === 'executing').length,
	completed: $tasks.filter((t) => t.status === 'completed').length,
	failed: $tasks.filter((t) => t.status === 'failed').length,
	total: $tasks.length
}));

export const todoStatuses = writable<Map<string, string>>(new Map());

// Human phases that need developer attention (Step-In)
export const humanNeededPhases = writable<HumanPhaseAlert[]>([]);

export async function initializeTaskListeners() {
	if (!isTauri()) return;

	const { listen } = await import('@tauri-apps/api/event');
	const { invoke } = await import('@tauri-apps/api/core');

	// Load existing queue
	try {
		const queue = await invoke<TaskQueueEntry[]>('get_task_queue');
		tasks.set(queue);
	} catch (e) {
		console.error('Failed to load task queue:', e);
	}

	// Load available phases from registry
	try {
		const reg = await invoke<{ missions?: Array<{ mission_id: string; title: string; available_phases?: Array<{ phase_id: string; phase_name: string; todo_count: number; status: string }> }> } | null>('get_registry');
		if (reg?.missions) {
			const phases: AvailablePhase[] = [];
			for (const m of reg.missions) {
				for (const p of m.available_phases ?? []) {
					phases.push({
						missionId: m.mission_id,
						missionTitle: m.title,
						phaseId: p.phase_id,
						phaseName: p.phase_name,
						todoCount: p.todo_count,
						status: p.status,
					});
				}
			}
			availablePhases.set(phases);
		}
	} catch {}

	// Listen for new assignments
	listen<unknown>('assignment-received', () => {
		invoke<TaskQueueEntry[]>('get_task_queue').then((queue) => tasks.set(queue));
	});

	// Listen for todo completions
	listen<TodoCompletion>('todo-completed', (event) => {
		todoStatuses.update((map) => {
			map.set(event.payload.todoId, event.payload.status);
			return new Map(map);
		});
		invoke<TaskQueueEntry[]>('get_task_queue').then((queue) => tasks.set(queue));
	});

	// Listen for human phase notifications (Step-In)
	listen<HumanPhaseAlert>('autopilot-human-needed', (event) => {
		humanNeededPhases.update((list) => {
			// Deduplicate by missionId:phaseId
			const key = `${event.payload.missionId}:${event.payload.phaseId}`;
			if (list.some((p) => `${p.missionId}:${p.phaseId}` === key)) return list;
			return [...list, event.payload];
		});
	});

	// Listen for registry updates (new available phases)
	listen<unknown>('mqtt-registry', (event) => {
		const reg = event.payload as { missions?: Array<{ mission_id: string; title: string; available_phases?: Array<{ phase_id: string; phase_name: string; todo_count: number; status: string }> }> };
		if (reg?.missions) {
			const phases: AvailablePhase[] = [];
			for (const m of reg.missions) {
				for (const p of m.available_phases ?? []) {
					phases.push({
						missionId: m.mission_id,
						missionTitle: m.title,
						phaseId: p.phase_id,
						phaseName: p.phase_name,
						todoCount: p.todo_count,
						status: p.status,
					});
				}
			}
			availablePhases.set(phases);
		}
	});
}

export async function acceptPhase(missionId: string, phaseId: string) {
	if (!isTauri()) return;
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		await invoke('accept_phase_cmd', { missionId, phaseId });
	} catch (e) {
		console.error('Failed to accept phase:', e);
	}
}
