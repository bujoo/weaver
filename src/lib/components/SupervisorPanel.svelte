<script lang="ts">
	import {
		getObservationsForMission,
		getInterventionsForMission,
		supervisorActive,
		autopilotEnabled,
		type SupervisorObservation,
		type SupervisorIntervention,
	} from '$lib/stores/supervisor';

	interface Props {
		missionId: string;
	}

	let { missionId }: Props = $props();

	let obsStore = $derived(getObservationsForMission(missionId));
	let intStore = $derived(getInterventionsForMission(missionId));

	let missionObs = $derived($obsStore);
	let missionInt = $derived($intStore);

	let isActive = $derived($supervisorActive);
	let isAutopilot = $derived($autopilotEnabled);

	// Sort observations newest first
	let sortedObs = $derived([...missionObs].sort((a, b) => b.timestamp - a.timestamp));
	let sortedInt = $derived([...missionInt].sort((a, b) => b.timestamp - a.timestamp));

	function formatTime(ts: number): string {
		const d = new Date(ts);
		return d.toLocaleTimeString('en-GB', {
			hour: '2-digit',
			minute: '2-digit',
			hour12: false,
		});
	}

	function severityIcon(severity: SupervisorObservation['severity']): string {
		if (severity === 'critical' || severity === 'warning') return '[!!]';
		return '[i]';
	}

	function severityClass(severity: SupervisorObservation['severity']): string {
		if (severity === 'critical') return 'severity-critical';
		if (severity === 'warning') return 'severity-warning';
		return 'severity-info';
	}

	function handleApplyFix(obs: SupervisorObservation) {
		console.log('[Supervisor] Apply fix for:', obs.type, obs.suggestion);
	}

	function handleSendCustom(obs: SupervisorObservation) {
		console.log('[Supervisor] Send custom for:', obs.type);
	}

	function handleEscalate(obs: SupervisorObservation) {
		console.log('[Supervisor] Escalate:', obs.type);
	}
</script>

