<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';

	interface Props {
		mission: UnifiedMission;
		selected?: boolean;
		onclick?: () => void;
	}

	let { mission, selected = false, onclick }: Props = $props();

	const statusColors: Record<string, string> = {
		incoming: 'var(--mission-incoming)',
		validating: 'var(--mission-incoming)',
		ready: 'var(--accent-amber)',
		executing: 'var(--mission-active)',
		completed: 'var(--mission-completed)',
		failed: 'var(--mission-failed)',
	};

	let statusColor = $derived(statusColors[mission.status] || 'var(--text-muted)');
	let progressPct = $derived(
		mission.phaseCount > 0 ? (mission.completedPhases / mission.phaseCount) * 100 : 0
	);
</script>

<button
	class="mission-item"
	class:selected
	class:attention={mission.needsAttention}
	{onclick}
	type="button"
>
	<div class="accent-border" style="background: {statusColor}"></div>
	<div class="mission-content">
		<div class="mission-title">{mission.title}</div>
		<div class="mission-meta">
			<span class="status-dot" style="background: {statusColor}"></span>
			<span class="status-label">{mission.status}</span>
			<span class="phase-progress">{mission.completedPhases}/{mission.phaseCount}</span>
			<div class="progress-bar">
				<div class="progress-fill" style="width: {progressPct}%; background: {statusColor}"></div>
			</div>
			{#if mission.activeSessions > 0}
				<span class="agent-badge">{mission.activeSessions}</span>
			{/if}
			{#if mission.needsAttention}
				<span class="attention-badge">
					<!-- Exclamation icon -->
					<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
						<line x1="12" y1="9" x2="12" y2="13" />
						<line x1="12" y1="17" x2="12.01" y2="17" />
					</svg>
				</span>
			{/if}
		</div>
	</div>
</button>

<style>
	.mission-item {
		display: flex;
		align-items: stretch;
		width: 100%;
		min-height: 44px;
		background: transparent;
		border: none;
		padding: 0;
		cursor: pointer;
		text-align: left;
		color: inherit;
		font: inherit;
		transition: background var(--transition-fast);
		position: relative;
	}

	.mission-item:hover {
		background: var(--bg-card-hover);
	}

	.mission-item.selected {
		background: var(--bg-card);
	}

	.accent-border {
		width: 2px;
		flex-shrink: 0;
		opacity: 0;
		transition: opacity var(--transition-fast);
	}

	.mission-item.selected .accent-border {
		opacity: 1;
	}

	.mission-content {
		flex: 1;
		min-width: 0;
		padding: 8px 12px 8px 10px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.mission-title {
		font-size: 13px;
		font-weight: 500;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 180px;
	}

	.mission-meta {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11px;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		flex-shrink: 0;
	}

	.status-label {
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.04em;
		font-size: 10px;
	}

	.phase-progress {
		font-family: var(--font-mono);
		color: var(--text-muted);
		font-size: 10px;
	}

	.progress-bar {
		flex: 1;
		height: 2px;
		background: var(--border-muted);
		min-width: 20px;
	}

	.progress-fill {
		height: 100%;
		transition: width 200ms ease-out;
	}

	.agent-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 14px;
		height: 14px;
		font-size: 9px;
		font-family: var(--font-mono);
		font-weight: 600;
		color: var(--text-primary);
		background: var(--bg-card-hover);
		border: 1px solid var(--border-default);
		flex-shrink: 0;
	}

	.attention-badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--accent-amber);
		flex-shrink: 0;
		animation: attention-pulse 2s ease-in-out infinite;
	}

	@keyframes attention-pulse {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.4;
		}
	}

	.mission-item.attention {
		background: oklch(0.74 0.17 55 / 0.04);
	}

	.mission-item.attention:hover {
		background: oklch(0.74 0.17 55 / 0.08);
	}
</style>
