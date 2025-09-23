import { Folder, Server, ServerConfig } from "@/types/server";
import { LazyStore } from "@tauri-apps/plugin-store";
import { defineStore } from "pinia";
import { computed, ref } from "vue";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { homeDir, BaseDirectory } from "@tauri-apps/api/path";
import { useKeysStore } from "@/stores/keys";

const store: LazyStore = new LazyStore("servers.json");
export const useServersStore = defineStore("servers", () => {
  const tree = ref<ServerConfig>([]);
  let loaded = false;

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
    if (loaded) return;
    const saved = await store.get<ServerConfig>("servers");
    tree.value = saved ?? [];
    loaded = true;
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

  async function getServer(id: Server["id"]): Promise<Server | undefined> {
    await load();
    return findServerById(tree.value, id);
  }

  function flatten(entries: ServerConfig, acc: Server[] = []): Server[] {
    for (const entry of entries) {
      if (isFolder(entry)) flatten(entry.contents, acc);
      else acc.push(entry);
    }
    return acc;
  }

  const getServers = computed<Server[]>(() => flatten(tree.value));

  function isUuid(value: string): boolean {
    return /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/.test(
      value,
    );
  }


  function serverExists(id: Server["id"]): boolean {
    return findServerById(tree.value, id) !== undefined;
  }

  async function connectToServer(serverId: Server["id"]): Promise<void> {
    const server = await getServer(serverId);
    if (!server) {
      throw new Error(`Server with id ${serverId} not found`);
    }

    const keysStore = useKeysStore();
    await keysStore.load();
    const key = await keysStore.getKeyById(server.keyID);
    
    if (!key) {
      throw new Error(`SSH key with id ${server.keyID} not found`);
    }

    const sshArgs = [
      "-i", key.private,
      "-o", "StrictHostKeyChecking=accept-new",
      "-o", "ConnectTimeout=10"
    ];

    if (server.username) {
      sshArgs.push(`${server.username}@${server.ip}`);
    } else {
      sshArgs.push(server.ip);
    }
    const { useConsolesStore } = await import("./consoles");
    const consolesStore = useConsolesStore();
    await consolesStore.launchNativeTerminalWithArgs(sshArgs);
  }

  return {
    tree,
    load,
    getServers,
    getServer,
    addServer,
    removeServer,
    updateServer,
    connectToServer,
    findServerById: (id: Server["id"]) => findServerById(tree.value, id),
    serverExists,
  };
});
