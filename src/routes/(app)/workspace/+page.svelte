<script lang="ts">
  import { onMount } from 'svelte';
  import {
    workspaceStatus,
    loading,
    refreshWorkspace,
    type RepoStatus,
    type ToolInfo,
  } from '$lib/stores/workspace';
  import { isTauri } from '$lib/ws';
  import PageHeader from '$lib/components/PageHeader.svelte';

  let ws = $derived($workspaceStatus);
  let isLoading = $derived($loading);

  onMount(() => {
    refreshWorkspace();
    // Refresh every 30s
    const interval = setInterval(refreshWorkspace, 30000);
    return () => clearInterval(interval);
  });

  async function openInEditor(path: string) {
    if (!isTauri()) return;
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('open_session', { pid: 0, projectPath: path });
    } catch (e) {
      // Fallback: try reveal in file manager
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('reveal_in_file_manager', { path });
      } catch {}
    }
  }
</script>

<PageHeader title="Workspace" />

<div class="page">
  <div class="header">
    <button class="btn-refresh" onclick={refreshWorkspace} disabled={isLoading}>
      {isLoading ? 'Scanning...' : 'Refresh'}
    </button>
  </div>

  {#if ws}
    <div class="mount-path">
      <span class="label">Mount</span>
      <span class="mono">{ws.mountPath}</span>
    </div>

    <section>
      <h2>Repositories ({ws.repos.length})</h2>
      {#if ws.repos.length === 0}
        <p class="empty">No git repositories found in workspace mount.</p>
      {:else}
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
      {/if}
    </section>

    <section>
      <h2>Tools</h2>
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
    </section>
  {:else if isLoading}
    <p class="empty">Scanning workspace...</p>
  {:else}
    <p class="empty">Configure workspace mount in Settings.</p>
  {/if}
</div>

<style>
  .page { padding: 24px; }

  .header {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 16px;
  }

  h2 {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted, #666);
    margin-bottom: 8px;
  }

  section {
    margin-bottom: 24px;
  }

  .mount-path {
    font-size: 12px;
    color: var(--text-muted);
    margin-bottom: 20px;
    display: flex;
    gap: 8px;
  }

  .label {
    color: var(--text-muted);
    text-transform: uppercase;
    font-size: 10px;
    letter-spacing: 0.05em;
  }

  .mono {
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  .btn-refresh {
    background: rgba(255, 255, 255, 0.06);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    padding: 4px 12px;
    font-size: 11px;
    cursor: pointer;
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

  .repo-card:hover {
    border-color: rgba(255, 255, 255, 0.12);
  }

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

  .repo-name {
    color: var(--text-primary);
    flex: 1;
  }

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
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 4px;
  }

  .tool-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    font-size: 12px;
  }

  .tool-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent-red);
  }

  .tool-dot.installed {
    background: var(--accent-green);
  }

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
  }

  .empty {
    font-size: 13px;
    color: var(--text-muted);
    padding: 24px 0;
  }
</style>
