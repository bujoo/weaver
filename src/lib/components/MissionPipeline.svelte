<script lang="ts">
	import { missions, selectedMissionId } from '$lib/stores/missions';
	import type { UnifiedMission } from '$lib/stores/missions';
	import MissionPipelineCard from './MissionPipelineCard.svelte';

	let allMissions = $derived($missions);

	let incoming = $derived(allMissions.filter((m) => m.status === 'incoming'));
	let validating = $derived(allMissions.filter((m) => m.status === 'validating'));
	let ready = $derived(allMissions.filter((m) => m.status === 'ready'));
	let setup = $derived(allMissions.filter((m) => m.status === 'setup'));
	let executing = $derived(allMissions.filter((m) => m.status === 'executing'));

	interface Column {
		key: string;
		label: string;
		missions: UnifiedMission[];
	}

	let columns: Column[] = $derived([
		{ key: 'incoming', label: 'RECEIVED', missions: incoming },
		{ key: 'validating', label: 'VALIDATED', missions: validating },
		{ key: 'ready', label: 'ACCEPTED', missions: ready },
		{ key: 'setup', label: 'SETUP', missions: setup },
		{ key: 'executing', label: 'EXECUTING', missions: executing },
	]);
</script>

<div class="pipeline">
	{#each columns as col (col.key)}
		<div class="pipeline-column">
			<div class="column-header">
				<span class="column-title">{col.label}</span>
				<span class="column-count">{col.missions.length}</span>
			</div>
			<div class="column-body">
				{#if col.missions.length > 0}
					{#each col.missions as mission (mission.missionId)}
						<MissionPipelineCard {mission} onaction={() => selectedMissionId.set(mission.missionId)} />
					{/each}
				{:else}
					<div class="column-empty">
						<span class="empty-dash">--</span>
					</div>
				{/if}
			</div>
		</div>
	{/each}
</div>

<style>
	.pipeline {
		display: grid;
		grid-template-columns: repeat(5, 1fr);
		gap: 0;
		width: 100%;
		min-height: 180px;
	}

	.pipeline-column {
		display: flex;
		flex-direction: column;
		border-left: 1px solid var(--border-muted);
		min-height: 0;
	}

	.pipeline-column:first-child {
		border-left: none;
	}

	.column-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--border-muted);
		flex-shrink: 0;
	}

	.column-title {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: var(--text-muted);
	}

	.column-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		opacity: 0.6;
	}

	.column-body {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 8px;
		flex: 1;
	}

	.column-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		min-height: 60px;
	}

	.empty-dash {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		opacity: 0.3;
	}
</style>
