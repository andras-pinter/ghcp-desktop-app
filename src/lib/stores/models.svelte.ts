/** Reactive models state using Svelte 5 runes. */

import { getModels as getModelsCmd, logFrontend } from "$lib/utils/commands";
import type { Model } from "$lib/types/message";

/** Model IDs that are not chat-capable (embeddings, legacy completions). */
const NON_CHAT_PREFIXES = ["text-embedding", "gpt-3.5-turbo-0613"];

function isChatModel(model: Model): boolean {
  return !NON_CHAT_PREFIXES.some((prefix) => model.id.startsWith(prefix));
}

/** Deduplicate models by id — keeps the first occurrence. */
function deduplicateModels(list: Model[]): Model[] {
  const seen = new Set<string>();
  return list.filter((m) => {
    if (seen.has(m.id)) return false;
    seen.add(m.id);
    return true;
  });
}

let models = $state<Model[]>([]);
let loaded = $state(false);

/** Fetch models from the backend — call once after auth. */
export async function initModels(): Promise<void> {
  try {
    const all = await getModelsCmd();
    models = deduplicateModels(all.filter(isChatModel));
  } catch (e) {
    logFrontend("error", `initModels failed: ${e}`);
    models = [];
  } finally {
    loaded = true;
  }
}

/** Reactive getters. */
export function getModelStore() {
  return {
    get models() {
      return models;
    },
    get loaded() {
      return loaded;
    },
  };
}
