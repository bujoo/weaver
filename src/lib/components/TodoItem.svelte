<script lang="ts">
	import LiveStream from './LiveStream.svelte';

	interface Todo {
		id: string;
		description: string;
		status: string;
	}

	interface Props {
		todo: Todo;
		label?: string;
		active?: boolean;
		missionId?: string;
		phaseId?: string;
	}

	let { todo, label = '', active = false, missionId = '', phaseId = '' }: Props = $props();

	let statusIcon = $derived(
		todo.status === 'done' || todo.status === 'completed'
			? '[x]'
			: todo.status === 'working' || todo.status === 'executing' || todo.status === 'in_progress'
				? '[>]'
				: '[ ]'
	);

	let statusLabel = $derived(
		todo.status === 'done' || todo.status === 'completed'
			? 'done'
			: todo.status === 'working' || todo.status === 'executing' || todo.status === 'in_progress'
				? 'working'
				: 'queued'
	);

	let statusClass = $derived(
		todo.status === 'done' || todo.status === 'completed'
			? 'done'
			: todo.status === 'working' || todo.status === 'executing' || todo.status === 'in_progress'
				? 'working'
				: 'queued'
	);
</script>

<div class="todo-item" class:active>
	<div class="todo-row">
		<span class="todo-icon {statusClass}">{statusIcon}</span>
		{#if label}<span class="todo-label">{label}</span>{/if}
		<span class="todo-description">{todo.description}</span>
		<span class="todo-status {statusClass}">{statusLabel}</span>
		<span class="todo-time">--</span>
	</div>

	{#if active}
		<div class="todo-active-content">
			{#if statusLabel === 'working'}
				<div class="working-hint">
					"Analyzing existing patterns..."
				</div>
			{/if}
			<LiveStream {missionId} {phaseId} todoId={todo.id} />
		</div>
	{/if}
</div>

<style>
	.todo-item {
		transition: background var(--transition-fast);
	}

	.todo-item:hover {
		background: var(--bg-card-hover);
	}

	.todo-item.active {
		background: oklch(0.72 0.15 290 / 0.04);
	}

	.todo-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-md);
		min-height: 32px;
	}

	.todo-icon {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 600;
		flex-shrink: 0;
		width: 24px;
		text-align: center;
	}

	.todo-icon.done {
		color: var(--accent-green);
	}

	.todo-icon.working {
		color: var(--accent-purple);
		animation: pulse-glow 2s ease-in-out infinite;
	}

	.todo-icon.queued {
		color: var(--task-queued);
	}

	.todo-label {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 700;
		color: var(--text-muted);
		flex-shrink: 0;
		min-width: 36px;
		letter-spacing: 0.02em;
	}

	.todo-description {
		flex: 1;
		font-size: 13px;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.todo-status {
		font-family: var(--font-mono);
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		flex-shrink: 0;
		width: 60px;
		text-align: right;
	}

	.todo-status.done {
		color: var(--accent-green);
	}

	.todo-status.working {
		color: var(--accent-purple);
	}

	.todo-status.queued {
		color: var(--task-queued);
	}

	.todo-time {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
		width: 52px;
		text-align: right;
	}

	.todo-active-content {
		padding: 0 var(--space-md) var(--space-sm);
		padding-left: calc(var(--space-md) + 24px + var(--space-sm));
	}

	.working-hint {
		font-size: 12px;
		font-style: italic;
		color: var(--text-muted);
		margin-bottom: var(--space-xs);
		padding-left: 2px;
	}
</style>
