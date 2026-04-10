/** A named project that groups conversations and pinned files (mirrors Rust Project). */
export interface Project {
  id: string;
  name: string;
  instructions: string | null;
  fileCount: number;
  createdAt: string;
  updatedAt: string;
}

/** Metadata for a file pinned inside a project. */
export interface ProjectFile {
  id: string;
  projectId: string;
  name: string;
  contentType: string;
  size: number;
  createdAt: string;
}

/** Payload for uploading a file to a project (base64-encoded content). */
export interface FileUpload {
  name: string;
  contentType: string;
  contentBase64: string;
}

/** File data returned from Rust after reading a file from disk. */
export interface ChatFileData {
  name: string;
  contentType: string;
  size: number;
  contentBase64: string;
  /** True while content is still being read from disk. */
  loading?: boolean;
}
