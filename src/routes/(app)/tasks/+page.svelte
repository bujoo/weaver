<script lang="ts">
  import { onMount } from 'svelte';
  import { tasks, taskCounts, initializeTaskListeners } from '$lib/stores/tasks';
  import TaskCard from '$lib/components/TaskCard.svelte';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let expandedId = $state<string | null>(null);
  let counts = $derived($taskCounts);
  let taskList = $derived($tasks);
  let activeTab = $state('queue');

  const tabs = [
    { id: 'queue', icon: '▶', label: 'QUEUE' },
    { id: 'completed', icon: '✓', label: 'COMPLETED' },
    { id: 'failed', icon: '✕', label: 'FAILED' },
  ];

  let filteredTasks = $derived(
    activeTab === 'queue'
      ? taskList.filter((t) => t.status === 'queued' || t.status === 'preparing' || t.status === 'executing')
      : activeTab === 'completed'
        ? taskList.filter((t) => t.status === 'completed')
        : taskList.filter((t) => t.status === 'failed')
  );

  onMount(() => {
    initializeTaskListeners();
  });

  function toggleExpand(key: string) {
    expandedId = expandedId === key ? null : key;
  }
</script>

<div class="dashboard">
  <PageHeader {tabs} {activeTab} onTabChange={(id) => (activeTab = id)} />

  <main class="grid-container">
    {#if filteredTasks.length === 0}
      <div class="empty">
        {#if activeTab === 'queue'}
          <p class="empty-title">No active tasks</p>
          <p class="empty-hint">Connect to Brain via MQTT in Settings, then trigger a weaver plan from Obsidian.</p>
        {:else if activeTab === 'completed'}
          <p class="empty-title">No completed tasks yet</p>
        {:else}
          <p class="empty-title">No failed tasks</p>
        {/if}
      </div>
    {:else}
      <div class="task-list">
        {#each filteredTasks as task (task.missionId + ':' + task.phaseId + ':' + task.receivedAt)}
          {@const key = task.missionId + ':' + task.phaseId + ':' + task.receivedAt}
          <TaskCard
            {task}
            expanded={expandedId === key}
            onexpand={() => toggleExpand(key)}
          />
        {/each}
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
  }
</style>
