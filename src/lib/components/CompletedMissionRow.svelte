<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';
	import { selectedMissionId } from '$lib/stores/missions';

	interface Props {
		mission: UnifiedMission;
	}

	let { mission }: Props = $props();

	function handleClick() {
		selectedMissionId.set(mission.missionId);
	}

	/**
	 * Format a rough duration from lastActivity to now.
	 */
	let durationText = $derived(() => {
		const now = Date.now();
		const elapsed = now - mission.lastActivity;
		const mins = Math.round(elapsed / 60_000);
		if (mins < 1) return '< 1 min';
		if (mins < 60) return `${mins} min`;
		const hrs = Math.round(mins / 60);
		if (hrs < 24) return `${hrs} hr`;
		const days = Math.round(hrs / 24);
		return `${days} d`;
	});

	let phaseText = $derived(`${mission.completedPhases}/${mission.phaseCount} phases`);
</script>

<button class="completed-row" onclick={handleClick} type="button">
	<span class="row-title">{mission.title}</span>
	<span class="row-duration">{durationText()}</span>
	<span class="row-phases">{phaseText}</span>
	<span class="row-interventions">0 interventions</span>
</button>

<style>
	.completed-row {
		display: flex;
		align-items: center;
		gap: 16px;
		width: 100%;
		padding: 8px 12px;
		background: transparent;
		border: 1px solid transparent;
		cursor: pointer;
		text-align: left;
		color: inherit;
		font: inherit;
		transition: background 100ms ease-out, border-color 100ms ease-out;
	}

	.completed-row:hover {
		background: var(--bg-card);
		border-color: var(--border-muted);
	}

	.row-title {
		flex: 1;
		min-width: 0;
		font-size: 13px;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.row-duration {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.row-phases {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.row-interventions {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
	}
</style>
