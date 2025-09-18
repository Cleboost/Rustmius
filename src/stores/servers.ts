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

  async function syncFromSshConfig(): Promise<void> {
    await load();
    const home = await homeDir();
    const sshConfigRel = ".ssh/config";
    let content: string;
    try {
      content = await readTextFile(sshConfigRel, {
        baseDir: BaseDirectory.Home,
      });
    } catch (e) {
      console.warn("[servers] ~/.ssh/config not found or unreadable", e);
      return;
    }

    const lines = content.split(/\r?\n/);
    type HostBlock = { name: string; options: Record<string, string> };
    const hosts: HostBlock[] = [];
    let current: HostBlock | null = null;
    for (const raw of lines) {
      const line = raw.trimEnd();
      if (!line || line.trimStart().startsWith("#")) continue;
      if (/^Host\s+/i.test(line)) {
        const name = line.replace(/^Host\s+/i, "").trim();
        current = { name, options: {} };
        hosts.push(current);
      } else if (current) {
        const m = line.trim().match(/^(\w[\w-]*)\s+(.+)$/);
        if (m) current.options[m[1].toLowerCase()] = m[2].trim();
      }
    }

    const keysStore = useKeysStore();
    await keysStore.load();
    const keys = await keysStore.getKeys();
    const keyByPath = new Map(keys.map((k) => [k.private, k]));

    let modified = false;
    const newLines: string[] = [];
    let i = 0;
    while (i < lines.length) {
      const raw = lines[i];
      const trimmed = raw.trim();
      if (/^Host\s+/i.test(trimmed)) {
        const originalAlias = trimmed.replace(/^Host\s+/i, "").trim();
        i++;
        const block: string[] = [raw];
        while (i < lines.length && !/^Host\s+/i.test(lines[i].trim())) {
          block.push(lines[i]);
          i++;
        }

        if (originalAlias.toLowerCase() === "aur.archlinux.org") {
          newLines.push(...block);
          continue;
        }

        const parsed = hosts.find((h) => h.name === originalAlias)!;
        const hostname = parsed.options["hostname"] ?? "";
        let identityFile = parsed.options["identityfile"];
        if (identityFile && identityFile.startsWith("~")) {
          identityFile = identityFile.replace(/^~\//, `${home}/`);
        }

        if (isUuid(originalAlias)) {
          let server = findServerById(tree.value, originalAlias);
          if (!server) {
            server = {
              id: originalAlias,
              name: originalAlias,
              ip: hostname,
              keyID: keyByPath.get(identityFile ?? "")?.id ?? 0,
            } as Server;
            await addServer(server);
          } else {
            const keyId = keyByPath.get(identityFile ?? "")?.id ?? server.keyID;
            const updated: Server = {
              ...server,
              ip: hostname || server.ip,
              keyID: keyId,
            };
            await updateServer(server.id, updated);
          }
          newLines.push(...block);
        } else {
          const newId = crypto.randomUUID();
          const server: Server = {
            id: newId,
            name: originalAlias,
            ip: hostname,
            keyID: keyByPath.get(identityFile ?? "")?.id ?? 0,
          } as Server;
          await addServer(server);
          const replaced = block.slice();
          replaced[0] = block[0].replace(/^(\s*Host\s+).+$/i, `$1${newId}`);
          newLines.push(...replaced);
          modified = true;
        }
      } else {
        newLines.push(raw);
        i++;
      }
    }

    if (modified) {
      await writeTextFile(sshConfigRel, newLines.join("\n"), {
        baseDir: BaseDirectory.Home,
      });
    }
  }

  return {
    load,
    getServers,
    getServer,
    addServer,
    removeServer,
    updateServer,
    syncFromSshConfig,
  };
});
