/** Reactive models state using Svelte 5 runes. */

import {
  getModels as getModelsCmd,
  getSetting,
  updateSetting,
  logFrontend,
} from "$lib/utils/commands";
import type { Model } from "$lib/types/message";

const DEFAULT_MODEL_KEY = "default_model";

/** Model IDs that are not chat-capable (embeddings, legacy completions). */
const NON_CHAT_PREFIXES = ["text-embedding", "gpt-3.5-turbo-0613"];

function isChatModel(model: Model): boolean {
  return !NON_CHAT_PREFIXES.some((prefix) => model.id.startsWith(prefix));
}

/** Deduplicate models by id — keeps the first occurrence. */
function deduplicateModels(list: Model[]): Model[] {
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- local non-reactive set used only for dedup
  const seen = new Set<string>();
  return list.filter((m) => {
    if (seen.has(m.id)) return false;
    seen.add(m.id);
    return true;
  });
}

let models = $state<Model[]>([]);
let loaded = $state(false);
let defaultModelId = $state<string | null>(null);

/** Fetch models from the backend — call once after auth. */
export async function initModels(): Promise<void> {
  try {
    const [all, savedDefault] = await Promise.all([getModelsCmd(), getSetting(DEFAULT_MODEL_KEY)]);
    models = deduplicateModels(all.filter(isChatModel));
    defaultModelId = savedDefault;
  } catch (e) {
    logFrontend("error", `initModels failed: ${e}`);
    models = [];
  } finally {
    loaded = true;
  }
}

/** Persist a model as the default. */
export async function setDefaultModel(modelId: string): Promise<void> {
  defaultModelId = modelId;
  await updateSetting(DEFAULT_MODEL_KEY, modelId);
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
    get defaultModelId() {
      return defaultModelId;
    },
  };
}
