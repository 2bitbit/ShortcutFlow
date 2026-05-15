import { Store, load } from '@tauri-apps/plugin-store';
import { ref } from 'vue';

let _store: Store | null = null;

import { invoke } from '@tauri-apps/api/core';

async function getStore() {
  if (!_store) {
    try {
      const baseDir = await invoke<string>('get_root_dir');
      const sanitized = baseDir.replace(/\\\\/g, '/');
      _store = await load(`${sanitized}/ui_settings.json`);
    } catch (e) {
      console.warn("Failed to get base dir, falling back to standard config", e);
      _store = await load('ui_settings.json');
    }
  }
  return _store;
}

export async function getCollapsedGroups(viewId: string): Promise<string[]> {
  try {
    const store = await getStore();
    const saved = await store.get<string[]>(`collapsed_${viewId}`);
    return saved || [];
  } catch (e) {
    console.warn("Failed to load collapsed groups:", e);
    return [];
  }
}

export async function saveCollapsedGroups(viewId: string, groups: string[]) {
  try {
    const store = await getStore();
    await store.set(`collapsed_${viewId}`, groups);
    await store.save();
  } catch (e) {
    console.warn("Failed to save collapsed groups:", e);
  }
}

export const runInBackgroundCache = ref(true);

export async function getRunInBackground(): Promise<boolean> {
  try {
    const store = await getStore();
    const v = await store.get<boolean>('run_in_background');
    runInBackgroundCache.value = v ?? true;
    return runInBackgroundCache.value;
  } catch (e) {
    console.warn("Failed to load run_in_background setting:", e);
    runInBackgroundCache.value = true;
    return true;
  }
}

export async function setRunInBackground(value: boolean) {
  try {
    runInBackgroundCache.value = value;
    const store = await getStore();
    await store.set('run_in_background', value);
    await store.save();
  } catch (e) {
    console.warn("Failed to save run_in_background setting:", e);
  }
}

// --- 轻量模式 ---

export const lightweightAutoCache = ref(false);
export const lightweightDelayCache = ref(30);

export async function getLightweightAuto(): Promise<boolean> {
  try {
    const store = await getStore();
    const v = await store.get<boolean>('lightweight_auto');
    lightweightAutoCache.value = v ?? false;
    return lightweightAutoCache.value;
  } catch {
    return false;
  }
}

export async function setLightweightAuto(value: boolean) {
  lightweightAutoCache.value = value;
  const store = await getStore();
  await store.set('lightweight_auto', value);
  await store.save();
}

export async function getLightweightDelay(): Promise<number> {
  try {
    const store = await getStore();
    const v = await store.get<number>('lightweight_delay');
    lightweightDelayCache.value = v ?? 30;
    return lightweightDelayCache.value;
  } catch {
    return 30;
  }
}

export async function setLightweightDelay(value: number) {
  lightweightDelayCache.value = value;
  const store = await getStore();
  await store.set('lightweight_delay', value);
  await store.save();
}

// --- 开机自启时最小化 ---

export const minimizeOnStartupCache = ref(true);

export async function getMinimizeOnStartup(): Promise<boolean> {
  try {
    const store = await getStore();
    const v = await store.get<boolean>('minimize_on_startup');
    minimizeOnStartupCache.value = v ?? true;
    return minimizeOnStartupCache.value;
  } catch {
    return true;
  }
}

export async function setMinimizeOnStartup(value: boolean) {
  minimizeOnStartupCache.value = value;
  const store = await getStore();
  await store.set('minimize_on_startup', value);
  await store.save();
}
