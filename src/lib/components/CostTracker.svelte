<script lang="ts">
	import { onMount } from 'svelte';
	import { getCostData } from '$lib/api';
	import type { CostData } from '$lib/types';

	// ── State ────────────────────────────────────────────────────────
	let costData = $state<CostData | null>(null);
	let loading = $state(true);
	let expandedDays = $state<Set<string>>(new Set());
	let collapsedProjects = $state<Set<string>>(new Set());

	// ── Helpers ──────────────────────────────────────────────────────
	function formatCost(n: number): string {
		return '$' + n.toFixed(2);
	}

	function formatDate(dateStr: string): string {
		const today = new Date().toISOString().slice(0, 10);
		const yesterday = new Date(Date.now() - 86400000).toISOString().slice(0, 10);
		if (dateStr === today) return 'TODAY';
		if (dateStr === yesterday) return 'YESTERDAY';
		const d = new Date(dateStr + 'T00:00:00');
		return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' }).toUpperCase();
	}

	function formatTime(timestamp: string): string {
		const d = new Date(timestamp);
		return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
	}

	function modelDisplayName(model: string): string {
		if (model.startsWith('claude-sonnet')) return 'Sonnet';
		if (model.startsWith('claude-opus')) return 'Opus';
		if (model.startsWith('claude-haiku')) return 'Haiku';
		return model;
	}

	function modelColor(model: string): string {
		if (model.startsWith('claude-sonnet')) return 'var(--accent-blue)';
		if (model.startsWith('claude-opus')) return 'var(--accent-amber)';
		return 'var(--text-muted)';
	}

	// ── Derived ──────────────────────────────────────────────────────
	let dailyCosts = $derived.by(() => {
		if (!costData) return [];
		return costData.dailyCosts.filter(d => d.cost > 0).slice(0, 14);
	});

	let maxDailyCost = $derived.by(() => {
		if (dailyCosts.length === 0) return 0;
		return Math.max(...dailyCosts.map(d => d.cost));
	});

	let allCollapsed = $derived(
		costData !== null &&
		costData.projectCosts.length > 0 &&
		costData.projectCosts.every(p => collapsedProjects.has(p.project))
	);

	let maxProjectCost = $derived.by(() => {
		if (!costData || costData.projectCosts.length === 0) return 0;
		return Math.max(...costData.projectCosts.map(p => p.totalCost));
	});

	// ── Actions ──────────────────────────────────────────────────────
	function toggleDay(date: string) {
		const next = new Set(expandedDays);
		if (next.has(date)) {
			next.delete(date);
		} else {
			next.add(date);
		}
		expandedDays = next;
	}

	function toggleProjectCollapse(project: string) {
		const next = new Set(collapsedProjects);
		if (next.has(project)) {
			next.delete(project);
		} else {
			next.add(project);
		}
		collapsedProjects = next;
	}

	function toggleAllProjects() {
		if (allCollapsed) {
			collapsedProjects = new Set();
		} else if (costData) {
			collapsedProjects = new Set(costData.projectCosts.map(p => p.project));
		}
	}

	// ── Lifecycle ────────────────────────────────────────────────────
	onMount(async () => {
		try {
			costData = await getCostData();
			if (costData) {
				collapsedProjects = new Set(costData.projectCosts.map(p => p.project));
			}
		} catch (e) {
			console.error('Failed to load cost data:', e);
		} finally {
			loading = false;
		}
	});
</script>

