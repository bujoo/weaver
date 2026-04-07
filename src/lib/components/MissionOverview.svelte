<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';
	import { workspaceStatus } from '$lib/stores/workspace';
	import { isTauri } from '$lib/ws';

	interface Props {
		mission: UnifiedMission;
	}

	let { mission }: Props = $props();

	let workspace = $derived($workspaceStatus);

	// Format start time from lastActivity
	let startedLabel = $derived(() => {
		const d = new Date(mission.lastActivity);
		return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	});

	// Phase progress status colors
	const phaseStatusColors: Record<string, string> = {
		executing: 'var(--task-executing)',
		completed: 'var(--task-completed)',
		queued: 'var(--task-queued)',
		preparing: 'var(--task-preparing)',
		failed: 'var(--task-failed)',
	};

	function phaseColor(status: string): string {
		return phaseStatusColors[status] || 'var(--task-queued)';
	}

	function phaseStatusLabel(status: string): string {
		return status || 'queued';
	}

	// Match workspace repos to mission repos
	let matchedRepos = $derived(() => {
		if (!workspace?.repos || !mission.repos) return [];
		return mission.repos.map((missionRepo) => {
			const wsRepo = workspace!.repos.find(
				(r) => r.name === extractRepoName(missionRepo.repoUrl ?? missionRepo.repoId)
			);
			return {
				name: extractRepoName(missionRepo.repoUrl ?? missionRepo.repoId),
				baseBranch: wsRepo?.branch ?? 'main',
				missionBranch: missionRepo.branch ?? '--',
				clean: wsRepo?.clean ?? true,
			};
		});
	});

	function extractRepoName(urlOrId: string): string {
		if (!urlOrId) return 'unknown';
		// Extract last segment from URL or path
		const parts = urlOrId.replace(/\.git$/, '').split('/');
		return parts[parts.length - 1] || urlOrId;
	}

	async function openWorkspace() {
		if (!isTauri() || !workspace?.mountPath) return;
		try {
			const { invoke } = await import('@tauri-apps/api/core');
			await invoke('open_workspace_cmd', { path: workspace.mountPath });
		} catch (e) {
			console.error('Failed to open workspace:', e);
		}
	}
</script>

