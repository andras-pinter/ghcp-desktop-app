export type MessageRole = "user" | "assistant" | "system" | "tool";

export interface MessageAttachment {
  name: string;
  type: string;
  size: number;
}

export interface Message {
  id: string;
  conversationId: string;
  role: MessageRole;
  content: string;
  thinkingContent?: string | null;
  tool_call_id?: string | null;
  tool_name?: string | null;
  attachments?: MessageAttachment[] | null;
  createdAt: string;
  sortOrder: number;
}

/** A chat message in the API format (sent to Copilot). */
export interface ChatMessage {
  role: MessageRole;
  content: string;
  name?: string | null;
  tool_call_id?: string | null;
}

/** A Copilot model descriptor. */
export interface Model {
  id: string;
  name?: string | null;
  version?: string | null;
}
