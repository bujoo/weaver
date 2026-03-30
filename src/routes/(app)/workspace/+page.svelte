<script lang="ts">
  import { onMount } from 'svelte';
  import {
    workspaceStatus,
    loading,
    refreshWorkspace,
    registry,
    initRegistryListener,
  } from '$lib/stores/workspace';
  import { isTauri } from '$lib/ws';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let ws = $derived($workspaceStatus);
  let reg = $derived($registry);
  let isLoading = $derived($loading);
  let activeTab = $state('missions');

  const tabs = [
    { id: 'missions', icon: '◈', label: 'MISSIONS' },
    { id: 'repos', icon: '◇', label: 'REPOS' },
    { id: 'tools', icon: '⚙', label: 'TOOLS' },
  ];

  let cloning = $state<string | null>(null);

  onMount(() => {
    refreshWorkspace();
    initRegistryListener();
    const interval = setInterval(refreshWorkspace, 30000);
    return () => clearInterval(interval);
  });

  async function openInEditor(path: string) {
    if (!isTauri()) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('open_session', { pid: 0, projectPath: path });
    } catch {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('reveal_in_file_manager', { path });
      } catch {}
    }
  }

  async function cloneRepo(repoId: string, repoUrl: string | null, branch: string | null) {
    if (!isTauri() || !repoUrl) return;
    cloning = repoId;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('clone_repo_cmd', { url: repoUrl, branch });
      await refreshWorkspace();
    } catch (e) {
      console.error('Clone failed:', e);
    }
    cloning = null;
  }

  async function setupMission(missionId: string) {
    if (!reg) return;
    const mission = reg.missions.find((m) => m.mission_id === missionId);
    if (!mission) return;
    for (const repo of mission.repos) {
      if (repo.repo_url) {
        await cloneRepo(repo.repo_id, repo.repo_url, repo.branch);
      }
    }
  }

  let allWorktrees = $derived(
    ws?.repos.flatMap((r) =>
      r.worktrees.filter((wt) => wt.branch && wt.path !== r.path).map((wt) => ({
        ...wt,
        repoName: r.name,
      }))
    ) || []
  );
</script>

