<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { MILESTONES, tokensToHeight, formatHeight, getCurrentMilestone } from './milestones';
	import type { Milestone } from './milestones';

	let { totalTokens, onclose }: { totalTokens: number; onclose: () => void } = $props();

	// ── Animation phases ─────────────────────────────────────────
	// stacking: zoomed in, single column of dots growing upward
	// zoomout:  brief transition
	// final:    zoomed-out full tower with height markers
	type Phase = 'stacking' | 'zoomout' | 'final';
	let phase = $state<Phase>('stacking');

	let currentTokens = $state(0);
	let lastPassedMilestone = $state<string | null>(null);
	let milestoneFlash = $state(false);
	let flashTimeoutId: ReturnType<typeof setTimeout> | null = null;
	let animFrameId: number | null = null;

	// ── Stacking phase ───────────────────────────────────────────
	// Each dot = `scale` tokens. The viewport shows VISIBLE_DOTS rows.
	// Total dots = totalTokens / scale.
	const VISIBLE_DOTS = 30; // how many dots visible at once in the viewport

	// Scale: how many tokens per dot. Aim for ~2000 total dots so animation looks good.
	const scale = $derived(Math.max(1, Math.ceil(totalTokens / 2000)));
	const totalDots = $derived(Math.ceil(totalTokens / scale));
	let currentDots = $state(0);

	// The viewport is a sliding window showing VISIBLE_DOTS rows.
	// We always show the top of the stack (where new dots are landing).
	const viewportDots = $derived.by((): Array<{ dot: boolean; milestone: Milestone | null }> => {
		if (phase !== 'stacking' || totalTokens === 0) return [];

		const rows: Array<{ dot: boolean; milestone: Milestone | null }> = [];
		// viewport top = currentDots, viewport bottom = currentDots - VISIBLE_DOTS
		const viewBottom = Math.max(0, currentDots - VISIBLE_DOTS);

		for (let i = VISIBLE_DOTS - 1; i >= 0; i--) {
			const dotIdx = viewBottom + i; // which dot position (0 = ground)
			const tokensAtDot = dotIdx * scale;
			const isFilled = dotIdx < currentDots;

			// Check if a milestone lands at this dot position
			const milestone = MILESTONES.find(m => {
				const mDot = Math.ceil(m.tokens / scale);
				return mDot === dotIdx && currentTokens >= m.tokens;
			}) ?? null;

			rows.push({ dot: isFilled, milestone });
		}

		return rows;
	});

	// ── Final phase ──────────────────────────────────────────────
	const reached = $derived(getCurrentMilestone(totalTokens));
	const passedMilestones = $derived(MILESTONES.filter(m => totalTokens >= m.tokens));

	const TOWER_HEIGHT = 320; // px
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

		const duration = 5000;
		const startTime = performance.now();
		let lastMilestoneIdx = -1;

		function tick(now: number) {
			const elapsed = now - startTime;
			const t = Math.min(elapsed / duration, 1);

			// 3-phase easing: slow → fast → slow
			let eased: number;
			if (t < 0.1) {
				// Very slow start — see individual dots
				const local = t / 0.1;
				eased = 0.015 * (local * local);
			} else if (t < 0.6) {
				// Accelerate
				const local = (t - 0.1) / 0.5;
				eased = 0.015 + 0.685 * local;
			} else {
				// Decelerate to landing
				const local = (t - 0.6) / 0.4;
				eased = 0.7 + 0.3 * (1 - Math.pow(1 - local, 3));
			}

			currentTokens = Math.round(eased * totalTokens);
			currentDots = Math.round(eased * totalDots);

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
				currentDots = totalDots;
				phase = 'zoomout';
				setTimeout(() => { phase = 'final'; }, 1000);
			}
		}

		animFrameId = requestAnimationFrame(tick);
	}

	function skipToFinal() {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		currentTokens = totalTokens;
		currentDots = totalDots;
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
			<!-- ── Phase 1: Zoomed-in single column stacking ──── -->
			<div class="stacking-view">
				<!-- Dot column -->
				<div class="dot-column">
					{#each viewportDots as row}
						<div class="dot-row">
							{#if row.dot}
								<span class="grain">·</span>
							{:else}
								<span class="grain empty"> </span>
							{/if}
							{#if row.milestone}
								<span class="row-milestone">◄ {row.milestone.label}</span>
							{/if}
						</div>
					{/each}
					<div class="ground-line">─</div>
				</div>

				<!-- Stats -->
				<div class="stacking-stats">
					<div class="token-count-sm">{formatTokenCount(currentTokens)}</div>
					<div class="stat-label">tokens</div>
					<div class="height-sm">{formatHeight(currentHeight)}</div>
					<div class="stat-label">height</div>

					{#if lastPassedMilestone}
						<div class="milestone-flash" class:flash={milestoneFlash}>
							{lastPassedMilestone}
						</div>
					{/if}

					<div class="milestone-track">
						{#each MILESTONES as m}
							<span class="track-dot" class:reached={currentTokens >= m.tokens}>
								{currentTokens >= m.tokens ? '●' : '○'}
							</span>
						{/each}
					</div>
				</div>
			</div>

		{:else if phase === 'zoomout'}
			<!-- ── Phase 2: Zoom-out transition ────────────────── -->
			<div class="zoomout-view">
				<div class="zoomout-text">ZOOMING OUT</div>
			</div>

		{:else}
			<!-- ── Phase 3: Final full tower view ──────────────── -->
			<div class="final-view">
				<!-- Tower bar with milestone rulers -->
				<div class="tower-area">
					<div class="tower-container" style="height: {TOWER_HEIGHT}px">
						<div class="tower-bar"></div>
						{#each milestoneMarkers as m}
							<div class="tower-milestone" style="bottom: {m.pct}%">
								<span class="tm-line"></span>
								<span class="tm-label">{m.label}</span>
								<span class="tm-height">{m.height}</span>
							</div>
						{/each}
					</div>
				</div>

				<!-- Final stats -->
				<div class="final-stats">
					<div class="rice-icon">🍚</div>
					<div class="final-count">{formatTokenCount(totalTokens)}</div>
					<div class="stat-label">tokens stacked</div>
					<div class="final-height">{formatHeight(tokensToHeight(totalTokens))}</div>
					<div class="stat-label">rice stack height</div>
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

	/* ── Phase 1: Stacking ───────────────── */
	.stacking-view {
		display: flex;
		gap: var(--space-xl);
		align-items: center;
		justify-content: center;
		width: 100%;
		animation: fadeIn 0.3s ease-out;
	}

	.dot-column {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		flex-shrink: 0;
	}

	.dot-row {
		display: flex;
		align-items: center;
		height: 14px;
		line-height: 14px;
	}

	.grain {
		font-family: var(--font-mono);
		font-size: 18px;
		color: var(--accent-amber);
		width: 12px;
		text-align: center;
		text-shadow: 0 0 4px var(--status-permission-glow);
	}

	.grain.empty {
		color: transparent;
		text-shadow: none;
	}

	.row-milestone {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-primary);
		text-transform: uppercase;
		margin-left: 8px;
		white-space: nowrap;
		animation: fadeIn 0.3s ease-out;
	}

	.ground-line {
		font-family: var(--font-mono);
		font-size: 14px;
		color: var(--border-default);
		width: 12px;
		text-align: center;
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

	.stat-label {
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		color: var(--text-muted);
		letter-spacing: 0.1em;
	}

	.height-sm {
		font-family: var(--font-pixel);
		font-size: 20px;
		color: var(--text-primary);
		margin-top: var(--space-sm);
	}

	.milestone-flash {
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

	.milestone-flash.flash {
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

	.track-dot {
		font-size: 6px;
		color: var(--border-default);
		transition: color 0.3s;
	}

	.track-dot.reached { color: var(--accent-amber); }

	/* ── Phase 2: Zoom-out ───────────────── */
	.zoomout-view {
		display: flex;
		align-items: center;
		justify-content: center;
		min-height: 400px;
		animation: shrinkDown 1s ease-in-out;
	}

	.zoomout-text {
		font-family: var(--font-pixel);
		font-size: 14px;
		color: var(--text-muted);
		text-transform: uppercase;
		letter-spacing: 0.2em;
		animation: blink 0.5s infinite;
	}

	@keyframes shrinkDown {
		0% { transform: scaleY(1) scaleX(1); opacity: 1; }
		50% { transform: scaleY(0.5) scaleX(1.2); opacity: 0.4; }
		100% { transform: scaleY(1) scaleX(1); opacity: 1; }
	}

	/* ── Phase 3: Final tower view ───────── */
	.final-view {
		display: flex;
		gap: var(--space-xl);
		align-items: stretch;
		justify-content: center;
		width: 100%;
		animation: revealIn 0.8s ease-out;
	}

	@keyframes revealIn {
		from { opacity: 0; transform: scale(0.85); }
		to { opacity: 1; transform: scale(1); }
	}

	.tower-area {
		display: flex;
		align-items: flex-end;
		flex-shrink: 0;
	}

	.tower-container {
		position: relative;
		width: 40px;
	}

	.tower-bar {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		height: 100%;
		background: linear-gradient(
			to top,
			var(--accent-amber),
			color-mix(in srgb, var(--accent-amber) 40%, transparent)
		);
		border: 1px solid var(--accent-amber);
		animation: towerGrow 0.8s ease-out;
		transform-origin: bottom;
	}

	@keyframes towerGrow {
		from { transform: scaleY(0); }
		to { transform: scaleY(1); }
	}

	.tower-milestone {
		position: absolute;
		left: calc(100% + 8px);
		display: flex;
		align-items: center;
		gap: 6px;
		white-space: nowrap;
		transform: translateY(50%);
	}

	.tm-line {
		display: block;
		width: 10px;
		height: 1px;
		background: var(--border-default);
	}

	.tm-label {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-primary);
		text-transform: uppercase;
	}

	.tm-height {
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

	.final-count {
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
