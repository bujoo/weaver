<script lang="ts">
  interface Props {
    title: string;
  }

  let { title }: Props = $props();
</script>

<div class="page-header" data-tauri-drag-region>
  <span class="page-title">{title}</span>
  <div class="drag-region" data-tauri-drag-region>
    <span class="drag-dots">&#x2817; &#x2817; &#x2817;</span>
  </div>
</div>

<style>
  .page-header {
    height: 28px;
    width: 100%;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    background: transparent;
    z-index: 1000;
    position: relative;
    padding: 0 var(--space-md, 12px) 0 16px;
    -webkit-app-region: drag;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .page-title {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--text-muted, #666);
    -webkit-app-region: no-drag;
    white-space: nowrap;
  }

  .drag-region {
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
  }
</style>
