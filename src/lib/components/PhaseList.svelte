<script lang="ts">
	import PhaseAccordion from './PhaseAccordion.svelte';
	import type { UnifiedMission } from '$lib/stores/missions';

	interface Props {
		mission: UnifiedMission;
	}

	let { mission }: Props = $props();

	// Track which phases are expanded
	let expandedPhases: Set<string> = $state(new Set());

	// Auto-expand executing phases -- derived, never written back into expandedPhases
	let autoExpandedPhases = $derived.by(() => {
		const next = new Set<string>();
		for (const phase of mission.availablePhases) {
			if (
				phase.status === 'executing' ||
				phase.status === 'in_progress' ||
				phase.status === 'active'
			) {
				next.add(phase.phaseId);
			}
		}
		if (next.size === 0 && mission.availablePhases.length > 0) {
			next.add(mission.availablePhases[0].phaseId);
		}
		return next;
	});

	// Merge user toggles with auto-expanded
	function isExpanded(phaseId: string): boolean {
		return expandedPhases.has(phaseId) || autoExpandedPhases.has(phaseId);
	}

	function togglePhase(phaseId: string) {
		expandedPhases = new Set(expandedPhases);
		if (expandedPhases.has(phaseId)) {
			expandedPhases.delete(phaseId);
		} else {
			expandedPhases.add(phaseId);
		}
	}
</script>

<div class="phase-list">
	{#if mission.availablePhases.length > 0}
		{#each mission.availablePhases as phase (phase.phaseId)}
			<PhaseAccordion
				{phase}
				missionId={mission.missionId}
				expanded={isExpanded(phase.phaseId)}
				ontoggle={() => togglePhase(phase.phaseId)}
			/>
		{/each}
	{:else}
		<div class="empty-state">
			<span class="empty-label">NO PHASES</span>
			<span class="empty-sub">Waiting for phase data from mission registry</span>
		</div>
	{/if}
</div>

<style>
	.phase-list {
		display: flex;
		flex-direction: column;
		gap: 0;
		padding: var(--space-sm) 0;
	}

	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-3xl) var(--space-lg);
		gap: var(--space-sm);
	}

	.empty-label {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-muted);
	}

	.empty-sub {
		font-size: 12px;
		color: var(--text-muted);
		opacity: 0.6;
	}
</style>
