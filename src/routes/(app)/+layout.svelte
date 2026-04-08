<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { goto } from '$app/navigation';
	import { initializeSessionListeners, sessions } from '$lib/stores/sessions';
	import { getSessions } from '$lib/api';
	import { loadDemoDataIfActive } from '$lib/demo';
	import { checkForUpdates } from '$lib/updater';
	import { isTauri } from '$lib/ws';
	import { startMqttPolling, stopMqttPolling } from '$lib/stores/mqtt';
	import { initRegistryListener } from '$lib/stores/workspace';
	import { initializeTaskListeners } from '$lib/stores/tasks';
	import { missions, selectedMissionId, startPhasePolling } from '$lib/stores/missions';
	import { initActivityListeners } from '$lib/stores/activity';
	import { registerShortcuts } from '$lib/shortcuts';
	import MissionSidebar from '$lib/components/MissionSidebar.svelte';
	import WeavySidebar from '$lib/components/WeavySidebar.svelte';
	import KeyboardShortcutOverlay from '$lib/components/KeyboardShortcutOverlay.svelte';
	import FuzzyMissionSearch from '$lib/components/FuzzyMissionSearch.svelte';

	let showShortcuts = $state(false);
	let showMissionSearch = $state(false);
	let allMissions = $derived($missions);

	let cleanupShortcuts: (() => void) | null = null;
	let cleanupPhasePolling: (() => void) | null = null;

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
			cleanupPhasePolling = startPhasePolling();
			initActivityListeners();
		}

		cleanupShortcuts = registerShortcuts({
			onSwitchMission: (index: number) => {
				const list = get(missions);
				if (list[index]) {
					selectedMissionId.set(list[index].missionId);
				}
			},
			onJumpToMission: () => {
				showMissionSearch = true;
			},
			onOpenVSCode: () => {
				// VS Code integration handled at mission level
			},
			onToggleStream: () => {
				// Stream toggle handled at mission view level
			},
			onBackToControl: () => {
				if (!showShortcuts && !showMissionSearch) {
					selectedMissionId.set(null);
					goto('/');
				}
			},
			onShowShortcuts: () => {
				showShortcuts = !showShortcuts;
			},
			onSendGuidance: () => {
				// Guidance submission handled by the focused input component
			},
		});
	});

	onDestroy(() => {
		stopMqttPolling();
		cleanupShortcuts?.();
		cleanupPhasePolling?.();
	});

	function handleMissionSelect(missionId: string) {
		selectedMissionId.set(missionId);
		showMissionSearch = false;
	}
</script>

<div class="app-shell">
	<MissionSidebar />
	<main class="content">
		<slot />
	</main>
	<WeavySidebar />
</div>

{#if showShortcuts}
	<KeyboardShortcutOverlay onclose={() => (showShortcuts = false)} />
{/if}

{#if showMissionSearch}
	<FuzzyMissionSearch
		missions={allMissions}
		onselect={handleMissionSelect}
		onclose={() => (showMissionSearch = false)}
	/>
{/if}

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
