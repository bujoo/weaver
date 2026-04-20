<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';

	interface Props {
		missions: UnifiedMission[];
		onselect: (missionId: string) => void;
		onclose: () => void;
	}

	let { missions, onselect, onclose }: Props = $props();

	let query = $state('');
	let selectedIndex = $state(0);
	let inputEl: HTMLInputElement | undefined = $state();

	const filtered = $derived.by(() => {
		if (!query.trim()) return missions.slice(0, 10);
		const q = query.toLowerCase();
		return missions
			.filter((m) => fuzzyMatch(m.title.toLowerCase(), q))
			.slice(0, 10);
	});

	// Reset selection when results change
	$effect(() => {
		if (filtered.length > 0 && selectedIndex >= filtered.length) {
			selectedIndex = 0;
		}
	});

	// Auto-focus input on mount
	$effect(() => {
		if (inputEl) {
			inputEl.focus();
		}
	});

	function fuzzyMatch(text: string, pattern: string): boolean {
		let pi = 0;
		for (let ti = 0; ti < text.length && pi < pattern.length; ti++) {
			if (text[ti] === pattern[pi]) {
				pi++;
			}
		}
		return pi === pattern.length;
	}

	function statusLabel(status: UnifiedMission['status']): string {
		return status;
	}

	function statusClass(status: UnifiedMission['status']): string {
		switch (status) {
			case 'executing': return 'status-executing';
			case 'ready': return 'status-ready';
			case 'incoming': return 'status-incoming';
			case 'validating': return 'status-validating';
			case 'completed': return 'status-completed';
			case 'failed': return 'status-failed';
			default: return '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			e.preventDefault();
			e.stopPropagation();
			onclose();
			return;
		}

		if (e.key === 'ArrowDown') {
			e.preventDefault();
			if (filtered.length > 0) {
				selectedIndex = (selectedIndex + 1) % filtered.length;
			}
			return;
		}

		if (e.key === 'ArrowUp') {
			e.preventDefault();
			if (filtered.length > 0) {
				selectedIndex = (selectedIndex - 1 + filtered.length) % filtered.length;
			}
			return;
		}

		if (e.key === 'Enter') {
			e.preventDefault();
			if (filtered.length > 0 && filtered[selectedIndex]) {
				onselect(filtered[selectedIndex].missionId);
			}
			return;
		}
	}

	function handleBackdropClick(e: MouseEvent) {
		if (e.target === e.currentTarget) {
			onclose();
		}
	}

	function selectResult(missionId: string) {
		onselect(missionId);
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="overlay-backdrop"
	onclick={handleBackdropClick}
	onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
	role="dialog"
	aria-modal="true"
	aria-label="Jump to mission"
	tabindex="-1"
>
	<div class="search-panel">
		<div class="search-header">
			<span class="search-title">JUMP TO MISSION</span>
		</div>

		<div class="search-input-wrap">
			<input
				bind:this={inputEl}
				bind:value={query}
				type="text"
				class="search-input"
				placeholder="Search..."
				autocomplete="off"
				spellcheck="false"
			/>
		</div>

		<div class="search-results" role="listbox" aria-label="Mission search results">
			{#each filtered as mission, i (mission.missionId)}
				<button
					class="result-row"
					class:selected={i === selectedIndex}
					onclick={() => selectResult(mission.missionId)}
					role="option"
					aria-selected={i === selectedIndex}
					type="button"
				>
					<span class="result-title">{mission.title}</span>
					<span class="result-status {statusClass(mission.status)}">{statusLabel(mission.status)}</span>
				</button>
			{/each}

			{#if filtered.length === 0}
				<div class="no-results">
					<span>No missions found</span>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.overlay-backdrop {
		position: fixed;
		inset: 0;
		z-index: 9999;
		display: flex;
		align-items: flex-start;
		justify-content: center;
		padding-top: 20vh;
		background: var(--bg-overlay);
		animation: fade-in 100ms linear;
	}

	.search-panel {
		width: 100%;
		max-width: 440px;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		display: flex;
		flex-direction: column;
		max-height: 60vh;
		animation: scale-in 100ms linear;
	}

	.search-header {
		display: flex;
		align-items: center;
		padding: var(--space-md) var(--space-lg);
		border-bottom: 1px solid var(--border-muted);
	}

	.search-title {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-secondary);
	}

	.search-input-wrap {
		padding: var(--space-md) var(--space-lg);
		border-bottom: 1px solid var(--border-muted);
	}

	.search-input {
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		font-size: 14px;
		color: var(--text-primary);
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		outline: none;
		font-family: var(--font-sans);
	}

	.search-input::placeholder {
		color: var(--text-muted);
	}

	.search-input:focus {
		border-color: var(--border-focus);
	}

	.search-results {
		overflow-y: auto;
		flex: 1;
	}

	.result-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-sm) var(--space-lg);
		gap: var(--space-md);
		cursor: pointer;
		background: none;
		border: none;
		color: inherit;
		font: inherit;
		text-align: left;
		transition: background var(--transition-fast);
	}

	.result-row:hover,
	.result-row.selected {
		background: var(--bg-card-hover);
	}

	.result-row.selected {
		border-left: 2px solid var(--text-primary);
	}

	.result-title {
		font-size: 13px;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
		min-width: 0;
	}

	.result-status {
		font-family: var(--font-mono);
		font-size: 11px;
		flex-shrink: 0;
		color: var(--text-muted);
	}

	.result-status.status-executing {
		color: var(--status-working);
	}

	.result-status.status-ready {
		color: var(--accent-green);
	}

	.result-status.status-incoming {
		color: var(--accent-blue);
	}

	.result-status.status-validating {
		color: var(--accent-amber);
	}

	.result-status.status-completed {
		color: var(--text-muted);
	}

	.result-status.status-failed {
		color: var(--accent-red);
	}

	.no-results {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl);
		font-size: 13px;
		color: var(--text-muted);
	}
</style>
