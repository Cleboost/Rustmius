import { createApp } from "vue";
import { useColorMode } from "@vueuse/core";
import App from "@/App.vue";
import router from "@/router";
import { createPinia } from "pinia";
import { useSettingsStore } from "@/stores/settings";
import { useKeysStore } from "./stores/keys";
import { useServerConfigStore } from "./stores/servers";

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

  await useKeysStore().syncWithFs();
  await useServerConfigStore().load();
}
