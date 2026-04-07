<script lang="ts">
	import { activeMissions } from '$lib/stores/missions';
	import { mqttConnected } from '$lib/stores/mqtt';
	import { sessions } from '$lib/stores/sessions';

	let active = $derived($activeMissions);
	let connected = $derived($mqttConnected);
	let sessionList = $derived($sessions);

	let capacityText = $derived(
		active.length === 0 ? 'idle' : `${active.length} active`
	);

	let mqttText = $derived(connected ? 'connected' : 'disconnected');
</script>

<div class="device-status">
	<div class="status-item">
		<span class="status-label">CAPACITY</span>
		<span class="status-value">{capacityText}</span>
	</div>
	<div class="status-sep"></div>
	<div class="status-item">
		<span class="status-label">MQTT</span>
		<span class="status-value" class:ok={connected} class:err={!connected}>{mqttText}</span>
	</div>
	<div class="status-sep"></div>
	<div class="status-item">
		<span class="status-label">SUPERVISOR</span>
		<span class="status-value ok">active</span>
	</div>
	<div class="status-sep"></div>
	<div class="status-item">
		<span class="status-label">SESSIONS</span>
		<span class="status-value">{sessionList.length}</span>
	</div>
	<div class="status-sep"></div>
	<div class="status-item">
		<span class="status-label">UPTIME</span>
		<span class="status-value">--</span>
	</div>
</div>

<style>
	.device-status {
		display: flex;
		align-items: center;
		gap: 0;
		padding: 8px 0;
		border-top: 1px solid var(--border-muted);
		flex-wrap: wrap;
	}

	.status-item {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 0 12px;
	}

	.status-label {
		font-family: var(--font-mono);
		font-size: 9px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-muted);
	}

	.status-value {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-secondary);
	}

	.status-value.ok {
		color: var(--accent-green);
	}

	.status-value.err {
		color: var(--accent-red);
	}

	.status-sep {
		width: 1px;
		height: 12px;
		background: var(--border-muted);
		flex-shrink: 0;
	}
</style>
