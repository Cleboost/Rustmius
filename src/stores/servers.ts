import { Folder, Server, ServerConfig } from "@/types/server";
import { LazyStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { homeDir, BaseDirectory } from "@tauri-apps/api/path";
import { useKeysStore } from "@/stores/keys";

const store: LazyStore = new LazyStore("servers.json");
export const useServerConfigStore = defineStore("serversconfig", () => {
  const tree = ref<ServerConfig>([]);
  load();

  function isFolder(entry: Folder | Server): entry is Folder {
    return (entry as Folder).contents !== undefined;
  }

  function findServerById(
    entries: ServerConfig,
    id: Server["id"],
  ): Server | undefined {
    for (const entry of entries) {
      if (isFolder(entry)) {
        const found = findServerById(entry.contents, id);
        if (found) return found;
      } else if (entry.id === id) {
        return entry;
      }
    }
    return undefined;
  }

  async function load(): Promise<void> {
    const saved = await store.get<ServerConfig>("servers");
    tree.value = saved ?? [];
  }

  async function save(): Promise<void> {
    await store.set("servers", tree.value);
    await store.save();
  }

  function ensurePath(path: string): ServerConfig {
    const parts = path.split("/").filter(Boolean);
    let current = tree.value;
    for (const name of parts) {
      let folder = current.find((e) => isFolder(e) && e.name === name) as
        | Folder
        | undefined;
      if (!folder) {
        folder = { id: crypto.randomUUID(), name, contents: [] };
        current.push(folder);
      }
      current = folder.contents;
    }
    return current;
  }

  async function addServer(server: Server, path: string = "/"): Promise<void> {
    await load();
    const target = path === "/" ? tree.value : ensurePath(path);
    target.push(server);
    await save();
  }

  function removeEntryById(entries: ServerConfig, id: Server["id"]): boolean {
    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i];
      if (isFolder(entry)) {
        if (removeEntryById(entry.contents, id)) return true;
      } else if (entry.id === id) {
        entries.splice(i, 1);
        return true;
      }
    }
    return false;
  }

  async function removeServer(id: Server["id"]): Promise<boolean> {
    await load();
    if (removeEntryById(tree.value, id)) {
      await save();
      return true;
    }
    return false;
  }

  function updateEntryById(
    entries: ServerConfig,
    id: Server["id"],
    server: Server,
  ): boolean {
    for (let i = 0; i < entries.length; i++) {
      const entry = entries[i];
      if (isFolder(entry)) {
        if (updateEntryById(entry.contents, id, server)) return true;
      } else if (entry.id === id) {
        entries[i] = server;
        return true;
      }
    }
    return false;
  }

  async function updateServer(id: Server["id"], server: Server): Promise<void> {
    await load();
    if (updateEntryById(tree.value, id, server)) {
      await save();
    }
  }

  function getServer(id: Server["id"]): Server | undefined {
    const search = (entries: ServerConfig): Server | undefined => {
      for (const entry of entries) {
        if (isFolder(entry)) {
          const found = search(entry.contents);
          if (found) return found;
        } else if (entry.id === id) return entry;
      }
      return undefined;
    };
    return search(tree.value);
  }

  function listServers(): Server[] {
    const servers: Server[] = [];
    const traverse = (entries: ServerConfig) => {
      for (const entry of entries) {
        if (isFolder(entry)) {
          traverse(entry.contents);
        } else {
          servers.push(entry);
        }
      }
    };
    traverse(tree.value);
    return servers;
  }

  function serverExists(id: Server["id"]): boolean {
    return findServerById(tree.value, id) !== undefined;
  }

  return {
    load,
    getServer,
    listServers,
    addServer,
    removeServer,
    updateServer,
    serverExists,
  };
});