<div class="overview">
	<!-- Mission Brief -->
	<section class="overview-section">
		<h3 class="section-header">MISSION BRIEF</h3>
		<div class="brief-content">
			<div class="brief-row">
				<span class="brief-label">Scope:</span>
				<span class="brief-value">{mission.title}</span>
			</div>
			<div class="brief-row">
				<span class="brief-label">Started:</span>
				<span class="brief-value">{startedLabel()}</span>
				<span class="brief-sep">|</span>
				<span class="brief-label">Duration:</span>
				<span class="brief-value dim">--</span>
				<span class="brief-sep">|</span>
				<span class="brief-label">Agents:</span>
				<span class="brief-value">{mission.activeSessions}</span>
			</div>
		</div>
	</section>

	<!-- Phase Progress -->
	<section class="overview-section">
		<h3 class="section-header">PHASE PROGRESS</h3>
		{#if mission.availablePhases.length > 0}
			<div class="phase-grid">
				{#each mission.availablePhases as phase (phase.phaseId)}
					<div class="phase-item">
						<div class="phase-bar">
							<div
								class="phase-fill"
								class:pulse={phase.status === 'executing'}
								style="background: {phaseColor(phase.status)}; width: {phase.status === 'completed' ? '100%' : phase.status === 'executing' ? '60%' : '0%'};"
							></div>
						</div>
						<span class="phase-name">{phase.phaseName}</span>
						<span class="phase-status">{phaseStatusLabel(phase.status)}</span>
					</div>
				{/each}
			</div>
		{:else}
			<div class="empty-state">
				<span class="empty-dash">--</span>
				<span class="empty-text">No phases available</span>
			</div>
		{/if}
	</section>

	<!-- Repositories -->
	<section class="overview-section">
		<h3 class="section-header">REPOSITORIES</h3>
		{#if mission.repos.length > 0}
			<div class="repo-list">
				{#each matchedRepos() as repo (repo.name)}
					<div class="repo-row">
						<span class="repo-name">{repo.name}</span>
						<span class="repo-branches">
							{repo.baseBranch} <span class="branch-arrow">-></span> {repo.missionBranch}
						</span>
						<span class="repo-status" class:dirty={!repo.clean}>
							{repo.clean ? 'clean' : 'dirty'}
						</span>
					</div>
				{/each}
			</div>
			{#if isTauri()}
				<button class="workspace-btn" onclick={openWorkspace} type="button">
					Open VS Code Workspace
				</button>
			{/if}
		{:else}
			<div class="empty-state">
				<span class="empty-dash">--</span>
				<span class="empty-text">No repositories linked</span>
			</div>
		{/if}
	</section>

	<!-- Quick Stats -->
	<section class="overview-section">
		<h3 class="section-header">QUICK STATS</h3>
		<div class="stats-row">
			<div class="stat">
				<span class="stat-label">Todos</span>
				<span class="stat-value">{mission.completedTodos}/{mission.todoCount} done</span>
			</div>
			<span class="stat-sep">|</span>
			<div class="stat">
				<span class="stat-label">Retries</span>
				<span class="stat-value dim">--</span>
			</div>
			<span class="stat-sep">|</span>
			<div class="stat">
				<span class="stat-label">Interventions</span>
				<span class="stat-value dim">--</span>
			</div>
			<span class="stat-sep">|</span>
			<div class="stat">
				<span class="stat-label">Est</span>
				<span class="stat-value dim">--</span>
			</div>
		</div>
	</section>
</div>

<style>
	.overview {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
		padding-top: var(--space-lg);
	}

	/* ── Section ─────────────────────────────────────────────────── */
	.overview-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.section-header {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.05em;
		color: var(--text-muted);
		text-transform: uppercase;
		margin: 0;
	}

	/* ── Brief ───────────────────────────────────────────────────── */
	.brief-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.brief-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		flex-wrap: wrap;
	}

	.brief-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.brief-value {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-primary);
	}

	.brief-value.dim {
		color: var(--text-muted);
	}

	.brief-sep {
		color: var(--border-default);
		font-size: 12px;
	}

	/* ── Phase Progress ──────────────────────────────────────────── */
	.phase-grid {
		display: flex;
		gap: var(--space-sm);
		overflow-x: auto;
	}

	.phase-item {
		flex: 1;
		min-width: 80px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.phase-bar {
		height: 6px;
		background: var(--border-muted);
		overflow: hidden;
		position: relative;
	}

	.phase-fill {
		height: 100%;
		transition: width var(--transition-normal);
	}

	.phase-fill.pulse {
		animation: pulse-glow 2s ease-in-out infinite;
	}

	.phase-name {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-secondary);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.phase-status {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		text-transform: lowercase;
	}

	/* ── Repositories ────────────────────────────────────────────── */
	.repo-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.repo-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		background: var(--bg-card);
		border: 1px solid var(--border-muted);
	}

	.repo-name {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-primary);
		min-width: 120px;
	}

	.repo-branches {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		flex: 1;
	}

	.branch-arrow {
		color: var(--border-default);
		margin: 0 2px;
	}

	.repo-status {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.04em;
		color: var(--accent-green);
		text-transform: uppercase;
	}

	.repo-status.dirty {
		color: var(--accent-amber);
	}

	.workspace-btn {
		align-self: flex-start;
		margin-top: var(--space-sm);
		padding: 6px 16px;
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		color: var(--text-primary);
		background: transparent;
		border: 1px solid var(--border-default);
		cursor: pointer;
		transition: background var(--transition-fast), border-color var(--transition-fast);
	}

	.workspace-btn:hover {
		background: rgba(255, 255, 255, 0.06);
		border-color: var(--text-secondary);
	}

	/* ── Quick Stats ──────────────────────────────────────────────── */
	.stats-row {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		flex-wrap: wrap;
	}

	.stat {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.stat-label {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.stat-value {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-primary);
	}

	.stat-value.dim {
		color: var(--text-muted);
	}

	.stat-sep {
		color: var(--border-default);
		font-size: 12px;
	}

	/* ── Empty state ─────────────────────────────────────────────── */
	.empty-state {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-md) 0;
	}

	.empty-dash {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.empty-text {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	/* ── Mobile ──────────────────────────────────────────────────── */
	@media (max-width: 768px) {
		.phase-grid {
			flex-wrap: wrap;
		}

		.phase-item {
			min-width: 60px;
		}

		.repo-row {
			flex-wrap: wrap;
			gap: var(--space-sm);
		}

		.repo-name {
			min-width: 0;
		}

		.stats-row {
			gap: var(--space-sm);
		}
	}
</style>
