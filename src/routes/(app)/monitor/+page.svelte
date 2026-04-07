<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import {
		sortedSessions,
		expandedSessionId,
		currentConversation,
		statusSummary
	} from '$lib/stores/sessions';
	import { getConversation, stopSession, openSession, renameSession } from '$lib/api';
	import SessionCard from '$lib/components/SessionCard.svelte';
	import ExpandedCardOverlay from '$lib/components/ExpandedCardOverlay.svelte';
	import CostTracker from '$lib/components/CostTracker.svelte';
	import type { Session } from '$lib/types';
	import { SessionStatus } from '$lib/types';

	let sessions = $derived($sortedSessions);
	let summary = $derived($statusSummary);
	let expandedId = $derived($expandedSessionId);
	let conversation = $derived($currentConversation);

	let activeCount = $derived(sessions.length);

	// Group sessions by status priority
	let attentionSessions = $derived(
		sessions.filter(
			(s) =>
				s.status === SessionStatus.NeedsAttention ||
				s.status === SessionStatus.WaitingForInput
		)
	);

	let workingSessions = $derived(
		sessions.filter(
			(s) =>
				s.status === SessionStatus.Working ||
				s.status === SessionStatus.Connecting
		)
	);

	let idleSessions = $derived(
		sessions.filter(
			(s) =>
				s.status !== SessionStatus.NeedsAttention &&
				s.status !== SessionStatus.WaitingForInput &&
				s.status !== SessionStatus.Working &&
				s.status !== SessionStatus.Connecting
		)
	);

	let expandedSession = $derived(sessions.find((s) => s.id === expandedId) || null);

	// Fetch conversation when a session is expanded
	$effect(() => {
		if (expandedId) {
			getConversation(expandedId)
				.then((conv) => {
					currentConversation.set(conv);
				})
				.catch((error) => {
					console.error('Failed to fetch conversation:', error);
					currentConversation.set(null);
				});
		} else {
			currentConversation.set(null);
		}
	});

	function handleExpand(session: Session) {
		expandedSessionId.set(session.id);
	}

	function handleClose() {
		expandedSessionId.set(null);
	}

	async function handleStop(pid: number) {
		try {
			await stopSession(pid);
		} catch (error) {
			console.error('Failed to stop session:', error);
		}
	}

	async function handleOpen(pid: number, projectPath: string) {
		try {
			await openSession(pid, projectPath);
		} catch (error) {
			console.error('Failed to open session:', error);
		}
	}

	function handleRename(session: Session) {
		const newName = prompt('Rename session:', session.customTitle || session.summary || session.firstPrompt || '');
		if (newName !== null && newName.trim() !== '') {
			renameSession(session.id, newName.trim()).catch((error) => {
				console.error('Failed to rename session:', error);
			});
		}
	}

	// Keyboard shortcuts
	function handleKeydown(e: KeyboardEvent) {
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA') return;

		if (e.key === 'Escape' && expandedId) {
			handleClose();
			return;
		}

		// Number keys to quick-expand sessions
		if (e.key >= '1' && e.key <= '9' && !expandedId) {
			const index = parseInt(e.key) - 1;
			if (index < sessions.length) {
				handleExpand(sessions[index]);
			}
		}

		// Tab to focus first attention session
		if (e.key === 'Tab' && !expandedId) {
			if (attentionSessions.length > 0) {
				e.preventDefault();
				handleExpand(attentionSessions[0]);
			}
		}
	}

	let showCostSummary = $state(true);
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="monitor-page">
	<!-- Header -->
	<header class="monitor-header">
		<div class="header-left">
			<h1 class="page-title">CLAUDE CODE SESSIONS</h1>
		</div>
		<div class="header-right">
			<span class="session-count">{activeCount} active</span>
		</div>
	</header>

	<!-- Session Groups -->
	<div class="session-groups">
		<!-- Needs Attention -->
		<section class="session-group">
			<div class="group-header">
				<span class="group-label attention-label">NEEDS ATTENTION</span>
				<span class="group-count">({attentionSessions.length})</span>
			</div>
			{#if attentionSessions.length > 0}
				<div class="session-grid">
					{#each attentionSessions as session (session.id)}
						<SessionCard
							{session}
							onexpand={() => handleExpand(session)}
							onstop={() => handleStop(session.pid)}
							onopen={() => handleOpen(session.pid, session.projectPath)}
							onrename={() => handleRename(session)}
						/>
					{/each}
				</div>
			{:else}
				<p class="empty-group">No sessions need attention</p>
			{/if}
		</section>

		<!-- Working -->
		<section class="session-group">
			<div class="group-header">
				<span class="group-label working-label">WORKING</span>
				<span class="group-count">({workingSessions.length})</span>
			</div>
			{#if workingSessions.length > 0}
				<div class="session-grid">
					{#each workingSessions as session (session.id)}
						<SessionCard
							{session}
							onexpand={() => handleExpand(session)}
							onstop={() => handleStop(session.pid)}
							onopen={() => handleOpen(session.pid, session.projectPath)}
							onrename={() => handleRename(session)}
						/>
					{/each}
				</div>
			{:else}
				<p class="empty-group">No working sessions</p>
			{/if}
		</section>

		<!-- Idle -->
		<section class="session-group">
			<div class="group-header">
				<span class="group-label idle-label">IDLE</span>
				<span class="group-count">({idleSessions.length})</span>
			</div>
			{#if idleSessions.length > 0}
				<div class="session-grid">
					{#each idleSessions as session (session.id)}
						<SessionCard
							{session}
							onexpand={() => handleExpand(session)}
							onstop={() => handleStop(session.pid)}
							onopen={() => handleOpen(session.pid, session.projectPath)}
							onrename={() => handleRename(session)}
						/>
					{/each}
				</div>
			{:else}
				<p class="empty-group">No idle sessions</p>
			{/if}
		</section>
	</div>

	<!-- Cost Summary -->
	<section class="cost-section">
		<button
			class="cost-toggle"
			onclick={() => (showCostSummary = !showCostSummary)}
			type="button"
		>
			<span class="cost-toggle-label">COST SUMMARY</span>
			<svg
				class="cost-chevron"
				class:collapsed={!showCostSummary}
				width="12"
				height="12"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<polyline points="6 9 12 15 18 9" />
			</svg>
		</button>
		{#if showCostSummary}
			<div class="cost-container">
				<CostTracker />
			</div>
		{/if}
	</section>

	<!-- Empty state when no sessions at all -->
	{#if sessions.length === 0}
		<div class="empty-state">
			<div class="empty-icon">
				<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<path d="M22 12h-4l-3 9L9 3l-3 9H2" />
				</svg>
			</div>
			<p class="empty-title">No active sessions</p>
			<p class="empty-sub">Claude Code sessions on this device will appear here</p>
		</div>
	{/if}
</div>

<!-- Expanded session overlay -->
{#if expandedSession}
	<ExpandedCardOverlay
		session={expandedSession}
		{conversation}
		onclose={handleClose}
		onstop={() => handleStop(expandedSession.pid)}
		onopen={() => handleOpen(expandedSession.pid, expandedSession.projectPath)}
	/>
{/if}

<style>
	.monitor-page {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
		padding: var(--space-xl) var(--space-2xl);
		gap: var(--space-xl);
	}

	/* Header */
	.monitor-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-lg) 0;
		border-bottom: 1px solid var(--border-default);
	}

	.page-title {
		font-family: var(--font-mono);
		font-size: 13px;
		font-weight: 600;
		letter-spacing: 0.12em;
		color: var(--text-primary);
		margin: 0;
	}

	.session-count {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-muted);
		letter-spacing: 0.05em;
		text-transform: uppercase;
	}

	/* Session Groups */
	.session-groups {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.session-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.group-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.group-label {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
		text-transform: uppercase;
	}

	.attention-label {
		color: var(--status-permission);
	}

	.working-label {
		color: var(--status-working);
	}

	.idle-label {
		color: var(--text-muted);
	}

	.group-count {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.session-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
		gap: var(--space-md);
	}

	.empty-group {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
		opacity: 0.5;
		padding: var(--space-sm) 0;
		letter-spacing: 0.05em;
	}

	/* Cost section */
	.cost-section {
		border-top: 1px solid var(--border-default);
		padding-top: var(--space-lg);
	}

	.cost-toggle {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		background: none;
		border: none;
		padding: var(--space-sm) 0;
		cursor: pointer;
		color: var(--text-muted);
		transition: color var(--transition-fast);
	}

	.cost-toggle:hover {
		color: var(--text-secondary);
	}

	.cost-toggle-label {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.1em;
	}

	.cost-chevron {
		transition: transform 150ms ease-out;
	}

	.cost-chevron.collapsed {
		transform: rotate(-90deg);
	}

	.cost-container {
		margin-top: var(--space-md);
	}

	/* Empty state */
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-2xl) 0;
		gap: var(--space-md);
	}

	.empty-icon {
		color: var(--text-muted);
		opacity: 0.3;
	}

	.empty-title {
		font-family: var(--font-mono);
		font-size: 13px;
		font-weight: 500;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.empty-sub {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		opacity: 0.6;
		letter-spacing: 0.05em;
	}

	/* Responsive */
	@media (max-width: 768px) {
		.monitor-page {
			padding: var(--space-lg) var(--space-md);
		}

		.session-grid {
			grid-template-columns: 1fr;
		}

		.page-title {
			font-size: 12px;
		}
	}
</style>
