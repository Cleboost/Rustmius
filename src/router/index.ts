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