<div class="supervisor-panel">
	<!-- Border Control Status -->
	<section class="panel-section">
		<div class="section-header">
			<span class="section-title">BORDER CONTROL STATUS</span>
			<div class="header-controls">
				<button
					class="toggle-pill"
					class:active={isActive}
					onclick={() => supervisorActive.update((v) => !v)}
				>
					{isActive ? 'ACTIVE' : 'PAUSED'}
				</button>
				<button
					class="toggle-pill autopilot"
					class:active={isAutopilot}
					onclick={() => autopilotEnabled.update((v) => !v)}
				>
					AUTOPILOT {isAutopilot ? 'ON' : 'OFF'}
				</button>
			</div>
		</div>
		<div class="border-rows">
			<div class="border-row">
				<span class="border-label">Brain -> Weaver</span>
				<span class="border-value">2 received, 2 validated</span>
			</div>
			<div class="border-row">
				<span class="border-label">Weaver -> Claude</span>
				<span class="border-value">1 sent, context complete</span>
			</div>
			<div class="border-row">
				<span class="border-label">Claude -> Weaver</span>
				<span class="border-value">3 completed, 1 in progress</span>
			</div>
			<div class="border-row">
				<span class="border-label">Weaver -> Brain</span>
				<span class="border-value">3 published</span>
			</div>
		</div>
	</section>

	<!-- Active Observations -->
	<section class="panel-section">
		<div class="section-header">
			<span class="section-title">ACTIVE OBSERVATIONS</span>
			<span class="section-count">{sortedObs.length}</span>
		</div>
		<div class="obs-list">
			{#if sortedObs.length === 0}
				<div class="empty-row">No observations</div>
			{:else}
				{#each sortedObs as obs (obs.timestamp + obs.type)}
					<div class="obs-row {severityClass(obs.severity)}">
						<div class="obs-main">
							<span class="obs-icon">{severityIcon(obs.severity)}</span>
							<span class="obs-message">{obs.message}</span>
							<span class="obs-time">{formatTime(obs.timestamp)}</span>
						</div>
						{#if obs.suggestion}
							<div class="obs-detail">
								<span class="obs-pattern">
									{obs.phaseId ? `Phase: ${obs.phaseId}` : ''}
								</span>
								<span class="obs-suggestion">Suggested: {obs.suggestion}</span>
								<div class="obs-actions">
									<button class="action-btn primary" onclick={() => handleApplyFix(obs)}>
										Apply Fix
									</button>
									<button class="action-btn" onclick={() => handleSendCustom(obs)}>
										Send Custom
									</button>
									<button class="action-btn danger" onclick={() => handleEscalate(obs)}>
										Escalate
									</button>
								</div>
							</div>
						{/if}
					</div>
				{/each}
			{/if}
		</div>
	</section>

	<!-- Intervention History -->
	<section class="panel-section">
		<div class="section-header">
			<span class="section-title">INTERVENTION HISTORY</span>
			<span class="section-count">{sortedInt.length}</span>
		</div>
		<div class="int-list">
			{#if sortedInt.length === 0}
				<div class="empty-row">No interventions</div>
			{:else}
				{#each sortedInt as int (int.timestamp + int.type)}
					<div class="int-row">
						<span class="int-time">{formatTime(int.timestamp)}</span>
						<span class="int-badge" class:auto={int.automated}>
							{int.automated ? 'AUTO' : 'HUMAN'}
						</span>
						<span class="int-message">{int.message}</span>
					</div>
				{/each}
			{/if}
		</div>
	</section>

	<!-- Mission Insights -->
	<section class="panel-section">
		<div class="section-header">
			<span class="section-title">MISSION INSIGHTS</span>
		</div>
		<div class="insights">
			<div class="insight-row">
				<span class="insight-label">Auth todos</span>
				<span class="insight-value">avg 8min, 30% retry</span>
			</div>
			<div class="insight-row">
				<span class="insight-label">This mission</span>
				<span class="insight-value">15% faster than average</span>
			</div>
			<div class="insight-row">
				<span class="insight-label">Session stability</span>
				<span class="insight-value">98% uptime (last 1h)</span>
			</div>
		</div>
	</section>
</div>

<style>
	.supervisor-panel {
		display: flex;
		flex-direction: column;
		gap: 0;
		flex: 1;
		min-height: 0;
		overflow-y: auto;
	}

	/* ── Section ──────────────────────────────────────────────────── */

	.panel-section {
		border-bottom: 1px solid var(--border-muted);
		padding: var(--space-md) 0;
	}

	.panel-section:last-child {
		border-bottom: none;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 0 var(--space-md) var(--space-sm);
	}

	.section-title {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: var(--text-muted);
	}

	.section-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		opacity: 0.6;
	}

	.header-controls {
		margin-left: auto;
		display: flex;
		gap: var(--space-sm);
	}

	.toggle-pill {
		font-family: var(--font-mono);
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.05em;
		padding: 2px 8px;
		border: 1px solid var(--border-default);
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.toggle-pill:hover {
		border-color: var(--text-secondary);
		color: var(--text-secondary);
	}

	.toggle-pill.active {
		background: var(--accent-green);
		border-color: var(--accent-green);
		color: #000;
	}

	.toggle-pill.autopilot.active {
		background: var(--accent-amber);
		border-color: var(--accent-amber);
	}

	/* ── Border Control ──────────────────────────────────────────── */

	.border-rows {
		display: flex;
		flex-direction: column;
	}

	.border-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-xs) var(--space-md);
	}

	.border-row:hover {
		background: var(--bg-card-hover);
	}

	.border-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		min-width: 140px;
	}

	.border-value {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	/* ── Observations ────────────────────────────────────────────── */

	.obs-list {
		display: flex;
		flex-direction: column;
	}

	.obs-row {
		border-left: 2px solid transparent;
		padding: var(--space-sm) var(--space-md);
	}

	.obs-row:hover {
		background: var(--bg-card-hover);
	}

	.obs-row.severity-critical {
		border-left-color: var(--accent-red);
	}

	.obs-row.severity-warning {
		border-left-color: var(--accent-amber);
	}

	.obs-row.severity-info {
		border-left-color: var(--supervisor-info);
	}

	.obs-main {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.obs-icon {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 700;
		color: var(--text-muted);
		flex-shrink: 0;
		min-width: 28px;
	}

	.severity-critical .obs-icon {
		color: var(--accent-red);
	}

	.severity-warning .obs-icon {
		color: var(--accent-amber);
	}

	.severity-info .obs-icon {
		color: var(--supervisor-info);
	}

	.obs-message {
		flex: 1;
		font-size: 13px;
		color: var(--text-primary);
	}

	.obs-time {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.obs-detail {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		padding: var(--space-sm) 0 0 36px;
	}

	.obs-pattern {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.obs-suggestion {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-secondary);
	}

	.obs-actions {
		display: flex;
		gap: var(--space-sm);
		margin-top: var(--space-xs);
	}

	.action-btn {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		padding: 3px 10px;
		border: 1px solid var(--border-default);
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.action-btn:hover {
		border-color: var(--text-primary);
		color: var(--text-primary);
	}

	.action-btn.primary {
		border-color: var(--accent-green);
		color: var(--accent-green);
	}

	.action-btn.primary:hover {
		background: var(--accent-green);
		color: #000;
	}

	.action-btn.danger {
		border-color: var(--accent-red);
		color: var(--accent-red);
	}

	.action-btn.danger:hover {
		background: var(--accent-red);
		color: #000;
	}

	/* ── Interventions ───────────────────────────────────────────── */

	.int-list {
		display: flex;
		flex-direction: column;
	}

	.int-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-md);
	}

	.int-row:hover {
		background: var(--bg-card-hover);
	}

	.int-time {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
		min-width: 40px;
	}

	.int-badge {
		font-family: var(--font-mono);
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.05em;
		padding: 1px 6px;
		border: 1px solid var(--text-muted);
		color: var(--text-muted);
		flex-shrink: 0;
		min-width: 48px;
		text-align: center;
	}

	.int-badge.auto {
		border-color: var(--accent-amber);
		color: var(--accent-amber);
	}

	.int-message {
		font-size: 13px;
		color: var(--text-secondary);
	}

	/* ── Insights ────────────────────────────────────────────────── */

	.insights {
		display: flex;
		flex-direction: column;
	}

	.insight-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-xs) var(--space-md);
	}

	.insight-row:hover {
		background: var(--bg-card-hover);
	}

	.insight-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.insight-value {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	/* ── Empty ───────────────────────────────────────────────────── */

	.empty-row {
		padding: var(--space-md) var(--space-md);
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		opacity: 0.5;
	}
</style>
