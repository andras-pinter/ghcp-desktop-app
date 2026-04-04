export interface Project {
  id: string;
  name: string;
  instructions: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface ProjectFile {
  id: string;
  projectId: string;
  name: string;
  contentType: string;
  createdAt: string;
}
