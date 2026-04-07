<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';
	import { selectedMissionId } from '$lib/stores/missions';

	interface Props {
		mission: UnifiedMission;
		onaction?: () => void;
	}

	let { mission, onaction }: Props = $props();

	const statusColors: Record<string, string> = {
		incoming: 'var(--mission-incoming)',
		validating: 'var(--mission-incoming)',
		ready: 'var(--accent-amber)',
		executing: 'var(--mission-active)',
		completed: 'var(--mission-completed)',
		failed: 'var(--mission-failed)',
	};

	let statusColor = $derived(statusColors[mission.status] || 'var(--text-muted)');
	let isSelected = $derived($selectedMissionId === mission.missionId);

	function handleClick() {
		selectedMissionId.set(mission.missionId);
	}

	function handleAction(e: MouseEvent) {
		e.stopPropagation();
		onaction?.();
	}

	let actionLabel = $derived(
		mission.status === 'incoming' || mission.status === 'validating'
			? 'Review'
			: mission.status === 'ready'
				? 'Start'
				: null
	);

	let supervisorText = $derived(
		mission.status === 'validating'
			? 'SUPERVISOR: checking...'
			: mission.status === 'incoming'
				? 'SUPERVISOR: pending review'
				: mission.status === 'ready'
					? 'SUPERVISOR: approved'
					: mission.status === 'executing'
						? 'SUPERVISOR: monitoring'
						: ''
	);
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="pipeline-card"
	class:selected={isSelected}
	class:attention={mission.needsAttention}
	onclick={handleClick}
	onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') handleClick(); }}
	role="button"
	tabindex="0"
>
	<div class="card-accent" style="background: {statusColor}"></div>
	<div class="card-body">
		<div class="card-title">{mission.title}</div>
		<div class="card-meta">
			<span class="meta-item">{mission.phaseCount} phases</span>
			<span class="meta-sep">/</span>
			<span class="meta-item">{mission.todoCount} todos</span>
		</div>
		<div class="card-status">
			<span class="status-dot" style="background: {statusColor}"></span>
			<span class="status-text">{mission.status}</span>
		</div>
		{#if supervisorText}
			<div class="supervisor-line">{supervisorText}</div>
		{/if}
		{#if actionLabel}
			<button class="action-btn" onclick={handleAction} type="button">
				{actionLabel}
			</button>
		{/if}
	</div>
</div>

<style>
	.pipeline-card {
		display: flex;
		align-items: stretch;
		width: 100%;
		background: var(--bg-card);
		border: 1px solid var(--border-muted);
		padding: 0;
		cursor: pointer;
		text-align: left;
		color: inherit;
		font: inherit;
		transition: border-color 150ms ease-out;
		position: relative;
	}

	.pipeline-card:hover {
		border-color: var(--border-default);
	}

	.pipeline-card.selected {
		border-color: var(--text-primary);
	}

	.pipeline-card.attention {
		border-color: var(--accent-amber);
	}

	.card-accent {
		width: 2px;
		flex-shrink: 0;
	}

	.card-body {
		flex: 1;
		min-width: 0;
		padding: 10px 12px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.card-title {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.card-meta {
		display: flex;
		align-items: center;
		gap: 4px;
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.meta-sep {
		color: var(--border-default);
	}

	.card-status {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		flex-shrink: 0;
	}

	.status-text {
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--text-muted);
	}

	.supervisor-line {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		letter-spacing: 0.02em;
		opacity: 0.7;
	}

	.action-btn {
		align-self: flex-start;
		margin-top: 2px;
		padding: 3px 10px;
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--text-primary);
		background: transparent;
		border: 1px solid var(--border-default);
		cursor: pointer;
		transition: background 100ms ease-out, border-color 100ms ease-out;
	}

	.action-btn:hover {
		background: rgba(255, 255, 255, 0.06);
		border-color: var(--text-secondary);
	}
</style>
