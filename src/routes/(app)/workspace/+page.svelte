<script lang="ts">
  import { onMount } from 'svelte';
  import {
    workspaceStatus,
    loading,
    refreshWorkspace,
  } from '$lib/stores/workspace';
  import { isTauri } from '$lib/ws';

  let ws = $derived($workspaceStatus);
  let isLoading = $derived($loading);
  let activeTab = $state<'repos' | 'tools'>('repos');

  onMount(() => {
    refreshWorkspace();
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
</script>

<div class="dashboard">
  <div class="page-header">
    <span class="page-title">WORKSPACE</span>
    <div class="tab-bar">
      <button class="tab-btn" class:active={activeTab === 'repos'} onclick={() => activeTab = 'repos'}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        REPOS
      </button>
      <button class="tab-btn" class:active={activeTab === 'tools'} onclick={() => activeTab = 'tools'}>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        TOOLS
      </button>
    </div>
    <div class="spacer"></div>
    <button class="btn-refresh" onclick={refreshWorkspace} disabled={isLoading}>
      {isLoading ? '...' : 'Refresh'}
    </button>
  </div>

  <main class="grid-container">
    {#if activeTab === 'repos'}
      {#if ws && ws.repos.length > 0}
        <div class="mount-path">
          <span class="label">MOUNT</span>
          <span class="mono">{ws.mountPath}</span>
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
          <p class="empty-hint">Configure your workspace mount path in settings.</p>
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

  .page-header {
    display: flex;
    align-items: center;
    gap: var(--space-md);
    padding: var(--space-sm) var(--space-lg);
    border-bottom: 1px solid var(--border-muted);
  }

  .page-title {
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--text-muted);
  }

  .tab-bar {
    display: flex;
    gap: 0;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs);
    padding: var(--space-sm) var(--space-md);
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.04em;
    color: var(--text-muted);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
  }

  .tab-btn:hover { color: var(--text-secondary); }
  .tab-btn.active { color: var(--text-primary); border-bottom-color: var(--text-primary); }

  .spacer { flex: 1; }

  .btn-refresh {
    font-family: var(--font-mono);
    font-size: 10px;
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-secondary);
    padding: 3px 10px;
    cursor: pointer;
  }

  .btn-refresh:hover { background: rgba(255, 255, 255, 0.1); }
  .btn-refresh:disabled { opacity: 0.4; cursor: not-allowed; }

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

  .empty { padding: 48px 0; text-align: center; }
  .empty-title { font-family: var(--font-mono); font-size: 14px; color: var(--text-muted); }
  .empty-hint { font-size: 12px; color: var(--text-muted); margin-top: 8px; }
</style>
