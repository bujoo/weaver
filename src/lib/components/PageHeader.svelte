<script lang="ts">
  import { fade } from 'svelte/transition';

  interface Tab {
    id: string;
    icon: string;
    label: string;
  }

  interface Props {
    tabs: Tab[];
    activeTab: string;
    onTabChange: (id: string) => void;
  }

  let { tabs, activeTab, onTabChange }: Props = $props();
</script>

<div class="tab-bar" data-tauri-drag-region>
  {#each tabs as tab}
    <button
      class="tab-btn"
      class:active={activeTab === tab.id}
      onclick={() => onTabChange(tab.id)}
    >
      <span class="tab-icon">{tab.icon}</span>
      <span class="tab-label">{tab.label}</span>
    </button>
  {/each}
  <div class="tab-drag-region" data-tauri-drag-region>
    <span class="drag-dots" transition:fade={{ duration: 250 }}>&#x2817; &#x2817; &#x2817;</span>
  </div>
</div>

<style>
  .tab-bar {
    height: 28px;
    width: 100%;
    flex-shrink: 0;
    display: flex;
    align-items: stretch;
    background: transparent;
    z-index: 1000;
    position: relative;
    padding: 0 var(--space-md, 12px) 0 36px;
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
    color: var(--text-muted, #666);
    opacity: 0.3;
    pointer-events: none;
    user-select: none;
    line-height: 1;
  }

  .tab-drag-region:hover .drag-dots {
    opacity: 0.7;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: var(--space-xs, 4px);
    padding: 0 var(--space-md, 12px);
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-muted, #666);
    cursor: pointer;
    font-family: var(--font-pixel, monospace);
    font-size: 10px;
    letter-spacing: 0.08em;
    transition: color 100ms;
    -webkit-app-region: no-drag;
  }

  .tab-btn:hover {
    color: var(--text-secondary, #888);
  }

  .tab-btn.active {
    color: var(--text-primary, #fff);
    border-bottom-color: var(--text-primary, #fff);
  }

  .tab-icon {
    font-size: 8px;
  }

  .tab-label {
    font-size: 10px;
  }
</style>
