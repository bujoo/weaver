<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import { getSessionHistory, deepSearchSessions, getConversation } from '$lib/api';
	import type { HistoryEntry, Conversation } from '$lib/types';
	import HistoryCardOverlay from './HistoryCardOverlay.svelte';

	// ── State ────────────────────────────────────────────────────────
	let allEntries = $state<HistoryEntry[]>([]);
	let loading = $state(true);
	let error = $state<string | null>(null);

	let query = $state('');
	let sortOrder = $state<'newest' | 'oldest'>('newest');
	let groupByProject = $state(false);

	let deepSearching = $state(false);
	let deepSearchResults = $state<Set<string> | null>(null); // null = no search run yet

	// Conversation viewer state
	let selectedEntry = $state<HistoryEntry | null>(null);
	let conversation = $state<Conversation | null>(null);
	// ── Persistence ──────────────────────────────────────────────────
	onMount(async () => {
		if (browser) {
			const savedSort = localStorage.getItem('historySort');
			if (savedSort === 'newest' || savedSort === 'oldest') sortOrder = savedSort;
			const savedGroup = localStorage.getItem('historyGroup');
			if (savedGroup === 'true') groupByProject = true;
		}

		try {
			allEntries = await getSessionHistory();
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	});

	$effect(() => {
		if (browser) localStorage.setItem('historySort', sortOrder);
	});

	$effect(() => {
		if (browser) localStorage.setItem('historyGroup', String(groupByProject));
	});

	// Debounced deep search: fires 300ms after the query settles.
	// deepSearching is set only after the timer fires — no spinner during the
	// debounce window itself, which avoids flicker on every keystroke.
	$effect(() => {
		const q = query;
		if (!q.trim()) {
			deepSearchResults = null;
			deepSearching = false;
			return;
		}
		deepSearchResults = null; // clear stale results from the previous query immediately
		let cancelled = false;
		const timer = setTimeout(async () => {
			deepSearching = true;
			try {
				const ids = await deepSearchSessions(q);
				if (!cancelled) deepSearchResults = new Set(ids);
			} catch (e) {
				if (!cancelled) console.error('Deep search failed:', e);
			} finally {
				if (!cancelled) deepSearching = false;
			}
		}, 300);
		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});

	// ── Filtering & sorting ──────────────────────────────────────────
	let filtered = $derived.by(() => {
		let entries = allEntries;

		if (query.trim()) {
			const q = query.toLowerCase();
			entries = entries.filter(
				(e) =>
					e.display.toLowerCase().includes(q) ||
					e.projectName.toLowerCase().includes(q)
			);

			// If deep search has run, also include sessions that matched full content
			if (deepSearchResults !== null) {
				const metaIds = new Set(entries.map((e) => e.sessionId));
				const deepOnly = allEntries.filter(
					(e) => deepSearchResults!.has(e.sessionId) && !metaIds.has(e.sessionId)
				);
				entries = [...entries, ...deepOnly];
			}
		} else if (deepSearchResults !== null) {
			// No text query but deep search ran — show all deep search hits
			entries = allEntries.filter((e) => deepSearchResults!.has(e.sessionId));
		}

		return [...entries].sort((a, b) =>
			sortOrder === 'newest' ? b.timestamp - a.timestamp : a.timestamp - b.timestamp
		);
	});

	// ── Grouping ─────────────────────────────────────────────────────
	let groups = $derived.by(() => {
		if (!groupByProject) return null;

		const map = new Map<string, { project: string; projectName: string; entries: HistoryEntry[] }>();
		for (const entry of filtered) {
			if (!map.has(entry.project)) {
				map.set(entry.project, { project: entry.project, projectName: entry.projectName, entries: [] });
			}
			map.get(entry.project)!.entries.push(entry);
		}

		return [...map.values()];
	});

	// ── Actions ──────────────────────────────────────────────────────
	async function handleSelectEntry(entry: HistoryEntry) {
		selectedEntry = entry;
		conversation = null;
		try {
			conversation = await getConversation(entry.sessionId);
		} catch (e) {
			console.error('Failed to load conversation:', e);
		}
	}

	function handleCloseConversation() {
		selectedEntry = null;
		conversation = null;
	}

	// ── Helpers ──────────────────────────────────────────────────────
	function relativeTime(ms: number): string {
		const diff = Date.now() - ms;
		const mins = Math.floor(diff / 60_000);
		if (mins < 1) return 'just now';
		if (mins < 60) return `${mins}m ago`;
		const hours = Math.floor(mins / 60);
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		if (days === 1) return 'yesterday';
		if (days < 7) return `${days}d ago`;
		return new Date(ms).toLocaleDateString();
	}

</script>

<!-- ── Search bar & controls ──────────────────────────────────────── -->
<div class="history-container">
	<div class="controls">
		<div class="search-row">
			<input
				class="search-input"
				type="text"
				placeholder="Search sessions..."
				bind:value={query}
			/>
		</div>

		<div class="options-row">
			<div class="sort-group">
				<button
					class="option-btn"
					class:active={sortOrder === 'newest'}
					onclick={() => (sortOrder = 'newest')}
				>NEWEST</button>
				<button
					class="option-btn"
					class:active={sortOrder === 'oldest'}
					onclick={() => (sortOrder = 'oldest')}
				>OLDEST</button>
			</div>

			<div class="sort-group">
				<button
					class="option-btn"
					class:active={!groupByProject}
					onclick={() => (groupByProject = false)}
				>FLAT</button>
				<button
					class="option-btn"
					class:active={groupByProject}
					onclick={() => (groupByProject = true)}
				>BY PROJECT</button>
			</div>
		</div>
	</div>

	{#if deepSearching}
		<div class="searching-indicator">Searching...</div>
	{/if}

	<!-- ── List ──────────────────────────────────────────────────── -->
	<div class="list-area">
		{#if loading}
			<div class="state-msg">Loading history...</div>
		{:else if error}
			<div class="state-msg error">Error: {error}</div>
		{:else if filtered.length === 0}
			<div class="state-msg">No sessions found.</div>
		{:else if groupByProject && groups}
			{#each groups as group}
				<div class="project-group">
					<div class="group-header">
						<span class="group-name">{group.projectName.toUpperCase()}</span>
						<span class="group-count">{group.entries.length}</span>
					</div>
					{#each group.entries as entry (entry.sessionId)}
						<button class="session-row" onclick={() => handleSelectEntry(entry)}>
							<span class="row-prompt">{entry.display || '(no prompt)'}</span>
							<span class="row-time">{relativeTime(entry.timestamp)}</span>
						</button>
					{/each}
				</div>
			{/each}
		{:else}
			{#each filtered as entry (entry.sessionId)}
				<button class="session-row" onclick={() => handleSelectEntry(entry)}>
					<div class="row-top">
						<span class="row-project">{entry.projectName.toUpperCase()}</span>
						<span class="row-time">{relativeTime(entry.timestamp)}</span>
					</div>
					<span class="row-prompt">{entry.display || '(no prompt)'}</span>
				</button>
			{/each}
		{/if}

	</div>
</div>

<!-- ── Conversation overlay ───────────────────────────────────────── -->
{#if selectedEntry}
	<HistoryCardOverlay entry={selectedEntry} {conversation} onclose={handleCloseConversation} />
{/if}

<style>
	.history-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.controls {
		flex-shrink: 0;
		padding: var(--space-xl) var(--space-xl) var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		border-bottom: 1px solid var(--border-default);
	}

	.search-input {
		width: 100%;
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		color: var(--text-primary);
		font-family: var(--font-mono);
		font-size: 13px;
		padding: var(--space-sm) var(--space-md);
		outline: none;
		box-sizing: border-box;
	}

	.search-input:focus {
		border-color: var(--border-focus);
	}

	.search-input::placeholder {
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.options-row {
		display: flex;
		gap: var(--space-md);
	}

	.sort-group {
		display: flex;
		border: 1px solid var(--border-default);
	}

	.option-btn {
		font-family: var(--font-pixel);
		font-size: 10px;
		letter-spacing: 0.05em;
		padding: 4px var(--space-sm);
		background: transparent;
		border: none;
		color: var(--text-muted);
		cursor: pointer;
	}

	.option-btn.active {
		background: rgba(255, 255, 255, 0.1);
		color: var(--text-primary);
	}

	.list-area {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md) var(--space-xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.state-msg {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		padding: var(--space-xl) 0;
		text-align: center;
	}

	.state-msg.error {
		color: var(--accent-red);
	}

	.session-row {
		width: 100%;
		text-align: left;
		background: var(--bg-card);
		border: 1px solid var(--border-muted);
		padding: var(--space-md);
		cursor: pointer;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		transition: border-color var(--transition-fast);
	}

	.session-row:hover {
		border-color: var(--border-default);
		background: var(--bg-card-hover);
	}

	.row-top {
		display: flex;
		justify-content: space-between;
		align-items: baseline;
	}

	.row-project {
		font-family: var(--font-pixel);
		font-size: 12px;
		font-weight: 600;
		color: var(--text-primary);
		letter-spacing: 0.1em;
	}

	.row-time {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.row-prompt {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.project-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		margin-bottom: var(--space-xl);
	}

	.group-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-default);
		margin-bottom: var(--space-sm);
	}

	.group-name {
		font-family: var(--font-pixel);
		font-size: 16px;
		color: var(--text-primary);
		letter-spacing: 0.1em;
	}

	.group-count {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-muted);
	}

	.searching-indicator {
		flex-shrink: 0;
		padding: var(--space-xs) var(--space-xl);
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

</style>
