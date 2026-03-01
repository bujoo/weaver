<script lang="ts">
	import { onMount } from 'svelte';
	import { marked } from 'marked';
	import DOMPurify from 'dompurify';
	import { getMemoryFiles, revealInFileManager } from '$lib/api';
	import type { ProjectMemory } from '$lib/types';

	let projects = $state<ProjectMemory[]>([]);
	let loading = $state(true);
	let selectedIndex = $state(0);

	async function refresh() {
		loading = true;
		projects = await getMemoryFiles();
		if (selectedIndex >= projects.length) {
			selectedIndex = 0;
		}
		loading = false;
	}

	onMount(() => {
		refresh();
	});

	let selectedProject = $derived(projects[selectedIndex] ?? null);

	function renderMarkdown(content: string): string {
		const renderer = new marked.Renderer();

		renderer.code = function ({ text, lang }) {
			const language = lang || 'code';
			return `
				<div class="code-block-wrapper">
					<div class="code-header">
						<span class="code-lang">${language}</span>
					</div>
					<pre><code class="language-${language}">${text}</code></pre>
				</div>
			`;
		};

		const rawHtml = marked.parse(content, {
			async: false,
			breaks: true,
			gfm: true,
			renderer
		});
		return DOMPurify.sanitize(rawHtml as string);
	}

	async function handleReveal() {
		if (!selectedProject) return;
		await revealInFileManager(selectedProject.memoryDirPath);
	}
</script>

