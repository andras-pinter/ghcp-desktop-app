/** A persisted chat conversation (mirrors Rust Conversation). */
export interface Conversation {
  id: string;
  title: string | null;
  agentId: string | null;
  projectId: string | null;
  model: string | null;
  isFavourite: boolean;
  /** ISO 8601 timestamp. */
  createdAt: string;
  /** ISO 8601 timestamp. */
  updatedAt: string;
}
