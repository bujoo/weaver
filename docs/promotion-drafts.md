# c9watch Promotion Drafts

> Post drafts for promoting c9watch across communities.
> GitHub: https://github.com/minchenlee/c9watch

---
Keywords to Search

 English

 Claude Code dashboard
 Claude Code session monitor
 Claude Code menu bar / tray app
 Claude Code multi-project
 AI coding assistant monitor
 macOS menu bar app developer
 Tauri macOS app
 indie macOS app open source
 Show HN macOS developer tool

 Traditional Chinese (ZHTW)

 Claude Code 監控工具
 Claude Code 工作階段管理
 Claude Code 開發者工具
 macOS 狀態列工具 / 選單列 app
 macOS 開源工具推薦
 AI 工具推薦
 工程師必備 macOS 工具
 AI session 管理
 Claude Code 好用工具推薦

---

## Search & Reply Strategy — X and Threads

### Keywords to search (filter by Latest, not Top)

**High intent (people actively frustrated):**
- `"Claude Code" sessions tab`
- `"Claude Code" permission waiting`
- `"Claude Code" multiple terminals`
- `"Claude Code" parallel agents`
- `"Claude Code" switching`
- `claude code monitor`
- `claude code dashboard`

**Broader discovery:**
- `claude code workflow`
- `claude code tips`
- `"running claude code"`

**Threads — search is weak, browse hashtags instead:**
- `#ClaudeCode`
- `#claudecode`

**Traditional Chinese (ZHTW) — X and Threads:**

High intent:
- `Claude Code session`
- `Claude Code 多個`
- `Claude Code 切換`
- `Claude Code 等待`
- `Claude Code permission`
- `Claude Code 同時跑`
- `Claude Code 開好幾個`

Broader discovery:
- `Claude Code 工作流`
- `Claude Code 怎麼用`
- `Claude Code 平行`
- `多個 agent`

Target reply signals: "一直在切"、"不知道哪個在跑"、"同時開好幾個不好管"

### Who to reply to
Look for posts expressing:
- Frustration managing multiple sessions
- Questions like "how do you monitor Claude Code?"
- People sharing multi-agent workflow setups

Skip promotional posts from other tools.

### Reply template
Don't lead with the link. One sentence of empathy, one sentence of what it does, one link:

> Same problem here — I ended up building a menu bar app for this. Scans processes automatically so it works with whatever terminal you're already using: github.com/minchenlee/c9watch

### Cadence
5–10 replies per session, spaced over a few days. Don't batch all at once.
 
---

## Reddit — r/ClaudeCode

### Comment on Agent Conductor post
> https://www.reddit.com/r/ClaudeCode/comments/1rg4wdy/