<div class="memory-viewer">
	<div class="memory-header">
		<span class="memory-title">Memory Files</span>
		<div class="memory-actions">
			{#if selectedProject}
				<button class="action-btn" onclick={handleReveal} title="Reveal in Finder">
					⌕
				</button>
			{/if}
			<button class="action-btn" onclick={refresh} title="Refresh" disabled={loading}>
				↻
			</button>
		</div>
	</div>

	{#if loading}
		<div class="loading-state">
			<span class="loading-spinner">◌</span>
			<span>Scanning memory files…</span>
		</div>
	{:else if projects.length === 0}
		<div class="empty-state">
			<p>No memory files found.</p>
			<p class="empty-hint">Claude Code stores memory in ~/.claude/projects/*/memory/</p>
		</div>
	{:else}
		<div class="two-panel">
			<aside class="project-list">
				{#each projects as project, i}
					<button
						class="project-item"
						class:selected={i === selectedIndex}
						onclick={() => (selectedIndex = i)}
					>
						<span class="project-item-name">{project.projectName}</span>
						<span class="project-item-count">{project.files.length}</span>
					</button>
				{/each}
			</aside>

			<div class="memory-content">
				{#if selectedProject}
					<div class="content-header">
						<span class="content-path">{selectedProject.projectPath}</span>
					</div>
					{#each selectedProject.files as file}
						<div class="file-section">
							<div class="file-header">{file.filename}</div>
							<div class="markdown-body">
								{@html renderMarkdown(file.content)}
							</div>
						</div>
					{/each}
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.memory-viewer {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
	}

	.memory-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 16px;
		border-bottom: 1px solid var(--border);
		flex-shrink: 0;
	}

	.memory-title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--text-secondary);
	}

	.memory-actions {
		display: flex;
		gap: 8px;
	}

	.action-btn {
		background: none;
		border: 1px solid var(--border);
		color: var(--text-secondary);
		padding: 4px 8px;
		border-radius: 4px;
		cursor: pointer;
		font-size: 13px;
		transition: all 0.15s ease;
	}

	.action-btn:hover {
		color: var(--text-primary);
		border-color: var(--text-secondary);
	}

	.action-btn:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.loading-state,
	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		padding: 48px 16px;
		color: var(--text-secondary);
		font-size: 13px;
	}

	.loading-spinner {
		animation: spin 1s linear infinite;
		font-size: 18px;
	}

	@keyframes spin {
		from { transform: rotate(0deg); }
		to { transform: rotate(360deg); }
	}

	.empty-hint {
		font-size: 11px;
		opacity: 0.6;
		font-family: var(--font-mono);
	}

	.two-panel {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.project-list {
		width: 200px;
		min-width: 160px;
		border-right: 1px solid var(--border);
		overflow-y: auto;
		flex-shrink: 0;
	}

	.project-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: 10px 12px;
		background: none;
		border: none;
		border-bottom: 1px solid var(--border);
		color: var(--text-secondary);
		cursor: pointer;
		text-align: left;
		font-size: 12px;
		transition: all 0.15s ease;
	}

	.project-item:hover {
		background: rgba(255, 255, 255, 0.03);
		color: var(--text-primary);
	}

	.project-item.selected {
		background: rgba(255, 255, 255, 0.06);
		color: var(--text-primary);
	}

	.project-item-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.project-item-count {
		font-size: 10px;
		opacity: 0.5;
		flex-shrink: 0;
		margin-left: 8px;
	}

	.memory-content {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
	}

	.content-header {
		margin-bottom: 16px;
		padding-bottom: 8px;
		border-bottom: 1px solid var(--border);
	}

	.content-path {
		font-size: 11px;
		color: var(--text-secondary);
		font-family: var(--font-mono);
	}

	.file-section {
		margin-bottom: 24px;
	}

	.file-header {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--accent);
		margin-bottom: 12px;
		padding: 4px 0;
		border-bottom: 1px solid var(--border);
	}

	.markdown-body {
		font-size: 13px;
		line-height: 1.6;
		color: var(--text-primary);
	}

	.markdown-body :global(h1),
	.markdown-body :global(h2),
	.markdown-body :global(h3) {
		margin: 16px 0 8px;
		font-weight: 600;
		color: var(--text-primary);
	}

	.markdown-body :global(h1) { font-size: 16px; }
	.markdown-body :global(h2) { font-size: 14px; }
	.markdown-body :global(h3) { font-size: 13px; }

	.markdown-body :global(p) {
		margin: 8px 0;
	}

	.markdown-body :global(ul),
	.markdown-body :global(ol) {
		margin: 8px 0;
		padding-left: 20px;
	}

	.markdown-body :global(li) {
		margin: 4px 0;
	}

	.markdown-body :global(code) {
		background: rgba(255, 255, 255, 0.06);
		padding: 2px 5px;
		border-radius: 3px;
		font-size: 12px;
		font-family: var(--font-mono);
	}

	.markdown-body :global(strong) {
		color: var(--text-primary);
		font-weight: 600;
	}

	.markdown-body :global(.code-block-wrapper) {
		margin: 12px 0;
		border: 1px solid var(--border);
		border-radius: 6px;
		overflow: hidden;
	}

	.markdown-body :global(.code-header) {
		padding: 6px 12px;
		font-size: 11px;
		color: var(--text-secondary);
		border-bottom: 1px solid var(--border);
		background: rgba(255, 255, 255, 0.02);
	}

	.markdown-body :global(pre) {
		margin: 0;
		padding: 12px;
		overflow-x: auto;
		font-size: 12px;
		line-height: 1.5;
	}

	.markdown-body :global(pre code) {
		background: none;
		padding: 0;
		border-radius: 0;
	}

	.markdown-body :global(a) {
		color: var(--accent);
		text-decoration: none;
	}

	.markdown-body :global(a:hover) {
		text-decoration: underline;
	}

	.markdown-body :global(hr) {
		border: none;
		border-top: 1px solid var(--border);
		margin: 16px 0;
	}

	.markdown-body :global(blockquote) {
		margin: 8px 0;
		padding: 4px 12px;
		border-left: 2px solid var(--border);
		color: var(--text-secondary);
	}

	.markdown-body :global(table) {
		width: 100%;
		border-collapse: collapse;
		margin: 12px 0;
		font-size: 12px;
	}

	.markdown-body :global(th),
	.markdown-body :global(td) {
		padding: 6px 10px;
		border: 1px solid var(--border);
		text-align: left;
	}

	.markdown-body :global(th) {
		font-weight: 600;
		background: rgba(255, 255, 255, 0.03);
	}
</style>
