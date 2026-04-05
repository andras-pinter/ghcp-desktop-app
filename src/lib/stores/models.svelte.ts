/** Reactive models state using Svelte 5 runes. */

import { getModels as getModelsCmd } from "$lib/utils/commands";
import type { Model } from "$lib/types/message";

/** Model IDs that are not chat-capable (embeddings, legacy completions). */
const NON_CHAT_PREFIXES = ["text-embedding", "gpt-3.5-turbo-0613"];

function isChatModel(model: Model): boolean {
  return !NON_CHAT_PREFIXES.some((prefix) => model.id.startsWith(prefix));
}

let models = $state<Model[]>([]);
let loaded = $state(false);

/** Fetch models from the backend — call once after auth. */
export async function initModels(): Promise<void> {
  try {
    const all = await getModelsCmd();
    models = all.filter(isChatModel);
  } catch (e) {
    console.error("Failed to fetch models:", e);
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
