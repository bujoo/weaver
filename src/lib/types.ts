/**
 * TypeScript type definitions for c9watch
 */

/**
 * Session status enumeration
 */
export enum SessionStatus {
  Working = 'Working',              // Executing tools/thinking
  NeedsPermission = 'NeedsPermission', // Waiting for user approval
  WaitingForInput = 'WaitingForInput', // Idle, ready for prompt
  Connecting = 'Connecting'            // Session starting up
}

/**
 * A Claude Code session
 */
export interface Session {
  /** Session UUID */
  id: string;

  /** Process ID of the running Claude instance */
  pid: number;

  /** Custom session name (defaults to project directory name) - shown as small badge */
  sessionName: string;

  /** Custom title override for the session - if set, shown instead of summary/firstPrompt */
  customTitle: string | null;

  /** Full path to project directory */
  projectPath: string;

  /** Git branch name (if available) */
  gitBranch: string | null;

  /** Summary of the first prompt (shown in list view) */
  firstPrompt: string;

  /** AI-generated summary of the session (from sessions-index.json) */
  summary: string | null;

  /** Total number of messages in the conversation */
  messageCount: number;

  /** Timestamp of last activity (ISO 8601 string) */
  modified: string;

  /** Current status of the session */
  status: SessionStatus;

  /** Content of the latest message */
  latestMessage: string;

  /** Name of the tool currently awaiting user permission (if status is NeedsPermission) */
  pendingToolName: string | null;
}

/**
 * Message type in conversation
 */
export type MessageType = 'User' | 'Assistant' | 'Thinking' | 'ToolUse' | 'ToolResult';

/**
 * A message in a conversation
 */
export interface Message {
  /** Message timestamp (ISO 8601 string) */
  timestamp: string;

  /** Message type */
  messageType: MessageType;

  /** Message content text */
  content: string;
}

/**
 * A conversation containing all messages for a session
 */
export interface Conversation {
  /** Session ID this conversation belongs to */
  sessionId: string;

  /** Array of messages in chronological order */
  messages: Message[];
}

/**
 * A single result from the deep search command.
 * Contains the session ID and a short snippet from the matching message.
 */
export interface DeepSearchHit {
  sessionId: string;
  /** ~200-char snippet from the first matching message line, with '…' padding if truncated. */
  snippet: string;
}

/**
 * A single entry from ~/.claude/history.jsonl (deduplicated by sessionId)
 */
export interface HistoryEntry {
  /** Session UUID */
  sessionId: string;

  /** The user's prompt text as displayed in Claude Code. May be an empty string. */
  display: string;

  /**
   * Timestamp in milliseconds since epoch (raw integer from history.jsonl).
   * Note: unlike other timestamps in this file which are ISO 8601 strings,
   * this is a Unix-ms number matching the raw format Claude Code writes.
   */
  timestamp: number;

  /** Full project path, e.g. /Users/you/Documents/GitHub/myproject */
  project: string;

  /** Last path segment of project, e.g. "myproject" */
  projectName: string;
}
