import { writable, derived, get, type Readable, type Writable } from 'svelte/store';
import { isTauri } from '$lib/ws';

// ── Types ──────────────────────────────────────────────────────────

export interface ActivityEvent {
	id: string;
	timestamp: number;
	missionId: string;
	phaseId?: string;
	todoId?: string;
	source: 'mqtt' | 'claude_code' | 'supervisor' | 'human' | 'system';
	eventType: string;
	severity: 'info' | 'warning' | 'error';
	message: string;
	detail?: string;
}

export interface ActivityFilter {
	sources: Set<string>;
	missionId: string | null;
}

// ── Stores ─────────────────────────────────────────────────────────

let idCounter = 0;

function generateId(): string {
	idCounter += 1;
	return `evt-${Date.now()}-${idCounter}`;
}

/**
 * All activity events, newest last.
 */
export const activityEvents: Writable<ActivityEvent[]> = writable([]);

/**
 * Current filter state.
 */
export const activityFilter: Writable<ActivityFilter> = writable({
	sources: new Set<string>(),
	missionId: null,
});

/**
 * Derived store of events after applying filters.
 */
export const filteredEvents: Readable<ActivityEvent[]> = derived(
	[activityEvents, activityFilter],
	([$events, $filter]) => {
		let result = $events;

		// Filter by mission if set
		if ($filter.missionId) {
			result = result.filter((e) => e.missionId === $filter.missionId);
		}

		// Filter by source if any selected (empty set means show all)
		if ($filter.sources.size > 0) {
			result = result.filter((e) => $filter.sources.has(e.source));
		}

		return result;
	}
);

// ── Public API ─────────────────────────────────────────────────────

/**
 * Add a new event to the activity log.
 */
export function addActivityEvent(event: Omit<ActivityEvent, 'id'>): void {
	const full: ActivityEvent = { ...event, id: generateId() };
	activityEvents.update((list) => [...list, full]);
}

/**
 * Get a derived store of events for a specific mission.
 */
export function getEventsForMission(missionId: string): Readable<ActivityEvent[]> {
	return derived(activityEvents, ($events) =>
		$events.filter((e) => e.missionId === missionId)
	);
}

// ── Tauri Event Listeners ──────────────────────────────────────────

/**
 * Initialize Tauri event listeners that feed into the activity log.
 * Safely no-ops if not running in Tauri.
 */
