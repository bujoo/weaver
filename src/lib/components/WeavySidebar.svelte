<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import { isTauri } from '$lib/ws';
  import { missions } from '$lib/stores/missions';

  // ── Robot Poses ────────────────────────────────────────────────────

  interface RobotPose {
    left: string;
    eyes: string;
    right: string;
  }

  type Mood = 'idle' | 'working' | 'happy' | 'thinking' | 'alert' | 'flexing';

  const moodPoses: Record<Mood, RobotPose[]> = {
    idle: [
      { left: '', eyes: '[-_-]', right: '' },
      { left: '', eyes: '[-_-]', right: 'z' },
      { left: '', eyes: '[-_-]', right: 'zZ' },
      { left: '', eyes: '[-_-]', right: 'zZz' },
      { left: '', eyes: '[~_~]', right: '' },
      { left: '\\', eyes: '[-_-]', right: '/' },
      { left: '\u00AF\\_', eyes: '[-_-]', right: '_/\u00AF' },
    ],
    working: [
      { left: '', eyes: '[*_*]', right: '/' },
      { left: '\\', eyes: '[*_*]', right: '' },
      { left: '', eyes: '[\u2022_\u2022]', right: '/' },
      { left: '\\', eyes: '[\u2022_\u2022]', right: '/' },
      { left: '', eyes: '[\u2022_-]', right: '' },
      { left: '', eyes: '[-_\u2022]', right: '' },
      { left: '\\', eyes: '[\u2022_\u2022]', right: '' },
    ],
    happy: [
      { left: '\\', eyes: '[^_^]', right: '/' },
      { left: '', eyes: '[^_^]', right: '' },
      { left: '\\', eyes: '[*_*]', right: '/' },
      { left: '', eyes: '[\u2022_\u2022]', right: '/' },
    ],
    thinking: [
      { left: '', eyes: '[o_o]', right: '' },
      { left: '\u00AF\\_', eyes: '[\u2022_\u2022]', right: '_/\u00AF' },
      { left: '', eyes: '[\u2022_\u2022]', right: '' },
      { left: '', eyes: '[-_\u2022]', right: '' },
      { left: '', eyes: '[\u2022_-]', right: '...' },
    ],
    alert: [
      { left: '\\', eyes: '[!_!]', right: '/' },
      { left: '\\', eyes: '[o_o]', right: '/' },
      { left: '', eyes: '[>_<]', right: '' },
      { left: '\u1D66', eyes: '[!_!]', right: '\u1D64' },
    ],
    flexing: [
      { left: '\u1D66', eyes: '[^_^]', right: '\u1D64' },
      { left: '\u1D66', eyes: '[*_*]', right: '\u1D64' },
      { left: '\\', eyes: '[^_^]', right: '/' },
      { left: '\u1D66', eyes: '[\u2022_\u2022]', right: '\u1D64' },
    ],
  };

  // ── Level System ───────────────────────────────────────────────────

  const levels = [
    { name: 'Intern', xp: 0 },
    { name: 'Junior', xp: 100 },
    { name: 'Mid', xp: 300 },
    { name: 'Senior', xp: 600 },
    { name: 'Architect', xp: 1000 },
    { name: 'CTO', xp: 2000 },
  ];

  function getLevel(xp: number): { level: number; name: string; nextXp: number; percent: number } {
    let lvl = 0;
    for (let i = levels.length - 1; i >= 0; i--) {
      if (xp >= levels[i].xp) { lvl = i; break; }
    }
    const next = lvl < levels.length - 1 ? levels[lvl + 1].xp : levels[lvl].xp;
    const prev = levels[lvl].xp;
    const range = next - prev || 1;
    return {
      level: lvl + 1,
      name: levels[lvl].name,
      nextXp: next,
      percent: Math.min(100, ((xp - prev) / range) * 100),
    };
  }

  // ── State ──────────────────────────────────────────────────────────

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

  let mood = $state<Mood>('idle');
  let currentPose = $state<RobotPose>({ left: '', eyes: '[-_-]', right: '' });
  let xp = $state(0);
  let levelInfo = $derived(getLevel(xp));

  let poseInterval: ReturnType<typeof setInterval>;
  let moodTimeout: ReturnType<typeof setTimeout>;
  let lastActivity = $state(Date.now());

  function randomPose(m: Mood): RobotPose {
    const poses = moodPoses[m];
    return poses[Math.floor(Math.random() * poses.length)];
  }

  function setMood(m: Mood, durationMs?: number) {
    mood = m;
    currentPose = randomPose(m);
    lastActivity = Date.now();
    if (moodTimeout) clearTimeout(moodTimeout);
    if (durationMs) {
      moodTimeout = setTimeout(() => {
        mood = 'idle';
        currentPose = randomPose('idle');
      }, durationMs);
    }
  }

  async function scrollToBottom() {
    await tick();
    if (messagesEnd) {
      messagesEnd.scrollIntoView({ behavior: 'smooth' });
    }
  }

  // ── Lifecycle ──────────────────────────────────────────────────────

  onMount(async () => {
    // Rotate poses every 3s
    poseInterval = setInterval(() => {
      // Auto-idle after 30s of no events
      if (Date.now() - lastActivity > 30000 && mood !== 'alert') {
        mood = 'idle';
      }
      currentPose = randomPose(mood);
    }, 3000);

    // Welcome
    messages.push({
      id: 'welcome',
      timestamp: Date.now(),
      role: 'weavy',
      content: 'Hey! I\'m Weavy, your AI dev sidekick. I watch over your Claude Code sessions and keep things moving.',
    });

    if (isTauri()) {
      const { listen } = await import('@tauri-apps/api/event');

      // Conductor decisions
      listen<{ model_used?: string; decision?: { action?: string; reason?: string; phase_id?: string; mission_id?: string }; input_tokens?: number; output_tokens?: number }>('conductor-decision', (event) => {
        const p = event.payload;
        const d = p.decision;
        const action = d?.action ?? '?';
        const reason = d?.reason ?? '';
        setMood('thinking', 3000);
        messages = [...messages, {
          id: `dec-${Date.now()}`,
          timestamp: Date.now(),
          role: 'weavy',
          content: reason,
          decision: action,
          model: p.model_used?.split('.').pop() ?? '?',
        }];
        scrollToBottom();
      });

      // Conductor actions
      listen<{ action?: string; mission_id?: string; phase_id?: string; reason?: string }>('conductor-action', (event) => {
        const p = event.payload;
        setMood('working');
        messages = [...messages, {
          id: `act-${Date.now()}`,
          timestamp: Date.now(),
          role: 'system',
          content: `Executed: ${p.action} ${p.phase_id ?? ''} -- ${p.reason ?? ''}`,
        }];
        scrollToBottom();
      });

      // Escalations
      listen<{ mission_id?: string; reason?: string }>('conductor-escalation', (event) => {
        setMood('alert');
        messages = [...messages, {
          id: `esc-${Date.now()}`,
          timestamp: Date.now(),
          role: 'weavy',
          content: `I need your help: ${event.payload.reason ?? 'Unknown issue'}`,
        }];
        scrollToBottom();
      });

      // Claude activity
      listen<{ event_type?: string; message?: string; todo_id?: string }>('claude-activity', (event) => {
        const p = event.payload;
        if (p.event_type === 'todo_completed') {
          setMood('happy', 5000);
          xp += 10;
          messages = [...messages, {
            id: `cc-${Date.now()}`,
            timestamp: Date.now(),
            role: 'system',
            content: p.message ?? 'Todo completed',
          }];
          scrollToBottom();
        } else if (p.event_type === 'channel_reply') {
          messages = [...messages, {
            id: `cc-${Date.now()}`,
            timestamp: Date.now(),
            role: 'system',
            content: p.message ?? 'Claude Code update',
          }];
          scrollToBottom();
        } else if (p.event_type === 'tool_use') {
          setMood('working');
        }
      });

      // Phase complete (from MQTT events)
      listen<{ mission_id?: string }>('mission-phases-updated', () => {
        setMood('flexing', 8000);
        xp += 50;
      });
    }
  });

  onDestroy(() => {
    clearInterval(poseInterval);
    if (moodTimeout) clearTimeout(moodTimeout);
  });

  // ── Chat ───────────────────────────────────────────────────────────

  async function sendMessage() {
    if (!input.trim()) return;
    const text = input.trim();
    input = '';

    messages = [...messages, {
      id: `user-${Date.now()}`,
      timestamp: Date.now(),
      role: 'user',
      content: text,
    }];
    scrollToBottom();
    setMood('thinking', 3000);

    const lower = text.toLowerCase();
    const reply = handleChat(lower, text);
    messages = [...messages, {
      id: `weavy-${Date.now()}`,
      timestamp: Date.now(),
      role: 'weavy',
      content: reply,
    }];
    scrollToBottom();
  }

  function handleChat(lower: string, _original: string): string {
    const missionList = $missions;
    setMood('thinking', 3000);

    // Mission queries
    if (lower.includes('mission') || lower.includes('what are') || lower.includes('status')) {
      if (missionList.length === 0) {
        return 'No missions loaded yet. Make sure MQTT is connected and Brain has published a registry.';
      }
      const summary = missionList.map((m, i) =>
        `${i + 1}. ${m.title} (${m.status}) -- ${m.phaseCount} phases, ${m.todoCount} todos`
      ).join('\n');
      return `Here are the current missions:\n\n${summary}`;
    }

    // Push phase
    if (lower.startsWith('push p') || lower.startsWith('start p')) {
      const phaseMatch = lower.match(/p(\d+)/i);
      if (phaseMatch) {
        setMood('working');
        return `Got it! Pushing phase P${phaseMatch[1]} to the active session... (conductor wiring pending)`;
      }
    }

    // Help
    if (lower.includes('help') || lower === '?') {
      return 'I can help with:\n- "missions" or "status" -- show current missions\n- "push P1" -- push a phase to Claude Code\n- "what phase" -- show active phase\n- Ask me anything about the current work!';
    }

    // Phase queries
    if (lower.includes('phase') || lower.includes('current')) {
      if (missionList.length > 0) {
        const m = missionList[0];
        return `Working on: ${m.title}\nPhases: ${m.phaseCount} total\nTodos: ${m.todoCount}\nStatus: ${m.status}`;
      }
      return 'No active mission. Waiting for Brain to assign work.';
    }

    // Greeting
    if (lower.includes('hello') || lower.includes('hi') || lower.includes('hey')) {
      setMood('happy', 3000);
      return 'Hey there! Ready to help with your missions. Ask me about current status or tell me to push a phase.';
    }

    // Default
    return `I heard you. Once the conductor is fully wired to Bedrock, I'll be able to make smarter decisions. For now, try "missions" or "help".`;
  }

  function formatTime(ts: number): string {
    return new Date(ts).toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
  }
