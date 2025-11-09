import type { KeysConfig, KeyPair } from "@/types/key";
import { LazyStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { readDir, exists } from "@tauri-apps/plugin-fs";
import { homeDir, join, BaseDirectory } from "@tauri-apps/api/path";

const store: LazyStore = new LazyStore("keys.json");
const SSH_RELATIVE_DIR = ".ssh";
const SSH_PUB_SUFFIX = ".pub";

export const useKeysStore = defineStore("keys", () => {
  const keys = ref<KeysConfig>([]);

  async function load(): Promise<void> {
    const saved = await store.get<KeysConfig>("keys");
    keys.value = saved ?? [];  
  }

  async function save(): Promise<void> {
    await store.set("keys", keys.value);
    await store.save();
  }

  /**
   * Get a key by its ID.
   * @param id - The ID of the key.
   * @returns The key.
   */
  function getKey(id: number): KeyPair | undefined {
    return keys.value.find((k) => k.id === id);
  }

  /**
   * Get all keys.
   * @returns The keys.
   */
  function listKeys(): KeyPair[] {
    return keys.value;
  }

  /**
   * @deprecated Use listKeys instead.
   * Get all keys.
   * @returns A promise that resolves to the keys.
   */
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

  async function updateKeyMetadata(
    id: number,
    data: Partial<Omit<KeyPair, "id">>,
  ): Promise<KeyPair | undefined> {
    await load();
    const index = keys.value.findIndex((k) => k.id === id);
    if (index === -1) return undefined;
    const updated: KeyPair = {
      ...(keys.value[index] as KeyPair),
      ...data,
      id,
    };
    keys.value.splice(index, 1, updated);
    await save();
    return updated;
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

  async function syncWithFs(): Promise<KeyPair[]> {
    await load();

    const home = await homeDir();
    const normalizedHome = home.endsWith("/") ? home : `${home}/`;
    const sshAbs = await join(home, SSH_RELATIVE_DIR);
    const discovered: Array<{ name: string; private: string; public?: string }> = [];

    const entries = await readDir(SSH_RELATIVE_DIR, {
      baseDir: BaseDirectory.Home,
    });

    for (const entry of entries) {
      if (entry.isDirectory) continue;
      const fileName = entry.name ?? "";
      if (!fileName.endsWith(SSH_PUB_SUFFIX)) continue;
      const stem = fileName.slice(0, -SSH_PUB_SUFFIX.length);
      if (!stem || stem === "config" || stem.startsWith("known_hosts")) continue;

      const privateRel = `${SSH_RELATIVE_DIR}/${stem}`;
      const privateExists = await exists(privateRel, {
        baseDir: BaseDirectory.Home,
      });
      if (!privateExists) continue;

      const privatePath = await join(sshAbs, stem);
      const publicPath = await join(sshAbs, fileName);
      discovered.push({ name: stem, private: privatePath, public: publicPath });
    }

    const discoveredPrivates = new Set(discovered.map((k) => k.private));
    const byPrivate = new Map<string, KeyPair>(
      keys.value.map((k) => [k.private, k as KeyPair]),
    );

    for (const info of discovered) {
      const existing = byPrivate.get(info.private);
      if (existing) {
        if (existing.name !== info.name || existing.public !== info.public) {
          const index = keys.value.findIndex((k) => k.id === existing.id);
          if (index !== -1) {
            keys.value.splice(index, 1, {
              ...(keys.value[index] as KeyPair),
              name: info.name,
              public: info.public,
            });
          }
        }
      } else {
        const id = generateId();
        const created: KeyPair = {
          id,
          name: info.name,
          private: info.private,
          public: info.public,
        };
        keys.value.push(created);
      }
    }

    const kept: KeyPair[] = [];
    for (const key of keys.value) {
      if (discoveredPrivates.has(key.private)) {
        kept.push(key as KeyPair);
        continue;
      }

      const relativePrivate = key.private.startsWith(normalizedHome)
        ? key.private.slice(normalizedHome.length)
        : null;

      let existsOnDisk = false;
      try {
        if (relativePrivate) {
          existsOnDisk = await exists(relativePrivate, {
            baseDir: BaseDirectory.Home,
          });
        } else {
          existsOnDisk = await exists(key.private);
        }
      } catch {
        existsOnDisk = false;
      }

      if (!existsOnDisk && key.public) {
        const relativePublic = key.public.startsWith(normalizedHome)
          ? key.public.slice(normalizedHome.length)
          : null;
        try {
          if (relativePublic) {
            existsOnDisk = await exists(relativePublic, {
              baseDir: BaseDirectory.Home,
            });
          } else if (key.public) {
            existsOnDisk = await exists(key.public);
          }
        } catch {
          // ignore
        }
      }

      if (existsOnDisk) {
        kept.push(key as KeyPair);
      }
    }

    if (kept.length !== keys.value.length) {
      keys.value = kept;
    }

    await save();
    return keys.value;
  }

  function generateId(): number {
    const maxId = keys.value.reduce((m, k) => Math.max(m, k.id), 0);
    return maxId + 1;
  }

  const byId = computed(() => new Map(keys.value.map((k) => [k.id, k])));

  return {
    load,
    getKeys,
    addOrUpdateKey,
    removeKey,
    byId,
    getKey,
    listKeys,
    updateKeyMetadata,
    syncWithFs,
  };
});
