import { writable } from 'svelte/store';
import { isTauri } from '$lib/ws';

export const mqttConnected = writable(false);
export const instanceId = writable('');

let pollInterval: ReturnType<typeof setInterval> | null = null;

export async function startMqttPolling() {
	if (!isTauri()) return;
	const { invoke } = await import('@tauri-apps/api/core');

	async function poll() {
		try {
			const connected = await invoke<boolean>('get_mqtt_status');
			mqttConnected.set(connected);
		} catch {
			mqttConnected.set(false);
		}
		try {
			const settings = await invoke<{ instanceId: string }>('get_settings');
			instanceId.set(settings.instanceId || '');
		} catch {}
	}

	poll();
	pollInterval = setInterval(poll, 5000);
}

export function stopMqttPolling() {
	if (pollInterval) {
		clearInterval(pollInterval);
		pollInterval = null;
	}
}
