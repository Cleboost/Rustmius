import {
  createMemoryHistory,
  createRouter,
  type RouteRecordRaw,
} from "vue-router";

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
    path: "/server/:id/docker/images",
    name: "images",
    component: () => import("@/pages/server/docker/images.vue"),
  },
  {
    path: "/server/:id/docker/containers",
    name: "containers",
    component: () => import("@/pages/server/docker/containers.vue"),
  },
  {
    path: "/server/:id/docker/container/:cid",
    name: "container-details",
    component: () => import("@/pages/server/docker/container/index.vue"),
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
