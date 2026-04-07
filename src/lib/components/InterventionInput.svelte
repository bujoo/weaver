<script lang="ts">
	interface Props {
		missionId: string;
		phaseId: string;
		todoId: string;
	}

	let { missionId, phaseId, todoId }: Props = $props();

	type ActionMode = 'btw' | 'guidance' | null;
	let activeAction: ActionMode = $state(null);
	let inputText = $state('');

	function handleAction(action: string) {
		if (action === 'btw') {
			activeAction = activeAction === 'btw' ? null : 'btw';
			inputText = '';
		} else if (action === 'guidance') {
			activeAction = activeAction === 'guidance' ? null : 'guidance';
			inputText = '';
		} else {
			// For other actions, just log for now
			console.log(`[Intervention] ${action}`, { missionId, phaseId, todoId });
		}
	}

	function submit() {
		if (!inputText.trim() || !activeAction) return;
		console.log(`[Intervention] ${activeAction}:`, inputText, { missionId, phaseId, todoId });
		inputText = '';
		activeAction = null;
	}

	function handleKeydown(e: KeyboardEvent) {
		if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
			e.preventDefault();
			submit();
		}
	}

	// Auto-grow textarea
	function autoGrow(node: HTMLTextAreaElement) {
		function resize() {
			node.style.height = 'auto';
			node.style.height = Math.min(node.scrollHeight, 160) + 'px';
		}
		node.addEventListener('input', resize);
		resize();
		return {
			destroy() {
				node.removeEventListener('input', resize);
			},
		};
	}

	const actions = [
		{ key: 'btw', label: 'SEND /BTW' },
		{ key: 'guidance', label: 'SEND GUIDANCE' },
		{ key: 'vscode', label: 'OPEN VS CODE' },
		{ key: 'retry', label: 'RETRY TODO' },
		{ key: 'skip', label: 'SKIP TODO' },
		{ key: 'attach', label: 'ATTACH TO SESSION' },
	];
</script>

<div class="intervention">
	<div class="intervention-header">
		<span class="intervention-label">ACTIONS</span>
	</div>
	<div class="actions-row">
		{#each actions as action}
			<button
				class="action-btn"
				class:active={activeAction === action.key}
				onclick={() => handleAction(action.key)}
				type="button"
			>
				{action.label}
			</button>
		{/each}
	</div>

	{#if activeAction}
		<div class="input-area">
			<textarea
				class="guidance-input"
				placeholder={activeAction === 'btw'
					? 'Type /btw message for Claude Code...'
					: 'Type guidance for Claude Code...'}
				bind:value={inputText}
				onkeydown={handleKeydown}
				use:autoGrow
				rows="1"
			></textarea>
			<div class="input-footer">
				<span class="hint">Cmd+Enter to send</span>
				<button class="send-btn" onclick={submit} type="button" disabled={!inputText.trim()}>
					SEND
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.intervention {
		border: 1px solid var(--border-muted);
		background: var(--bg-card);
		margin-top: var(--space-md);
	}

	.intervention-header {
		padding: var(--space-xs) var(--space-sm);
		border-bottom: 1px solid var(--border-muted);
	}

	.intervention-label {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-muted);
	}

	.actions-row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-xs);
		padding: var(--space-sm);
	}

	.action-btn {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 500;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding: 4px 10px;
		color: var(--text-muted);
		background: transparent;
		border: 1px solid var(--border-muted);
		cursor: pointer;
		transition: color var(--transition-fast), border-color var(--transition-fast),
			background var(--transition-fast);
	}

	.action-btn:hover {
		color: var(--text-secondary);
		border-color: var(--border-default);
	}

	.action-btn.active {
		color: var(--text-primary);
		border-color: var(--text-primary);
		background: var(--bg-card-hover);
	}

	.input-area {
		padding: 0 var(--space-sm) var(--space-sm);
	}

	.guidance-input {
		width: 100%;
		min-height: 32px;
		max-height: 160px;
		padding: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 12px;
		line-height: 1.5;
		color: var(--text-primary);
		background: var(--bg-base);
		border: 1px solid var(--border-muted);
		resize: none;
		outline: none;
		transition: border-color var(--transition-fast);
	}

	.guidance-input::placeholder {
		color: var(--text-muted);
	}

	.guidance-input:focus {
		border-color: var(--border-default);
	}

	.input-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-top: var(--space-xs);
	}

	.hint {
		font-size: 10px;
		color: var(--text-muted);
		font-family: var(--font-mono);
	}

	.send-btn {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.04em;
		padding: 4px 12px;
		color: var(--bg-base);
		background: var(--text-primary);
		border: none;
		cursor: pointer;
		transition: opacity var(--transition-fast);
	}

	.send-btn:hover {
		opacity: 0.85;
	}

	.send-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}
</style>
