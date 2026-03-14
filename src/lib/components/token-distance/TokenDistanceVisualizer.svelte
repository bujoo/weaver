<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { MILESTONES, tokensToHeight, formatHeight, getCurrentMilestone, getNextMilestone } from './milestones';
	import { getSceneForTokens } from './asciiArt';

	let { totalTokens, onclose }: { totalTokens: number; onclose: () => void } = $props();

	let currentTokens = $state(0);
	let animationDone = $state(false);
	let lastPassedMilestone = $state<string | null>(null);
	let milestoneFlash = $state(false);
	let animFrameId: number | null = null;

	const currentHeight = $derived(tokensToHeight(currentTokens));
	const currentScene = $derived(getSceneForTokens(currentTokens));
	const reached = $derived(getCurrentMilestone(currentTokens));
	const next = $derived(getNextMilestone(currentTokens));

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

		const duration = 4000;
		const startTime = performance.now();
		let lastMilestoneIdx = -1;

		function tick(now: number) {
			const elapsed = now - startTime;
			const progress = Math.min(elapsed / duration, 1);
			const eased = 1 - Math.pow(1 - progress, 3);

			currentTokens = Math.round(eased * totalTokens);

			const currentIdx = MILESTONES.findLastIndex(m => currentTokens >= m.tokens);
			if (currentIdx > lastMilestoneIdx && currentIdx >= 0) {
				lastMilestoneIdx = currentIdx;
				lastPassedMilestone = MILESTONES[currentIdx].label;
				milestoneFlash = true;
				setTimeout(() => milestoneFlash = false, 600);
			}

			if (progress < 1) {
				animFrameId = requestAnimationFrame(tick);
			} else {
				currentTokens = totalTokens;
				animationDone = true;
			}
		}

		animFrameId = requestAnimationFrame(tick);
	}

	function skipAnimation() {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		currentTokens = totalTokens;
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
		setTimeout(startAnimation, 300);
	});

	onDestroy(() => {
		if (animFrameId) cancelAnimationFrame(animFrameId);
		document.removeEventListener('keydown', handleKeydown);
	});
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={animationDone ? onclose : skipAnimation}>
	<div class="visualizer" onclick={(e) => e.stopPropagation()}>
		<div class="viz-header">
			<span class="viz-title">YOUR TOKEN JOURNEY</span>
			<button class="viz-close" onclick={onclose}>✕</button>
		</div>

		<div class="scene-container">
			<pre class="ascii-scene {currentScene.theme}">{currentScene.lines.join('\n')}</pre>
		</div>

		{#if lastPassedMilestone}
			<div class="milestone-label" class:flash={milestoneFlash}>
				{lastPassedMilestone}
			</div>
		{/if}

		<div class="milestone-track">
			{#each MILESTONES as m}
				<div class="milestone-dot" class:reached={currentTokens >= m.tokens}>
					{#if currentTokens >= m.tokens}
						<span class="dot-filled">●</span>
					{:else}
						<span class="dot-empty">○</span>
					{/if}
				</div>
			{/each}
		</div>

		<div class="counter-section">
			<div class="rice-icon">🍚</div>
			<div class="token-count">{formatTokenCount(currentTokens)}</div>
			<div class="token-label">tokens</div>
			<div class="height-display">{formatHeight(currentHeight)}</div>
			{#if reached}
				<div class="reached-label">Past {reached.label}!</div>
			{/if}
			{#if next && !animationDone}
				<div class="next-label">Next: {next.label}</div>
			{/if}
		</div>

		{#if totalTokens === 0}
			<div class="empty-state">No tokens yet — start a Claude session to begin your journey!</div>
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
		max-width: 600px;
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

	.scene-container {
		width: 100%;
		display: flex;
		justify-content: center;
		min-height: 180px;
		transition: opacity 0.5s ease;
	}

	.ascii-scene {
		font-family: var(--font-mono);
		font-size: 14px;
		line-height: 1.4;
		white-space: pre;
		margin: 0;
		transition: color 0.5s ease;
	}

	.ascii-scene.ground { color: var(--accent-amber); }
	.ascii-scene.mountain { color: var(--text-primary); }
	.ascii-scene.sky { color: var(--accent-purple, #7928ca); }
	.ascii-scene.space { color: var(--text-muted); }

	.milestone-label {
		font-family: var(--font-pixel);
		font-size: 18px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-primary);
		min-height: 24px;
		transition: opacity 0.3s;
	}

	.milestone-label.flash {
		color: var(--accent-amber);
		text-shadow: 0 0 10px var(--status-permission-glow);
		animation: milestoneFlash 0.6s ease-out;
	}

	@keyframes milestoneFlash {
		0% { transform: scale(1.3); opacity: 1; }
		100% { transform: scale(1); opacity: 0.8; }
	}

	.milestone-track {
		display: flex;
		gap: 6px;
		align-items: center;
	}

	.milestone-dot {
		font-size: 8px;
		transition: color 0.3s;
	}

	.milestone-dot .dot-filled { color: var(--accent-amber); }
	.milestone-dot .dot-empty { color: var(--border-default); }

	.counter-section {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
	}

	.rice-icon { font-size: 32px; }

	.token-count {
		font-family: var(--font-pixel);
		font-size: 48px;
		color: var(--accent-amber);
		text-shadow: 0 0 30px var(--status-permission-glow);
	}

	.token-label {
		font-family: var(--font-mono);
		font-size: 13px;
		text-transform: uppercase;
		color: var(--text-muted);
		letter-spacing: 0.1em;
	}

	.height-display {
		font-family: var(--font-mono);
		font-size: 16px;
		color: var(--text-muted);
	}

	.reached-label {
		font-family: var(--font-pixel);
		font-size: 14px;
		color: var(--accent-amber);
		text-transform: uppercase;
	}

	.next-label {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-transform: uppercase;
	}

	.empty-state {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-muted);
		text-transform: uppercase;
		text-align: center;
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
