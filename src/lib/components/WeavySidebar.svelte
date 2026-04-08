<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri } from '$lib/ws';

  interface WeavyMessage {
    id: string;
    timestamp: number;
    role: 'weavy' | 'user' | 'system';
    content: string;
    decision?: string;
    model?: string;
  }

  let messages = $state<WeavyMessage[]>([]);
  let input = $state('');
  let isExpanded = $state(true);
  let messagesEnd: HTMLElement | undefined;

  function scrollToBottom() {
    if (messagesEnd) {
      messagesEnd.scrollIntoView({ behavior: 'smooth' });
    }
  }

  onMount(async () => {
    // Welcome message
    messages.push({
      id: 'welcome',
      timestamp: Date.now(),
      role: 'weavy',
      content: 'Hey! I\'m Weavy, your AI dev sidekick. I watch over your Claude Code sessions and make decisions about phase transitions, error recovery, and context injection. Ask me anything or tell me what to do.',
    });

    if (isTauri()) {
      const { listen } = await import('@tauri-apps/api/event');

      // Listen for conductor decisions
      listen<{ model_used?: string; decision?: { action?: string; reason?: string; phase_id?: string; mission_id?: string }; input_tokens?: number; output_tokens?: number }>('conductor-decision', (event) => {
        const p = event.payload;
        const d = p.decision;
        const action = d?.action ?? '?';
        const reason = d?.reason ?? '';
        messages.push({
          id: `dec-${Date.now()}`,
          timestamp: Date.now(),
          role: 'weavy',
          content: reason,
          decision: action,
          model: p.model_used?.split('.').pop() ?? '?',
        });
        messages = [...messages];
        scrollToBottom();
      });

      // Listen for conductor actions (executed decisions)
      listen<{ action?: string; mission_id?: string; phase_id?: string; reason?: string }>('conductor-action', (event) => {
        const p = event.payload;
        messages.push({
          id: `act-${Date.now()}`,
          timestamp: Date.now(),
          role: 'system',
          content: `Executed: ${p.action} ${p.phase_id ?? ''} -- ${p.reason ?? ''}`,
        });
        messages = [...messages];
        scrollToBottom();
      });

      // Listen for escalations
      listen<{ mission_id?: string; reason?: string }>('conductor-escalation', (event) => {
        messages.push({
          id: `esc-${Date.now()}`,
          timestamp: Date.now(),
          role: 'weavy',
          content: `I need your help: ${event.payload.reason ?? 'Unknown issue'}`,
        });
        messages = [...messages];
        scrollToBottom();
      });

      // Listen for claude activity highlights
      listen<{ event_type?: string; message?: string; todo_id?: string }>('claude-activity', (event) => {
        const p = event.payload;
        if (p.event_type === 'todo_completed' || p.event_type === 'channel_reply') {
          messages.push({
            id: `cc-${Date.now()}`,
            timestamp: Date.now(),
            role: 'system',
            content: p.message ?? 'Claude Code update',
          });
          messages = [...messages];
          scrollToBottom();
        }
      });
    }
  });

  async function sendMessage() {
    if (!input.trim()) return;
    const text = input.trim();
    input = '';

    messages.push({
      id: `user-${Date.now()}`,
      timestamp: Date.now(),
      role: 'user',
      content: text,
    });
    messages = [...messages];
    scrollToBottom();

    // Handle simple commands
    const lower = text.toLowerCase();
    if (lower.startsWith('push p') || lower.startsWith('start p')) {
      const phaseMatch = text.match(/p(\d+)/i);
      if (phaseMatch) {
        const phaseId = `P${phaseMatch[1]}`;
        messages.push({
          id: `weavy-${Date.now()}`,
          timestamp: Date.now(),
          role: 'weavy',
          content: `Got it. Pushing phase ${phaseId} to the active session...`,
        });
        messages = [...messages];

        // TODO: call Tauri command to push phase via channel
        // For now, show what would happen
        messages.push({
          id: `sys-${Date.now()}`,
          timestamp: Date.now(),
          role: 'system',
          content: `Phase ${phaseId} push requested (conductor wiring pending)`,
        });
        messages = [...messages];
        scrollToBottom();
        return;
      }
    }

    // Default: acknowledge and explain
    messages.push({
      id: `weavy-${Date.now()}`,
      timestamp: Date.now(),
      role: 'weavy',
      content: `I'll handle that once the conductor is fully wired. For now, I'm watching Claude Code sessions and logging decisions.`,
    });
    messages = [...messages];
    scrollToBottom();
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  }
</script>

