<script lang="ts">
  import type { TaskQueueEntry } from '$lib/stores/tasks';
  import { todoStatuses } from '$lib/stores/tasks';

  interface Props {
    task: TaskQueueEntry;
    expanded?: boolean;
    onexpand?: () => void;
  }

  let { task, expanded = false, onexpand }: Props = $props();
  let statuses = $derived($todoStatuses);

  let completedCount = $derived(
    task.todos.filter((id) => statuses.get(id) === 'completed').length
  );

  let elapsed = $derived(() => {
    const start = new Date(task.receivedAt).getTime();
    const diff = Date.now() - start;
    const mins = Math.floor(diff / 60000);
    if (mins < 1) return 'just now';
    if (mins < 60) return `${mins}m`;
    return `${Math.floor(mins / 60)}h ${mins % 60}m`;
  });

  const statusColors: Record<string, string> = {
    queued: 'var(--task-queued)',
    preparing: 'var(--task-preparing)',
    executing: 'var(--task-executing)',
    completed: 'var(--task-completed)',
    failed: 'var(--task-failed)',
  };
</script>

<button class="task-card" class:expanded onclick={onexpand}>
  <div class="task-header">
    <span class="status-dot" style="background: {statusColors[task.status] || '#444'}"></span>
    <span class="phase-name">{task.phaseName}</span>
    <span class="todo-count">{completedCount}/{task.todos.length}</span>
    <span class="status-label">{task.status}</span>
    <span class="elapsed">{elapsed()}</span>
  </div>

  {#if expanded}
    <div class="task-body">
      <div class="mission-id">{task.missionId.slice(0, 8)}</div>
      <div class="todo-list">
        {#each task.todos as todoId}
          {@const todoStatus = statuses.get(todoId) || 'pending'}
          <div class="todo-item">
            <span
              class="todo-dot"
              style="background: {todoStatus === 'completed'
                ? 'var(--task-completed)'
                : todoStatus === 'in_progress'
                  ? 'var(--task-executing)'
                  : todoStatus === 'failed'
                    ? 'var(--task-failed)'
                    : '#333'}"
            ></span>
            <span class="todo-id">{todoId}</span>
            <span class="todo-status">{todoStatus}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</button>

<style>
  .task-card {
    display: block;
    width: 100%;
    background: var(--bg-card, #111);
    border: 1px solid rgba(255, 255, 255, 0.06);
    padding: 0;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
    margin-bottom: 2px;
  }

  .task-card:hover {
    border-color: rgba(255, 255, 255, 0.12);
  }

  .task-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    font-size: 13px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .phase-name {
    flex: 1;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .todo-count {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .status-label {
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .elapsed {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .task-body {
    padding: 0 12px 12px;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
  }

  .mission-id {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    margin: 8px 0;
  }

  .todo-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .todo-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
    font-size: 12px;
  }

  .todo-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .todo-id {
    font-family: var(--font-mono);
    color: var(--text-secondary);
    flex: 1;
  }

  .todo-status {
    font-size: 10px;
    color: var(--text-muted);
    text-transform: uppercase;
  }
</style>
