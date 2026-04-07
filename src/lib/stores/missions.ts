import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { registry, type MissionSummary } from './workspace';
import { tasks, availablePhases, humanNeededPhases, type TaskQueueEntry, type AvailablePhase, type HumanPhaseAlert } from './tasks';
import { sessions } from './sessions';
import { isTauri } from '$lib/ws';
import type { Session } from '$lib/types';

export interface CachedTodo {
	todo_id: string;
	description: string;
	status: string;
	role: string;
}

export interface CachedPhase {
	phase_id: string;
	name: string;
	order: number;
	status: string;
	todo_count: number;
	completed_count: number;
	todos: CachedTodo[];
}

/**
 * Cached phase data from Rust state cache, keyed by mission_id.
 */
export const cachedPhases: Writable<Record<string, CachedPhase[]>> = writable({});

/**
 * Fetch phases for a mission from the Rust state cache.
 */
export async function fetchMissionPhases(missionId: string): Promise<void> {
	if (!isTauri()) return;
	try {
		const { invoke } = await import('@tauri-apps/api/core');
		const phases = await invoke<CachedPhase[]>('get_mission_phases', { missionId });
		if (phases && phases.length > 0) {
			cachedPhases.update((cache) => ({ ...cache, [missionId]: phases }));
		}
	} catch (e) {
		console.error('[missions] Failed to fetch phases for', missionId, e);
	}
}

/**
 * Start polling phases for all missions in the registry.
 */
export function startPhasePolling(): () => void {
	const interval = setInterval(() => {
		const reg = get(registry);
		if (reg?.missions) {
			for (const m of reg.missions) {
				fetchMissionPhases(m.mission_id);
			}
		}
	}, 5000);
	// Initial fetch
	const reg = get(registry);
	if (reg?.missions) {
		for (const m of reg.missions) {
			fetchMissionPhases(m.mission_id);
		}
	}
	return () => clearInterval(interval);
}

export interface UnifiedMission {
	missionId: string;
	title: string;
	status: 'incoming' | 'validating' | 'ready' | 'executing' | 'completed' | 'failed';
	phaseCount: number;
	todoCount: number;
	completedPhases: number;
	completedTodos: number;
	repos: Array<{ repoId: string; repoUrl: string | null; branch: string | null }>;
	availablePhases: Array<{ phaseId: string; phaseName: string; todoCount: number; status: string; todos?: CachedTodo[] }>;
	taskQueue: TaskQueueEntry[];
	activeSessions: number;
	needsAttention: boolean;
	lastActivity: number;
}

/**
 * Currently selected mission ID in the sidebar
 */
export const selectedMissionId: Writable<string | null> = writable(null);

/**
 * Status priority for sorting (lower = higher priority)
 */
const STATUS_PRIORITY: Record<string, number> = {
	executing: 1,
	ready: 2,
	validating: 3,
	incoming: 4,
	completed: 5,
	failed: 6,
};

/**
 * Unified missions derived from all existing stores.
 * Aggregates registry, tasks, available phases, human alerts, and sessions.
 */
export const missions: Readable<UnifiedMission[]> = derived(
	[registry, tasks, availablePhases, humanNeededPhases, sessions, cachedPhases],
	([$registry, $tasks, $availablePhases, $humanNeededPhases, $sessions, $cachedPhases]) => {
		if (!$registry?.missions) return [];

		return $registry.missions
			.map((m: MissionSummary) => buildUnifiedMission(m, $tasks, $availablePhases, $humanNeededPhases, $sessions, $cachedPhases))
			.sort(missionSortComparator);
	}
);

/**
 * Active missions (currently executing)
 */
export const activeMissions: Readable<UnifiedMission[]> = derived(missions, ($missions) =>
	$missions.filter((m) => m.status === 'executing')
);

/**
 * Incoming missions (from registry, not yet accepted)
 */
export const incomingMissions: Readable<UnifiedMission[]> = derived(missions, ($missions) =>
	$missions.filter((m) => m.status === 'incoming' || m.status === 'validating' || m.status === 'ready')
);

/**
 * Completed missions
 */
export const completedMissions: Readable<UnifiedMission[]> = derived(missions, ($missions) =>
	$missions.filter((m) => m.status === 'completed')
);

/**
 * Currently selected mission object
 */
export const selectedMission: Readable<UnifiedMission | null> = derived(
	[missions, selectedMissionId],
	([$missions, $selectedMissionId]) => {
		if (!$selectedMissionId) return $missions[0] ?? null;
		return $missions.find((m) => m.missionId === $selectedMissionId) ?? null;
	}
);

/**
 * Whether any missions are currently executing
 */
export const hasExecutingMissions: Readable<boolean> = derived(activeMissions, ($active) => $active.length > 0);

/**
 * Count of missions needing human attention
 */
export const attentionCount: Readable<number> = derived(missions, ($missions) =>
	$missions.filter((m) => m.needsAttention).length
);

