<script lang="ts">
	import { slide, fade } from 'svelte/transition';
	import { flip } from 'svelte/animate';
	import { quintOut } from 'svelte/easing';
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';
	import {
		sortedSessions,
		expandedSessionId,
		currentConversation,
		statusSummary
	} from '$lib/stores/sessions';
	import {
		missions,
		activeMissions,
		completedMissions,
		selectedMissionId,
		hasExecutingMissions,
		attentionCount,
		autoSelectMission,
	} from '$lib/stores/missions';
	import { getConversation, stopSession, openSession } from '$lib/api';
	import { isDemoMode, toggleDemoMode } from '$lib/demo';
	import { isTauri } from '$lib/ws';
	import StatusBar from '$lib/components/StatusBar.svelte';
	import SessionCard from '$lib/components/SessionCard.svelte';
	import ExpandedCardOverlay from '$lib/components/ExpandedCardOverlay.svelte';
	import ToastNotifications from '$lib/components/ToastNotifications.svelte';
	import QRCodeModal from '$lib/components/QRCodeModal.svelte';
	import ConnectionScreen from '$lib/components/ConnectionScreen.svelte';
	import MissionPipeline from '$lib/components/MissionPipeline.svelte';
	import CompletedMissionRow from '$lib/components/CompletedMissionRow.svelte';
	import DeviceStatus from '$lib/components/DeviceStatus.svelte';
	import type { Session } from '$lib/types';
	import { SessionStatus } from '$lib/types';
	import SessionHistory from '$lib/components/SessionHistory.svelte';
	import CostTracker from '$lib/components/CostTracker.svelte';
	import MemoryViewer from '$lib/components/MemoryViewer.svelte';
	import FdaBanner from '$lib/components/FdaBanner.svelte';
	import DebugConsole from '$lib/components/DebugConsole.svelte';
	import type { DetectionDiagnostics } from '$lib/types';

	let demoActive = $derived($isDemoMode);
	let showQRModal = $state(false);

	let needsConnection = $state(!isTauri());

	let sessions = $derived($sortedSessions);
	let activeSessionIds = $derived(new Set(sessions.map((s) => s.id)));
	let summary = $derived($statusSummary);
	let expandedId = $derived($expandedSessionId);
	let conversation = $derived($currentConversation);

	// Mission stores
	let allMissions = $derived($missions);
	let active = $derived($activeMissions);
	let completed = $derived($completedMissions);
	let isExecuting = $derived($hasExecutingMissions);
	let attention = $derived($attentionCount);

	let viewMode = $state<'project' | 'all'>('project');

	let isCompact = $state(false);

	let activeTab = $state<'monitor' | 'history' | 'cost' | 'memory'>('monitor');
	let fdaLikelyNeeded = $state(false);
	let showDebugConsole = $state(false);
	let showRenameHint = $state(false);

	let isFullscreen = $state(false);

	// Auto-select highest priority mission
	$effect(() => {
		autoSelectMission(allMissions, $selectedMissionId);
	});

	onMount(() => {
		if (browser) {
			const saved = localStorage.getItem('sessionViewMode');
			if (saved === 'project' || saved === 'all') {
				viewMode = saved;
			}
			const savedCompact = localStorage.getItem('sessionViewCompact');
			if (savedCompact === 'true') {
				isCompact = true;
			}
		}

		if (!isTauri()) return;

		let unlisten: (() => void) | null = null;
		let timer: ReturnType<typeof setTimeout> | null = null;

		(async () => {
			const { getCurrentWindow } = await import('@tauri-apps/api/window');
			const win = getCurrentWindow();

			isFullscreen = await win.isFullscreen();

			unlisten = await win.onResized(async () => {
				if (timer) clearTimeout(timer);
				timer = setTimeout(async () => {
					isFullscreen = await win.isFullscreen();
				}, 150);
			});
		})();

		if (isTauri()) {
			import('@tauri-apps/api/event').then(({ listen }) => {
				listen<DetectionDiagnostics>('diagnostic-update', (event) => {
					fdaLikelyNeeded = event.payload.fdaLikelyNeeded;
				});
			});
		}

		return () => {
			unlisten?.();
			if (timer) clearTimeout(timer);
		};
	});

	$effect(() => {
		if (browser) {
			localStorage.setItem('sessionViewCompact', String(isCompact));
		}
	});

	// Session grouping helpers (kept for active mission view)
	function groupByProjectAndStatus(sessions: Session[]) {
		const groups: Array<{
			path: string;
			displayName: string;
			attention: Session[];
			idle: Session[];
			working: Session[];
			lastModified: number;
		}> = [];

		sessions.forEach(session => {
			let group = groups.find(g => g.path === session.projectPath);
			if (!group) {
				const parts = session.projectPath.split(/[/\\]/);
				const folderName = parts.filter(Boolean).pop() || session.projectPath;
				group = {
					path: session.projectPath,
					displayName: folderName,
					attention: [],
					idle: [],
					working: [],
					lastModified: 0
				};
				groups.push(group);
			}

			addToGroup(group, session);
		});

		return sortGroups(groups);
	}

	function groupSessionsByStatus(sessions: Session[]) {
		const groups = {
			attention: [] as Session[],
			idle: [] as Session[],
			working: [] as Session[]
		};

		sessions.forEach(session => {
			if (session.status === SessionStatus.NeedsAttention) {
				groups.attention.push(session);
			} else if (session.status === SessionStatus.WaitingForInput) {
				groups.idle.push(session);
			} else if (session.status === SessionStatus.Working || session.status === SessionStatus.Connecting) {
				groups.working.push(session);
			}
		});

		const sortSessions = (a: Session, b: Session) => new Date(a.modified).getTime() - new Date(b.modified).getTime();

		return [
			{ id: 'attention', label: 'Needs Attention', sessions: groups.attention.sort(sortSessions), type: 'attention' },
			{ id: 'idle', label: 'Idle', sessions: groups.idle.sort(sortSessions), type: 'idle' },
			{ id: 'working', label: 'Working', sessions: groups.working.sort(sortSessions), type: 'working' }
		].filter(g => g.sessions.length > 0);
	}

	function addToGroup(group: any, session: Session) {
		const modified = new Date(session.modified).getTime();
		if (modified > group.lastModified) {
			group.lastModified = modified;
		}

		if (session.status === SessionStatus.NeedsAttention) {
			group.attention.push(session);
		} else if (session.status === SessionStatus.WaitingForInput) {
			group.idle.push(session);
		} else if (session.status === SessionStatus.Working || session.status === SessionStatus.Connecting) {
			group.working.push(session);
		}
	}

	function sortGroups(groups: any[]) {
		return groups.sort((a, b) => {
			const aNeedsAttention = a.attention.length > 0;
			const bNeedsAttention = b.attention.length > 0;
			if (aNeedsAttention !== bNeedsAttention) return aNeedsAttention ? -1 : 1;

			const aNeedsIdle = a.idle.length > 0;
			const bNeedsIdle = b.idle.length > 0;
			if (aNeedsIdle !== bNeedsIdle) return aNeedsIdle ? -1 : 1;

			return b.lastModified - a.lastModified;
		});
	}

	let projectGroups = $derived(groupByProjectAndStatus(sessions));
	let allStatusGroups = $derived(groupSessionsByStatus(sessions));

	let expandedSession = $derived(sessions.find((s) => s.id === expandedId) || null);

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

	function handleKeydown(e: KeyboardEvent) {
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA') return;

		if ((e.key === 'd' || e.key === 'D') && e.shiftKey && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			showDebugConsole = !showDebugConsole;
			return;
		}

		if (e.key === 'd' && (e.metaKey || e.ctrlKey)) {
			e.preventDefault();
			toggleDemoMode();
			return;
		}
		if (e.key >= '1' && e.key <= '9' && !expandedId) {
			const index = parseInt(e.key) - 1;
			if (index < sessions.length) {
				handleExpand(sessions[index]);
			}
		}
		if (e.key === 'Tab' && !expandedId) {
			const needsAction = sessions.filter(s =>
				s.status === SessionStatus.NeedsAttention ||
				s.status === SessionStatus.WaitingForInput
			);
			if (needsAction.length > 0) {
				e.preventDefault();
				handleExpand(needsAction[0]);
			}
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

{#if needsConnection}
	<ConnectionScreen onconnected={() => (needsConnection = false)} />
{:else}
<div class="dashboard">
	<FdaBanner {fdaLikelyNeeded} />
	<div class="tab-bar" class:fullscreen={isFullscreen} data-tauri-drag-region>
		<button
			class="tab-btn"
			class:active={activeTab === 'monitor'}
			onclick={() => (activeTab = 'monitor')}
		>
			<span class="tab-icon">&#9632;</span>
			<span class="tab-label">MONITOR</span>
		</button>
		<button
			class="tab-btn"
			class:active={activeTab === 'history'}
			onclick={() => (activeTab = 'history')}
		>
			<span class="tab-icon">&#9109;</span>
			<span class="tab-label">HISTORY</span>
		</button>
		<button
			class="tab-btn"
			class:active={activeTab === 'cost'}
			onclick={() => (activeTab = 'cost')}
		>
			<span class="tab-icon">$</span>
			<span class="tab-label">COST</span>
		</button>
		<button
			class="tab-btn"
			class:active={activeTab === 'memory'}
			onclick={() => (activeTab = 'memory')}
		>
			<span class="tab-icon">&#9670;</span>
			<span class="tab-label">MEMORY</span>
		</button>
		<div class="tab-drag-region" data-tauri-drag-region>
			{#if !isFullscreen}
				<span class="drag-dots" transition:fade={{ duration: 250 }}>&#10239; &#10239; &#10239;</span>
			{/if}
		</div>
	</div>

	{#if activeTab === 'history'}
	<main class="grid-container history-main">
		<SessionHistory {activeSessionIds} />
	</main>
	{:else if activeTab === 'cost'}
	<main class="grid-container history-main">
		<CostTracker />
	</main>
	{:else if activeTab === 'memory'}
	<main class="grid-container history-main">
		<MemoryViewer />
	</main>
	{:else}

	<!-- ── Monitor tab: Mission Control (idle) vs Active Mission Focus ── -->
	{#if !isExecuting}
		<!-- MISSION CONTROL - idle dashboard -->
		<main class="grid-container">
			<div class="mission-control">
				<header class="mc-header">
					<span class="mc-title">MISSION CONTROL</span>
					<span class="mc-subtitle">
						{allMissions.length} mission{allMissions.length !== 1 ? 's' : ''}
					</span>
					<div class="mc-spacer"></div>
					<button
						class="toggle-btn demo-toggle"
						class:active={demoActive}
						onclick={() => toggleDemoMode()}
						title="Try with Sample Data (Cmd+D)"
					>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
							<path d="M10 2v7.31" />
							<path d="M14 2v7.31" />
							<path d="M8.5 2h7" />
							<path d="M14 9.3c.7.4 1.3.9 1.8 1.5l3.8 4.4a3 3 0 0 1-2.3 4.8H6.7a3 3 0 0 1-2.3-4.8l3.8-4.4c.5-.6 1.1-1.1 1.8-1.5" />
						</svg>
						<span class="demo-label">DEMO</span>
					</button>
					{#if isTauri()}
						<button
							class="toggle-btn mobile-connect-btn"
							onclick={() => (showQRModal = true)}
							title="Connect Mobile Device"
						>
							<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
								<rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
								<line x1="12" y1="18" x2="12.01" y2="18" />
							</svg>
							<span class="mobile-label">MOBILE</span>
						</button>
					{/if}
				</header>

				<!-- Pipeline funnel -->
				<section class="mc-pipeline">
					<MissionPipeline />
				</section>

				<!-- Completed missions -->
				{#if completed.length > 0}
					<section class="mc-completed">
						<div class="mc-section-header">
							<span class="mc-section-title">COMPLETED (last 24h)</span>
							<span class="mc-section-count">{completed.length}</span>
						</div>
						{#each completed as m (m.missionId)}
							<CompletedMissionRow mission={m} />
						{/each}
					</section>
				{/if}

				<!-- Sessions overview (if any sessions exist) -->
				{#if sessions.length > 0}
					<section class="mc-sessions">
						<div class="mc-section-header">
							<span class="mc-section-title">CLAUDE CODE SESSIONS</span>
							<span class="mc-section-count">{sessions.length}</span>
						</div>
						<div class="system-status-container">
							<StatusBar total={sessions.length} {summary} />
						</div>
					</section>
				{/if}

				<!-- Device status bar -->
				<DeviceStatus />
			</div>
		</main>
	{:else}
		<!-- ACTIVE MISSION FOCUS - Phase 3 -->
		<main class="grid-container">
			<div class="active-focus-placeholder">
				<div class="placeholder-border"></div>
				<div class="placeholder-content">
					<span class="placeholder-label">ACTIVE MISSION FOCUS</span>
					<span class="placeholder-sub">Coming in Phase 3</span>
					<span class="placeholder-count">{active.length} executing</span>
				</div>

				<!-- Keep sessions visible in active mode -->
				{#if sessions.length > 0}
					<section class="system-section">
						<div class="project-header">
							<span class="project-name">System status</span>
							<span class="project-count">{sessions.length}</span>
							<button
								class="toggle-btn demo-toggle"
								class:active={demoActive}
								onclick={() => toggleDemoMode()}
								title="Try with Sample Data (Cmd+D)"
							>
								<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
									<path d="M10 2v7.31" />
									<path d="M14 2v7.31" />
									<path d="M8.5 2h7" />
									<path d="M14 9.3c.7.4 1.3.9 1.8 1.5l3.8 4.4a3 3 0 0 1-2.3 4.8H6.7a3 3 0 0 1-2.3-4.8l3.8-4.4c.5-.6 1.1-1.1 1.8-1.5" />
								</svg>
								<span class="demo-label">DEMO</span>
							</button>
							{#if isTauri()}
								<button
									class="toggle-btn mobile-connect-btn"
									onclick={() => (showQRModal = true)}
									title="Connect Mobile Device"
								>
									<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<rect x="5" y="2" width="14" height="20" rx="2" ry="2" />
										<line x1="12" y1="18" x2="12.01" y2="18" />
									</svg>
									<span class="mobile-label">MOBILE</span>
								</button>
							{/if}
							<div class="header-spacer"></div>
							<div class="view-toggle">
								<button
									class="toggle-btn"
									class:active={isCompact}
									onclick={() => isCompact = !isCompact}
									title="Compact View"
								>
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<polyline points="4 14 10 14 10 20" />
										<polyline points="20 10 14 10 14 4" />
										<line x1="14" y1="10" x2="21" y2="3" />
										<line x1="3" y1="21" x2="10" y2="14" />
									</svg>
								</button>
							</div>
							<div class="view-toggle">
								<button
									class="toggle-btn"
									class:active={viewMode === 'project'}
									onclick={() => viewMode = 'project'}
									title="Group by Project"
								>
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
										<line x1="8" y1="6" x2="21" y2="6"></line>
										<line x1="8" y1="12" x2="21" y2="12"></line>
										<line x1="8" y1="18" x2="21" y2="18"></line>
										<line x1="3" y1="6" x2="3.01" y2="6"></line>
										<line x1="3" y1="12" x2="3.01" y2="12"></line>
										<line x1="3" y1="18" x2="3.01" y2="18"></line>
									</svg>
								</button>
								<button
									class="toggle-btn"
									class:active={viewMode === 'all'}
									onclick={() => viewMode = 'all'}
									title="Show All"
								>
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
										<rect x="3" y="3" width="7" height="7" rx="1" />
										<rect x="14" y="3" width="7" height="7" rx="1" />
										<rect x="14" y="14" width="7" height="7" rx="1" />
										<rect x="3" y="14" width="7" height="7" rx="1" />
									</svg>
								</button>
							</div>
						</div>

						{#if sessions.length > 0}
							<div class="system-status-container">
								<StatusBar total={sessions.length} {summary} />
							</div>
						{/if}
					</section>

					<div class="sections-container">
						{#if viewMode === 'project'}
							{#each projectGroups as group (group.path)}
								<section class="project-section" animate:flip={{ duration: 400 }}>
									<div class="project-header">
										<span class="project-name">{group.displayName}</span>
										<span class="project-count">
											{group.attention.length + group.idle.length + group.working.length}
										</span>
									</div>

									<div class="status-groups">
										<div class="status-group" class:empty={group.attention.length === 0} class:compact={isCompact}>
											<div class="status-header attention">
												<span class="status-indicator attention"></span>
												<span class="status-title">Needs Attention</span>
												<span class="status-count">{group.attention.length}</span>
											</div>
											<div class="session-grid">
												{#each group.attention as session (session.id)}
													<div
														class="card-wrapper"
														transition:slide={{ duration: 400, easing: quintOut }}
														animate:flip={{ duration: 400 }}
													>
														<SessionCard
															{session}
															compact={isCompact}
															onexpand={() => handleExpand(session)}
															onstop={() => handleStop(session.pid)}
															onopen={() => handleOpen(session.pid, session.projectPath)}
															onrename={() => showRenameHint = true}
														/>
													</div>
												{/each}
											</div>
										</div>

										<div class="status-group" class:empty={group.idle.length === 0} class:compact={isCompact}>
											<div class="status-header idle">
												<span class="status-indicator idle"></span>
												<span class="status-title">Idle</span>
												<span class="status-count">{group.idle.length}</span>
											</div>
											<div class="session-grid">
												{#each group.idle as session (session.id)}
													<div
														class="card-wrapper"
														transition:slide={{ duration: 400, easing: quintOut }}
														animate:flip={{ duration: 400 }}
													>
														<SessionCard
															{session}
															compact={isCompact}
															onexpand={() => handleExpand(session)}
															onstop={() => handleStop(session.pid)}
															onopen={() => handleOpen(session.pid, session.projectPath)}
															onrename={() => showRenameHint = true}
														/>
													</div>
												{/each}
											</div>
										</div>

										<div class="status-group" class:empty={group.working.length === 0} class:compact={isCompact}>
											<div class="status-header working">
												<span class="status-indicator working"></span>
												<span class="status-title">Working</span>
												<span class="status-count">{group.working.length}</span>
											</div>
											<div class="session-grid">
												{#each group.working as session (session.id)}
													<div
														class="card-wrapper"
														transition:slide={{ duration: 400, easing: quintOut }}
														animate:flip={{ duration: 400 }}
													>
														<SessionCard
															{session}
															compact={isCompact}
															onexpand={() => handleExpand(session)}
															onstop={() => handleStop(session.pid)}
															onopen={() => handleOpen(session.pid, session.projectPath)}
															onrename={() => showRenameHint = true}
														/>
													</div>
												{/each}
											</div>
										</div>
									</div>
								</section>
							{/each}
						{:else}
							{#each allStatusGroups as group (group.id)}
								<section class="project-section" animate:flip={{ duration: 400 }}>
									<div class="status-header all-view {group.type}">
										<span class="status-indicator {group.type}" style="width: 8px; height: 8px;"></span>
										<span class="project-name" style="font-size: 16px;">{group.label}</span>
										<span class="project-count">{group.sessions.length}</span>
									</div>

									<div class="all-sessions-grid" class:compact={isCompact}>
										{#each group.sessions as session (session.id)}
											<div
												class="card-wrapper"
												transition:slide={{ duration: 400, easing: quintOut }}
												animate:flip={{ duration: 400 }}
											>
												<SessionCard
													{session}
													compact={isCompact}
													onexpand={() => handleExpand(session)}
													onstop={() => handleStop(session.pid)}
													onopen={() => handleOpen(session.pid, session.projectPath)}
													onrename={() => showRenameHint = true}
												/>
											</div>
										{/each}
									</div>
								</section>
							{/each}
						{/if}
					</div>
				{/if}
			</div>
		</main>
	{/if}

	{/if}

	{#if expandedSession}
		<ExpandedCardOverlay
			session={expandedSession}
			{conversation}
			onclose={handleClose}
			onstop={() => handleStop(expandedSession.pid)}
			onopen={() => handleOpen(expandedSession.pid, expandedSession.projectPath)}
		/>
	{/if}

	{#if showQRModal}
		<QRCodeModal onclose={() => (showQRModal = false)} />
	{/if}

	<ToastNotifications />
	<DebugConsole visible={showDebugConsole} onclose={() => (showDebugConsole = false)} />

	{#if showRenameHint}
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div class="rename-hint-backdrop" transition:fade={{ duration: 150 }} onclick={() => showRenameHint = false}>
			<div class="rename-hint-modal" onclick={(e) => e.stopPropagation()}>
				<div class="rename-hint-header">
					<span class="rename-hint-icon">
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
							<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
							<path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" />
						</svg>
					</span>
					<span class="rename-hint-title">Rename Session</span>
				</div>
				<p class="rename-hint-text">Use the <code>/rename</code> command in your Claude Code session to rename it.</p>
				<div class="rename-hint-example">/rename my-task-name</div>
				<p class="rename-hint-sub">The new name will automatically appear in c9watch.</p>
				<button type="button" class="rename-hint-close" onclick={() => showRenameHint = false}>GOT IT</button>
			</div>
		</div>
	{/if}
</div>
{/if}

<style>
	.dashboard {
		display: flex;
		flex-direction: column;
		height: 100%;
		width: 100%;
		overflow: hidden;
		background: var(--bg-base);
	}

	/* ── Tab bar ───────────────────────────────────────────────────── */
	.tab-bar {
		height: 28px;
		width: 100%;
		flex-shrink: 0;
		display: flex;
		align-items: stretch;
		background: transparent;
		z-index: 1000;
		position: relative;
		padding: 0 var(--space-md) 0 36px;
		transition: padding-left 0.35s ease;
		-webkit-app-region: drag;
	}

	.tab-drag-region {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: center;
		-webkit-app-region: drag;
		cursor: grab;
	}

	.drag-dots {
		position: absolute;
		left: 50%;
		top: 50%;
		transform: translate(-50%, -50%);
		font-size: 14px;
		letter-spacing: 3px;
		color: var(--text-muted);
		opacity: 0.5;
		user-select: none;
		line-height: 1;
		pointer-events: none;
		transition: opacity var(--transition-fast);
	}

	.tab-drag-region:hover .drag-dots {
		opacity: 0.85;
	}

	.tab-btn {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: 0 var(--space-md);
		background: transparent;
		border: none;
		border-bottom: 2px solid transparent;
		color: var(--text-muted);
		cursor: pointer;
		font-family: var(--font-pixel);
		font-size: 10px;
		letter-spacing: 0.08em;
		transition: color var(--transition-fast);
		-webkit-app-region: no-drag;
	}

	.tab-btn:hover {
		color: var(--text-secondary);
	}

	.tab-btn.active {
		color: var(--text-primary);
		border-bottom-color: var(--text-primary);
	}

	.tab-icon {
		font-size: 8px;
	}

	.tab-label {
		font-size: 10px;
	}

	/* ── Grid container ────────────────────────────────────────────── */
	.grid-container {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xl);
	}

	.history-main {
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}

	/* ── Mission Control (idle dashboard) ──────────────────────────── */
	.mission-control {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
		max-width: 1200px;
		margin: 0 auto;
		width: 100%;
	}

	.mc-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-md);
		border-bottom: 1px solid var(--text-primary);
	}

	.mc-title {
		font-family: var(--font-pixel);
		font-size: 22px;
		font-weight: 600;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		line-height: 1;
	}

	.mc-subtitle {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.mc-spacer {
		flex: 1;
	}

	.mc-pipeline {
		border: 1px solid var(--border-muted);
	}

	.mc-completed {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.mc-section-header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 0 0 8px 0;
	}

	.mc-section-title {
		font-family: var(--font-mono);
		font-size: 10px;
		font-weight: 600;
		letter-spacing: 0.1em;
		color: var(--text-muted);
	}

	.mc-section-count {
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--text-muted);
		opacity: 0.6;
	}

	.mc-sessions {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	/* ── Active Mission Focus placeholder ─────────────────────────── */
	.active-focus-placeholder {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
		max-width: 1200px;
		margin: 0 auto;
		width: 100%;
	}

	.placeholder-border {
		height: 2px;
		background: var(--mission-active);
		opacity: 0.4;
	}

	.placeholder-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-3xl) 0;
	}

	.placeholder-label {
		font-family: var(--font-pixel);
		font-size: 18px;
		font-weight: 600;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.12em;
	}

	.placeholder-sub {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.placeholder-count {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--mission-active);
	}

	/* ── Sections (kept from old sessions view for active mode) ───── */
	.sections-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-3xl);
		max-width: 1200px;
		margin: 0 auto;
		width: 100%;
	}

	.system-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.system-status-container {
		margin-top: var(--space-sm);
	}

	.project-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.project-header {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		padding-bottom: var(--space-md);
		border-bottom: 1px solid var(--text-primary);
		margin-bottom: var(--space-md);
	}

	.project-name {
		font-family: var(--font-pixel);
		font-size: 22px;
		font-weight: 600;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
		line-height: 1;
	}

	.project-count {
		font-family: var(--font-pixel);
		font-size: 18px;
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
	}

	.status-groups {
		display: flex;
		flex-direction: row;
		gap: var(--space-xl);
		overflow-x: auto;
		padding-bottom: var(--space-lg);
	}

	.status-group {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		min-width: 350px;
		max-width: 400px;
		flex: 1;
	}

	.status-header {
		display: flex;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		background: rgba(255, 255, 255, 0.03);
		border-left: 3px solid var(--border-default);
		gap: var(--space-sm);
	}

	.status-header.attention { border-left-color: var(--status-permission); }
	.status-header.idle { border-left-color: var(--status-input); }
	.status-header.working { border-left-color: var(--status-working); }
	.status-header.connecting { border-left-color: var(--status-connecting); }

	.status-header.all-view {
		background: transparent;
		padding-left: 0;
		margin-bottom: var(--space-md);
		border-left: none;
	}

	.status-group.empty {
		opacity: 0.5;
	}

	.status-group.empty .status-header {
		background: transparent;
		border-left-style: dashed;
	}

	.status-indicator {
		width: 6px;
		height: 6px;
	}

	.status-indicator.attention {
		background: var(--status-permission);
	}

	.status-indicator.idle {
		background: var(--status-input);
	}

	.status-indicator.working {
		background: var(--status-working);
	}

	.status-indicator.connecting {
		background: var(--status-connecting);
	}

	.status-title {
		font-family: var(--font-mono);
		font-size: 12px;
		font-weight: 500;
		color: var(--text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.status-count {
		font-family: var(--font-mono);
		font-size: 12px;
		color: var(--text-muted);
	}

	.session-grid {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
	}

	.header-spacer {
		flex: 1;
	}

	.view-toggle {
		display: flex;
		gap: var(--space-xs);
		background: rgba(255, 255, 255, 0.03);
		padding: 2px;
		border: 1px solid var(--border-default);
	}

	.toggle-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		border: 1px solid transparent;
		background: transparent;
		color: var(--text-muted);
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.toggle-btn:hover {
		color: var(--text-secondary);
		background: rgba(255, 255, 255, 0.05);
	}

	.toggle-btn.active {
		color: var(--text-primary);
		background: rgba(255, 255, 255, 0.1);
		border-color: var(--border-default);
	}

	.all-sessions-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
		gap: var(--space-lg);
	}

	.all-sessions-grid.compact {
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: var(--space-md);
	}

	.demo-toggle {
		width: auto;
		padding: 0 var(--space-sm);
		gap: var(--space-xs);
		color: var(--accent-amber);
	}

	.demo-toggle:hover {
		color: var(--accent-amber);
		background: rgba(255, 102, 0, 0.1);
	}

	.demo-toggle.active {
		background: var(--accent-amber);
		color: #000;
		border-color: var(--accent-amber);
	}

	.demo-label {
		font-family: var(--font-pixel);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	.mobile-connect-btn {
		width: auto;
		padding: 0 var(--space-sm);
		gap: var(--space-xs);
		color: var(--accent-blue);
	}

	.mobile-connect-btn:hover {
		color: var(--accent-blue);
		background: rgba(0, 112, 243, 0.1);
	}

	.mobile-label {
		font-family: var(--font-pixel);
		font-size: 10px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	/* ── Fullscreen ────────────────────────────────────────────────── */
	.tab-bar.fullscreen {
		padding-left: var(--space-xl);
	}

	/* ── Mobile Responsive ─────────────────────────────────────────── */
	@media (max-width: 768px) {
		.tab-bar {
			height: 28px;
		}

		.tab-btn {
			display: none;
		}

		.grid-container {
			padding: var(--space-md);
		}

		.sections-container {
			gap: var(--space-xl);
		}

		.project-header {
			flex-wrap: wrap;
			gap: var(--space-sm);
		}

		.project-name {
			font-size: 16px;
		}

		.project-count {
			font-size: 14px;
		}

		.status-groups {
			flex-direction: column;
			overflow-x: visible;
			padding-bottom: 0;
		}

		.status-group {
			min-width: 0;
			max-width: 100%;
		}

		.all-sessions-grid {
			grid-template-columns: 1fr;
		}

		.all-sessions-grid.compact {
			grid-template-columns: 1fr;
		}

		.view-toggle {
			padding: 1px;
		}

		.toggle-btn {
			width: 32px;
			height: 32px;
		}

		.demo-toggle {
			padding: 0 var(--space-xs);
		}

		.mc-title {
			font-size: 16px;
		}
	}

	/* ── Rename Hint Modal ───────────────────────────────────────── */
	.rename-hint-backdrop {
		position: fixed;
		inset: 0;
		background: var(--bg-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 2000;
	}

	.rename-hint-modal {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-2xl) var(--space-2xl);
		background: var(--bg-card);
		border: 1px solid var(--border-default);
		box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
		max-width: 380px;
		width: 100%;
	}

	.rename-hint-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.rename-hint-icon {
		color: var(--text-muted);
	}

	.rename-hint-title {
		font-family: var(--font-pixel);
		font-size: 16px;
		font-weight: 600;
		color: var(--text-primary);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.rename-hint-text {
		font-family: var(--font-mono);
		font-size: 13px;
		color: var(--text-secondary);
		text-align: center;
		margin: 0;
		line-height: 1.5;
	}

	.rename-hint-text code {
		font-family: var(--font-mono);
		color: var(--text-primary);
		background: var(--bg-elevated);
		padding: 2px 6px;
		border: 1px solid var(--border-default);
	}

	.rename-hint-example {
		font-family: var(--font-mono);
		font-size: 14px;
		color: var(--status-input);
		background: var(--bg-elevated);
		padding: 8px 16px;
		border: 1px solid var(--border-default);
		letter-spacing: 0.02em;
		width: 100%;
		text-align: center;
	}

	.rename-hint-sub {
		font-family: var(--font-mono);
		font-size: 11px;
		color: var(--text-muted);
		text-align: center;
		margin: 0;
	}

	.rename-hint-close {
		font-family: var(--font-mono);
		font-size: 11px;
		font-weight: 600;
		color: var(--bg-base);
		background: var(--text-primary);
		border: 1px solid var(--text-primary);
		padding: 6px 24px;
		text-transform: uppercase;
		letter-spacing: 0.1em;
		cursor: pointer;
		transition: all var(--transition-fast);
		margin-top: var(--space-xs);
	}

	.rename-hint-close:hover {
		background: var(--text-secondary);
		border-color: var(--text-secondary);
	}
</style>
