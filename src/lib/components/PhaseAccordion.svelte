<script lang="ts">
	import TodoItem from './TodoItem.svelte';
	import InterventionInput from './InterventionInput.svelte';
	import { isTauri } from '$lib/ws';

	import type { CachedTodo } from '$lib/stores/missions';

	let starting = $state(false);
	let started = $state(false);

	let canStart = $derived(
		isTauri() && !started && !starting &&
		phase.status !== 'completed' && phase.status !== 'done' &&
		phase.status !== 'blocked'
	);

	async function handleStartPhase() {
		if (!isTauri() || starting) return;
		starting = true;
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			const sessionName = await invoke<string>('start_phase_manually', {
				missionId,
				phaseId: phase.phaseId,
			});
			console.log('[Phase] Started session:', sessionName);
			started = true;
		} catch (e) {
			console.error('[Phase] Failed to start:', e);
		}
		starting = false;
	}

	interface Phase {
		phaseId: string;
		phaseName: string;
		todoCount: number;
		status: string;
		todos?: CachedTodo[];
		blockedBy?: string[];
	}

	interface Props {
		phase: Phase;
		missionId: string;
		expanded?: boolean;
		ontoggle?: () => void;
	}

	let { phase, missionId, expanded = false, ontoggle }: Props = $props();

	// Extract phase number from phaseId (e.g. "P1" -> "1", "P3" -> "3")
	let phaseOrder = $derived(
		phase.phaseId.startsWith('P') ? phase.phaseId.slice(1) : phase.phaseId.slice(0, 4)
	);

	let statusLabel = $derived(
		phase.status === 'completed' || phase.status === 'done'
			? 'DONE'
			: phase.status === 'executing' || phase.status === 'in_progress' || phase.status === 'active'
				? 'EXECUTING'
				: 'QUEUED'
	);

	let statusClass = $derived(
		phase.status === 'completed' || phase.status === 'done'
			? 'done'
			: phase.status === 'executing' || phase.status === 'in_progress' || phase.status === 'active'
				? 'executing'
				: 'queued'
	);

	// Use real todo data from state cache, fall back to placeholder if empty
	let todos = $derived(() => {
		if (phase.todos && phase.todos.length > 0) {
			return phase.todos
				.map((t) => ({
					id: t.todo_id,
					description: t.description,
					status: t.status === 'completed' || t.status === 'done' ? 'done'
						: t.status === 'executing' || t.status === 'in_progress' || t.status === 'running' ? 'working'
						: 'queued',
				}))
				.sort((a, b) => {
					// Sort by numeric part of ID: P1.1 < P1.2 < P1.10
					const numA = parseFloat(a.id.replace(/^P/, '')) || 0;
					const numB = parseFloat(b.id.replace(/^P/, '')) || 0;
					return numA - numB;
				});
		}
		// Fallback: generate placeholders from todoCount
		return Array.from({ length: phase.todoCount }, (_, i) => ({
			id: `${phase.phaseId}-todo-${i}`,
			description: `Todo ${i + 1}`,
			status: 'queued' as string,
		}));
	});

	let completedCount = $derived(
		todos().filter((t) => t.status === 'done' || t.status === 'completed').length
	);

	let activeTodo = $derived(
		todos().find((t) => t.status === 'working' || t.status === 'executing' || t.status === 'in_progress')
	);
</script>

