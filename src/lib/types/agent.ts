export interface Agent {
  id: string;
  name: string;
  avatar: string | null;
  systemPrompt: string;
  isDefault: boolean;
  createdAt: string;
  updatedAt: string;
}
