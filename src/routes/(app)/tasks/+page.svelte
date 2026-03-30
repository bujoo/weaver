<script lang="ts">
  import { onMount } from 'svelte';
  import { tasks, taskCounts, initializeTaskListeners } from '$lib/stores/tasks';
  import TaskCard from '$lib/components/TaskCard.svelte';

  let expandedId = $state<string | null>(null);
  let counts = $derived($taskCounts);
  let taskList = $derived($tasks);

  onMount(() => {
    initializeTaskListeners();
  });

  function toggleExpand(phaseId: string) {
    expandedId = expandedId === phaseId ? null : phaseId;
  }
</script>

<div class="page">
  <div class="header">
    <h1>Tasks</h1>
    <div class="counts">
      {#if counts.executing > 0}
        <span class="count executing">{counts.executing} executing</span>
      {/if}
      {#if counts.queued > 0}
        <span class="count queued">{counts.queued} queued</span>
      {/if}
      {#if counts.completed > 0}
        <span class="count completed">{counts.completed} done</span>
      {/if}
    </div>
  </div>

  {#if taskList.length === 0}
    <div class="empty">
      <p>No tasks assigned yet.</p>
      <p class="hint">Connect to Brain via MQTT in Settings, then trigger a weaver plan from Obsidian.</p>
    </div>
  {:else}
    <div class="task-list">
      {#each taskList as task (task.phaseId)}
        <TaskCard
          {task}
          expanded={expandedId === task.phaseId}
          onexpand={() => toggleExpand(task.phaseId)}
        />
      {/each}
    </div>
  {/if}
</div>

<style>
  .page {
    padding: 24px;
  }

  .header {
    display: flex;
    align-items: center;
    gap: 16px;
    margin-bottom: 16px;
  }

  h1 {
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-secondary, #888);
  }

  .counts {
    display: flex;
    gap: 8px;
  }

  .count {
    font-size: 11px;
    font-family: var(--font-mono);
    padding: 2px 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .count.executing {
    color: var(--task-executing);
    border-color: var(--task-executing);
  }

  .count.queued {
    color: var(--text-muted);
  }

  .count.completed {
    color: var(--task-completed);
  }

  .empty {
    padding: 48px 0;
    text-align: center;
  }

  .empty p {
    font-size: 13px;
    color: var(--text-muted, #666);
  }

  .empty .hint {
    font-size: 12px;
    margin-top: 8px;
    color: var(--text-muted, #555);
  }

  .task-list {
    display: flex;
    flex-direction: column;
  }
</style>
