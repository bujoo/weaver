<script lang="ts">
	import { selectedSessionId } from '$lib/stores/sessions';

	interface Props {
		onsend?: (prompt: string) => void;
	}

	let { onsend }: Props = $props();

	let promptText = $state('');
	let textareaElement: HTMLTextAreaElement;
	let hasSelectedSession = $derived($selectedSessionId !== null);

	function handleSend() {
		const trimmedPrompt = promptText.trim();
		if (trimmedPrompt && onsend) {
			onsend(trimmedPrompt);
			promptText = '';
			// Reset textarea height
			if (textareaElement) {
				textareaElement.style.height = 'auto';
			}
		}
	}

	function handleKeyDown(event: KeyboardEvent) {
		// Send on Ctrl+Enter or Cmd+Enter
		if ((event.ctrlKey || event.metaKey) && event.key === 'Enter') {
			event.preventDefault();
			handleSend();
		}
	}

	function handleInput() {
		// Auto-resize textarea
		if (textareaElement) {
			textareaElement.style.height = 'auto';
			textareaElement.style.height = textareaElement.scrollHeight + 'px';
		}
	}
</script>

<div class="prompt-input">
	<div class="input-container">
		<textarea
			bind:this={textareaElement}
			bind:value={promptText}
			onkeydown={handleKeyDown}
			oninput={handleInput}
			placeholder={hasSelectedSession
				? 'Type your message... (Ctrl+Enter to send)'
				: 'Select a session to send a message'}
			disabled={!hasSelectedSession}
			rows="1"
		></textarea>
		<button
			class="send-button"
			onclick={handleSend}
			disabled={!hasSelectedSession || !promptText.trim()}
			type="button"
			aria-label="Send message"
		>
			<svg
				width="20"
				height="20"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="22" y1="2" x2="11" y2="13"></line>
				<polygon points="22 2 15 22 11 13 2 9 22 2"></polygon>
			</svg>
		</button>
	</div>
</div>

<style>
	.prompt-input {
		border-top: 1px solid var(--border-default);
		background: var(--bg-elevated);
		padding: 16px 20px;
	}

	.input-container {
		display: flex;
		gap: 12px;
		align-items: flex-end;
	}

	textarea {
		flex: 1;
		padding: 12px 16px;
		border: 1px solid var(--border-default);
		border-radius: var(--radius-md);
		font-size: 14px;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		resize: none;
		max-height: 200px;
		overflow-y: auto;
		transition: border-color 0.2s;
		background: var(--bg-card);
		color: var(--text-primary);
	}

	textarea:focus {
		outline: none;
		border-color: var(--weavy);
	}

	textarea:disabled {
		background: var(--bg-base);
		color: var(--text-muted);
		cursor: not-allowed;
	}

	.send-button {
		padding: 12px 16px;
		background: var(--weavy);
		color: var(--bg-base);
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: background-color 0.2s;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.send-button:hover:not(:disabled) {
		background: var(--weavy);
		filter: brightness(1.15);
	}

	.send-button:disabled {
		background: var(--border-default);
		color: var(--text-muted);
		cursor: not-allowed;
	}
</style>