<div class="dashboard">
  <PageHeader {tabs} {activeTab} onTabChange={(id) => (activeTab = id)} />

  <main class="grid-container">
    {#if activeTab === 'missions'}
        {#if reg && reg.missions.length > 0}
          <div class="mission-list">
            {#each reg.missions as mission}
              <div class="mission-card">
                <div class="mission-header">
                  <span class="mission-title">{mission.title}</span>
                  <span class="mission-status" class:executing={mission.status === 'executing'}>{mission.status}</span>
                </div>
                <div class="mission-meta">
                  <span class="mono">{mission.mission_id.slice(0, 8)}</span>
                  <span>{mission.phase_count} phases</span>
                  <span>{mission.todo_count} todos</span>
                </div>
                {#if mission.repos.length > 0}
                  <div class="mission-repos">
                    {#each mission.repos as repo}
                      <div class="mission-repo-row">
                        <span class="repo-id">{repo.repo_id}</span>
                        {#if repo.branch}
                          <span class="branch">{repo.branch}</span>
                        {/if}
                        {#if repo.repo_url}
                          <button class="btn-clone" onclick={() => cloneRepo(repo.repo_id, repo.repo_url, repo.branch)} disabled={cloning === repo.repo_id}>
                            {cloning === repo.repo_id ? '...' : 'Clone'}
                          </button>
                        {/if}
                      </div>
                    {/each}
                  </div>
                {/if}
                <button class="btn-setup" onclick={() => setupMission(mission.mission_id)} disabled={!!cloning}>
                  Setup All Repos
                </button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty">
            <p class="empty-title">No missions discovered</p>
            <p class="empty-hint">Brain publishes missions via MQTT. Make sure a weaver plan exists.</p>
          </div>
        {/if}

      {:else if activeTab === 'repos'}
        {#if ws && ws.repos.length > 0}
          <div class="mount-path">
            <span class="label">MOUNT</span>
            <span class="mono">{ws.mountPath}</span>
            <button class="btn-refresh" onclick={refreshWorkspace} disabled={isLoading}>
              {isLoading ? '...' : '↻'}
            </button>
          </div>
          <div class="repo-list">
            {#each ws.repos as repo}
              <button class="repo-card" onclick={() => openInEditor(repo.path)}>
                <div class="repo-header">
                  <span class="status-dot" class:clean={repo.clean} class:dirty={!repo.clean}></span>
                  <span class="repo-name">{repo.name}</span>
                  <span class="branch">{repo.branch}</span>
                  {#if repo.worktrees.length > 1}
                    <span class="wt-count">{repo.worktrees.length - 1} wt</span>
                  {/if}
                </div>
              </button>
            {/each}
          </div>
        {:else}
          <div class="empty">
            <p class="empty-title">{isLoading ? 'Scanning...' : 'No repositories found'}</p>
          </div>
        {/if}

      {:else if activeTab === 'tools'}
        {#if ws && ws.tools.length > 0}
          <div class="tool-grid">
            {#each ws.tools as tool}
              <div class="tool-item">
                <span class="tool-dot" class:installed={tool.installed}></span>
                <span class="tool-name">{tool.name}</span>
                {#if tool.version}
                  <span class="tool-version">{tool.version.split('\n')[0].slice(0, 30)}</span>
                {/if}
              </div>
            {/each}
          </div>
        {:else}
          <div class="empty">
            <p class="empty-title">No tools detected</p>
          </div>
        {/if}
      {/if}
  </main>
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--bg-base);
  }

  .grid-container {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-xl);
  }

  .mount-path {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 20px;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .label {
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .btn-refresh {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    padding: 2px 8px;
    font-size: 12px;
    cursor: pointer;
    margin-left: auto;
  }

  .btn-refresh:hover { background: rgba(255, 255, 255, 0.1); }
  .btn-refresh:disabled { opacity: 0.4; cursor: not-allowed; }

  .repo-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .repo-card {
    display: block;
    width: 100%;
    background: var(--bg-card, #111);
    border: 1px solid rgba(255, 255, 255, 0.06);
    padding: 8px 12px;
    cursor: pointer;
    text-align: left;
    color: inherit;
    font: inherit;
  }

  .repo-card:hover { border-color: rgba(255, 255, 255, 0.12); }

  .repo-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .status-dot.clean { background: var(--accent-green); }
  .status-dot.dirty { background: var(--accent-amber); }

  .repo-name { color: var(--text-primary); flex: 1; }

  .branch {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
  }

  .wt-count {
    font-size: 10px;
    color: var(--task-executing);
    border: 1px solid var(--task-executing);
    padding: 1px 4px;
  }

  .wt-path {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tool-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 4px;
  }

  .tool-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    font-size: 12px;
    background: var(--bg-card, #111);
    border: 1px solid rgba(255, 255, 255, 0.04);
  }

  .tool-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent-red);
  }

  .tool-dot.installed { background: var(--accent-green); }

  .tool-name {
    color: var(--text-secondary);
    font-family: var(--font-mono);
  }

  .tool-version {
    font-size: 10px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-left: auto;
  }

  .mission-list { display: flex; flex-direction: column; gap: 8px; }

  .mission-card {
    background: var(--bg-card, #111);
    border: 1px solid rgba(255, 255, 255, 0.06);
    padding: 12px;
  }

  .mission-header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }

  .mission-title { color: var(--text-primary); font-size: 13px; flex: 1; }

  .mission-status {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
    padding: 1px 6px;
    border: 1px solid rgba(255, 255, 255, 0.08);
  }

  .mission-status.executing {
    color: var(--task-executing);
    border-color: var(--task-executing);
  }

  .mission-meta {
    display: flex;
    gap: 12px;
    font-size: 11px;
    color: var(--text-muted);
    margin-bottom: 8px;
  }

  .mission-repos { margin-bottom: 8px; }

  .mission-repo-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 3px 0;
    font-size: 12px;
  }

  .repo-id { font-family: var(--font-mono); color: var(--text-secondary); flex: 1; }

  .btn-clone, .btn-setup {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    padding: 2px 8px;
    font-size: 11px;
    cursor: pointer;
  }

  .btn-clone:hover, .btn-setup:hover { background: rgba(255, 255, 255, 0.1); }
  .btn-clone:disabled, .btn-setup:disabled { opacity: 0.4; cursor: not-allowed; }

  .empty { padding: 48px 0; text-align: center; }
  .empty-title { font-family: var(--font-pixel, monospace); font-size: 16px; color: var(--text-muted); }
  .empty-hint { font-size: 12px; color: var(--text-muted); margin-top: 8px; }
</style>
