import { createMemoryHistory, createRouter } from "vue-router";

const routes = [
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
    path: "/server/:id/console",
    name: "server-console",
    component: () => import("@/pages/server/console.vue"),
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
