import { writable, derived, type Readable, type Writable } from 'svelte/store';
import { isTauri } from '$lib/ws';

// ── Types ──────────────────────────────────────────────────────────

export interface SupervisorObservation {
	type: string;
	missionId: string;
	phaseId?: string;
	todoId?: string;
	message: string;
	severity: 'info' | 'warning' | 'critical';
	timestamp: number;
	suggestion?: string;
}

export interface SupervisorIntervention {
	type: string;
	missionId: string;
	message: string;
	timestamp: number;
	automated: boolean;
}

// ── Stores ─────────────────────────────────────────────────────────

export const observations: Writable<SupervisorObservation[]> = writable([]);
export const interventions: Writable<SupervisorIntervention[]> = writable([]);
export const supervisorActive: Writable<boolean> = writable(true);
export const autopilotEnabled: Writable<boolean> = writable(false);

// ── Derived stores ─────────────────────────────────────────────────

/**
 * Get observations for a specific mission.
 */
export function getObservationsForMission(missionId: string): Readable<SupervisorObservation[]> {
	return derived(observations, ($obs) =>
		$obs.filter((o) => o.missionId === missionId)
	);
}

/**
 * Get interventions for a specific mission.
 */
export function getInterventionsForMission(missionId: string): Readable<SupervisorIntervention[]> {
	return derived(interventions, ($int) =>
		$int.filter((i) => i.missionId === missionId)
	);
}

/**
 * Most recent critical or warning observation (for SupervisorBar).
 */
/**
 * Set of dismissed observation timestamps (used to hide them from the bar).
 */
export const dismissedObservations: Writable<Set<number>> = writable(new Set());

export const latestUrgentObservation: Readable<SupervisorObservation | null> = derived(
	[observations, dismissedObservations],
	([$obs, $dismissed]) => {
		const urgent = $obs
			.filter((o) => (o.severity === 'critical' || o.severity === 'warning') && !$dismissed.has(o.timestamp))
			.sort((a, b) => b.timestamp - a.timestamp);
		return urgent[0] ?? null;
	}
);

/**
 * Dismiss an observation so it no longer shows in the bar.
 */
export function dismissObservation(timestamp: number): void {
	dismissedObservations.update((set) => {
		set.add(timestamp);
		return new Set(set);
	});
}

// ── Seed data for development ──────────────────────────────────────

function seedDemoData(): void {
	const now = Date.now();

	observations.set([
		{
			type: 'RetryLoop',
			missionId: '*',
			phaseId: 'phase-2',
			todoId: 'todo-jwt-1',
			message: 'JWT validation: 4 retries detected',
			severity: 'warning',
			timestamp: now - 120_000,
			suggestion: 'Send tsconfig context to resolve module resolution',
		},
		{
			type: 'IdleDetected',
			missionId: '*',
			message: 'Phase 3 ready to queue',
			severity: 'info',
			timestamp: now - 240_000,
		},
		{
			type: 'SessionCrashed',
			missionId: '*',
			phaseId: 'phase-1',
			message: 'Claude Code session exited unexpectedly',
			severity: 'critical',
			timestamp: now - 480_000,
			suggestion: 'Restart session with preserved context',
		},
	]);

	interventions.set([
		{
			type: 'RestartedSession',
			missionId: '*',
			message: 'Restarted session',
			timestamp: now - 600_000,
			automated: true,
		},
		{
			type: 'SentHint',
			missionId: '*',
			message: 'Sent .env hint',
			timestamp: now - 420_000,
			automated: true,
		},
	]);
}

// ── Tauri Event Listeners ──────────────────────────────────────────

/**
 * Initialize Tauri event listeners for supervisor data.
 * Seeds demo data when no real events are available.
 */
export async function initSupervisorListeners(): Promise<void> {
	// Demo data removed -- real observations come from the Rust supervisor via Tauri events

	if (!isTauri()) return;

	const { listen } = await import('@tauri-apps/api/event');

	listen<SupervisorObservation>('supervisor-observation', (event) => {
		const obs = event.payload;
		observations.update((list) => [...list, obs]);
	});

	listen<SupervisorIntervention>('supervisor-intervention', (event) => {
		const int = event.payload;
		interventions.update((list) => [...list, int]);
	});
}
