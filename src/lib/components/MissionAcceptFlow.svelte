<script lang="ts">
	import type { UnifiedMission } from '$lib/stores/missions';
	import { isTauri } from '$lib/ws';

	interface Props {
		mission: UnifiedMission;
		onaccept: () => void;
		onreject: () => void;
	}

	let { mission, onaccept, onreject }: Props = $props();

	let currentStep = $state(0);
	const STEPS = ['Received', 'Supervisor Check', 'Your Review', 'Accept'];
	const STEP_COUNT = STEPS.length;

	// Transition direction
	let direction = $state<'forward' | 'back'>('forward');

	function nextStep() {
		if (currentStep < STEP_COUNT - 1) {
			direction = 'forward';
			currentStep += 1;
		}
	}

	function prevStep() {
		if (currentStep > 0) {
			direction = 'back';
			currentStep -= 1;
		}
	}

	// Accept actions
	async function handleAcceptStart() {
		if (isTauri()) {
			try {
				const { invoke } = await import('@tauri-apps/api/core');
				// Accept first available phase
				const firstPhase = mission.availablePhases[0];
				if (firstPhase) {
					await invoke('accept_phase_cmd', {
						missionId: mission.missionId,
						phaseId: firstPhase.phaseId,
					});
				}
			} catch (err) {
				console.error('[AcceptFlow] accept_phase_cmd failed:', err);
			}
		}
		onaccept();
	}

	function handleAcceptQueue() {
		console.log('[AcceptFlow] Queued mission:', mission.missionId);
		onaccept();
	}

	let rejectReason = $state('');

	function handleReject() {
		console.log('[AcceptFlow] Rejected mission:', mission.missionId, 'Reason:', rejectReason);
		onreject();
	}

	// Placeholder supervisor validation data
	const validationResults = [
		{ label: 'Context available', status: 'pass', detail: 'All referenced repos accessible' },
		{ label: 'Repo access', status: 'pass', detail: '2 repos cloned, branches ready' },
		{ label: 'Risk assessment', status: 'warn', detail: 'Auth module -- moderate complexity' },
		{ label: 'Estimated effort', status: 'info', detail: '~45 min (4 phases, 12 todos)' },
	];

	function statusIcon(status: string): string {
		if (status === 'pass') return '[ok]';
		if (status === 'warn') return '[!!]';
		if (status === 'fail') return '[xx]';
		return '[--]';
	}

	function statusClass(status: string): string {
		if (status === 'pass') return 'status-pass';
		if (status === 'warn') return 'status-warn';
		if (status === 'fail') return 'status-fail';
		return 'status-info';
	}
</script>

