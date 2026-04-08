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
    const reply = await handleChat(lower, text);
    messages = [...messages, {
      id: `weavy-${Date.now()}`,
      timestamp: Date.now(),
      role: 'weavy',
      content: reply,
    }];
    scrollToBottom();
  }

  // ── Weavy Command Engine ──────────────────────────────────────────
  // Weavy has full control: queries state, starts/stops sessions, pushes phases

  async function handleChat(lower: string, original: string): Promise<string> {
    setMood('thinking', 3000);
    const invoke = isTauri() ? (await import('@tauri-apps/api/core')).invoke : null;

    // ── Slash commands: instant local responses ──
    if (original.startsWith('/')) {
      return handleSlashCommand(lower.slice(1), original.slice(1), invoke);
    }

    // ── AI-first: try Bedrock for everything ──
    if (invoke) {
      try {
        const aiReply = await invoke<string>('weavy_chat', { message: original });
        setMood('happy', 3000);
        xp += 1;
        return aiReply;
      } catch (e) {
        const err = String(e);
        if (err.includes('not_enabled')) {
          // Conductor disabled -- fall through to local handlers
        } else if (err.includes('bedrock_error')) {
          // SSO not logged in or network issue -- fall through
        }
        // Fall through to local keyword handlers
      }
    }

    // ── Offline fallback: keyword handlers ──
    return handleOffline(lower, original, invoke);
  }

  async function handleSlashCommand(cmd: string, original: string, invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    const lower = cmd.toLowerCase().trim();

    if (lower === 'help' || lower === '?') {
      return `Slash commands:\n/status -- system overview\n/watch -- see Claude Code output\n/continue -- push next phase\n/push P1 -- push specific phase\n/send <msg> -- send to Claude Code\n/missions -- list missions\n/sessions -- active sessions\n/workspace -- repos and worktrees\n/regenerate -- refresh CLAUDE.md\n\nAnything else: just type naturally and I'll answer with AI.`;
    }
    if (lower === 'status' || lower === 'sitrep') {
      return handleOfflineStatus(invoke);
    }
    if (lower === 'watch') {
      return handleOfflineWatch(invoke);
    }
    if (lower.startsWith('continue') || lower.startsWith('next')) {
      return handleOfflineContinue(lower, original, invoke);
    }
    if (lower.startsWith('push') || lower.startsWith('send') || lower.startsWith('tell')) {
      return handleOfflineContinue(lower, original, invoke);
    }
    if (lower === 'missions') {
      return handleOfflineMissions(invoke);
    }
    if (lower === 'sessions') {
      return handleOfflineSessions(invoke);
    }
    if (lower === 'workspace' || lower === 'repos') {
      return handleOfflineWorkspace(invoke);
    }
    if (lower === 'regenerate') {
      return handleOfflineRegenerate(invoke);
    }
    if (lower.startsWith('load')) {
      return handleOfflineLoadFixture(invoke);
    }
    return `Unknown command: /${lower}. Type /help for available commands.`;
  }

  async function handleOffline(lower: string, original: string, invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    // Status queries
    if (lower.includes('status') || lower.includes('what') || lower.includes('happening') || lower.includes('going on') || lower.includes('sitrep') || lower.includes('overview')) {
      return handleOfflineStatus(invoke);
    }
    if (lower.includes('mission') || lower.includes('list')) {
      return handleOfflineMissions(invoke);
    }
    if (lower.includes('watch') || lower.includes('session') || lower.includes('tmux') || lower.includes('claude code')) {
      return handleOfflineWatch(invoke);
    }
    if (lower.startsWith('continue') || lower.startsWith('push') || lower.startsWith('send') || lower.startsWith('next phase')) {
      return handleOfflineContinue(lower, original, invoke);
    }
    if (lower.includes('hello') || lower.includes('hi') || lower === 'hey') {
      setMood('happy', 3000);
      return `Hey! I'm in offline mode (Bedrock not available). Type /help to see what I can do locally, or enable CONDUCTOR_ENABLED=1 with AWS SSO for full AI mode.`;
    }
    return `I'm in offline mode (no Bedrock). Use slash commands for local actions:\n/status, /watch, /continue, /missions, /help`;
  }

  async function handleOfflineStatus(invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    try {
      const mqtt = invoke ? await invoke<boolean>('get_mqtt_status') : false;
      const state = invoke ? await invoke<{ plans: number; phases: number; todos: number }>('get_mission_state') : null;
      const missionList = $missions;
      const active = missionList.filter(m => m.status === 'executing' || m.status === 'ready');
      let s = `MQTT: ${mqtt ? 'connected' : 'disconnected'}\nMissions: ${missionList.length} total, ${active.length} active\n`;
      if (state) s += `Cache: ${state.plans} plans, ${state.phases} phases, ${state.todos} todos\n`;
      active.forEach(m => { s += `\n  ${m.title} (${m.status}) -- ${m.phaseCount} phases`; });
      setMood('working');
      return s;
    } catch (e) { return `Error: ${e}`; }
  }

  async function handleOfflineMissions(_invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    const list = $missions;
    if (list.length === 0) return 'No missions loaded. Check MQTT or use /load.';
    return list.map((m, i) => `${i + 1}. ${m.title}\n   ${m.missionId.slice(0, 8)}... | ${m.status} | ${m.phaseCount} phases`).join('\n\n');
  }

  async function handleOfflineSessions(invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    try {
      const sessions = invoke ? await invoke<Array<{ name: string }>>('list_weaver_sessions') : [];
      if (sessions.length === 0) return 'No active sessions.';
      return `Sessions:\n${sessions.map(s => `  ${s.name}`).join('\n')}\n\nUse /watch to see output.`;
    } catch { return 'Could not list sessions.'; }
  }

  async function handleOfflineWatch(invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    try {
      const sessions = invoke ? await invoke<Array<{ name: string }>>('list_weaver_sessions') : [];
      if (sessions.length === 0) return 'No active sessions to watch.';
      const content = invoke ? await invoke<string>('read_weaver_session', { sessionName: sessions[0].name, lines: 30 }) : '';
      const trimmed = content.trim().split('\n').filter((l: string) => l.trim()).slice(-15).join('\n');
      setMood('working');
      return `${sessions[0].name}:\n\n${trimmed || '(empty)'}`;
    } catch (e) { return `Error: ${e}`; }
  }

  async function handleOfflineContinue(lower: string, original: string, invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    const sessions = invoke ? await invoke<Array<{ name: string }>>('list_weaver_sessions').catch(() => []) : [];
    if (sessions.length === 0) return 'No active session.';

    let msg: string;
    if (lower.startsWith('continue') || lower.startsWith('next')) {
      msg = 'Continue with the next phase. Read .weaver/specs/ for todo specs.';
    } else if (lower.startsWith('push')) {
      const m = original.match(/p(\d+)/i);
      msg = `Execute Phase ${m ? `P${m[1]}` : 'P0'}. Read .weaver/specs/ for each todo spec.`;
    } else {
      msg = original.replace(/^(send |tell )/i, '');
    }

    const port = await findChannelPort(invoke);
    if (port) {
      const sent = await sendToChannel(port, { type: 'message', content: msg, mission_id: $missions[0]?.missionId ?? '' });
      if (sent) { setMood('working'); xp += 5; return `Sent via channel: "${msg.slice(0, 80)}"`; }
    }
    return `Channel not available.\ntmux send-keys -t ${sessions[0].name} "${msg.slice(0, 60)}" Enter`;
  }

  async function handleOfflineWorkspace(invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    try {
      const ws = invoke ? await invoke<{ mountPath: string; repos: Array<{ name: string; branch: string; clean: boolean }> }>('get_workspace_status') : null;
      if (!ws) return 'Cannot read workspace.';
      let out = `Mount: ${ws.mountPath}\n`;
      ws.repos.forEach(r => { out += `  ${r.name} (${r.branch})${r.clean ? '' : ' -- dirty'}\n`; });
      return out;
    } catch (e) { return `Error: ${e}`; }
  }

  async function handleOfflineRegenerate(invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    const list = $missions;
    if (list.length === 0) return 'No missions to regenerate.';
    try {
      const ok = invoke ? await invoke<boolean>('regenerate_workspace_context', { missionId: list[0].missionId }) : false;
      setMood('happy', 3000);
      return ok ? `Regenerated context for ${list[0].missionId.slice(0, 8)}.` : 'Nothing to regenerate.';
    } catch (e) { return `Error: ${e}`; }
  }

  async function handleOfflineLoadFixture(invoke: ((c: string, a?: Record<string, unknown>) => Promise<unknown>) | null): Promise<string> {
    try {
      const r = invoke ? await invoke<{ missionId: string; title: string; phases: number; todos: number }>('load_fixture', { path: null }) : null;
      if (r) { setMood('happy', 5000); xp += 20; return `Loaded: ${r.title} (${r.phases} phases, ${r.todos} todos)`; }
      return 'Loaded.';
    } catch (e) { return `Error: ${e}`; }
  }
  }

  async function findChannelPort(invoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null): Promise<number | null> {
    if (!invoke) return null;
    try {
      const port = await invoke('get_channel_port', { missionId: null }) as number | null;
      if (!port) return null;
      // Verify the channel is actually alive
      const resp = await fetch(`http://127.0.0.1:${port}/health`, { signal: AbortSignal.timeout(1000) });
      if (resp.ok) return port;
    } catch { /* channel not responding */ }
    return null;
  }

  async function sendToChannel(port: number, payload: Record<string, unknown>): Promise<boolean> {
    try {
      const resp = await fetch(`http://127.0.0.1:${port}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
        signal: AbortSignal.timeout(3000),
      });
      return resp.ok;
    } catch {
      return false;
    }
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
