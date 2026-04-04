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
  toolCallId?: string | null;
  toolName?: string | null;
  attachments?: MessageAttachment[] | null;
  createdAt: string;
  sortOrder: number;
}
