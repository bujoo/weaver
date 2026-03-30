import { writable, derived } from 'svelte/store';
import { isTauri } from '$lib/ws';

export interface TaskQueueEntry {
	missionId: string;
	phaseId: string;
	phaseName: string;
	todos: string[];
	status: string; // queued | preparing | executing | completed | failed
	receivedAt: string;
	contextBundles: unknown[];
}

export interface TodoCompletion {
	todoId: string;
	status: string;
}

export const tasks = writable<TaskQueueEntry[]>([]);

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

// Track per-todo completion status
export const todoStatuses = writable<Map<string, string>>(new Map());

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

	// Listen for new assignments
	listen<unknown>('assignment-received', () => {
		// Refresh queue from Rust
		invoke<TaskQueueEntry[]>('get_task_queue').then((queue) => tasks.set(queue));
	});

	// Listen for todo completions
	listen<TodoCompletion>('todo-completed', (event) => {
		todoStatuses.update((map) => {
			map.set(event.payload.todoId, event.payload.status);
			return new Map(map);
		});
		// Refresh queue
		invoke<TaskQueueEntry[]>('get_task_queue').then((queue) => tasks.set(queue));
	});
}
