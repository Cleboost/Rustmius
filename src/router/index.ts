import { createMemoryHistory, createRouter, type RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    redirect: "/home",
    children: [
      {
        path: "home",
        name: "home",
        component: () => import("@/pages/home.vue"),
      },
    ],
  },
  {
    path: "/keys",
    name: "keys",
    component: () => import("@/pages/keys.vue"),
  },
  {
    path: "/server/:id",
    name: "server",
    component: () => import("@/pages/server/index.vue"),
  },
  {
    path: "/server/:id/docker",
    name: "docker",
    component: () => import("@/pages/server/docker.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("@/pages/settings.vue"),
  },
];

const router = createRouter({
  history: createMemoryHistory(),
  routes,
});

export default router;