<div class="accept-flow">
	<!-- Step indicator -->
	<div class="step-indicator">
		{#each STEPS as step, i (step)}
			<div class="step-dot-group">
				<div
					class="step-dot"
					class:active={i === currentStep}
					class:completed={i < currentStep}
				></div>
				<span
					class="step-label"
					class:active={i === currentStep}
				>
					{step}
				</span>
			</div>
			{#if i < STEP_COUNT - 1}
				<div class="step-line" class:filled={i < currentStep}></div>
			{/if}
		{/each}
	</div>

	<!-- Step content -->
	<div class="step-content">
		{#key currentStep}
			<div class="step-slide" class:slide-forward={direction === 'forward'} class:slide-back={direction === 'back'}>
				{#if currentStep === 0}
					<!-- Step 1: Received -->
					<div class="step-body">
						<div class="step-title">MISSION RECEIVED</div>
						<div class="detail-grid">
							<div class="detail-row">
								<span class="detail-label">Title</span>
								<span class="detail-value">{mission.title}</span>
							</div>
							<div class="detail-row">
								<span class="detail-label">Phases</span>
								<span class="detail-value">{mission.phaseCount}</span>
							</div>
							<div class="detail-row">
								<span class="detail-label">Todos</span>
								<span class="detail-value">{mission.todoCount}</span>
							</div>
							<div class="detail-row">
								<span class="detail-label">Repos</span>
								<span class="detail-value">
									{mission.repos.length > 0
										? mission.repos.map((r) => r.repoUrl ?? r.repoId).join(', ')
										: 'None specified'}
								</span>
							</div>
							<div class="detail-row">
								<span class="detail-label">Source</span>
								<span class="detail-value">Brain API</span>
							</div>
						</div>
					</div>
				{:else if currentStep === 1}
					<!-- Step 2: Supervisor Check -->
					<div class="step-body">
						<div class="step-title">SUPERVISOR VALIDATION</div>
						<div class="validation-list">
							{#each validationResults as v (v.label)}
								<div class="validation-row {statusClass(v.status)}">
									<span class="validation-icon">{statusIcon(v.status)}</span>
									<span class="validation-label">{v.label}</span>
									<span class="validation-detail">{v.detail}</span>
								</div>
							{/each}
						</div>
					</div>
				{:else if currentStep === 2}
					<!-- Step 3: Your Review -->
					<div class="step-body">
						<div class="step-title">MISSION BRIEF</div>
						<div class="brief-section">
							<div class="brief-label">OVERVIEW</div>
							<p class="brief-text">{mission.title}</p>
						</div>
						<div class="brief-section">
							<div class="brief-label">PHASE BREAKDOWN</div>
							{#if mission.availablePhases.length > 0}
								<div class="phase-breakdown">
									{#each mission.availablePhases as phase, i (phase.phaseId)}
										<div class="phase-row">
											<span class="phase-index">{i + 1}.</span>
											<span class="phase-name">{phase.phaseName}</span>
											<span class="phase-todos">{phase.todoCount} todo{phase.todoCount !== 1 ? 's' : ''}</span>
										</div>
									{/each}
								</div>
							{:else}
								<p class="brief-text muted">Phase data not yet available</p>
							{/if}
						</div>
						<div class="brief-section">
							<div class="brief-label">REPOSITORIES</div>
							{#if mission.repos.length > 0}
								{#each mission.repos as repo (repo.repoId)}
									<div class="repo-row">
										<span class="repo-url">{repo.repoUrl ?? repo.repoId}</span>
										{#if repo.branch}
											<span class="repo-branch">{repo.branch}</span>
										{/if}
									</div>
								{/each}
							{:else}
								<p class="brief-text muted">No repos specified</p>
							{/if}
						</div>
					</div>
				{:else if currentStep === 3}
					<!-- Step 4: Accept -->
					<div class="step-body">
						<div class="step-title">CONFIRM ACTION</div>
						<div class="accept-actions">
							<button class="accept-btn primary" onclick={handleAcceptStart}>
								Accept & Start
							</button>
							<button class="accept-btn secondary" onclick={handleAcceptQueue}>
								Accept & Queue
							</button>
						</div>
						<div class="reject-section">
							<div class="reject-label">Or reject with reason:</div>
							<input
								type="text"
								class="reject-input"
								placeholder="Optional reason..."
								bind:value={rejectReason}
							/>
							<button class="accept-btn danger" onclick={handleReject}>
								Reject Mission
							</button>
						</div>
					</div>
				{/if}
			</div>
		{/key}
	</div>

	<!-- Navigation -->
	<div class="step-nav">
		<button
			class="nav-btn"
			disabled={currentStep === 0}
			onclick={prevStep}
		>
			Back
		</button>
		<span class="step-counter">{currentStep + 1} / {STEP_COUNT}</span>
		{#if currentStep < STEP_COUNT - 1}
			<button class="nav-btn primary" onclick={nextStep}>
				Next
			</button>
		{/if}
	</div>
</div>

<style>
	.accept-flow {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		padding: var(--space-lg) 0;
	}

	/* ── Step Indicator ──────────────────────────────────────────── */

	.step-indicator {
		display: flex;
		align-items: center;
		gap: 0;
		padding: 0 var(--space-md);
	}

	.step-dot-group {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-xs);
		flex-shrink: 0;
	}

	.step-dot {
		width: 8px;
		height: 8px;
		background: var(--border-default);
		transition: background var(--transition-normal);
	}

	.step-dot.active {
		background: var(--text-primary);
	}

	.step-dot.completed {
		background: var(--accent-green);
	}

	.step-label {
		font-family: var(--font-mono);
		font-size: 9px;
		letter-spacing: 0.06em;
		color: var(--text-muted);
		text-transform: uppercase;
		white-space: nowrap;
	}

	.step-label.active {
		color: var(--text-primary);
	}

	.step-line {
		flex: 1;
		height: 1px;
		background: var(--border-muted);
		margin: 0 var(--space-sm);
		margin-bottom: 20px;
		transition: background var(--transition-normal);
	}

	.step-line.filled {
		background: var(--accent-green);
	}

	/* ── Step Content ────────────────────────────────────────────── */

	.step-content {
		min-height: 240px;
		overflow: hidden;
		position: relative;
	}

	.step-slide {
		animation: step-enter 200ms ease-out;
	}

	@keyframes step-enter {
		from {
			opacity: 0;
			transform: translateX(24px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	.step-slide.slide-back {
		animation: step-enter-back 200ms ease-out;
	}

	@keyframes step-enter-back {
		from {
			opacity: 0;
			transform: translateX(-24px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	.step-body {
		padding: 0 var(--space-md);
	}

	.step-title {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.1em;
		color: var(--text-muted);
		margin-bottom: var(--space-lg);
	}

	/* ── Detail Grid (Step 1) ────────────────────────────────────── */

	.detail-grid {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.detail-row {
		display: flex;
		align-items: baseline;
		gap: var(--space-md);
	}

	.detail-label {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.06em;
		color: var(--text-muted);
		min-width: 80px;
		text-transform: uppercase;
	}

	.detail-value {
		font-size: 13px;
		color: var(--text-primary);
	}

	/* ── Validation (Step 2) ─────────────────────────────────────── */

	.validation-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.validation-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) 0;
		border-left: 2px solid transparent;
		padding-left: var(--space-sm);
	}

	.validation-row.status-pass {
		border-left-color: var(--accent-green);
	}

	.validation-row.status-warn {
		border-left-color: var(--accent-amber);
	}

	.validation-row.status-fail {
		border-left-color: var(--accent-red);
	}

	.validation-row.status-info {
		border-left-color: var(--supervisor-info);
	}

	.validation-icon {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 700;
		min-width: 32px;
		flex-shrink: 0;
	}

	.status-pass .validation-icon {
		color: var(--accent-green);
	}

	.status-warn .validation-icon {
		color: var(--accent-amber);
	}

	.status-fail .validation-icon {
		color: var(--accent-red);
	}

	.status-info .validation-icon {
		color: var(--supervisor-info);
	}

	.validation-label {
		font-size: 13px;
		color: var(--text-primary);
		min-width: 120px;
		flex-shrink: 0;
	}

	.validation-detail {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	/* ── Brief (Step 3) ──────────────────────────────────────────── */

	.brief-section {
		margin-bottom: var(--space-lg);
	}

	.brief-label {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: var(--text-muted);
		margin-bottom: var(--space-sm);
	}

	.brief-text {
		font-size: 13px;
		color: var(--text-primary);
		line-height: 1.5;
	}

	.brief-text.muted {
		color: var(--text-muted);
	}

	.phase-breakdown {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.phase-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) 0;
	}

	.phase-index {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		min-width: 20px;
	}

	.phase-name {
		font-size: 13px;
		color: var(--text-primary);
		flex: 1;
	}

	.phase-todos {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.repo-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) 0;
	}

	.repo-url {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-secondary);
	}

	.repo-branch {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		padding: 1px 6px;
		border: 1px solid var(--border-default);
	}

	/* ── Accept Actions (Step 4) ─────────────────────────────────── */

	.accept-actions {
		display: flex;
		gap: var(--space-md);
		margin-bottom: var(--space-xl);
	}

	.accept-btn {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 600;
		letter-spacing: 0.04em;
		padding: var(--space-sm) var(--space-lg);
		border: 1px solid var(--border-default);
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.accept-btn:hover {
		border-color: var(--text-primary);
		color: var(--text-primary);
	}

	.accept-btn.primary {
		background: var(--text-primary);
		border-color: var(--text-primary);
		color: var(--bg-base);
	}

	.accept-btn.primary:hover {
		background: var(--text-secondary);
		border-color: var(--text-secondary);
	}

	.accept-btn.secondary {
		border-color: var(--accent-green);
		color: var(--accent-green);
	}

	.accept-btn.secondary:hover {
		background: var(--accent-green);
		color: #000;
	}

	.accept-btn.danger {
		border-color: var(--accent-red);
		color: var(--accent-red);
	}

	.accept-btn.danger:hover {
		background: var(--accent-red);
		color: #000;
	}

	.reject-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		border-top: 1px solid var(--border-muted);
		padding-top: var(--space-lg);
	}

	.reject-label {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.reject-input {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-primary);
		background: var(--bg-elevated);
		border: 1px solid var(--border-default);
		padding: var(--space-sm) var(--space-md);
		width: 100%;
		transition: border-color var(--transition-fast);
	}

	.reject-input::placeholder {
		color: var(--text-muted);
	}

	.reject-input:focus {
		outline: none;
		border-color: var(--text-primary);
	}

	/* ── Navigation ──────────────────────────────────────────────── */

	.step-nav {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 var(--space-md);
		border-top: 1px solid var(--border-muted);
		padding-top: var(--space-md);
	}

	.step-counter {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
	}

	.nav-btn {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.04em;
		padding: var(--space-xs) var(--space-md);
		border: 1px solid var(--border-default);
		background: transparent;
		color: var(--text-secondary);
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.nav-btn:hover:not(:disabled) {
		border-color: var(--text-primary);
		color: var(--text-primary);
	}

	.nav-btn:disabled {
		opacity: 0.3;
		cursor: default;
	}

	.nav-btn.primary {
		background: var(--text-primary);
		border-color: var(--text-primary);
		color: var(--bg-base);
	}

	.nav-btn.primary:hover {
		background: var(--text-secondary);
		border-color: var(--text-secondary);
	}
</style>
