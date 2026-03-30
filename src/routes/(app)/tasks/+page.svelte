<script lang="ts">
  import { onMount } from 'svelte';
  import { tasks, taskCounts, availablePhases, initializeTaskListeners, acceptPhase } from '$lib/stores/tasks';
  import TaskCard from '$lib/components/TaskCard.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let expandedId = $state<string | null>(null);
  let counts = $derived($taskCounts);
  let taskList = $derived($tasks);
  let available = $derived($availablePhases);
  let activeTab = $state('my-tasks');
  let accepting = $state<string | null>(null);

  const tabs = [
    { id: 'my-tasks', icon: '▶', label: 'MY TASKS' },
    { id: 'available', icon: '◈', label: 'AVAILABLE' },
    { id: 'fleet', icon: '⊞', label: 'FLEET' },
  ];

  onMount(() => {
    initializeTaskListeners();
  });

  function toggleExpand(key: string) {
    expandedId = expandedId === key ? null : key;
  }

  async function handleAccept(missionId: string, phaseId: string) {
    accepting = `${missionId}:${phaseId}`;
    await acceptPhase(missionId, phaseId);
    accepting = null;
  }
</script>

<div class="dashboard">
  <PageHeader {tabs} {activeTab} onTabChange={(id) => (activeTab = id)} />

  <main class="grid-container">
    {#if activeTab === 'my-tasks'}
      {#if taskList.length === 0}
        <div class="empty">
          <p class="empty-title">No active tasks</p>
          <p class="empty-hint">Tasks appear here when Brain assigns work to this device, or when you accept a phase from Available.</p>
        </div>
      {:else}
        <div class="task-list">
          {#each taskList as task (task.missionId + ':' + task.phaseId + ':' + task.receivedAt)}
            {@const key = task.missionId + ':' + task.phaseId + ':' + task.receivedAt}
            <TaskCard
              {task}
              expanded={expandedId === key}
              onexpand={() => toggleExpand(key)}
            />
          {/each}
        </div>
      {/if}

    {:else if activeTab === 'available'}
      {#if available.length === 0}
        <div class="empty">
          <p class="empty-title">No available phases</p>
          <p class="empty-hint">When a mission needs human involvement, phases appear here for you to accept.</p>
        </div>
      {:else}
        <div class="task-list">
          {#each available as phase (phase.missionId + ':' + phase.phaseId)}
            <div class="available-card">
              <div class="available-header">
                <span class="phase-name">{phase.phaseName}</span>
                <span class="todo-count">{phase.todoCount} todos</span>
                <span class="mission-badge">{phase.missionTitle}</span>
              </div>
              <div class="available-meta">
                <span class="mono">{phase.missionId.slice(0, 8)}</span>
                <span class="phase-id">{phase.phaseId}</span>
                <span class="status-label">{phase.status}</span>
              </div>
              <button
                class="btn-accept"
                onclick={() => handleAccept(phase.missionId, phase.phaseId)}
                disabled={accepting === `${phase.missionId}:${phase.phaseId}`}
              >
                {accepting === `${phase.missionId}:${phase.phaseId}` ? 'Accepting...' : 'Accept'}
              </button>
            </div>
          {/each}
        </div>
      {/if}

    {:else if activeTab === 'fleet'}
      <div class="empty">
        <p class="empty-title">Fleet monitoring</p>
        <p class="empty-hint">Connected workers and their heartbeat status will appear here.</p>
      </div>
    {/if}
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

  .empty {
    padding: 48px 0;
    text-align: center;
  }

  .empty-title {
    font-family: var(--font-pixel, monospace);
    font-size: 16px;
    color: var(--text-muted, #666);
    margin-bottom: 8px;
  }

  .empty-hint {
    font-size: 12px;
    color: var(--text-muted, #555);
  }

  .task-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .available-card {
    background: var(--bg-card, #111);
    border: 1px solid rgba(255, 255, 255, 0.06);
    padding: 12px;
  }

  .available-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .phase-name {
    color: var(--text-primary);
    font-size: 13px;
    flex: 1;
  }

  .todo-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .mission-badge {
    font-size: 10px;
    color: var(--text-muted);
    padding: 1px 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 200px;
  }

  .available-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .mono { font-family: var(--font-mono); }
  .phase-id { font-family: var(--font-mono); }
  .status-label { text-transform: uppercase; letter-spacing: 0.05em; }

  .btn-accept {
    background: rgba(0, 255, 136, 0.1);
    border: 1px solid var(--accent-green, #00ff88);
    color: var(--accent-green, #00ff88);
    padding: 4px 16px;
    font-size: 11px;
    cursor: pointer;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .btn-accept:hover {
    background: rgba(0, 255, 136, 0.2);
  }

  .btn-accept:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
