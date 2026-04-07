<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { initializeSessionListeners, sessions } from '$lib/stores/sessions';
	import { getSessions } from '$lib/api';
	import { loadDemoDataIfActive } from '$lib/demo';
	import { checkForUpdates } from '$lib/updater';
	import { isTauri } from '$lib/ws';
	import { startMqttPolling, stopMqttPolling } from '$lib/stores/mqtt';
	import { initRegistryListener } from '$lib/stores/workspace';
	import { initializeTaskListeners } from '$lib/stores/tasks';
	import MissionSidebar from '$lib/components/MissionSidebar.svelte';

	onMount(async () => {
		const demoActive = loadDemoDataIfActive();

		if (isTauri()) {
			await initializeSessionListeners();
			await initRegistryListener();
			await initializeTaskListeners();

			if (!demoActive) {
				const initialSessions = await getSessions();
				sessions.set(initialSessions);
			}

			checkForUpdates();
			startMqttPolling();
		}
	});

	onDestroy(() => {
		stopMqttPolling();
	});
</script>

<div class="app-shell">
	<MissionSidebar />
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

	.content {
		flex: 1;
		overflow-y: auto;
		min-width: 0;
	}
</style>
