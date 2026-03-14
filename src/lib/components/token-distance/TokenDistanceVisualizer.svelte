<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { MILESTONES, tokensToHeight, formatHeight, getCurrentMilestone } from './milestones';
	import type { Milestone } from './milestones';

	let { totalTokens, onclose }: { totalTokens: number; onclose: () => void } = $props();

	// ── Animation phases ─────────────────────────────────────────
	// Phase 1: "stacking" — zoomed in, grains appear, viewport scrolls up
	// Phase 2: "zoomout"  — transition to zoomed-out full-stack view
	// Phase 3: "final"    — static final view with height markers

	type Phase = 'stacking' | 'zoomout' | 'final';
	let phase = $state<Phase>('stacking');

	let currentTokens = $state(0);
	let lastPassedMilestone = $state<string | null>(null);
	let milestoneFlash = $state(false);
	let flashTimeoutId: ReturnType<typeof setTimeout> | null = null;
	let animFrameId: number | null = null;

	// ── Stacking phase state ─────────────────────────────────────
	// The viewport shows VIEWPORT_ROWS rows of COLS width.
	// The total stack can be much taller — we track the scroll offset.
	const COLS = 30;
	const VIEWPORT_ROWS = 20;

	// Total rows needed = totalTokens / COLS (each row = COLS grains)
	// We render a sliding window of VIEWPORT_ROWS from the top of the fill.
	let filledGrains = $state(0);

	const totalRows = $derived(Math.ceil(totalTokens / COLS));
	const filledRows = $derived(Math.ceil(filledGrains / COLS));
	const topPartialCount = $derived(filledGrains % COLS);

	// The viewport shows rows around the current fill line
	const viewportLines = $derived.by((): string[] => {
		if (phase !== 'stacking' || totalTokens === 0) return [];

		const lines: string[] = [];
		// Show VIEWPORT_ROWS rows, with the fill-line near the top (row 3-4)
		const fillRow = filledRows; // 1-indexed: row where grains are being added
		const viewTop = Math.max(0, fillRow - 3); // scroll so fill is near top

		for (let i = 0; i < VIEWPORT_ROWS; i++) {
			const rowIdx = viewTop + VIEWPORT_ROWS - 1 - i; // top-down rendering

			// Check if a milestone lands at this row
			const milestoneAtRow = MILESTONES.find(m => {
				const mRow = Math.ceil(m.tokens / COLS);
				return mRow === rowIdx && currentTokens >= m.tokens;
			});

			let content: string;
			if (rowIdx > filledRows) {
				// Above fill — empty
				content = ' '.repeat(COLS);
			} else if (rowIdx === filledRows && topPartialCount > 0) {
				// Partially filled top row
				content = '·'.repeat(topPartialCount) + ' '.repeat(COLS - topPartialCount);
			} else if (rowIdx <= filledRows && rowIdx > 0) {
				// Fully filled row
				content = '·'.repeat(COLS);
			} else {
				content = ' '.repeat(COLS);
			}

			const marker = milestoneAtRow ? ` ◄ ${milestoneAtRow.label}` : '';
			lines.push(`│${content}│${marker}`);
		}

		// Ground line at bottom (only if we can see the ground)
		if (viewTop <= 0) {
			lines[lines.length - 1] = `└${'─'.repeat(COLS)}┘`;
		}

		return lines;
	});

	// ── Zoom-out phase: full stack with milestone rulers ──────────
	const reached = $derived(getCurrentMilestone(totalTokens));
	const passedMilestones = $derived(MILESTONES.filter(m => totalTokens >= m.tokens));

	// For the zoomed-out view, we show a compact bar with milestone markers
	const ZOOMOUT_HEIGHT = 300; // px height of the zoomed-out bar
	const milestoneMarkers = $derived.by((): Array<{ label: string; pct: number; height: string }> => {
		if (totalTokens === 0) return [];
		return passedMilestones.map(m => ({
			label: m.label,
			pct: Math.min((m.tokens / totalTokens) * 100, 100),
			height: formatHeight(tokensToHeight(m.tokens)),
		}));
	});

	// ── Formatting ───────────────────────────────────────────────
	const currentHeight = $derived(tokensToHeight(currentTokens));

	function formatTokenCount(n: number): string {
		if (n >= 1_000_000_000) return (n / 1_000_000_000).toFixed(1) + 'B';
		if (n >= 1_000_000) return (n / 1_000_000).toFixed(1) + 'M';
		if (n >= 1_000) return (n / 1_000).toFixed(1) + 'K';
		return n.toString();
	}

	// ── Animation ────────────────────────────────────────────────
	function startAnimation() {
		if (totalTokens === 0) {
			phase = 'final';
			return;
		}

		const stackDuration = 4500; // ms for stacking phase
		const startTime = performance.now();
		let lastMilestoneIdx = -1;

		function tick(now: number) {
			const elapsed = now - startTime;
			const t = Math.min(elapsed / stackDuration, 1);

			// 3-phase easing: slow → fast → slow
			let eased: number;
			if (t < 0.12) {
				// Phase A: Very slow start — individual grains visible
				const local = t / 0.12;
				eased = 0.02 * (local * local);
			} else if (t < 0.65) {
				// Phase B: Accelerate through the middle
				const local = (t - 0.12) / 0.53;
				eased = 0.02 + 0.68 * local;
			} else {
				// Phase C: Decelerate to landing
				const local = (t - 0.65) / 0.35;
				eased = 0.7 + 0.3 * (1 - Math.pow(1 - local, 3));
			}

			currentTokens = Math.round(eased * totalTokens);
			filledGrains = Math.round(eased * totalTokens);

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
				// Stacking done — transition to zoom-out
				currentTokens = totalTokens;
				filledGrains = totalTokens;
				phase = 'zoomout';
				// After a brief pause, switch to final
				setTimeout(() => {
					phase = 'final';
				}, 1200);
			}
		}

		animFrameId = requestAnimationFrame(tick);
	}

	function skipToFinal() {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		currentTokens = totalTokens;
		filledGrains = totalTokens;
		phase = 'final';
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			if (phase === 'final') onclose();
			else skipToFinal();
		}
		if (e.key === ' ') {
			e.preventDefault();
			if (phase !== 'final') skipToFinal();
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
<div class="overlay" onclick={phase === 'final' ? onclose : skipToFinal}>
	<div class="visualizer" onclick={(e) => e.stopPropagation()}>
		<!-- Header -->
		<div class="viz-header">
			<span class="viz-title">YOUR TOKEN JOURNEY</span>
			<button class="viz-close" onclick={onclose}>✕</button>
		</div>

		{#if totalTokens === 0}
			<div class="empty-state">No tokens yet — start a Claude session to begin your journey!</div>

		{:else if phase === 'stacking'}
			<!-- ── Phase 1: Zoomed-in stacking ─────────────────── -->
			<div class="stacking-view">
				<div class="stack-viewport">
					<pre class="rice-stack">{viewportLines.join('\n')}</pre>
				</div>

				<div class="stacking-stats">
					<div class="token-count-sm">{formatTokenCount(currentTokens)}</div>
					<div class="token-label">tokens</div>
					<div class="height-display-sm">{formatHeight(currentHeight)}</div>

					{#if lastPassedMilestone}
						<div class="milestone-flash-label" class:flash={milestoneFlash}>
							{lastPassedMilestone}
						</div>
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

		{:else if phase === 'zoomout'}
			<!-- ── Phase 2: Zoom-out transition ────────────────── -->
			<div class="zoomout-view">
				<div class="zoomout-text">ZOOMING OUT...</div>
			</div>

		{:else}
			<!-- ── Phase 3: Final zoomed-out view ──────────────── -->
			<div class="final-view">
				<!-- Full stack bar with milestone rulers -->
				<div class="final-stack-area">
					<div class="final-bar-container" style="height: {ZOOMOUT_HEIGHT}px">
						<!-- The filled bar -->
						<div class="final-bar"></div>

						<!-- Milestone markers -->
						{#each milestoneMarkers as m}
							<div class="final-milestone" style="bottom: {m.pct}%">
								<span class="final-milestone-line"></span>
								<span class="final-milestone-label">{m.label}</span>
								<span class="final-milestone-height">{m.height}</span>
							</div>
						{/each}
					</div>
				</div>

				<!-- Final stats -->
				<div class="final-stats">
					<div class="rice-icon">🍚</div>
					<div class="final-token-count">{formatTokenCount(totalTokens)}</div>
					<div class="token-label">tokens stacked</div>
					<div class="final-height">{formatHeight(tokensToHeight(totalTokens))}</div>
					<div class="height-sub">rice stack height</div>
					{#if reached}
						<div class="final-reached">Past {reached.label}!</div>
					{/if}
				</div>
			</div>
		{/if}

		{#if phase !== 'final'}
			<div class="skip-hint">PRESS SPACE TO SKIP</div>
		{:else if totalTokens > 0}
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
		max-width: 750px;
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

	/* ── Phase 1: Stacking view ──────────── */
	.stacking-view {
		display: flex;
		gap: var(--space-xl);
		align-items: center;
		justify-content: center;
		width: 100%;
		animation: fadeIn 0.3s ease-out;
	}

	.stack-viewport {
		flex-shrink: 0;
		overflow: hidden;
	}

	.rice-stack {
		font-family: var(--font-mono);
		font-size: 12px;
		line-height: 1.15;
		white-space: pre;
		margin: 0;
		color: var(--accent-amber);
	}

	.stacking-stats {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		min-width: 140px;
	}

	.token-count-sm {
		font-family: var(--font-pixel);
		font-size: 36px;
		color: var(--accent-amber);
		text-shadow: 0 0 20px var(--status-permission-glow);
		line-height: 1;
	}

	.token-label {
		font-family: var(--font-mono);
		font-size: 11px;
		text-transform: uppercase;
		color: var(--text-muted);
		letter-spacing: 0.1em;
	}

	.height-display-sm {
		font-family: var(--font-pixel);
		font-size: 18px;
		color: var(--text-primary);
		margin-top: var(--space-sm);
	}

	.milestone-flash-label {
		font-family: var(--font-pixel);
		font-size: 13px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-primary);
		min-height: 18px;
		margin-top: var(--space-md);
		text-align: center;
		transition: opacity 0.3s;
	}

	.milestone-flash-label.flash {
		color: var(--accent-amber);
		text-shadow: 0 0 10px var(--status-permission-glow);
		animation: milestoneFlash 0.8s ease-out;
	}

	@keyframes milestoneFlash {
		0% { transform: scale(1.4); opacity: 1; }
		100% { transform: scale(1); opacity: 0.8; }
	}

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

	/* ── Phase 2: Zoom-out transition ────── */
	.zoomout-view {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 350px;
		animation: zoomPulse 1.2s ease-in-out;
	}

	.zoomout-text {
		font-family: var(--font-pixel);
		font-size: 16px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.2em;
		animation: blink 0.6s infinite;
	}

	@keyframes zoomPulse {
		0% { transform: scale(1); opacity: 1; }
		50% { transform: scale(0.8); opacity: 0.5; }
		100% { transform: scale(1); opacity: 1; }
	}

	/* ── Phase 3: Final zoomed-out view ──── */
	.final-view {
		display: flex;
		gap: var(--space-xl);
		align-items: stretch;
		justify-content: center;
		width: 100%;
		animation: revealIn 0.8s ease-out;
	}

	@keyframes revealIn {
		from { opacity: 0; transform: scale(0.9); }
		to { opacity: 1; transform: scale(1); }
	}

	.final-stack-area {
		display: flex;
		align-items: flex-end;
		flex-shrink: 0;
	}

	.final-bar-container {
		position: relative;
		width: 60px;
	}

	.final-bar {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 100%;
		background: linear-gradient(
			to top,
			var(--accent-amber),
			color-mix(in srgb, var(--accent-amber) 60%, transparent)
		);
		border: 1px solid var(--accent-amber);
		animation: barGrow 0.8s ease-out;
		transform-origin: bottom;
	}

	@keyframes barGrow {
		from { transform: scaleY(0); }
		to { transform: scaleY(1); }
	}

	.final-milestone {
		position: absolute;
		left: calc(100% + 8px);
		display: flex;
		align-items: center;
		gap: 6px;
		white-space: nowrap;
		transform: translateY(50%);
	}

	.final-milestone-line {
		display: block;
		width: 12px;
		height: 1px;
		background: var(--border-default);
	}

	.final-milestone-label {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-primary);
		text-transform: uppercase;
	}

	.final-milestone-height {
		font-family: var(--font-mono);
		font-size: 9px;
		color: var(--text-muted);
	}

	.final-stats {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm);
		min-width: 160px;
	}

	.rice-icon { font-size: 32px; }

	.final-token-count {
		font-family: var(--font-pixel);
		font-size: 48px;
		color: var(--accent-amber);
		text-shadow: 0 0 30px var(--status-permission-glow);
		line-height: 1;
	}

	.final-height {
		font-family: var(--font-pixel);
		font-size: 28px;
		color: var(--text-primary);
		margin-top: var(--space-sm);
	}

	.height-sub {
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		color: var(--text-muted);
		letter-spacing: 0.05em;
	}

	.final-reached {
		font-family: var(--font-pixel);
		font-size: 16px;
		color: var(--accent-amber);
		text-transform: uppercase;
		margin-top: var(--space-md);
	}

	/* ── Shared ──────────────────────────── */
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
