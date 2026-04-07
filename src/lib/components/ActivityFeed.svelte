<script lang="ts">
	import {
		activityEvents,
		activityFilter,
		filteredEvents,
		type ActivityEvent,
		type ActivityFilter,
	} from '$lib/stores/activity';

	interface Props {
		missionId: string;
	}

	let { missionId }: Props = $props();

	// Filter the activity store by the passed missionId
	$effect(() => {
		activityFilter.update((f) => ({ ...f, missionId }));
	});

	let filter = $derived($activityFilter);
	let events = $derived($filteredEvents);

	// Reverse for newest-first display
	let displayEvents = $derived([...events].reverse());

	// Expanded event detail
	let expandedId: string | null = $state(null);

	// Auto-scroll / pin-to-bottom
	let pinToBottom = $state(true);
	let scrollContainer: HTMLDivElement | undefined = $state(undefined);

	$effect(() => {
		if (pinToBottom && scrollContainer && events.length > 0) {
			// Scroll to top since newest is first in reversed list
			scrollContainer.scrollTop = 0;
		}
	});

	// ── Source filter logic ─────────────────────────────────────────

	type SourceKey = 'mqtt' | 'claude_code' | 'supervisor' | 'human' | 'system';

	const SOURCE_LABELS: Record<SourceKey, string> = {
		mqtt: 'MQTT',
		claude_code: 'Claude Code',
		supervisor: 'Supervisor',
		human: 'Human',
		system: 'System',
	};

	const ALL_SOURCES: SourceKey[] = ['mqtt', 'claude_code', 'supervisor', 'human', 'system'];

	let activeSource: SourceKey | null = $derived(
		filter.sources.size === 1 ? ([...filter.sources][0] as SourceKey) : null
	);
	let isAll = $derived(filter.sources.size === 0);

	function selectAll() {
		activityFilter.update((f) => ({ ...f, sources: new Set<string>() }));
	}

	function selectSource(source: SourceKey) {
		activityFilter.update((f) => {
			if (f.sources.size === 1 && f.sources.has(source)) {
				// Clicking the active filter clears it -> show all
				return { ...f, sources: new Set<string>() };
			}
			return { ...f, sources: new Set<string>([source]) };
		});
	}

	// ── Helpers ─────────────────────────────────────────────────────

	function formatTime(ts: number): string {
		const d = new Date(ts);
		return d.toLocaleTimeString('en-GB', {
			hour: '2-digit',
			minute: '2-digit',
			second: '2-digit',
			hour12: false,
		});
	}

	function sourceBadgeClass(source: string): string {
		return `badge badge-${source}`;
	}

	function sourceBadgeLabel(source: string): string {
		if (source === 'claude_code') return 'CLAUDE';
		return source.toUpperCase();
	}

	function toggleExpand(id: string) {
		expandedId = expandedId === id ? null : id;
	}
</script>

