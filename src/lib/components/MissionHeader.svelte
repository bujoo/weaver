<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';

	interface Props {
		mission: UnifiedMission;
	}

	let { mission }: Props = $props();

	const statusColors: Record<string, string> = {
		incoming: 'var(--mission-incoming)',
		validating: 'var(--mission-incoming)',
		ready: 'var(--accent-amber)',
		executing: 'var(--mission-active)',
		completed: 'var(--mission-completed)',
		failed: 'var(--mission-failed)',
	};

	const statusLabels: Record<string, string> = {
		incoming: 'INCOMING',
		validating: 'VALIDATING',
		ready: 'READY',
		executing: 'EXECUTING',
		completed: 'COMPLETED',
		failed: 'FAILED',
	};

	let statusColor = $derived(statusColors[mission.status] || 'var(--text-muted)');
	let statusLabel = $derived(statusLabels[mission.status] || mission.status.toUpperCase());
	let truncatedId = $derived(mission.missionId.slice(0, 8));

	/**
	 * Pipeline stages mapped from mission status.
	 * Each stage is either 'completed', 'current', or 'future'.
	 */
	const PIPELINE_STAGES = ['received', 'validated', 'accepted', 'setup', 'executing'] as const;

	const STATUS_TO_STAGE_INDEX: Record<string, number> = {
		incoming: 0,
		validating: 1,
		ready: 2,
		executing: 4, // all 5 stages (0-4) should be filled
		completed: 5, // all done
		failed: -1,
	};

	let currentStageIndex = $derived(STATUS_TO_STAGE_INDEX[mission.status] ?? 0);

	function stageState(index: number): 'completed' | 'current' | 'future' {
		if (mission.status === 'completed') return 'completed';
		if (mission.status === 'failed') return index <= currentStageIndex ? 'completed' : 'future';
		if (index < currentStageIndex) return 'completed';
		if (index === currentStageIndex) return 'current';
		return 'future';
	}
</script>

<div class="mission-header">
	<div class="header-top">
		<h1 class="mission-title">{mission.title}</h1>
		<span class="status-badge" style="background: {statusColor}; color: #000;">
			{statusLabel}
		</span>
	</div>

	<div class="header-meta">
		<span class="meta-id">{truncatedId}</span>
		<span class="meta-sep">|</span>
		<span class="meta-item">{mission.phaseCount} phases</span>
		<span class="meta-sep">|</span>
		<span class="meta-item">{mission.todoCount} todos</span>
		<span class="meta-sep">|</span>
		<span class="meta-item">{mission.activeSessions} agent{mission.activeSessions !== 1 ? 's' : ''}</span>
	</div>

	<div class="pipeline-bar">
		<span class="pipeline-label">PIPELINE</span>
		<div class="pipeline-segments">
			{#each PIPELINE_STAGES as stage, i (stage)}
				{@const state = stageState(i)}
				<div class="pipeline-segment {state}" title="{stage} ({state})">
					<div class="segment-fill"></div>
				</div>
			{/each}
		</div>
		<div class="pipeline-labels">
			{#each PIPELINE_STAGES as stage (stage)}
				<span class="pipeline-stage-label">{stage}</span>
			{/each}
		</div>
	</div>
</div>

<style>
	.mission-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		padding-bottom: var(--space-lg);
		border-bottom: 1px solid var(--border-default);
	}

	.header-top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-lg);
	}

	.mission-title {
		font-family: var(--font-sans);
		font-size: 24px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: -0.02em;
		line-height: 1.2;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		min-width: 0;
	}

	.status-badge {
		flex-shrink: 0;
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.08em;
		padding: 3px 10px;
		text-transform: uppercase;
	}

	.header-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.meta-id {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.meta-sep {
		color: var(--border-default);
	}

	.meta-item {
		color: var(--text-muted);
	}

	/* ── Pipeline ────────────────────────────────────────────────── */
	.pipeline-bar {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		margin-top: var(--space-xs);
	}

	.pipeline-label {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-muted);
		text-transform: uppercase;
	}

	.pipeline-segments {
		display: flex;
		gap: 2px;
		height: 6px;
	}

	.pipeline-segment {
		flex: 1;
		background: var(--border-muted);
		overflow: hidden;
		position: relative;
	}

	.pipeline-segment .segment-fill {
		position: absolute;
		inset: 0;
		background: transparent;
		transition: background var(--transition-normal);
	}

	.pipeline-segment.completed .segment-fill {
		background: var(--accent-green);
	}

	.pipeline-segment.current .segment-fill {
		background: var(--task-executing);
		animation: pulse-glow 2s ease-in-out infinite;
	}

	.pipeline-segment.future .segment-fill {
		background: transparent;
	}

	.pipeline-labels {
		display: flex;
		gap: 2px;
	}

	.pipeline-stage-label {
		flex: 1;
		font-family: var(--font-mono);
		font-size: 9px;
		color: var(--text-muted);
		text-transform: lowercase;
		letter-spacing: 0.02em;
	}

	/* ── Mobile ──────────────────────────────────────────────────── */
	@media (max-width: 768px) {
		.mission-title {
			font-size: 18px;
		}

		.header-meta {
			flex-wrap: wrap;
			gap: var(--space-xs);
		}
	}
</style>
