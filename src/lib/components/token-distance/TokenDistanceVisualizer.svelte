<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { MILESTONES, tokensToHeight, formatHeight, getCurrentMilestone, getNextMilestone } from './milestones';
	import type { Milestone } from './milestones';

	let { totalTokens, onclose }: { totalTokens: number; onclose: () => void } = $props();

	let currentTokens = $state(0);
	let animationDone = $state(false);
	let lastPassedMilestone = $state<string | null>(null);
	let milestoneFlash = $state(false);
	let flashTimeoutId: ReturnType<typeof setTimeout> | null = null;
	let animFrameId: number | null = null;

	// Rice stack visual state
	const STACK_ROWS = 24;
	const STACK_COLS = 20;
	let grainCount = $state(0); // number of grain cells filled in the visual stack

	const currentHeight = $derived(tokensToHeight(currentTokens));
	const reached = $derived(getCurrentMilestone(currentTokens));
	const next = $derived(getNextMilestone(currentTokens));

	// Milestones that are within the "visible" range relative to current progress
	const visibleMilestones = $derived.by((): Array<Milestone & { row: number }> => {
		if (totalTokens === 0) return [];
		// Show milestones as markers along the stack height
		return MILESTONES
			.filter(m => m.tokens <= totalTokens * 1.2) // show slightly beyond current
			.map(m => {
				// Map token position to row (0 = bottom, STACK_ROWS-1 = top)
				const ratio = Math.min(m.tokens / totalTokens, 1);
				const row = Math.floor(ratio * (STACK_ROWS - 1));
				return { ...m, row };
			});
	});

	// Build the visual rice stack as an array of strings (bottom to top)
	const stackLines = $derived.by((): string[] => {
		const totalCells = STACK_ROWS * STACK_COLS;
		const filledCells = Math.min(grainCount, totalCells);
		const filledRows = Math.ceil(filledCells / STACK_COLS);
		const lines: string[] = [];

		for (let row = STACK_ROWS - 1; row >= 0; row--) {
			// Find if a milestone marker is at this row
			const milestone = visibleMilestones.find(m => m.row === row && currentTokens >= m.tokens);
			const isAboveFill = row >= filledRows;

			let rowStr: string;
			if (isAboveFill) {
				// Empty space above the rice
				rowStr = ' '.repeat(STACK_COLS);
			} else if (row === filledRows - 1 && filledCells % STACK_COLS !== 0) {
				// Partially filled top row
				const filled = filledCells % STACK_COLS;
				rowStr = '·'.repeat(filled) + ' '.repeat(STACK_COLS - filled);
			} else {
				// Fully filled row
				rowStr = '·'.repeat(STACK_COLS);
			}

			// Add milestone label on the right if this row has one
			if (milestone) {
				const marker = ` ◄ ${milestone.label}`;
				lines.push(`│${rowStr}│${marker}`);
			} else {
				lines.push(`│${rowStr}│`);
			}
		}

		// Ground line
		lines.push(`└${'─'.repeat(STACK_COLS)}┘`);
		return lines;
	});

	function formatTokenCount(n: number): string {
		if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(1) + 'B';
		if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
		if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
		return n.toString();
	}

	function startAnimation() {
		if (totalTokens === 0) {
			animationDone = true;
			return;
		}

		const totalCells = STACK_ROWS * STACK_COLS;
		const duration = 5000; // 5 seconds
		const startTime = performance.now();
		let lastMilestoneIdx = -1;

		function tick(now: number) {
			const elapsed = now - startTime;
			const t = Math.min(elapsed / duration, 1);

			// 3-phase easing: slow start → accelerate → decelerate
			// Using a custom curve: ease-in for first 20%, linear for 20-70%, ease-out for 70-100%
			let eased: number;
			if (t < 0.15) {
				// Slow start (ease-in quadratic) — grains appear one by one
				const local = t / 0.15;
				eased = 0.05 * (local * local);
			} else if (t < 0.7) {
				// Accelerate (linear ramp through the middle)
				const local = (t - 0.15) / 0.55;
				eased = 0.05 + 0.65 * local;
			} else {
				// Decelerate (ease-out cubic) — slow landing
				const local = (t - 0.7) / 0.3;
				eased = 0.7 + 0.3 * (1 - Math.pow(1 - local, 3));
			}

			currentTokens = Math.round(eased * totalTokens);
			grainCount = Math.round(eased * totalCells);

			// Check milestones
			const currentIdx = MILESTONES.findLastIndex(m => currentTokens >= m.tokens);
			if (currentIdx > lastMilestoneIdx && currentIdx >= 0) {
				lastMilestoneIdx = currentIdx;
				lastPassedMilestone = MILESTONES[currentIdx].label;
				if (flashTimeoutId) clearTimeout(flashTimeoutId);
				milestoneFlash = true;
				flashTimeoutId = setTimeout(() => milestoneFlash = false, 800);
			}

			if (t < 1) {
				animFrameId = requestAnimationFrame(tick);
			} else {
				currentTokens = totalTokens;
				grainCount = totalCells;
				animationDone = true;
			}
		}

		animFrameId = requestAnimationFrame(tick);
	}

	function skipAnimation() {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		currentTokens = totalTokens;
		grainCount = STACK_ROWS * STACK_COLS;
		animationDone = true;
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (animationDone) onclose();
			else skipAnimation();
		}
		if (e.key === ' ') {
			e.preventDefault();
			if (!animationDone) skipAnimation();
		}
	}

	onMount(() => {
		document.addEventListener('keydown', handleKeydown);
		setTimeout(startAnimation, 400);
	});

	onDestroy(() => {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		if (flashTimeoutId) clearTimeout(flashTimeoutId);
		document.removeEventListener('keydown', handleKeydown);
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={animationDone ? onclose : skipAnimation}>
	<div class="visualizer" onclick={(e) => e.stopPropagation()}>
		<!-- Header -->
		<div class="viz-header">
			<span class="viz-title">YOUR TOKEN JOURNEY</span>
			<button class="viz-close" onclick={onclose}>✕</button>
		</div>

		{#if totalTokens === 0}
			<div class="empty-state">No tokens yet — start a Claude session to begin your journey!</div>
		{:else}
			<!-- Main content: rice stack + stats side by side -->
			<div class="viz-body">
				<!-- Rice stack column -->
				<div class="stack-container">
					<pre class="rice-stack">{stackLines.join('\n')}</pre>
				</div>

				<!-- Stats panel -->
				<div class="stats-panel">
					<div class="rice-icon">🍚</div>
					<div class="token-count">{formatTokenCount(currentTokens)}</div>
					<div class="token-label">tokens</div>
					<div class="height-value">{formatHeight(currentHeight)}</div>
					<div class="height-label">rice stack height</div>

					<!-- Milestone flash -->
					{#if lastPassedMilestone}
						<div class="milestone-label" class:flash={milestoneFlash}>
							{lastPassedMilestone}
						</div>
					{/if}

					{#if reached && animationDone}
						<div class="reached-label">Past {reached.label}!</div>
					{/if}
					{#if next && !animationDone}
						<div class="next-label">Next: {next.label}</div>
					{/if}

					<!-- Milestone dot track -->
					<div class="milestone-track">
						{#each MILESTONES as m}
							<span class="dot" class:reached={currentTokens >= m.tokens}>
								{currentTokens >= m.tokens ? '●' : '○'}
							</span>
						{/each}
					</div>
				</div>
			</div>
		{/if}

		{#if !animationDone}
			<div class="skip-hint">PRESS SPACE TO SKIP</div>
		{:else}
			<div class="skip-hint">PRESS ESC TO CLOSE</div>
		{/if}
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		z-index: 9999;
		display: flex;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.3s ease-out;
	}

	@keyframes fadeIn {
		from { opacity: 0; }
		to { opacity: 1; }
	}

	.visualizer {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-lg);
		padding: var(--space-xl);
		max-width: 700px;
		width: 100%;
	}

	.viz-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
	}

	.viz-title {
		font-family: var(--font-pixel);
		font-size: 22px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		color: var(--accent-amber);
		text-shadow: 0 0 20px var(--status-permission-glow);
	}

	.viz-close {
		background: none;
		border: 1px solid var(--border-default);
		color: var(--text-muted);
		font-family: var(--font-mono);
		font-size: 14px;
		padding: 4px 8px;
		cursor: pointer;
		transition: color 0.15s, border-color 0.15s;
	}

	.viz-close:hover {
		color: var(--text-primary);
		border-color: var(--text-primary);
	}

	/* ── Main body layout ────────────────── */
	.viz-body {
		display: flex;
		gap: var(--space-xl);
		align-items: center;
		width: 100%;
		justify-content: center;
	}

	/* ── Rice stack column ────────────────── */
	.stack-container {
		flex-shrink: 0;
	}

	.rice-stack {
		font-family: var(--font-mono);
		font-size: 13px;
		line-height: 1.2;
		white-space: pre;
		margin: 0;
		color: var(--accent-amber);
	}

	/* ── Stats panel ─────────────────────── */
	.stats-panel {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		min-width: 160px;
	}

	.rice-icon { font-size: 28px; }

	.token-count {
		font-family: var(--font-pixel);
		font-size: 42px;
		color: var(--accent-amber);
		text-shadow: 0 0 30px var(--status-permission-glow);
		line-height: 1;
	}

	.token-label {
		font-family: var(--font-mono);
		font-size: 12px;
		text-transform: uppercase;
		color: var(--text-muted);
		letter-spacing: 0.1em;
	}

	.height-value {
		font-family: var(--font-pixel);
		font-size: 24px;
		color: var(--text-primary);
		margin-top: var(--space-sm);
	}

	.height-label {
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.milestone-label {
		font-family: var(--font-pixel);
		font-size: 14px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-primary);
		min-height: 20px;
		margin-top: var(--space-md);
		text-align: center;
		transition: opacity 0.3s;
	}

	.milestone-label.flash {
		color: var(--accent-amber);
		text-shadow: 0 0 10px var(--status-permission-glow);
		animation: milestoneFlash 0.8s ease-out;
	}

	@keyframes milestoneFlash {
		0% { transform: scale(1.4); opacity: 1; }
		100% { transform: scale(1); opacity: 0.8; }
	}

	.reached-label {
		font-family: var(--font-pixel);
		font-size: 13px;
		color: var(--accent-amber);
		text-transform: uppercase;
		text-align: center;
	}

	.next-label {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
	}

	/* ── Milestone dot track ─────────────── */
	.milestone-track {
		display: flex;
		gap: 4px;
		align-items: center;
		margin-top: var(--space-sm);
	}

	.dot {
		font-size: 6px;
		color: var(--border-default);
		transition: color 0.3s;
	}

	.dot.reached { color: var(--accent-amber); }

	/* ── Empty & hints ───────────────────── */
	.empty-state {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-muted);
		text-transform: uppercase;
		text-align: center;
		padding: var(--space-xl) 0;
	}

	.skip-hint {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		animation: blink 2s infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 0.5; }
		50% { opacity: 1; }
	}
</style>