export async function initActivityListeners(): Promise<void> {
	if (!isTauri()) return;

	const { listen } = await import('@tauri-apps/api/event');

	// sessions-updated -> system / state_change
	listen<unknown[]>('sessions-updated', (event) => {
		const sessions = event.payload;
		const count = Array.isArray(sessions) ? sessions.length : 0;
		addActivityEvent({
			timestamp: Date.now(),
			missionId: '*',
			source: 'system',
			eventType: 'state_change',
			severity: 'info',
			message: `Sessions updated (${count} active)`,
		});
	});

	// assignment-received -> mqtt / state_change
	listen<{ missionId?: string; phaseId?: string; phaseName?: string }>(
		'assignment-received',
		(event) => {
			const p = event.payload ?? {};
			addActivityEvent({
				timestamp: Date.now(),
				missionId: p.missionId ?? '*',
				phaseId: p.phaseId,
				source: 'mqtt',
				eventType: 'state_change',
				severity: 'info',
				message: `Assignment received: ${p.phaseName ?? p.phaseId ?? 'unknown phase'}`,
				detail: JSON.stringify(p, null, 2),
			});
		}
	);

	// todo-completed -> claude_code / state_change
	listen<{ todoId?: string; status?: string; missionId?: string; phaseId?: string }>(
		'todo-completed',
		(event) => {
			const p = event.payload ?? {};
			addActivityEvent({
				timestamp: Date.now(),
				missionId: p.missionId ?? '*',
				phaseId: p.phaseId,
				todoId: p.todoId,
				source: 'claude_code',
				eventType: 'state_change',
				severity: 'info',
				message: `Todo completed: ${p.todoId ?? 'unknown'}`,
				detail: JSON.stringify(p, null, 2),
			});
		}
	);

	// supervisor-observation -> supervisor / observation
	listen<{ missionId?: string; phaseId?: string; message?: string; detail?: string }>(
		'supervisor-observation',
		(event) => {
			const p = event.payload ?? {};
			addActivityEvent({
				timestamp: Date.now(),
				missionId: p.missionId ?? '*',
				phaseId: p.phaseId,
				source: 'supervisor',
				eventType: 'observation',
				severity: 'info',
				message: p.message ?? 'Supervisor observation',
				detail: p.detail,
			});
		}
	);

	// supervisor-intervention -> supervisor / intervention
	listen<{ missionId?: string; phaseId?: string; message?: string; detail?: string; severity?: string }>(
		'supervisor-intervention',
		(event) => {
			const p = event.payload ?? {};
			addActivityEvent({
				timestamp: Date.now(),
				missionId: p.missionId ?? '*',
				phaseId: p.phaseId,
				source: 'supervisor',
				eventType: 'intervention',
				severity: (p.severity as 'warning' | 'error') ?? 'warning',
				message: p.message ?? 'Supervisor intervention',
				detail: p.detail,
			});
		}
	);

	// mqtt-registry -> mqtt / state_change
	listen<{ missions?: Array<{ mission_id: string; title: string }> }>(
		'mqtt-registry',
		(event) => {
			const reg = event.payload;
			const count = reg?.missions?.length ?? 0;
			addActivityEvent({
				timestamp: Date.now(),
				missionId: '*',
				source: 'mqtt',
				eventType: 'state_change',
				severity: 'info',
				message: `Registry updated (${count} mission${count !== 1 ? 's' : ''})`,
				detail: JSON.stringify(reg, null, 2),
			});
		}
	);
}

// ── Demo Seed Data ─────────────────────────────────────────────────

const DEMO_MISSION_ID = 'demo-mission-001';

export function seedDemoEvents(): void {
	const now = Date.now();
	const events: Omit<ActivityEvent, 'id'>[] = [
		{
			timestamp: now - 45000,
			missionId: DEMO_MISSION_ID,
			source: 'system',
			eventType: 'state_change',
			severity: 'info',
			message: 'Worktree created',
			phaseId: 'setup',
		},
		{
			timestamp: now - 40000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'system',
			eventType: 'state_change',
			severity: 'info',
			message: 'Session started',
		},
		{
			timestamp: now - 35000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'mqtt',
			eventType: 'state_change',
			severity: 'info',
			message: 'PhaseState updated: executing',
		},
		{
			timestamp: now - 28000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'claude_code',
			eventType: 'tool_use',
			severity: 'info',
			message: 'Tool: Read src/auth/types.ts',
		},
		{
			timestamp: now - 22000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'claude_code',
			eventType: 'tool_use',
			severity: 'info',
			message: 'Tool: Edit auth/jwt.ts',
			detail: '--- a/auth/jwt.ts\n+++ b/auth/jwt.ts\n@@ -12,3 +12,5 @@\n+ validateToken(token: string): boolean {\n+   return this.verify(token);\n+ }',
		},
		{
			timestamp: now - 18000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'supervisor',
			eventType: 'observation',
			severity: 'warning',
			message: 'Detected retry loop on JWT validation',
			detail: 'The agent has attempted JWT validation 4 times with the same expired token. Consider clearing the token cache before retrying.',
		},
		{
			timestamp: now - 10000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'supervisor',
			eventType: 'intervention',
			severity: 'error',
			message: 'Blocked unsafe rm -rf on project root',
			detail: 'Command: rm -rf /Users/dev/project\nReason: destructive command targeting project root directory.',
		},
		{
			timestamp: now - 5000,
			missionId: DEMO_MISSION_ID,
			phaseId: 'phase-2',
			source: 'human',
			eventType: 'action',
			severity: 'info',
			message: 'Approved permission request: file write',
		},
	];

	for (const e of events) {
		addActivityEvent(e);
	}
}
