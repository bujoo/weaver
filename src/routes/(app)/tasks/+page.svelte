<script lang="ts">
  import { selectedMissionId } from '$lib/stores/missions';
  import { goto } from '$app/navigation';

  let missionId = $derived($selectedMissionId);

  function goHome() {
    goto('/');
  }

  function goToPhases() {
    if (missionId) {
      goto(`/missions/${missionId}?tab=phases`);
    }
  }
</script>

<div class="redirect-page">
  <h2>Tasks</h2>
  <p>Tasks have moved. Each mission now manages its own phases and todos in the Phases tab.</p>
  <p>Select a mission from the sidebar, then open the Phases tab to view and manage tasks.</p>
  <div class="actions">
    <button class="action-btn" onclick={goHome} type="button">Mission Control</button>
    {#if missionId}
      <button class="action-btn primary" onclick={goToPhases} type="button">Go to Phases</button>
    {/if}
  </div>
</div>

<style>
  .redirect-page {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: var(--space-md);
    padding: var(--space-xl);
    text-align: center;
  }

  h2 {
    font-family: var(--font-pixel);
    font-size: 22px;
    font-weight: 600;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin: 0;
  }

  p {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.5;
    max-width: 420px;
  }

  .actions {
    display: flex;
    gap: var(--space-md);
    margin-top: var(--space-md);
  }

  .action-btn {
    font-family: var(--font-mono);
    font-size: 12px;
    font-weight: 600;
    letter-spacing: 0.04em;
    color: var(--text-primary);
    background: transparent;
    border: 1px solid var(--border-default);
    padding: var(--space-sm) var(--space-lg);
    cursor: pointer;
    transition: background var(--transition-fast), border-color var(--transition-fast);
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--text-secondary);
  }

  .action-btn.primary {
    border-color: var(--accent-green);
    color: var(--accent-green);
  }

  .action-btn.primary:hover {
    background: rgba(var(--accent-green-rgb, 0, 255, 136), 0.08);
  }
</style>
