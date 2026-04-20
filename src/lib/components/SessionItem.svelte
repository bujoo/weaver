<script lang="ts">
	import type { Session } from '$lib/types';
	import { SessionStatus } from '$lib/types';

	interface Props {
		session: Session;
		selected?: boolean;
		onclick?: () => void;
	}

	let { session, selected = false, onclick }: Props = $props();

	// Get status color based on session status
	function getStatusColor(status: SessionStatus): string {
		switch (status) {
			case SessionStatus.Working:
				return 'var(--accent-blue)';
			case SessionStatus.NeedsAttention:
				return 'var(--accent-amber)';
			case SessionStatus.WaitingForInput:
				return 'var(--accent-green)';
			case SessionStatus.Connecting:
				return 'var(--accent-blue)';
			default:
				return 'var(--text-muted)';
		}
	}

	// Format time since last activity
	function formatTimeSince(isoTimestamp: string): string {
		const now = new Date().getTime();
		const then = new Date(isoTimestamp).getTime();
		const diffMs = now - then;
		const diffMins = Math.floor(diffMs / 60000);
		const diffHours = Math.floor(diffMs / 3600000);
		const diffDays = Math.floor(diffMs / 86400000);

		if (diffMins < 1) return 'just now';
		if (diffMins < 60) return `${diffMins}m ago`;
		if (diffHours < 24) return `${diffHours}h ago`;
		return `${diffDays}d ago`;
	}

	// Truncate first prompt to fit in list
	function truncatePrompt(text: string, maxLength: number = 60): string {
		if (text.length <= maxLength) return text;
		return text.substring(0, maxLength) + '...';
	}
</script>

<button
	class="session-item"
	class:selected
	onclick={onclick}
	type="button"
>
	<div class="status-indicator" style="background-color: {getStatusColor(session.status)}"></div>
	<div class="session-content">
		<div class="session-header">
			<span class="session-name">{session.sessionName}</span>
			{#if session.gitBranch}
				<span class="git-branch">({session.gitBranch})</span>
			{/if}
			<span class="time-since">{formatTimeSince(session.modified)}</span>
		</div>
		<div class="first-prompt">{truncatePrompt(session.summary || session.firstPrompt)}</div>
		<div class="message-count">{session.messageCount} messages</div>
	</div>
</button>

<style>
	.session-item {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 12px;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-sm);
		background: var(--bg-card);
		cursor: pointer;
		transition: all 0.2s;
		width: 100%;
		text-align: left;
	}

	.session-item:hover {
		background: var(--bg-card-hover);
		border-color: var(--border-default);
	}

	.session-item.selected {
		background: var(--weavy-subtle);
		border-color: var(--weavy);
	}

	.status-indicator {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		margin-top: 4px;
		flex-shrink: 0;
	}

	.session-content {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.session-header {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 14px;
	}

	.session-name {
		font-weight: 600;
		color: var(--text-primary);
	}

	.git-branch {
		color: var(--text-secondary);
		font-size: 12px;
	}

	.time-since {
		margin-left: auto;
		color: var(--text-muted);
		font-size: 11px;
		white-space: nowrap;
	}

	.first-prompt {
		color: var(--text-secondary);
		font-size: 13px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.message-count {
		color: var(--text-muted);
		font-size: 11px;
	}
</style>
