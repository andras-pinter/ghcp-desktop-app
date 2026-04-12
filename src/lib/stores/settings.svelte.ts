/** Reactive settings store using Svelte 5 runes. */

import { getSetting, updateSetting as updateSettingCmd } from "$lib/utils/commands";

// ── Setting keys ────────────────────────────────────────────────

export type ThemeMode = "system" | "light" | "dark";
export type SendShortcut = "enter" | "cmd-enter";
export type UpdateFrequency = "startup" | "daily" | "weekly";

/** All user-configurable setting keys. */
export const SETTING_KEYS = {
  theme: "theme",
  fontSize: "font_size",
  sendShortcut: "send_shortcut",
  globalHotkey: "global_hotkey",
  autoUpdateEnabled: "auto_update_enabled",
  autoUpdateFrequency: "auto_update_frequency",
  skippedVersion: "skipped_version",
  updateSnoozedUntil: "update_snoozed_until",
  aitmplEnabled: "aitmpl_enabled",
  defaultAgentId: "default_agent_id",
} as const;

// ── Reactive state ──────────────────────────────────────────────

let theme = $state<ThemeMode>("system");
let fontSize = $state(14);
let sendShortcut = $state<SendShortcut>("enter");
let globalHotkey = $state("CommandOrControl+Shift+Space");
let autoUpdateEnabled = $state(true);
let autoUpdateFrequency = $state<UpdateFrequency>("startup");
let skippedVersion = $state<string | null>(null);
let updateSnoozedUntil = $state<string | null>(null);
let aitmplEnabled = $state(true);
let defaultAgentId = $state<string>("default");
let loaded = $state(false);

// ── Theme application ───────────────────────────────────────────

function applyTheme(mode: ThemeMode): void {
  document.documentElement.dataset.theme = mode;
}

function applyFontSize(size: number): void {
  document.documentElement.style.setProperty("--font-size-base", `${size}px`);
  document.documentElement.style.setProperty("--font-size-sm", `${size - 1}px`);
  document.documentElement.style.setProperty("--font-size-xs", `${size - 2}px`);
  document.documentElement.style.setProperty("--font-size-lg", `${size + 2}px`);
}

// ── Public API ──────────────────────────────────────────────────

/** Load all settings from the backend. Call once on app startup. */
export async function initSettings(): Promise<void> {
  const [
    themeVal,
    fontVal,
    sendVal,
    hotkeyVal,
    autoUpdateVal,
    autoFreqVal,
    skippedVal,
    snoozedVal,
    aitmplVal,
    defaultAgentVal,
  ] = await Promise.all([
    getSetting(SETTING_KEYS.theme),
    getSetting(SETTING_KEYS.fontSize),
    getSetting(SETTING_KEYS.sendShortcut),
    getSetting(SETTING_KEYS.globalHotkey),
    getSetting(SETTING_KEYS.autoUpdateEnabled),
    getSetting(SETTING_KEYS.autoUpdateFrequency),
    getSetting(SETTING_KEYS.skippedVersion),
    getSetting(SETTING_KEYS.updateSnoozedUntil),
    getSetting(SETTING_KEYS.aitmplEnabled),
    getSetting(SETTING_KEYS.defaultAgentId),
  ]);

  if (themeVal) theme = themeVal as ThemeMode;
  if (fontVal) fontSize = parseInt(fontVal, 10) || 14;
  if (sendVal) sendShortcut = sendVal as SendShortcut;
  if (hotkeyVal) globalHotkey = hotkeyVal;
  if (autoUpdateVal) autoUpdateEnabled = autoUpdateVal === "true";
  if (autoFreqVal) autoUpdateFrequency = autoFreqVal as UpdateFrequency;
  skippedVersion = skippedVal ?? null;
  updateSnoozedUntil = snoozedVal ?? null;
  aitmplEnabled = aitmplVal !== "false";
  if (defaultAgentVal) defaultAgentId = defaultAgentVal;

  applyTheme(theme);
  applyFontSize(fontSize);
  loaded = true;
}

/** Update a setting value. Persists to backend and updates local state. */
export async function updateSetting(key: string, value: string): Promise<void> {
  await updateSettingCmd(key, value);

  switch (key) {
    case SETTING_KEYS.theme:
      theme = value as ThemeMode;
      applyTheme(theme);
      break;
    case SETTING_KEYS.fontSize:
      fontSize = parseInt(value, 10) || 14;
      applyFontSize(fontSize);
      break;
    case SETTING_KEYS.sendShortcut:
      sendShortcut = value as SendShortcut;
      break;
    case SETTING_KEYS.globalHotkey:
      globalHotkey = value;
      break;
    case SETTING_KEYS.autoUpdateEnabled:
      autoUpdateEnabled = value === "true";
      break;
    case SETTING_KEYS.autoUpdateFrequency:
      autoUpdateFrequency = value as UpdateFrequency;
      break;
    case SETTING_KEYS.skippedVersion:
      skippedVersion = value || null;
      break;
    case SETTING_KEYS.updateSnoozedUntil:
      updateSnoozedUntil = value || null;
      break;
    case SETTING_KEYS.aitmplEnabled:
      aitmplEnabled = value !== "false";
      break;
    case SETTING_KEYS.defaultAgentId:
      defaultAgentId = value || "default";
      break;
  }
}

/** Reactive getters for all settings. */
export function getSettings() {
  return {
    get theme() {
      return theme;
    },
    get fontSize() {
      return fontSize;
    },
    get sendShortcut() {
      return sendShortcut;
    },
    get globalHotkey() {
      return globalHotkey;
    },
    get autoUpdateEnabled() {
      return autoUpdateEnabled;
    },
    get autoUpdateFrequency() {
      return autoUpdateFrequency;
    },
    get skippedVersion() {
      return skippedVersion;
    },
    get updateSnoozedUntil() {
      return updateSnoozedUntil;
    },
    get aitmplEnabled() {
      return aitmplEnabled;
    },
    get defaultAgentId() {
      return defaultAgentId;
    },
    get loaded() {
      return loaded;
    },
  };
}
