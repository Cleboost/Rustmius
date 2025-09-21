import { createApp } from "vue";
import { useColorMode } from "@vueuse/core";
import App from "@/App.vue";
import router from "@/router";
import { createPinia } from "pinia";
import { useSettingsStore } from "@/stores/settings";
import { useKeysStore } from "./stores/keys";
import { useServersStore } from "./stores/servers";
import { readDir, exists } from "@tauri-apps/plugin-fs";
import { homeDir, join, BaseDirectory } from "@tauri-apps/api/path";

import "@/index.css";

const pinia = createPinia();
const app = createApp(App);

app.use(pinia);
app.use(router);

initAppSetttings();

app.mount("#app");

async function initAppSetttings() {
  const settingsStore = useSettingsStore();
  const mode = useColorMode();
  const theme = (await settingsStore.getSetting<string>("theme")) as
    | "light"
    | "dark"
    | "auto";
  if (!theme) {
    settingsStore.setSetting("theme", mode.value);
  } else {
    mode.value = theme;
  }

  await ensureSshKeysSynced();
  await ensureServersSynced();
}

async function ensureSshKeysSynced() {
  const keysStore = useKeysStore();
  await keysStore.load();

  const home = await homeDir();
  const sshRel = ".ssh";
  const sshAbs = await join(home, ".ssh");
  const entries = await readDir(sshRel, { baseDir: BaseDirectory.Home });
  const scanned: Array<{ name: string; private: string; public?: string }> = [];
  for (const e of entries) {
    if (e.isDirectory) continue;
    const name = e.name ?? "";
    if (!name.endsWith(".pub")) {
      continue;
    }
    const stem = name.slice(0, -4);
    if (!stem || stem === "config" || stem.startsWith("known_hosts")) {
      continue;
    }
    const privateRel = `${sshRel}/${stem}`;
    const hasPrivate = await exists(privateRel, {
      baseDir: BaseDirectory.Home,
    });
    if (!hasPrivate) {
      continue; // require private key pair
    }
    const publicPath = await join(sshAbs, name);
    const privatePath = await join(sshAbs, stem);
    scanned.push({ name: stem, private: privatePath, public: publicPath });
  }

  const existing = await keysStore.getKeys();
  const byPrivate = new Map(existing.map((k) => [k.private, k]));

  for (const s of scanned) {
    const found = byPrivate.get(s.private);
    if (found) {
      if (found.name !== s.name || found.public !== s.public) {
        await keysStore.addOrUpdateKey({
          id: found.id,
          name: s.name,
          private: s.private,
          public: s.public,
        });
      }
    } else {
      await keysStore.addOrUpdateKey({
        name: s.name,
        private: s.private,
        public: s.public,
      });
    }
  }
}

async function ensureServersSynced() {
  const serversStore = useServersStore();
  await serversStore.syncFromSshConfig();
}
