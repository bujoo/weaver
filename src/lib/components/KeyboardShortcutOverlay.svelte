<script lang="ts">
	import { SHORTCUTS, type ShortcutEntry } from '$lib/shortcuts';

	interface Props {
		onclose: () => void;
	}

	let { onclose }: Props = $props();

	const categories = ['Navigation', 'Actions', 'Views'] as const;

	function shortcutsByCategory(cat: string): ShortcutEntry[] {
		return SHORTCUTS.filter((s) => s.category === cat);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			onclose();
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="overlay-backdrop"
	onclick={handleBackdropClick}
	role="dialog"
	aria-modal="true"
	aria-label="Keyboard shortcuts"
>
	<div class="overlay-panel">
		<div class="overlay-header">
			<span class="overlay-title">KEYBOARD SHORTCUTS</span>
		</div>

		<div class="overlay-body">
			{#each categories as category}
				<div class="shortcut-category">
					<span class="category-label">{category.toUpperCase()}</span>
					{#each shortcutsByCategory(category) as shortcut}
						<div class="shortcut-row">
							<kbd class="shortcut-key">{shortcut.key}</kbd>
							<span class="shortcut-desc">{shortcut.description}</span>
						</div>
					{/each}
				</div>
			{/each}
		</div>

		<div class="overlay-footer">
			<span class="footer-hint">Press Escape to close</span>
		</div>
	</div>
</div>

<style>
	.overlay-backdrop {
		position: fixed;
		inset: 0;
		z-index: 9999;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.8);
		animation: fade-in 100ms linear;
	}

	.overlay-panel {
		width: 100%;
		max-width: 400px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		display: flex;
		flex-direction: column;
		max-height: 80vh;
		animation: scale-in 100ms linear;
	}

	.overlay-header {
		display: flex;
		align-items: center;
		padding: var(--space-md) var(--space-lg);
		border-bottom: 1px solid var(--border-muted);
	}

	.overlay-title {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-secondary);
	}

	.overlay-body {
		padding: var(--space-lg);
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.shortcut-category {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.category-label {
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: var(--text-muted);
		margin-bottom: var(--space-xs);
	}

	.shortcut-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
		padding: var(--space-xs) 0;
	}

	.shortcut-key {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-primary);
		background: var(--bg-card);
		border: 1px solid var(--border-muted);
		padding: 2px 8px;
		min-width: 80px;
		text-align: center;
		white-space: nowrap;
	}

	.shortcut-desc {
		font-size: 13px;
		color: var(--text-secondary);
		flex: 1;
		text-align: right;
	}

	.overlay-footer {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-md) var(--space-lg);
		border-top: 1px solid var(--border-muted);
	}

	.footer-hint {
		font-size: 11px;
		color: var(--text-muted);
	}
</style>
