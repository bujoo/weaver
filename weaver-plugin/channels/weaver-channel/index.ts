#!/usr/bin/env bun
/**
 * Weaver Channel Server
 *
 * Two-way channel between Weaver (ContextHub orchestration) and Claude Code.
 * - Weaver POSTs assignments/commands to localhost:8789
 * - Channel pushes them into Claude Code as <channel source="weaver"> events
 * - Claude replies via the reply tool -> forwarded to Weaver API
 * - Permission relay: Weaver can approve/deny tool calls remotely
 */

import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  ListToolsRequestSchema,
  CallToolRequestSchema,
} from "@modelcontextprotocol/sdk/types.js";
import { z } from "zod";

const WEAVER_CHANNEL_PORT = parseInt(
  process.env.WEAVER_CHANNEL_PORT || "8789"
);
const WEAVER_API_URL =
  process.env.WEAVER_API_URL || "http://localhost:9210";

// --- SSE listeners for dev/debug (GET /events) ---
const listeners = new Set<(chunk: string) => void>();
function broadcast(text: string) {
  const chunk =
    text
      .split("\n")
      .map((l) => `data: ${l}\n`)
      .join("") + "\n";
  for (const emit of listeners) emit(chunk);
}

// --- Forward reply to Weaver API ---
async function sendToWeaver(
  endpoint: string,
  payload: Record<string, unknown>
) {
  try {
    await fetch(`${WEAVER_API_URL}${endpoint}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
    });
  } catch {
    // Weaver API might not be running -- log but don't crash
  }
  broadcast(`[${endpoint}] ${JSON.stringify(payload)}`);
}

// --- MCP Server ---
const mcp = new Server(
  { name: "weaver", version: "1.0.0" },
  {
    capabilities: {
      experimental: {
        "claude/channel": {},
        "claude/channel/permission": {},
      },
      tools: {},
    },
    instructions: `You are connected to Weaver, the ContextHub orchestration system.

Phase assignments arrive as <channel source="weaver" type="assignment" phase_id="..." mission_id="..."> events.
When you receive an assignment:
1. Read CLAUDE.md for mission context
2. Read .weaver/specs/ for todo specs referenced in the assignment
3. Execute each todo following the behavior steps and constraints in the spec
4. After completing each todo, call the weaver_complete_todo tool
5. After all todos in the phase are done, call the weaver_phase_complete tool

Commands arrive as <channel source="weaver" type="command"> events.
Supported commands: abort, skip, pause, context_update.

Always reply through the weaver_reply tool to report status back to Weaver.`,
  }
);

// --- Tools: Claude calls these to communicate back to Weaver ---
mcp.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "weaver_reply",
      description:
        "Send a status message back to Weaver. Use for progress updates, completion reports, and help requests.",
      inputSchema: {
        type: "object" as const,
        properties: {
          mission_id: {
            type: "string",
            description: "The mission ID from the channel event",
          },
          type: {
            type: "string",
            enum: ["progress", "complete", "error", "help"],
            description: "Type of reply",
          },
          message: {
            type: "string",
            description: "Status message or completion summary",
          },
        },
        required: ["mission_id", "type", "message"],
      },
    },
    {
      name: "weaver_complete_todo",
      description:
        "Mark a specific todo as completed. Call this after finishing each todo.",
      inputSchema: {
        type: "object" as const,
        properties: {
          mission_id: {
            type: "string",
            description: "The mission ID",
          },
          todo_id: {
            type: "string",
            description: "The todo ID (e.g., P0.1)",
          },
          summary: {
            type: "string",
            description: "Brief summary of what was done",
          },
          files_modified: {
            type: "array",
            items: { type: "string" },
            description: "List of files that were modified",
          },
        },
        required: ["mission_id", "todo_id", "summary"],
      },
    },
    {
      name: "weaver_phase_complete",
      description:
        "Signal that all todos in the current phase are done. Weaver will assign the next phase.",
      inputSchema: {
        type: "object" as const,
        properties: {
          mission_id: {
            type: "string",
            description: "The mission ID",
          },
          phase_id: {
            type: "string",
            description: "The phase ID (e.g., P0)",
          },
          summary: {
            type: "string",
            description: "Phase completion summary",
          },
        },
        required: ["mission_id", "phase_id", "summary"],
      },
    },
  ],
}));

mcp.setRequestHandler(CallToolRequestSchema, async (req) => {
  const { name, arguments: args } = req.params;
  const typedArgs = args as Record<string, unknown>;

  switch (name) {
    case "weaver_reply":
      await sendToWeaver("/channel/reply", typedArgs);
      return {
        content: [{ type: "text" as const, text: "Reply sent to Weaver." }],
      };

    case "weaver_complete_todo":
      await sendToWeaver("/channel/todo-complete", typedArgs);
      return {
        content: [
          {
            type: "text" as const,
            text: `Todo ${typedArgs.todo_id} marked complete. Continue with the next todo or call weaver_phase_complete if all todos are done.`,
          },
        ],
      };

    case "weaver_phase_complete":
      await sendToWeaver("/channel/phase-complete", typedArgs);
      return {
        content: [
          {
            type: "text" as const,
            text: `Phase ${typedArgs.phase_id} complete. Weaver will assign the next phase.`,
          },
        ],
      };

    default:
      throw new Error(`Unknown tool: ${name}`);
  }
});

// --- Permission relay ---
const PermissionRequestSchema = z.object({
  method: z.literal("notifications/claude/channel/permission_request"),
  params: z.object({
    request_id: z.string(),
    tool_name: z.string(),
    description: z.string(),
    input_preview: z.string(),
  }),
});

mcp.setNotificationHandler(
  PermissionRequestSchema,
  async ({ params }) => {
    await sendToWeaver("/channel/permission-request", params);
    broadcast(
      `Permission: ${params.tool_name} - ${params.description}\n` +
        `Reply "yes ${params.request_id}" or "no ${params.request_id}"`
    );
  }
);

// --- Connect to Claude Code ---
await mcp.connect(new StdioServerTransport());

// --- HTTP server: Weaver POSTs here to push events into Claude Code ---
const PERMISSION_REPLY_RE = /^\s*(y|yes|n|no)\s+([a-km-z]{5})\s*$/i;

Bun.serve({
  port: WEAVER_CHANNEL_PORT,
  hostname: "127.0.0.1",
  idleTimeout: 0,
  async fetch(req) {
    const url = new URL(req.url);

    // GET /events: SSE stream for dev/debug
    if (req.method === "GET" && url.pathname === "/events") {
      const stream = new ReadableStream({
        start(ctrl) {
          ctrl.enqueue(": weaver channel connected\n\n");
          const emit = (chunk: string) => ctrl.enqueue(chunk);
          listeners.add(emit);
          req.signal.addEventListener("abort", () => listeners.delete(emit));
        },
      });
      return new Response(stream, {
        headers: {
          "Content-Type": "text/event-stream",
          "Cache-Control": "no-cache",
        },
      });
    }

    // GET /health
    if (req.method === "GET" && url.pathname === "/health") {
      return new Response(
        JSON.stringify({ status: "ok", port: WEAVER_CHANNEL_PORT }),
        { headers: { "Content-Type": "application/json" } }
      );
    }

    // POST: push event into Claude Code session
    const body = await req.text();

    // Check for permission verdict
    const m = PERMISSION_REPLY_RE.exec(body);
    if (m) {
      await mcp.notification({
        method: "notifications/claude/channel/permission",
        params: {
          request_id: m[2].toLowerCase(),
          behavior: m[1].toLowerCase().startsWith("y") ? "allow" : "deny",
        },
      });
      return new Response("verdict recorded");
    }

    // Parse as JSON if possible
    let content = body;
    let meta: Record<string, string> = { path: url.pathname };

    try {
      const json = JSON.parse(body);
      content = json.content || json.message || body;
      meta = {
        ...meta,
        type: json.type || "message",
        mission_id: json.mission_id || "",
        phase_id: json.phase_id || "",
      };
    } catch {
      // Plain text
    }

    await mcp.notification({
      method: "notifications/claude/channel",
      params: { content, meta },
    });

    return new Response("ok");
  },
});

console.error(
  `[weaver-channel] Listening on http://127.0.0.1:${WEAVER_CHANNEL_PORT}`
);