</script>

<aside class="weavy-sidebar" class:collapsed={!isExpanded}>
  <button class="toggle-btn" onclick={() => (isExpanded = !isExpanded)} type="button">
    {isExpanded ? '\u203A' : '\u2039'}
  </button>

  {#if isExpanded}
    <!-- Robot Avatar -->
    <div class="weavy-avatar">
      <div class="robot-pose" class:alert={mood === 'alert'} class:happy={mood === 'happy' || mood === 'flexing'}>
        <span class="arm-left">{currentPose.left}</span><span class="eyes">{currentPose.eyes}</span><span class="arm-right">{currentPose.right}</span>
      </div>
      <div class="xp-bar-container">
        <div class="xp-bar">
          <div class="xp-fill" style="width: {levelInfo.percent}%"></div>
        </div>
      </div>
      <span class="level-label">Lv.{levelInfo.level} {levelInfo.name} -- {xp} XP</span>
    </div>

    <!-- Messages -->
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

    <!-- Input -->
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
    font-size: 14px;
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

  /* Robot Avatar */
  .weavy-avatar {
    text-align: center;
    padding: 16px 8px 12px;
    border-bottom: 1px solid var(--border-muted, rgba(255, 255, 255, 0.06));
  }

  .robot-pose {
    font-family: var(--font-mono, monospace);
    font-size: 22px;
    color: #a78bfa;
    letter-spacing: 0;
    line-height: 1;
    transition: color 0.3s;
    white-space: nowrap;
    user-select: none;
  }

  .robot-pose.happy {
    color: #4ade80;
  }

  .robot-pose.alert {
    color: #f87171;
    animation: pulse 1s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .arm-left, .arm-right {
    font-size: 18px;
    vertical-align: middle;
  }

  .eyes {
    vertical-align: middle;
  }

  .xp-bar-container {
    display: flex;
    justify-content: center;
    margin: 10px 0 4px;
  }

  .xp-bar {
    width: 75%;
    height: 3px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 2px;
    overflow: hidden;
  }

  .xp-fill {
    height: 100%;
    background: linear-gradient(90deg, #a78bfa, #c4b5fd);
    transition: width 0.5s ease;
    border-radius: 2px;
  }

  .level-label {
    display: block;
    font-family: var(--font-mono, monospace);
    font-size: 9px;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    margin-top: 2px;
  }

  /* Messages */
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

  /* Input */
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