// ── Internal helpers ────────────────────────────────────────────────

function buildUnifiedMission(
	m: MissionSummary,
	$tasks: TaskQueueEntry[],
	$availablePhases: AvailablePhase[],
	$humanNeededPhases: HumanPhaseAlert[],
	$sessions: Session[],
	$cachedPhases: Record<string, Array<{ phase_id: string; name: string; order: number; status: string; todo_count: number; completed_count: number }>>
): UnifiedMission {
	// Tasks assigned to this mission
	const missionTasks = $tasks.filter((t) => t.missionId === m.mission_id);

	// Phases: prefer state cache (from Rust), then tasks store, then registry
	const stateCachePhases = ($cachedPhases[m.mission_id] ?? []).map((p) => ({
		phaseId: p.phase_id,
		phaseName: p.name,
		todoCount: p.todo_count,
		status: p.status,
		todos: p.todos ?? [],
	}));
	const taskPhases = $availablePhases
		.filter((p) => p.missionId === m.mission_id)
		.map((p) => ({
			phaseId: p.phaseId,
			phaseName: p.phaseName,
			todoCount: p.todoCount,
			status: p.status,
		}));
	const registryPhases = (m.available_phases ?? []).map((p) => ({
		phaseId: p.phase_id,
		phaseName: p.name,
		todoCount: p.todo_count,
		status: p.status,
	}));
	const missionPhases = stateCachePhases.length > 0 ? stateCachePhases : (taskPhases.length > 0 ? taskPhases : registryPhases);

	// Human-needed phases for this mission
	const humanAlerts = $humanNeededPhases.filter((h) => h.missionId === m.mission_id);

	// Count active Claude Code sessions by matching branch or CWD containing mission short ID
	const shortMid = m.mission_id.slice(0, 8);
	const branchPrefix = `weaver-${shortMid}`;
	const activeSessions = $sessions.filter(
		(s) => s.gitBranch?.startsWith(branchPrefix) || s.projectPath?.includes(shortMid)
	).length;

	// Calculate completed phases and todos from task data
	const completedPhases = missionTasks.filter((t) => t.status === 'completed').length;
	const completedTodos = missionTasks.reduce((acc, t) => {
		// Count completed todos in each task (rough estimate from status)
		if (t.status === 'completed') return acc + t.todos.length;
		return acc;
	}, 0);

	// Determine last activity timestamp
	const taskTimestamps = missionTasks
		.map((t) => new Date(t.receivedAt).getTime())
		.filter((ts) => !isNaN(ts));
	const lastActivity = taskTimestamps.length > 0 ? Math.max(...taskTimestamps) : Date.now();

	// Normalize status
	const status = normalizeStatus(m.status, missionTasks);

	return {
		missionId: m.mission_id,
		title: m.title,
		status,
		phaseCount: m.phase_count,
		todoCount: m.todo_count,
		completedPhases,
		completedTodos,
		repos: (m.repos ?? []).map((r) => ({
			repoId: r.repo_id,
			repoUrl: r.repo_url,
			branch: r.branch,
		})),
		availablePhases: missionPhases,
		taskQueue: missionTasks,
		activeSessions,
		needsAttention: humanAlerts.length > 0,
		lastActivity,
	};
}

function normalizeStatus(
	rawStatus: string,
	missionTasks: TaskQueueEntry[]
): UnifiedMission['status'] {
	const s = rawStatus.toLowerCase();
	if (s === 'incoming') return 'incoming';
	if (s === 'validating') return 'validating';
	if (s === 'ready') return 'ready';
	if (s === 'completed' || s === 'done') return 'completed';
	if (s === 'failed' || s === 'error') return 'failed';
	if (s === 'executing' || s === 'active' || s === 'in_progress') return 'executing';

	// Infer from tasks if status is ambiguous
	if (missionTasks.some((t) => t.status === 'executing')) return 'executing';
	if (missionTasks.every((t) => t.status === 'completed') && missionTasks.length > 0) return 'completed';
	return 'ready';
}

function missionSortComparator(a: UnifiedMission, b: UnifiedMission): number {
	// Attention needed always first
	if (a.needsAttention && !b.needsAttention) return -1;
	if (!a.needsAttention && b.needsAttention) return 1;

	// Then by status priority
	const pa = STATUS_PRIORITY[a.status] ?? 99;
	const pb = STATUS_PRIORITY[b.status] ?? 99;
	if (pa !== pb) return pa - pb;

	// Then by most recent activity
	return b.lastActivity - a.lastActivity;
}

/**
 * Auto-select the highest priority mission when none is selected.
 * Call this from a component $effect.
 */
export function autoSelectMission(missionList: UnifiedMission[], currentId: string | null): void {
	if (currentId && missionList.some((m) => m.missionId === currentId)) return;
	if (missionList.length > 0) {
		selectedMissionId.set(missionList[0].missionId);
	}
}
