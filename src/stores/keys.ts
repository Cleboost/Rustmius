import type { KeysConfig, KeyPair } from "@/types/key";
import { LazyStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { ref, computed } from "vue";

const store: LazyStore = new LazyStore("keys.json");

export const useKeysStore = defineStore("keys", () => {
  const keys = ref<KeysConfig>([]);
  let loaded = false;

  async function load(): Promise<void> {
    if (loaded) return;
    const saved = await store.get<KeysConfig>("keys");
    keys.value = saved ?? [];
    loaded = true;
  }

  async function save(): Promise<void> {
    await store.set("keys", keys.value);
    await store.save();
  }

  async function getKeys(): Promise<KeysConfig> {
    await load();
    return keys.value;
  }

  async function addOrUpdateKey(newKey: Omit<KeyPair, "id"> & { id?: number }): Promise<KeyPair> {
    await load();
    let existingIndex = -1;
    if (newKey.id != null) {
      existingIndex = keys.value.findIndex((k) => k.id === newKey.id);
    }
    if (existingIndex === -1) {
      const id = generateId();
      const created: KeyPair = { id, ...newKey } as KeyPair;
      keys.value.push(created);
      await save();
      return created;
    } else {
      const updated: KeyPair = { ...(keys.value[existingIndex] as KeyPair), ...newKey, id: keys.value[existingIndex]!.id };
      keys.value.splice(existingIndex, 1, updated);
      await save();
      return updated;
    }
  }

  async function removeKey(id: number): Promise<boolean> {
    await load();
    const index = keys.value.findIndex((k) => k.id === id);
    if (index !== -1) {
      keys.value.splice(index, 1);
      await save();
      return true;
    }
    return false;
  }

  function generateId(): number {
    const maxId = keys.value.reduce((m, k) => Math.max(m, k.id), 0);
    return maxId + 1;
  }

  const byId = computed(() => new Map(keys.value.map((k) => [k.id, k])));

  return { load, getKeys, addOrUpdateKey, removeKey, byId };
});