<aside class="weavy-sidebar" class:collapsed={!isExpanded}>
  <button class="toggle-btn" onclick={() => (isExpanded = !isExpanded)} type="button">
    {isExpanded ? '>' : '<'}
  </button>

  {#if isExpanded}
    <div class="sidebar-header">
      <span class="weavy-name">WEAVY</span>
      <span class="weavy-status">AI Sidekick</span>
    </div>

    <div class="messages-container">
      {#each messages as msg (msg.id)}
        <div class="message" class:weavy={msg.role === 'weavy'} class:user={msg.role === 'user'} class:system={msg.role === 'system'}>
          {#if msg.decision}
            <span class="decision-badge">{msg.decision}</span>
          {/if}
          <span class="msg-content">{msg.content}</span>
          <span class="msg-meta">
            {formatTime(msg.timestamp)}
            {#if msg.model}
              <span class="model-tag">{msg.model}</span>
            {/if}
          </span>
        </div>
      {/each}
      <div bind:this={messagesEnd}></div>
    </div>

    <form class="input-area" onsubmit={(e) => { e.preventDefault(); sendMessage(); }}>
      <input
        type="text"
        bind:value={input}
        placeholder="Talk to Weavy..."
        class="weavy-input"
      />
    </form>
  {/if}
</aside>

<style>
  .weavy-sidebar {
    width: 280px;
    min-width: 280px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--bg-surface, #0a0a0f);
    border-left: 1px solid var(--border-muted, rgba(255, 255, 255, 0.06));
    position: relative;
    transition: width 0.2s, min-width 0.2s;
  }

  .weavy-sidebar.collapsed {
    width: 28px;
    min-width: 28px;
  }

  .toggle-btn {
    position: absolute;
    left: -1px;
    top: 12px;
    width: 20px;
    height: 28px;
    background: var(--bg-surface, #0a0a0f);
    border: 1px solid var(--border-muted, rgba(255, 255, 255, 0.06));
    border-right: none;
    color: var(--text-muted);
    font-size: 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1;
  }

  .toggle-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.04);
  }

  .sidebar-header {
    padding: 12px 14px 8px;
    border-bottom: 1px solid var(--border-muted, rgba(255, 255, 255, 0.06));
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .weavy-name {
    font-family: var(--font-pixel, monospace);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: #a78bfa;
  }

  .weavy-status {
    font-family: var(--font-mono, monospace);
    font-size: 9px;
    color: var(--text-muted);
    letter-spacing: 0.04em;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .message {
    padding: 6px 10px;
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    line-height: 1.5;
    border-radius: 4px;
  }

  .message.weavy {
    background: rgba(167, 139, 250, 0.08);
    border-left: 2px solid #a78bfa;
    color: var(--text-primary, #e0e0e0);
  }

  .message.user {
    background: rgba(96, 165, 250, 0.08);
    border-left: 2px solid #60a5fa;
    color: var(--text-primary, #e0e0e0);
  }

  .message.system {
    background: rgba(255, 255, 255, 0.02);
    border-left: 2px solid rgba(255, 255, 255, 0.1);
    color: var(--text-muted);
    font-size: 10px;
  }

  .decision-badge {
    display: inline-block;
    font-size: 9px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: #a78bfa;
    background: rgba(167, 139, 250, 0.15);
    padding: 1px 6px;
    border-radius: 2px;
    margin-bottom: 4px;
  }

  .msg-content {
    display: block;
    word-wrap: break-word;
  }

  .msg-meta {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 4px;
    font-size: 9px;
    color: var(--text-muted);
  }

  .model-tag {
    font-size: 8px;
    padding: 0 4px;
    border: 1px solid rgba(167, 139, 250, 0.3);
    border-radius: 2px;
    color: #a78bfa;
  }

  .input-area {
    padding: 8px;
    border-top: 1px solid var(--border-muted, rgba(255, 255, 255, 0.06));
  }

  .weavy-input {
    width: 100%;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--text-primary, #e0e0e0);
    font-family: var(--font-mono, monospace);
    font-size: 11px;
    padding: 8px 10px;
    outline: none;
  }

  .weavy-input:focus {
    border-color: #a78bfa;
    background: rgba(167, 139, 250, 0.04);
  }

  .weavy-input::placeholder {
    color: var(--text-muted);
  }
</style>
