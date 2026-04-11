import { createRouter, createWebHashHistory } from "vue-router";
import { FEATURE_BACKUP } from "../features";
import { logger } from "../utils/logger";

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
      beforeEnter: () => FEATURE_BACKUP || { name: "dashboard" },
    },
    {
      path: "/settings",
      name: "settings",
      component: () => import("../views/SettingsView.vue"),
    },
  ],
});

router.afterEach((to, from) => {
  logger.debug("router", `${String(from.name ?? from.path)} → ${String(to.name ?? to.path)}`);
});

export default router;
