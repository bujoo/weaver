<script lang="ts">
	interface Props {
		missionId: string;
		phaseId: string;
		todoId: string;
	}

	let { missionId, phaseId, todoId }: Props = $props();

	type StreamMode = 'structured' | 'raw';
	let mode: StreamMode = $state('structured');

	interface StreamEvent {
		time: string;
		type: 'TOOL' | 'READ' | 'THINK' | 'WRITE' | 'TEST';
		detail: string;
	}

	// Placeholder demo data -- will be replaced with real event feed
	const demoEvents: StreamEvent[] = [
		{ time: '10:23', type: 'READ', detail: 'src/auth/middleware.ts' },
		{ time: '10:23', type: 'THINK', detail: 'Analyzing JWT validation flow...' },
		{ time: '10:24', type: 'TOOL', detail: 'Edit auth/jwt.ts (12 lines)' },
		{ time: '10:24', type: 'READ', detail: 'src/config/env.ts' },
		{ time: '10:25', type: 'WRITE', detail: 'src/auth/jwt.ts' },
		{ time: '10:25', type: 'TEST', detail: 'npm test -- auth.test.ts' },
		{ time: '10:26', type: 'THINK', detail: 'Verifying token refresh logic...' },
		{ time: '10:26', type: 'TOOL', detail: 'Edit auth/refresh.ts (8 lines)' },
	];

	const demoRawOutput = `$ reading src/auth/middleware.ts
> analyzing JWT validation patterns
> found existing validateToken() at line 42
$ edit auth/jwt.ts
  + import { verify } from 'jsonwebtoken';
  + export function validateJWT(token: string) {
  +   return verify(token, process.env.JWT_SECRET);
  + }
$ reading src/config/env.ts
> checking environment variable definitions
$ write src/auth/jwt.ts (12 lines modified)
$ test npm test -- auth.test.ts
  PASS auth.test.ts (3 tests passed)`;

	// Text icons for event types (no emoji)
	const typeIcons: Record<StreamEvent['type'], string> = {
		TOOL: '[T]',
		READ: '[R]',
		THINK: '[?]',
		WRITE: '[W]',
		TEST: '[!]',
	};

	const typeColors: Record<StreamEvent['type'], string> = {
		TOOL: 'var(--accent-amber)',
		READ: 'var(--accent-blue)',
		THINK: 'var(--accent-purple)',
		WRITE: 'var(--accent-green)',
		TEST: 'var(--accent-pink)',
	};

	let streamContainer: HTMLDivElement | undefined = $state(undefined);

	// Auto-scroll to bottom when mode changes
	$effect(() => {
		// Reference mode to track changes
		void mode;
		if (streamContainer) {
			streamContainer.scrollTop = streamContainer.scrollHeight;
		}
	});
</script>

<div class="live-stream">
	<div class="stream-header">
		<span class="stream-label">LIVE STREAM</span>
		<div class="mode-toggle">
			<button
				class="mode-btn"
				class:active={mode === 'structured'}
				onclick={() => (mode = 'structured')}
				type="button"
			>
				structured
			</button>
			<button
				class="mode-btn"
				class:active={mode === 'raw'}
				onclick={() => (mode = 'raw')}
				type="button"
			>
				raw
			</button>
		</div>
	</div>

	<div class="stream-body" bind:this={streamContainer}>
		{#if mode === 'structured'}
			<div class="structured-events">
				{#each demoEvents as event}
					<div class="event-row">
						<span class="event-time">{event.time}</span>
						<span class="event-icon" style="color: {typeColors[event.type]}">{typeIcons[event.type]}</span>
						<span class="event-type" style="color: {typeColors[event.type]}">{event.type}</span>
						<span class="event-detail">{event.detail}</span>
					</div>
				{/each}
			</div>
		{:else}
			<pre class="raw-output">{demoRawOutput}</pre>
		{/if}
	</div>
</div>

<style>
	.live-stream {
		border: 1px solid var(--border-muted);
		background: var(--bg-base);
		margin-top: var(--space-sm);
	}

	.stream-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-xs) var(--space-sm);
		border-bottom: 1px solid var(--border-muted);
	}

	.stream-label {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.08em;
		color: var(--text-muted);
	}

	.mode-toggle {
		display: flex;
		gap: 1px;
	}

	.mode-btn {
		font-family: var(--font-mono);
		font-size: 10px;
		padding: 2px 6px;
		color: var(--text-muted);
		background: transparent;
		border: 1px solid var(--border-muted);
		cursor: pointer;
		text-transform: lowercase;
		transition: color var(--transition-fast), background var(--transition-fast);
	}

	.mode-btn:hover {
		color: var(--text-secondary);
	}

	.mode-btn.active {
		color: var(--text-primary);
		background: var(--bg-card-hover);
		border-color: var(--border-default);
	}

	.stream-body {
		max-height: 200px;
		overflow-y: auto;
		padding: var(--space-xs) var(--space-sm);
	}

	/* Structured view */
	.structured-events {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.event-row {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
		padding: 2px 0;
		font-size: 12px;
		font-family: var(--font-mono);
	}

	.event-time {
		color: var(--text-muted);
		font-size: 11px;
		flex-shrink: 0;
		width: 36px;
	}

	.event-icon {
		font-size: 11px;
		flex-shrink: 0;
		width: 20px;
		text-align: center;
	}

	.event-type {
		font-size: 11px;
		font-weight: 600;
		flex-shrink: 0;
		width: 40px;
		text-transform: uppercase;
	}

	.event-detail {
		color: var(--text-secondary);
		font-size: 12px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	/* Raw view */
	.raw-output {
		font-family: var(--font-mono);
		font-size: 11px;
		line-height: 1.6;
		color: var(--text-secondary);
		white-space: pre-wrap;
		word-break: break-all;
		margin: 0;
	}
</style>