<div class="phase-accordion" class:expanded>
	<button class="phase-header" onclick={ontoggle} type="button">
		<div class="phase-title-row">
			<svg
				class="chevron"
				class:expanded
				width="12"
				height="12"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<polyline points="9 18 15 12 9 6" />
			</svg>
			<span class="phase-id-badge">{phase.phaseId}</span>
			<span class="phase-name">{phase.phaseName}</span>
		</div>
		<div class="phase-meta">
			{#if canStart}
				<button class="btn-start" onclick={(e) => { e.stopPropagation(); handleStartPhase(); }} disabled={starting} type="button">
					{starting ? 'Starting...' : 'Start'}
				</button>
			{/if}
			{#if started}
				<span class="started-badge">RUNNING</span>
			{/if}
			<span class="phase-progress">{completedCount}/{phase.todoCount}</span>
			<span class="status-badge {statusClass}">{statusLabel}</span>
		</div>
	</button>

	{#if expanded}
		<div class="phase-body">
			<!-- Column headers -->
			<div class="todo-header-row">
				<span class="col-todo">Todo</span>
				<span class="col-status">Status</span>
				<span class="col-time">Time</span>
			</div>

			<div class="todo-separator"></div>

			<!-- Todo items -->
			{#if todos().length === 0 && phase.todoCount === 0}
				<div class="phase-empty-state">
					<span class="empty-text">Tasks not yet decomposed</span>
					{#if phase.blockedBy && phase.blockedBy.length > 0}
						<span class="empty-blocked">Blocked by: {phase.blockedBy.join(', ')}</span>
					{/if}
				</div>
			{:else}
				{#each todos() as todo, i (todo.id)}
					<TodoItem
						{todo}
						label="P{phaseOrder}.{i + 1}"
						active={activeTodo?.id === todo.id}
						{missionId}
						phaseId={phase.phaseId}
					/>
				{/each}
			{/if}

			<!-- Intervention controls for active phases -->
			{#if statusClass === 'executing' && activeTodo}
				<InterventionInput
					{missionId}
					phaseId={phase.phaseId}
					todoId={activeTodo.id}
				/>
			{/if}
		</div>
	{/if}
</div>

<style>
	.phase-accordion {
		border: 1px solid var(--border-muted);
		background: var(--bg-card);
		margin-bottom: 2px;
		transition: border-color var(--transition-fast);
	}

	.phase-accordion:hover {
		border-color: var(--border-default);
	}

	.phase-accordion.expanded {
		border-color: var(--border-default);
	}

	.phase-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		min-height: 40px;
		background: none;
		border: none;
		cursor: pointer;
		color: inherit;
		font: inherit;
		transition: background var(--transition-fast);
	}

	.phase-header:hover {
		background: var(--bg-card-hover);
	}

	.phase-title-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		min-width: 0;
		flex: 1;
	}

	.chevron {
		flex-shrink: 0;
		color: var(--text-muted);
		transition: transform var(--transition-normal);
	}

	.chevron.expanded {
		transform: rotate(90deg);
	}

	.phase-id-badge {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 700;
		color: #a78bfa;
		background: rgba(167, 139, 250, 0.1);
		padding: 1px 6px;
		border-radius: 2px;
		letter-spacing: 0.02em;
		flex-shrink: 0;
	}

	.phase-name {
		font-size: 13px;
		font-weight: 600;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.phase-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		flex-shrink: 0;
	}

	.phase-progress {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.status-badge {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		padding: 2px 8px;
		border: 1px solid;
	}

	.status-badge.done {
		color: var(--accent-green);
		border-color: var(--accent-green);
		background: rgba(0, 255, 136, 0.06);
	}

	.status-badge.executing {
		color: var(--accent-purple);
		border-color: var(--accent-purple);
		background: rgba(121, 40, 202, 0.08);
		animation: pulse-glow 2s ease-in-out infinite;
	}

	.status-badge.queued {
		color: var(--task-queued);
		border-color: var(--task-queued);
		background: rgba(68, 68, 68, 0.1);
	}

	.btn-start {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		padding: 2px 10px;
		border: 1px solid var(--accent-green);
		background: transparent;
		color: var(--accent-green);
		cursor: pointer;
	}

	.btn-start:hover {
		background: var(--accent-green);
		color: #000;
	}

	.btn-start:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.started-badge {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		padding: 2px 8px;
		border: 1px solid var(--accent-purple);
		color: var(--accent-purple);
		background: rgba(121, 40, 202, 0.08);
	}

	.phase-body {
		border-top: 1px solid var(--border-muted);
	}

	.todo-header-row {
		display: flex;
		align-items: center;
		padding: var(--space-xs) var(--space-md);
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.06em;
		color: var(--text-muted);
		text-transform: uppercase;
	}

	.col-todo {
		flex: 1;
		padding-left: calc(24px + var(--space-sm));
	}

	.col-status {
		width: 60px;
		text-align: right;
	}

	.col-time {
		width: 52px;
		text-align: right;
	}

	.todo-separator {
		height: 1px;
		background: var(--border-muted);
		margin: 0 var(--space-md);
	}

	.phase-empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: var(--space-md) var(--space-lg);
		color: var(--text-muted);
	}

	.empty-text {
		font-family: var(--font-mono);
		font-size: 11px;
		letter-spacing: 0.04em;
	}

	.empty-blocked {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--task-queued);
	}
</style>
