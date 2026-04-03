import { createRouter, createWebHashHistory } from "vue-router";

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: "/",
      name: "dashboard",
      component: () => import("../views/DashboardView.vue"),
    },
    {
      path: "/catalog",
      name: "catalog",
      component: () => import("../views/CatalogView.vue"),
    },
    {
      path: "/catalog/:id",
      name: "package-detail",
      component: () => import("../views/PackageDetailView.vue"),
      props: true,
    },
    {
      path: "/installed",
      name: "installed",
      component: () => import("../views/InstalledView.vue"),
    },
    {
      path: "/backup",
      name: "backup",
      component: () => import("../views/BackupView.vue"),
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("../views/SettingsView.vue"),
    },
  ],
});

export default router;
