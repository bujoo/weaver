<script lang="ts">
	import MissionListItem from './MissionListItem.svelte';
	import {
		missions,
		activeMissions,
		incomingMissions,
		completedMissions,
		selectedMissionId,
		attentionCount,
		autoSelectMission,
	} from '$lib/stores/missions';
	import { mqttConnected } from '$lib/stores/mqtt';
	import { page } from '$app/stores';
	import { untrack } from 'svelte';

	let currentPath = $derived($page.url.pathname);
	let connected = $derived($mqttConnected);
	let allMissions = $derived($missions);
	let active = $derived($activeMissions);
	let incoming = $derived($incomingMissions);
	let completed = $derived($completedMissions);
	let currentSelectedId = $derived($selectedMissionId);
	let attention = $derived($attentionCount);

	let completedCollapsed = $state(true);

	// Auto-select highest priority mission when none is selected.
	// Uses $effect.pre to only run once per mission list change, with deferred write.
	let lastAutoSelectList = $state<string>('');
	$effect(() => {
		const list = allMissions; // tracked dependency
		const listKey = list.map(m => m.missionId).join(',');
		if (listKey !== lastAutoSelectList && list.length > 0) {
			lastAutoSelectList = listKey;
			const currentId = untrack(() => $selectedMissionId);
			if (!currentId || !list.some((m) => m.missionId === currentId)) {
				setTimeout(() => selectedMissionId.set(list[0].missionId), 0);
			}
		}
	});

	function selectMission(missionId: string) {
		selectedMissionId.set(missionId);
	}

	const navItems = [
		{ path: '/monitor', icon: 'M22 12h-4l-3 9L9 3l-3 9H2', label: 'Monitor' },
		{ path: '/', icon: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z', label: 'Sessions' },
		{ path: '/tasks', icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4', label: 'Tasks' },
		{ path: '/workspace', icon: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z', label: 'Workspace' },
		{ path: '/settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z', label: 'Settings' },
	];
</script>

<aside class="mission-sidebar">
	<div class="sidebar-header">
		<span class="logo-text">WEAVER</span>
		<span class="mqtt-status" title={connected ? 'MQTT connected' : 'MQTT disconnected'}>
			<span class="mqtt-dot" class:connected></span>
		</span>
	</div>

	<!-- Navigation -->
	<nav class="sidebar-nav">
		{#each navItems as item}
			<a
				href={item.path}
				class="nav-link"
				class:active={item.path === '/' ? currentPath === '/' : currentPath.startsWith(item.path)}
			>
				<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<path d={item.icon} />
				</svg>
				<span>{item.label}</span>
			</a>
		{/each}
	</nav>

	<div class="sidebar-divider"></div>

	<!-- Mission sections -->
	<div class="missions-scroll">
		<!-- Active missions -->
		{#if active.length > 0}
			<div class="section">
				<div class="section-header">
					<span class="section-title">ACTIVE</span>
					<span class="section-count">{active.length}</span>
				</div>
				{#each active as mission (mission.missionId)}
					<MissionListItem
						{mission}
						selected={currentSelectedId === mission.missionId}
						onclick={() => selectMission(mission.missionId)}
					/>
				{/each}
			</div>
		{/if}

		<!-- Incoming missions -->
		{#if incoming.length > 0}
			<div class="section">
				<div class="section-header">
					<span class="section-title">INCOMING</span>
					<span class="section-count">{incoming.length}</span>
				</div>
				{#each incoming as mission (mission.missionId)}
					<MissionListItem
						{mission}
						selected={currentSelectedId === mission.missionId}
						onclick={() => selectMission(mission.missionId)}
					/>
				{/each}
			</div>
		{/if}

		<!-- Completed missions -->
		{#if completed.length > 0}
			<div class="section">
				<button class="section-header section-toggle" onclick={() => (completedCollapsed = !completedCollapsed)} type="button">
					<span class="section-title">COMPLETED</span>
					<span class="section-count">{completed.length}</span>
					<svg
						class="chevron"
						class:collapsed={completedCollapsed}
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<polyline points="6 9 12 15 18 9" />
					</svg>
				</button>
				{#if !completedCollapsed}
					{#each completed as mission (mission.missionId)}
						<MissionListItem
							{mission}
							selected={currentSelectedId === mission.missionId}
							onclick={() => selectMission(mission.missionId)}
						/>
					{/each}
				{/if}
			</div>
		{/if}

		<!-- Empty state -->
		{#if allMissions.length === 0}
			<div class="empty-state">
				<span class="empty-text">No missions</span>
				<span class="empty-sub">Waiting for MQTT connection</span>
			</div>
		{/if}
	</div>

	<!-- Footer -->
	<div class="sidebar-footer">
		{#if attention > 0}
			<span class="attention-count" title="{attention} mission{attention !== 1 ? 's' : ''} need attention">
				{attention}
			</span>
		{/if}
		<div class="footer-spacer"></div>
		<a href="/settings" class="footer-icon" title="Settings">
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
				<path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
			</svg>
		</a>
		<span class="footer-mqtt" title={connected ? 'Connected' : 'Disconnected'}>
			<span class="mqtt-dot-sm" class:connected></span>
		</span>
	</div>
</aside>

<style>
	.mission-sidebar {
		width: var(--sidebar-width, 220px);
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		background: var(--bg-base);
		border-right: 1px solid var(--border-muted);
		height: 100vh;
		overflow: hidden;
	}

	/* Header - macOS traffic light clearance */
	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		padding-top: 40px;
		height: 60px;
		flex-shrink: 0;
	}

	.logo-text {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.12em;
		color: var(--text-secondary);
	}

	.mqtt-status {
		display: flex;
		align-items: center;
	}

	.mqtt-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--accent-red);
		transition: background 200ms;
	}

	.mqtt-dot.connected {
		background: var(--accent-green);
	}

	/* Navigation */
	.sidebar-nav {
		display: flex;
		flex-direction: column;
		padding: 0 8px;
		gap: 1px;
	}

	.nav-link {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px;
		min-height: 32px;
		color: var(--text-muted);
		text-decoration: none;
		font-size: 13px;
		transition: color var(--transition-fast), background var(--transition-fast);
		cursor: pointer;
	}

	.nav-link:hover {
		color: var(--text-secondary);
		background: var(--bg-card-hover);
	}

	.nav-link.active {
		color: var(--text-primary);
	}

	.sidebar-divider {
		height: 1px;
		background: var(--border-muted);
		margin: 8px 12px;
		flex-shrink: 0;
	}

	/* Missions scroll area */
	.missions-scroll {
		flex: 1;
		overflow-y: auto;
		min-height: 0;
	}

	/* Section */
	.section {
		margin-bottom: 4px;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 12px;
		cursor: default;
	}

	.section-toggle {
		cursor: pointer;
		width: 100%;
		background: none;
		border: none;
		color: inherit;
		font: inherit;
	}

	.section-toggle:hover {
		background: var(--bg-card-hover);
	}

	.section-title {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-muted);
	}

	.section-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
	}

	.chevron {
		margin-left: auto;
		color: var(--text-muted);
		transition: transform 150ms ease-out;
	}

	.chevron.collapsed {
		transform: rotate(-90deg);
	}

	/* Empty state */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 24px 12px;
		gap: 4px;
	}

	.empty-text {
		font-size: 12px;
		color: var(--text-muted);
	}

	.empty-sub {
		font-size: 11px;
		color: var(--text-muted);
		opacity: 0.6;
	}

	/* Footer */
	.sidebar-footer {
		display: flex;
		align-items: center;
		padding: 8px 12px;
		border-top: 1px solid var(--border-muted);
		gap: 8px;
		flex-shrink: 0;
	}

	.attention-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 18px;
		height: 18px;
		padding: 0 4px;
		font-size: 10px;
		font-weight: 600;
		font-family: var(--font-mono);
		color: var(--bg-base);
		background: var(--accent-amber);
		animation: attention-pulse 2s ease-in-out infinite;
	}

	@keyframes attention-pulse {
		0%, 100% {
			opacity: 1;
		}
		50% {
			opacity: 0.6;
		}
	}

	.footer-spacer {
		flex: 1;
	}

	.footer-icon {
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--text-muted);
		cursor: pointer;
		transition: color var(--transition-fast);
		text-decoration: none;
	}

	.footer-icon:hover {
		color: var(--text-secondary);
	}

	.footer-mqtt {
		display: flex;
		align-items: center;
	}

	.mqtt-dot-sm {
		width: 5px;
		height: 5px;
		border-radius: 50%;
		background: var(--accent-red);
		transition: background 200ms;
	}

	.mqtt-dot-sm.connected {
		background: var(--accent-green);
	}
</style>
