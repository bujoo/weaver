<script lang="ts">
	import {
		latestUrgentObservation,
		type SupervisorObservation,
	} from '$lib/stores/supervisor';

	let obs = $derived($latestUrgentObservation);

	function handleApply() {
		if (!obs) return;
		console.log('[Supervisor] Apply fix from bar:', obs.type, obs.suggestion);
	}

	function handleDismiss() {
		// For now just log; could remove from observations store
		console.log('[Supervisor] Dismissed bar observation');
	}

	function severityLabel(severity: SupervisorObservation['severity']): string {
		if (severity === 'critical') return 'CRITICAL';
		if (severity === 'warning') return 'WARNING';
		return 'INFO';
	}
</script>

{#if obs}
	<div class="supervisor-bar" class:critical={obs.severity === 'critical'} class:warning={obs.severity === 'warning'}>
		<span class="bar-prefix">SUPERVISOR:</span>
		<span class="bar-message">{obs.message}</span>
		{#if obs.suggestion}
			<button class="bar-btn apply" onclick={handleApply}>Apply</button>
		{/if}
		<button class="bar-btn dismiss" onclick={handleDismiss}>Dismiss</button>
	</div>
{/if}

<style>
	.supervisor-bar {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		height: 40px;
		padding: 0 var(--space-md);
		background: var(--bg-elevated);
		border-top: 1px solid var(--border-muted);
		flex-shrink: 0;
	}

	.supervisor-bar.warning {
		border-top-color: var(--accent-amber);
	}

	.supervisor-bar.critical {
		border-top-color: var(--accent-red);
	}

	.bar-prefix {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.08em;
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.supervisor-bar.warning .bar-prefix {
		color: var(--accent-amber);
	}

	.supervisor-bar.critical .bar-prefix {
		color: var(--accent-red);
	}

	.bar-message {
		flex: 1;
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.bar-btn {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		padding: 3px 10px;
		border: 1px solid var(--border-default);
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		flex-shrink: 0;
		transition: all var(--transition-fast);
	}

	.bar-btn:hover {
		border-color: var(--text-primary);
		color: var(--text-primary);
	}

	.bar-btn.apply {
		border-color: var(--accent-green);
		color: var(--accent-green);
	}

	.bar-btn.apply:hover {
		background: var(--accent-green);
		color: #000;
	}

	.bar-btn.dismiss {
		color: var(--text-muted);
	}

	.bar-btn.dismiss:hover {
		color: var(--text-secondary);
		border-color: var(--text-secondary);
	}
</style>
