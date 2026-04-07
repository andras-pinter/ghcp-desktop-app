/** Reactive project state using Svelte 5 runes. */

/* eslint-disable svelte/prefer-svelte-reactivity -- Date is used imperatively for ISO timestamps, not reactively */

import type { Project, ProjectFile, FileUpload } from "$lib/types/project";
import type { Conversation } from "$lib/types/conversation";
import {
  getProjects,
  createProject as createProjectCmd,
  updateProject as updateProjectCmd,
  deleteProject as deleteProjectCmd,
  getProjectFiles,
  addProjectFile as addProjectFileCmd,
  removeProjectFile as removeProjectFileCmd,
  getProjectConversations,
} from "$lib/utils/commands";

let projects = $state<Project[]>([]);
let activeProjectId = $state<string | null>(null);
let activeProjectFiles = $state<ProjectFile[]>([]);
let activeProjectConversations = $state<Conversation[]>([]);
let loading = $state(false);

// ── Initialization ──────────────────────────────────────────────

/** Load the project list from the backend. Call once on app startup. */
export async function initProjects(): Promise<void> {
  loading = true;
  try {
    projects = await getProjects();
  } catch (e) {
    console.error("Failed to load projects:", e);
    projects = [];
  } finally {
    loading = false;
  }
}

// ── Project CRUD ────────────────────────────────────────────────

/** Create a new project and make it active. */
export async function newProject(name: string, instructions?: string | null): Promise<Project> {
  const id = crypto.randomUUID();
  const proj = await createProjectCmd(id, name, instructions);
  projects = [proj, ...projects];
  await selectProject(proj.id);
  return proj;
}

/** Select a project — loads its files and conversations. */
export async function selectProject(id: string): Promise<void> {
  activeProjectId = id;
  try {
    const [files, convos] = await Promise.all([getProjectFiles(id), getProjectConversations(id)]);
    activeProjectFiles = files;
    activeProjectConversations = convos;
  } catch (e) {
    console.error("Failed to load project details:", e);
    activeProjectFiles = [];
    activeProjectConversations = [];
  }
}

/** Clear active project. */
export function clearActiveProject(): void {
  activeProjectId = null;
  activeProjectFiles = [];
  activeProjectConversations = [];
}

/** Rename a project. */
export async function renameProject(id: string, name: string): Promise<void> {
  await updateProjectCmd(id, name);
  projects = projects.map((p) =>
    p.id === id ? { ...p, name, updatedAt: new Date().toISOString() } : p,
  );
}

/** Update a project's instructions. */
export async function updateProjectInstructions(
  id: string,
  instructions: string | null,
): Promise<void> {
  await updateProjectCmd(id, null, instructions);
  projects = projects.map((p) =>
    p.id === id ? { ...p, instructions, updatedAt: new Date().toISOString() } : p,
  );
}

/** Delete a project. */
export async function removeProject(id: string): Promise<void> {
  await deleteProjectCmd(id);
  projects = projects.filter((p) => p.id !== id);
  if (activeProjectId === id) {
    clearActiveProject();
  }
}

// ── File management ─────────────────────────────────────────────

/** Upload a file to the active project. */
export async function uploadProjectFile(file: FileUpload): Promise<ProjectFile> {
  if (!activeProjectId) throw new Error("No active project");
  const pf = await addProjectFileCmd(activeProjectId, file);
  activeProjectFiles = [pf, ...activeProjectFiles];
  // Update file count
  projects = projects.map((p) =>
    p.id === activeProjectId ? { ...p, fileCount: p.fileCount + 1 } : p,
  );
  return pf;
}

/** Remove a file from the active project. */
export async function deleteProjectFile(fileId: string): Promise<void> {
  await removeProjectFileCmd(fileId);
  activeProjectFiles = activeProjectFiles.filter((f) => f.id !== fileId);
  projects = projects.map((p) =>
    p.id === activeProjectId ? { ...p, fileCount: Math.max(0, p.fileCount - 1) } : p,
  );
}

/** Reload the active project's files. */
export async function refreshProjectFiles(): Promise<void> {
  if (!activeProjectId) return;
  activeProjectFiles = await getProjectFiles(activeProjectId);
}

/** Reload the active project's conversations. */
export async function refreshProjectConversations(): Promise<void> {
  if (!activeProjectId) return;
  activeProjectConversations = await getProjectConversations(activeProjectId);
}

// ── Reactive getters ────────────────────────────────────────────

export function getProjectStore() {
  return {
    get projects() {
      return projects;
    },
    get activeProjectId() {
      return activeProjectId;
    },
    get activeProject(): Project | null {
      if (!activeProjectId) return null;
      return projects.find((p) => p.id === activeProjectId) ?? null;
    },
    get files() {
      return activeProjectFiles;
    },
    get conversations() {
      return activeProjectConversations;
    },
    get loading() {
      return loading;
    },
    get hasProjects() {
      return projects.length > 0;
    },
  };
}