<div class="cost-container">
	<!-- ── Header ─────────────────────────────────────────────────── -->
	<div class="section-header">
		<span class="section-title">COST TRACKER</span>
		{#if costData}
			<span class="section-total">{formatCost(costData.totalCost)}</span>
		{/if}
	</div>

	{#if loading}
		<div class="state-msg">Loading cost data...</div>
	{:else if !costData}
		<div class="state-msg">No cost data available.</div>
	{:else}
		<div class="list-area">
			<!-- ── DAILY COST ─────────────────────────────────────── -->
			<div class="cost-section">
				<div class="sub-header">DAILY COST</div>

				{#each dailyCosts as day (day.date)}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="day-row"
						onclick={() => toggleDay(day.date)}
						role="button"
						tabindex="0"
						aria-label={expandedDays.has(day.date) ? 'Collapse day' : 'Expand day'}
					>
						<span class="day-label">{formatDate(day.date)}</span>
						<div class="bar-track">
							<div
								class="bar-fill bar-amber"
								style="width: {maxDailyCost > 0 ? (day.cost / maxDailyCost) * 100 : 0}%"
							></div>
						</div>
						<span class="day-cost">{formatCost(day.cost)}</span>
					</div>

					{#if expandedDays.has(day.date)}
						<div class="day-sessions">
							{#each day.sessions as session (session.sessionId)}
								<div class="session-detail">
									<span class="detail-project">{session.projectName}</span>
									<span class="detail-time">{formatTime(session.timestamp)}</span>
									<span class="detail-model">{modelDisplayName(session.model)}</span>
									<span class="detail-cost">{formatCost(session.cost)}</span>
								</div>
							{/each}
						</div>
					{/if}
				{/each}
			</div>

			<!-- ── BY PROJECT ─────────────────────────────────────── -->
			<div class="cost-section">
				<div class="sub-header-row">
					<span class="sub-header">BY PROJECT</span>
					<div class="sort-group">
						<button class="option-btn" onclick={toggleAllProjects}>
							{allCollapsed ? 'EXPAND ALL' : 'COLLAPSE ALL'}
						</button>
					</div>
				</div>

				{#each costData.projectCosts as proj (proj.project)}
					<div class="project-group">
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<div
							class="group-header"
							onclick={() => toggleProjectCollapse(proj.project)}
							role="button"
							tabindex="0"
							aria-label={collapsedProjects.has(proj.project) ? 'Expand project' : 'Collapse project'}
						>
							<span class="collapse-toggle" aria-hidden="true">{collapsedProjects.has(proj.project) ? '▶' : '▼'}</span>
							<span class="group-name">{proj.projectName.toUpperCase()}</span>
							<span class="group-cost">{formatCost(proj.totalCost)}</span>
						</div>

						{#if !collapsedProjects.has(proj.project)}
							<div class="project-bar-track">
								<div
									class="bar-fill bar-amber"
									style="width: {maxProjectCost > 0 ? (proj.totalCost / maxProjectCost) * 100 : 0}%"
								></div>
							</div>

							{#each proj.sessions.slice(0, 5) as session (session.sessionId)}
								<div class="session-detail">
									<span class="detail-project">{session.projectName}</span>
									<span class="detail-time">{formatTime(session.timestamp)}</span>
									<span class="detail-model">{modelDisplayName(session.model)}</span>
									<span class="detail-cost">{formatCost(session.cost)}</span>
								</div>
							{/each}

							{#if proj.sessions.length > 5}
								<div class="more-sessions">
									{proj.sessions.length - 5} more sessions
								</div>
							{/if}
						{/if}
					</div>
				{/each}
			</div>

			<!-- ── BY MODEL ───────────────────────────────────────── -->
			<div class="cost-section">
				<div class="sub-header">BY MODEL</div>

				{#each costData.modelCosts as mc (mc.model)}
					<div class="model-row">
						<span class="model-label">{mc.displayName}</span>
						<div class="bar-track">
							<div
								class="bar-fill"
								style="width: {mc.percentage}%; background: {modelColor(mc.model)}"
							></div>
						</div>
						<span class="model-cost">{formatCost(mc.cost)}</span>
						<span class="model-pct">{mc.percentage.toFixed(0)}%</span>
					</div>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.cost-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.section-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-md);
		border-bottom: 1px solid var(--text-primary);
		margin-bottom: var(--space-md);
		flex-shrink: 0;
	}

	.section-title {
		font-family: var(--font-pixel);
		font-size: 22px;
		font-weight: 600;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		line-height: 1;
	}

	.section-total {
		font-family: var(--font-pixel);
		font-size: 18px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
	}

	.list-area {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-lg) 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.cost-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.sub-header {
		font-family: var(--font-pixel);
		font-size: 14px;
		text-transform: uppercase;
		color: var(--text-secondary);
		letter-spacing: 0.1em;
		line-height: 1;
	}

	.sub-header-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
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

	/* ── Daily cost rows ──────────────────────────────────────────── */
	.day-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-xs) var(--space-sm);
		cursor: pointer;
		transition: background var(--transition-fast, 150ms);
	}

	.day-row:hover {
		background: rgba(255, 255, 255, 0.03);
	}

	.day-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		min-width: 80px;
		flex-shrink: 0;
	}

	.bar-track {
		flex: 1;
		height: 4px;
		background: rgba(255, 255, 255, 0.05);
		overflow: hidden;
	}

	.bar-fill {
		height: 100%;
		transition: width 300ms ease;
	}

	.bar-amber {
		background: var(--accent-amber);
	}

	.day-cost {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		min-width: 60px;
		text-align: right;
		flex-shrink: 0;
	}

	/* ── Expanded day sessions ────────────────────────────────────── */
	.day-sessions {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		padding-left: var(--space-xl);
		margin-bottom: var(--space-sm);
	}

	.session-detail {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-xs) var(--space-sm);
		font-family: var(--font-mono);
		font-size: 11px;
	}

	.detail-project {
		color: var(--text-primary);
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.detail-time {
		color: var(--text-muted);
		flex-shrink: 0;
	}

	.detail-model {
		color: var(--text-muted);
		flex-shrink: 0;
		min-width: 50px;
	}

	.detail-cost {
		color: var(--text-secondary);
		flex-shrink: 0;
		min-width: 50px;
		text-align: right;
	}

	/* ── Project groups ───────────────────────────────────────────── */
	.project-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		margin-bottom: var(--space-md);
	}

	.group-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-sm);
		border-bottom: 1px solid var(--border-default);
		margin-bottom: var(--space-sm);
		cursor: pointer;
	}

	.group-header:hover .group-name {
		color: var(--text-primary);
	}

	.collapse-toggle {
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-size: 11px;
		line-height: 1;
		flex-shrink: 0;
	}

	.group-name {
		font-family: var(--font-pixel);
		font-size: 16px;
		color: var(--text-primary);
		letter-spacing: 0.1em;
		flex: 1;
	}

	.group-cost {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-secondary);
		flex-shrink: 0;
	}

	.project-bar-track {
		height: 4px;
		background: rgba(255, 255, 255, 0.05);
		overflow: hidden;
		margin-bottom: var(--space-xs);
	}

	.project-bar-track .bar-fill {
		height: 100%;
		transition: width 300ms ease;
	}

	.more-sessions {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		padding: var(--space-xs) var(--space-sm);
		cursor: pointer;
	}

	.more-sessions:hover {
		color: var(--text-secondary);
	}

	/* ── Sort/toggle buttons ──────────────────────────────────────── */
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

	.option-btn:hover {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.1);
	}

	/* ── Model rows ───────────────────────────────────────────────── */
	.model-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-xs) var(--space-sm);
	}

	.model-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-primary);
		min-width: 60px;
		flex-shrink: 0;
	}

	.model-cost {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
		min-width: 60px;
		text-align: right;
		flex-shrink: 0;
	}

	.model-pct {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		min-width: 35px;
		text-align: right;
		flex-shrink: 0;
	}
</style>
