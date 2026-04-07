<script lang="ts">
</script>

<div class="redirect-page">
  <h2>Tasks</h2>
  <p>Tasks are now managed per-mission.</p>
  <p>Select a mission from the sidebar to view its phases and todos.</p>
  <a href="/">Go to Mission Control</a>
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
  }

  a {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    text-decoration: none;
    border: 1px solid var(--border-default);
    padding: var(--space-sm) var(--space-lg);
    margin-top: var(--space-md);
    transition: background var(--transition-fast), border-color var(--transition-fast);
  }

  a:hover {
    background: rgba(255, 255, 255, 0.06);
    border-color: var(--text-secondary);
  }
</style>
