import { ref, computed, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

export type ThemeMode = "system" | "light" | "dark";

const currentTheme = ref<ThemeMode>("system");

const isDark = computed(() => {
  if (currentTheme.value === "dark") return true;
  if (currentTheme.value === "light") return false;
  return mediaQuery?.matches ?? true;
});

function applyDarkClass(dark: boolean) {
  if (dark) {
    document.documentElement.classList.add("app-dark");
  } else {
    document.documentElement.classList.remove("app-dark");
  }
}

let mediaQuery: MediaQueryList | null = null;
let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;

function watchSystem() {
  cleanupSystemWatch();
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  applyDarkClass(mediaQuery.matches);
  mediaHandler = (e) => applyDarkClass(e.matches);
  mediaQuery.addEventListener("change", mediaHandler);
}

function cleanupSystemWatch() {
  if (mediaQuery && mediaHandler) {
    mediaQuery.removeEventListener("change", mediaHandler);
    mediaQuery = null;
    mediaHandler = null;
  }
}

function applyTheme(mode: ThemeMode) {
  currentTheme.value = mode;
  cleanupSystemWatch();

  switch (mode) {
    case "system":
      watchSystem();
      break;
    case "dark":
      applyDarkClass(true);
      break;
    case "light":
      applyDarkClass(false);
      break;
  }
}

export function useTheme() {
  async function init() {
    try {
      const config = await invoke<Record<string, unknown>>("get_config");
      const general = config?.general as Record<string, unknown> | undefined;
      const theme = (general?.theme as ThemeMode) ?? "system";
      applyTheme(theme);
    } catch {
      applyTheme("system");
    }
  }

  async function setTheme(mode: ThemeMode) {
    applyTheme(mode);
    try {
      const config = await invoke<Record<string, unknown>>("get_config");
      const general = (config?.general as Record<string, unknown>) ?? {};
      general.theme = mode;
      await invoke("save_config", { config: { ...config, general } });
    } catch (e) {
      console.warn("Failed to save theme preference:", e);
    }
  }

  onUnmounted(() => {
    cleanupSystemWatch();
  });

  return {
    currentTheme,
    isDark,
    init,
    setTheme,
  };
}
