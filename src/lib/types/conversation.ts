export interface Conversation {
  id: string;
  title: string | null;
  agentId: string | null;
  projectId: string | null;
  model: string | null;
  isFavourite: boolean;
  createdAt: string;
  updatedAt: string;
}
