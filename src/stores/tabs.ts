import { reactive } from "vue";

export type SidebarRoute = { name: string; icon: string; path: string };

export const sidebarRoutes = reactive<SidebarRoute[]>([
  { name: "Servers", icon: "ph:house-duotone", path: "/home" },
  { name: "SSH Keys", icon: "ph:key-duotone", path: "/keys" },
]);

export function addServerTab(name: string, id: string) {
  const path = `/server/${id}/console`;
  if (!sidebarRoutes.find((r) => r.path === path)) {
    sidebarRoutes.splice(1, 0, { name, icon: "lucide:server", path });
  }
}