Nice work! I ran into the same problem and went a different direction — [c9watch](https://github.com/minchenlee/c9watch) is a native macOS tray app (Tauri/Rust) that does something similar but without the browser or server process. I wanted to keep using my own terminals and just have a menu bar icon that picks everything up automatically. Lighter scope too, if that fits better. Would love any feedback!

---

### New post — r/ClaudeCode

**Title:** I got tired of tab-switching between Claude Code sessions, so I built a macOS tray app

**Body:**
I was running 10+ Claude Code sessions across a few projects — Zed on one side, Ghostty on the other — and kept switching between them just to check which one was waiting for permission and which was done. Tried a few existing tools but they all wanted me to launch sessions from inside their app, and I just didn't want to change how I work.

So I wrote [c9watch](https://github.com/minchenlee/c9watch), a native macOS menu bar app that scans your running processes and picks up every Claude Code session automatically. No plugins, no workflow changes, works with whatever terminal you're already using.

What it shows you:
- Every session grouped by status — Working, Needs Permission, Idle, Done
- Model, project, current tool, elapsed time per session
- Sessions by project with git branch info
- A quick tray popover so you don't even have to open the full dashboard

And a few things beyond status monitoring:
- Expand any session to read the full conversation history with formatted code blocks
- Stop a session, rename it, or jump straight to its terminal — all from the dashboard
- Native macOS notifications when something needs your attention
- Scan a QR code to pull up a web client on your phone

Built with Tauri + Rust + Svelte, not Electron. You're already running a bunch of agents eating up memory — the monitoring tool shouldn't add to that.

MIT license, open source, no telemetry.

GitHub: https://github.com/minchenlee/c9watch
Demo: https://youtu.be/9PdN7joYmUk

Would love feedback, especially if something doesn't work with your setup.

---

### New post — r/ClaudeAI

**Title:** Built a macOS menu bar app to monitor all my Claude Code sessions at once

**Body:**
I've been running a lot of Claude Code sessions in parallel and kept losing track of which one needed permission, which was done, which was just sitting idle. So I built [c9watch](https://github.com/minchenlee/c9watch) — a native macOS tray app that shows everything in one place.

The thing I cared most about: it works with whatever terminal you're already using. Zed, VS Code, iTerm2, tmux, Ghostty — it scans running processes at the OS level and picks them all up automatically. No plugins, no setup, no changing how you start sessions.

Beyond the status overview, you can also expand any session to read the full conversation history, jump straight to the parent terminal, or get native macOS notifications when something needs your attention. There's also a QR code web client if you want to check in from your phone.

Built with Tauri + Rust + Svelte (not Electron), MIT license, no telemetry.

GitHub: https://github.com/minchenlee/c9watch
Demo: https://youtu.be/9PdN7joYmUk

Happy to hear any feedback or answer questions about how it works.

---

### New post — r/macapps

**Title:** c9watch — macOS menu bar app to monitor Claude Code sessions (Tauri/Rust, open source)

**Body:**
Sharing something I built for my own workflow in case it's useful to others here.

[c9watch](https://github.com/minchenlee/c9watch) lives in your menu bar and shows a real-time overview of every Claude Code session running on your machine. Click the tray icon for a quick status check, or open the full dashboard to see sessions grouped by status or by project.

It scans running processes at the OS level so it picks everything up automatically — doesn't matter where you launched the session from. VS Code, Zed, iTerm2, tmux, anything. No plugins to install, no workflows to change.

A few other things it does:
- Expand any session to read the full conversation with formatted code
- Stop sessions, rename them, or open their parent terminal/IDE from the dashboard
- Native macOS notifications when a session needs your attention
- QR code to open a web client on your phone

Built with Tauri + Rust + Svelte, not Electron. Small binary, low memory footprint.

- GitHub: https://github.com/minchenlee/c9watch
- Demo: https://youtu.be/9PdN7joYmUk
- Install: `curl -fsSL https://raw.githubusercontent.com/minchenlee/c9watch/main/install.sh | bash`

MIT license, completely open source, no telemetry.

---

### New post — r/SideProject

**Title:** Spent a few months of evenings building a macOS tray app to stop tab-switching between Claude Code sessions

**Body:**
I was running Claude Code across 10+ sessions and kept switching between terminal tabs just to check which agent was waiting for my input. It was getting in the way of actually getting work done.

I looked at existing tools but they all wanted to own how I start sessions. I just wanted to keep using my own terminals and have something that figures out what's running on its own. So I built [c9watch](https://github.com/minchenlee/c9watch).

It scans running processes at the OS level and picks up every Claude Code session automatically — no plugins, no changes to how you work. From the menu bar you get a real-time status overview. Open the full dashboard and you can also read the full conversation history per session, stop or rename sessions, jump to the parent terminal, and get native macOS notifications when something needs your attention. There's even a QR code web client for checking in from your phone.

Stack: Tauri + Rust + Svelte. Not Electron. Kept it as lightweight as possible.

MIT license, completely free.

GitHub: https://github.com/minchenlee/c9watch
Demo: https://youtu.be/9PdN7joYmUk

Would love to hear if it solves the problem for others, or if there's something that doesn't work for your setup.

---

## Hacker News — Show HN

**Title:** Show HN: c9watch – macOS menu bar app to monitor all Claude Code sessions

**Body:**
I was running Claude Code across 10+ terminal tabs and constantly switching between them to check which session needed permission, which was done, which was idle. Tried existing tools but they required launching sessions from within their app — I wanted to keep using my own terminals.

c9watch scans running processes at the OS level and reads from `~/.claude/` to detect every active Claude Code session automatically. Works with any terminal or IDE — VS Code, Zed, iTerm2, tmux, Ghostty — no plugins or workflow changes needed.

The tray popover gives you a quick overview. The full dashboard lets you:
- Group sessions by status or by project (with git branch info)
- Expand any session to read the full conversation — formatted markdown, code blocks, tool calls
- Stop sessions, rename them, or jump to the parent terminal/IDE
- Get native macOS notifications when sessions need attention
- Use a WebSocket-based web/mobile client via QR code

Built with Tauri (Rust + Svelte), not Electron. Rust handles process scanning and JSONL parsing. The binary is small and memory usage is minimal.

MIT license, no telemetry.

GitHub: https://github.com/minchenlee/c9watch
Demo: https://youtu.be/9PdN7joYmUk

---

## PTT — Soft_Job 板

**標題：** [心得] 自己做了一個 Claude Code session 監控工具，用 Tauri + Rust + Svelte

**內文：**
最近在跑很多個 Claude Code session，同時開著 10 幾個 terminal tab，一直在 Zed 和 Ghostty 之間切來切去，想找哪個 agent 在等我 approve、哪個已經跑完了。

本來想用現有的工具，但它們都要從它們的 app 裡面啟動 session，我只是想繼續用自己習慣的 terminal 而已。所以就自己做了一個。

[c9watch](https://github.com/minchenlee/c9watch) 是一個放在 macOS menu bar 的小 app，直接掃 OS 層的 process，不管你從哪裡開的 session 都能自動抓到。不用裝 plugin，不用改工作流程。

**功能：**
- 依狀態分類（Working / Needs Permission / Idle / Done），或依專案分群（含 git branch）
- 點開任一 session 可以看完整對話記錄，有格式化的 code block 和 tool call
- 可以直接從 dashboard 停止 session、改名、或跳回原本的 terminal / IDE
- 有 agent 需要你 approve 時會跳 macOS 原生通知
- 掃 QR code 可以在手機或其他裝置上看 session 狀態

**技術：** Tauri + Rust + Svelte，不是 Electron。binary 小，記憶體用量低，已經在跑一堆 agent 了，監控工具不應該再吃資源。

MIT 授權，完全免費，沒有 telemetry，開源。

GitHub: https://github.com/minchenlee/c9watch
Demo 影片: https://youtu.be/9PdN7joYmUk
安裝: `curl -fsSL https://raw.githubusercontent.com/minchenlee/c9watch/main/install.sh | bash`

有在用 Claude Code 的人可以試試看，有任何 bug 或建議歡迎回報！

---

## Dcard — 軟體工程師版 / AI工具交誼廳

**標題：** 自己做了個 macOS app 來監控 Claude Code sessions，因為一直切 terminal 切到煩了

**內文：**
最近同時在跑很多個 Claude Code session，一直在不同 terminal tab 之間切換，找哪個 agent 在等我、哪個已經跑完。

本來想用現有的工具，但它們都要從它們自己的 app 裡面啟動 session，我只是想繼續用 Zed 跟 Ghostty 而已，不想改工作流程。

所以就自己做了 [c9watch](https://github.com/minchenlee/c9watch)，一個 macOS menu bar app，直接掃 OS 層的 process，不管你從哪個 terminal 開的 session 都能自動抓到，不用裝任何 plugin。

**幾個功能：**
- 依狀態或依專案分群顯示所有 session
- 點開 session 可以看完整對話記錄（有 code block 格式）
- 可以直接停止 session、改名、或跳回原本的 terminal
- 有 agent 需要你 approve 時會跳原生 macOS 通知
- 掃 QR code 可以在手機上看 session 狀態

用 Tauri（Rust + Svelte）做的，不是 Electron，很輕量。MIT 授權，免費，開源，沒有 telemetry。

GitHub: https://github.com/minchenlee/c9watch
Demo: https://youtu.be/9PdN7joYmUk

有在用 Claude Code 的歡迎試試看，有任何建議都可以留言！

---

## GitHub PR — awesome-claude-code

**PR Title:** Add c9watch – native macOS tray app for monitoring Claude Code sessions

**PR Body:**
Adding c9watch to the list.

**c9watch** is a native macOS menu bar app (Tauri/Rust/Svelte) that monitors all running Claude Code sessions in real-time. It reads from `~/.claude/` and scans OS-level processes to auto-discover sessions regardless of which terminal they were launched from.

- Real-time session status grouped by status or project (with git branch info)
- Conversation viewer — expand any session to read full history with formatted code
- Session controls — stop, rename, open parent terminal/IDE
- Native macOS notifications + WebSocket-based mobile/web client (QR code access)
- Zero-integration — no plugins, works with any terminal
- MIT license, no telemetry

GitHub: https://github.com/minchenlee/c9watch

---

## GitHub PR — awesome-tauri

**PR Title:** Add c9watch

**List entry to add (Developer tools section, between `claws` and `CrabNebula DevTools`):**
```
- [c9watch](https://github.com/minchenlee/c9watch) - macOS menu bar app that monitors all running Claude Code sessions in real-time.
```

**PR Body:**
Adding c9watch to the Developer tools section.

Checklist:
- Original app, not too simple (real-time session monitoring, conversation viewer, session controls, mobile client)
- README is in English
- Built with Tauri 2.x (Rust + Svelte 5)
- Lightweight by design — Rust backend, no Electron

---

## X (Twitter) — @linhn95217033

### Post 1 — Launch/intro thread (post Tuesday–Wednesday 9–11pm TW time)

**Tweet 1 (hook):**
I was running 10+ Claude Code sessions across Zed and Ghostty and kept switching tabs just to check which one was waiting for permission.

So I built a macOS menu bar app that picks them all up automatically.

[attach screenshot of dashboard or tray popover]

**Tweet 2:**
It scans running processes at the OS level — doesn't matter which terminal you launched from. VS Code, Zed, iTerm2, tmux, Ghostty, anything. No plugins, no setup, no changing how you work.

**Tweet 3:**
Beyond status monitoring:
- expand any session to read the full conversation
- stop, rename, or jump to the parent terminal from the dashboard
- native macOS notifications when something needs your attention
- QR code to check in from your phone

**Tweet 4 (link):**
Built with Tauri + Rust + Svelte, not Electron. MIT license, no telemetry.

GitHub: https://github.com/minchenlee/c9watch
Demo: https://youtu.be/9PdN7joYmUk

---

## Threads — @lee_min_ch

### Post 2 — Popover update (post Saturday 12–1pm)

最近做了個 menu bar popover 功能，因為每次要確認 session 狀態都要先開 dashboard，有點多餘。現在點一下圖示就能看到所有 session，想跳到哪個直接點，不用切 app 不用切視窗。

c9watch 是之前做的一個 macOS menu bar app，會自動掃到你機器上所有跑著的 Claude Code session，不管你從哪個 terminal 開的都能抓到。開源，github.com/minchenlee/c9watch

[attach popover screenshot]
