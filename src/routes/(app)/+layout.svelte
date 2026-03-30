<script lang="ts">
	import { onMount } from 'svelte';
	import { initializeSessionListeners, sessions } from '$lib/stores/sessions';
	import { getSessions } from '$lib/api';
	import { loadDemoDataIfActive } from '$lib/demo';
	import { checkForUpdates } from '$lib/updater';
	import { isTauri } from '$lib/ws';
	import { page } from '$app/stores';

	onMount(async () => {
		const demoActive = loadDemoDataIfActive();

		if (isTauri()) {
			await initializeSessionListeners();

			if (!demoActive) {
				const initialSessions = await getSessions();
				sessions.set(initialSessions);
			}

			checkForUpdates();
		}
	});

	let currentPath = $derived($page.url.pathname);

	const navItems = [
		{ path: '/', icon: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z', label: 'Sessions' },
		{ path: '/tasks', icon: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4', label: 'Tasks' },
		{ path: '/workspace', icon: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z', label: 'Workspace' },
		{ path: '/settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z', label: 'Settings' },
	];
</script>

<div class="app-shell">
	<nav class="sidebar">
		{#each navItems as item}
			<a
				href={item.path}
				class="nav-item"
				class:active={item.path === '/' ? currentPath === '/' : currentPath.startsWith(item.path)}
				title={item.label}
			>
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<path d={item.icon} />
				</svg>
			</a>
		{/each}
	</nav>
	<main class="content">
		<slot />
	</main>
</div>

<style>
	.app-shell {
		display: flex;
		height: 100vh;
		overflow: hidden;
	}

	.sidebar {
		width: 44px;
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding-top: 12px;
		gap: 4px;
		border-right: 1px solid rgba(255, 255, 255, 0.06);
		background: var(--bg-base, #000);
	}

	.nav-item {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		color: var(--text-muted, #666);
		text-decoration: none;
		transition: color 100ms;
	}

	.nav-item:hover {
		color: var(--text-secondary, #888);
	}

	.nav-item.active {
		color: var(--text-primary, #fff);
	}

	.content {
		flex: 1;
		overflow-y: auto;
		min-width: 0;
	}
</style>
