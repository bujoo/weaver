<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '$lib/ws';
  import PageHeader from '$lib/components/PageHeader.svelte';

  interface WeaverSettings {
    mqttHost: string;
    mqttPort: number;
    mqttUsername: string;
    mqttPassword: string;
    instanceId: string;
    workspace: string;
    workspaceMount: string;
    capacity: number;
    brainApiUrl: string;
    autoConnect: boolean;
  }

  let settings: WeaverSettings = $state({
    mqttHost: 'localhost',
    mqttPort: 1883,
    mqttUsername: 'weaver-dev',
    mqttPassword: 'weaver-dev-secret',
    instanceId: '',
    workspace: 'dev',
    workspaceMount: '',
    capacity: 2,
    brainApiUrl: 'http://localhost:8000',
    autoConnect: false,
  });

  let saving = $state(false);
  let connecting = $state(false);
  let connected = $state(false);
  let message = $state('');
  let tauriAvailable = $state(false);
  let activeTab = $state('settings');

  const tabs = [
    { id: 'settings', icon: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z', label: 'SETTINGS' },
  ];

  onMount(async () => {
    tauriAvailable = isTauri();
    if (!tauriAvailable) {
      message = 'Running in browser mode. Tauri commands unavailable.';
      return;
    }

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const loaded = await invoke<WeaverSettings>('get_settings');
      if (loaded) settings = loaded;
      connected = await invoke<boolean>('get_mqtt_status');
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  });

  async function save() {
    if (!tauriAvailable) return;
    saving = true;
    message = '';
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('save_settings_cmd', { s: settings });
      message = 'Settings saved';
      setTimeout(() => (message = ''), 2000);
    } catch (e) {
      message = `Error: ${e}`;
    }
    saving = false;
  }

  async function connect() {
    if (!tauriAvailable) {
      message = 'Cannot connect in browser mode. Use the Tauri app.';
      return;
    }
    connecting = true;
    message = '';
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('connect_mqtt', {
        host: settings.mqttHost,
        port: settings.mqttPort,
        username: settings.mqttUsername,
        password: settings.mqttPassword,
        instanceId: settings.instanceId,
        workspace: settings.workspace,
      });
      connected = true;
      message = 'Connected to MQTT';
      setTimeout(() => (message = ''), 3000);
    } catch (e) {
      message = `Connection failed: ${e}`;
    }
    connecting = false;
  }
</script>

<div class="dashboard">
  <PageHeader {tabs} {activeTab} onTabChange={(id) => (activeTab = id)} />

  <main class="grid-container">
    {#if !tauriAvailable}
      <div class="browser-warning">
        Running in browser mode. Open the Tauri desktop app to configure MQTT.
      </div>
    {/if}

    <div class="settings">
      <section>
        <h2>MQTT Connection</h2>
        <div class="field-row">
          <div class="field">
            <label for="host">Host</label>
            <input id="host" bind:value={settings.mqttHost} />
          </div>
          <div class="field field-sm">
            <label for="port">Port</label>
            <input id="port" type="number" bind:value={settings.mqttPort} />
          </div>
        </div>
        <div class="field-row">
          <div class="field">
            <label for="user">Username</label>
            <input id="user" bind:value={settings.mqttUsername} />
          </div>
          <div class="field">
            <label for="pass">Password</label>
            <input id="pass" type="password" bind:value={settings.mqttPassword} />
          </div>
        </div>
        <div class="status-row">
          <button class="btn-connect" onclick={connect} disabled={connecting || !tauriAvailable}>
            {connecting ? 'Connecting...' : connected ? 'Reconnect' : 'Connect'}
          </button>
          <span class="status-dot" class:connected></span>
          <span class="status-text">{connected ? 'Connected' : 'Disconnected'}</span>
        </div>
      </section>

      <section>
        <h2>Instance</h2>
        <div class="field-row">
          <div class="field">
            <label for="iid">Instance ID</label>
            <input id="iid" bind:value={settings.instanceId} class="mono" />
          </div>
          <div class="field field-sm">
            <label for="cap">Capacity</label>
            <input id="cap" type="number" bind:value={settings.capacity} />
          </div>
        </div>
        <div class="field">
          <label for="ws">Workspace</label>
          <input id="ws" bind:value={settings.workspace} />
        </div>
      </section>

      <section>
        <h2>Paths</h2>
        <div class="field">
          <label for="mount">Workspace Mount</label>
          <input id="mount" bind:value={settings.workspaceMount} class="mono" />
        </div>
        <div class="field">
          <label for="brain">Brain API URL</label>
          <input id="brain" bind:value={settings.brainApiUrl} class="mono" />
        </div>
      </section>

      <div class="actions">
        <button class="btn-save" onclick={save} disabled={saving || !tauriAvailable}>
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
        {#if message}
          <span class="message" class:error={message.startsWith('Error') || message.startsWith('Connection failed') || message.startsWith('Cannot') || message.startsWith('Running')}>{message}</span>
        {/if}
      </div>
    </div>
  </main>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--bg-base);
  }

  .grid-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-xl);
  }

  .settings {
    max-width: 600px;
  }

  h2 {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-bottom: 12px;
  }

  section {
    margin-bottom: 24px;
    padding-bottom: 24px;
    border-bottom: 1px solid var(--border-muted);
  }

  .field-row {
    display: flex;
    gap: 12px;
  }

  .field {
    flex: 1;
    margin-bottom: 8px;
  }

  .field-sm {
    flex: 0 0 100px;
  }

  label {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 4px;
  }

  input {
    width: 100%;
    background: var(--bg-elevated);
    border: 1px solid var(--border-muted);
    color: var(--text-primary);
    padding: 6px 8px;
    font-size: 13px;
    font-family: inherit;
    box-sizing: border-box;
  }

  input:focus {
    outline: none;
    border-color: var(--border-default);
  }

  input.mono {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
  }

  .status-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 8px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent-red);
  }

  .status-dot.connected {
    background: var(--accent-green);
  }

  .status-text {
    font-size: 11px;
    color: var(--text-muted);
  }

  .btn-connect,
  .btn-save {
    background: var(--bg-elevated);
    border: 1px solid var(--border-default);
    color: var(--text-primary);
    padding: 6px 16px;
    font-size: 12px;
    cursor: pointer;
  }

  .btn-connect:hover,
  .btn-save:hover {
    background: var(--bg-card-hover);
  }

  .btn-connect:disabled,
  .btn-save:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .message {
    font-size: 12px;
    color: var(--accent-green);
  }

  .message.error {
    color: var(--accent-amber);
  }

  .browser-warning {
    background: oklch(0.74 0.17 55 / 0.1);
    border: 1px solid oklch(0.74 0.17 55 / 0.2);
    color: var(--accent-amber);
    padding: 8px 12px;
    font-size: 12px;
    margin-bottom: 20px;
  }
</style>