<div class="activity-feed">
	<div class="filter-bar">
		<span class="filter-label">FILTERS:</span>
		<button
			class="filter-btn"
			class:active={isAll}
			onclick={selectAll}
		>
			All
		</button>
		{#each ALL_SOURCES as source (source)}
			<button
				class="filter-btn"
				class:active={activeSource === source}
				onclick={() => selectSource(source)}
			>
				{SOURCE_LABELS[source]}
			</button>
		{/each}
	</div>

	<div class="event-list" bind:this={scrollContainer}>
		{#if displayEvents.length === 0}
			<div class="empty-state">
				<span class="empty-text">No activity events</span>
			</div>
		{:else}
			{#each displayEvents as event (event.id)}
				{@const hasDetail = !!event.detail}
				{@const isExpanded = expandedId === event.id}
				<div
					class="event-row"
					class:severity-warning={event.severity === 'warning'}
					class:severity-error={event.severity === 'error'}
					class:expandable={hasDetail}
					class:expanded={isExpanded}
				>
					<button
						class="event-main"
						onclick={() => hasDetail && toggleExpand(event.id)}
						disabled={!hasDetail}
					>
						<span class="event-time">{formatTime(event.timestamp)}</span>
						<span class={sourceBadgeClass(event.source)}>
							{sourceBadgeLabel(event.source)}
						</span>
						<span class="event-message">{event.message}</span>
						<span class="event-phase">
							{event.phaseId ?? '--'}
						</span>
					</button>

					{#if isExpanded && event.detail}
						<div class="event-detail">
							<pre>{event.detail}</pre>
						</div>
					{/if}
				</div>
			{/each}
		{/if}
	</div>

	<div class="feed-footer">
		<button
			class="pin-btn"
			class:active={pinToBottom}
			onclick={() => (pinToBottom = !pinToBottom)}
		>
			Pin to bottom: {pinToBottom ? 'ON' : 'OFF'}
		</button>
	</div>
</div>

<style>
	.activity-feed {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-height: 0;
		background: var(--bg-base);
	}

	/* ── Filter Bar ──────────────────────────────────────────────── */

	.filter-bar {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-bottom: 1px solid var(--border-muted);
		flex-shrink: 0;
	}

	.filter-label {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: var(--text-muted);
		margin-right: var(--space-xs);
	}

	.filter-btn {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		padding: var(--space-xs) var(--space-sm);
		border: none;
		background: none;
		cursor: pointer;
		border-bottom: 2px solid transparent;
		transition: color var(--transition-fast), border-color var(--transition-fast);
	}

	.filter-btn:hover {
		color: var(--text-secondary);
	}

	.filter-btn.active {
		color: var(--text-primary);
		border-bottom-color: var(--text-primary);
	}

	/* ── Event List ──────────────────────────────────────────────── */

	.event-list {
		flex: 1;
		overflow-y: auto;
		min-height: 0;
	}

	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-3xl);
	}

	.empty-text {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		opacity: 0.5;
	}

	/* ── Event Row ───────────────────────────────────────────────── */

	.event-row {
		border-bottom: 1px solid var(--border-muted);
		border-left: 2px solid transparent;
	}

	.event-row.severity-warning {
		border-left-color: var(--accent-amber);
	}

	.event-row.severity-error {
		border-left-color: var(--accent-red);
	}

	.event-main {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		width: 100%;
		padding: var(--space-sm) var(--space-md);
		text-align: left;
		background: none;
		border: none;
		color: inherit;
		font: inherit;
	}

	.event-row.expandable .event-main {
		cursor: pointer;
	}

	.event-row.expandable .event-main:hover {
		background: var(--bg-card-hover);
	}

	.event-main:disabled {
		cursor: default;
	}

	/* ── Event Fields ────────────────────────────────────────────── */

	.event-time {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
		min-width: 64px;
	}

	.badge {
		font-family: var(--font-mono);
		font-size: 9px;
		font-weight: 700;
		letter-spacing: 0.05em;
		padding: 1px 6px;
		flex-shrink: 0;
		min-width: 72px;
		text-align: center;
		border: 1px solid;
	}

	.badge-mqtt {
		color: var(--accent-blue);
		border-color: var(--accent-blue);
	}

	.badge-claude_code {
		color: var(--accent-purple);
		border-color: var(--accent-purple);
	}

	.badge-supervisor {
		color: var(--accent-amber);
		border-color: var(--accent-amber);
	}

	.badge-human {
		color: var(--accent-green);
		border-color: var(--accent-green);
	}

	.badge-system {
		color: var(--text-muted);
		border-color: var(--text-muted);
	}

	.event-message {
		flex: 1;
		font-size: 13px;
		color: var(--text-primary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.event-phase {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex-shrink: 0;
		text-align: right;
		min-width: 64px;
	}

	/* ── Expanded Detail ─────────────────────────────────────────── */

	.event-detail {
		padding: var(--space-sm) var(--space-md) var(--space-md);
		border-top: 1px solid var(--border-muted);
		background: var(--bg-elevated);
	}

	.event-detail pre {
		font-family: var(--font-mono);
		font-size: 11px;
		line-height: 1.5;
		color: var(--text-secondary);
		white-space: pre-wrap;
		word-break: break-all;
		margin: 0;
	}

	/* ── Footer ──────────────────────────────────────────────────── */

	.feed-footer {
		display: flex;
		justify-content: flex-end;
		padding: var(--space-xs) var(--space-md);
		border-top: 1px solid var(--border-muted);
		flex-shrink: 0;
	}

	.pin-btn {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		padding: var(--space-xs) var(--space-sm);
		border: 1px solid var(--border-muted);
		background: none;
		cursor: pointer;
		transition: color var(--transition-fast), border-color var(--transition-fast);
	}

	.pin-btn:hover {
		color: var(--text-secondary);
		border-color: var(--border-default);
	}

	.pin-btn.active {
		color: var(--text-primary);
		border-color: var(--text-primary);
	}
</style>
