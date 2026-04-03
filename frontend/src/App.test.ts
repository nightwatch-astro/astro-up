import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PrimeVue from "primevue/config";
import ToastService from "primevue/toastservice";
import { VueQueryPlugin, QueryClient } from "@tanstack/vue-query";
import router from "./router";
import App from "./App.vue";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(() => Promise.resolve([])),
}));

Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

function mountApp() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return mount(App, {
    global: {
      plugins: [PrimeVue, ToastService, router, [VueQueryPlugin, { queryClient }]],
    },
  });
}

describe("App", () => {
  it("mounts and contains app name", () => {
    const wrapper = mountApp();
    expect(wrapper.text()).toContain("Astro-Up");
  });

  it("displays version string", () => {
    const wrapper = mountApp();
    expect(wrapper.text()).toMatch(/v\d+\.\d+\.\d+/);
  });

  it("renders Toast component", () => {
    const wrapper = mountApp();
    expect(wrapper.findComponent({ name: "Toast" }).exists()).toBe(true);
  });

  it("renders sidebar navigation", () => {
    const wrapper = mountApp();
    expect(wrapper.text()).toContain("Dashboard");
    expect(wrapper.text()).toContain("Catalog");
    expect(wrapper.text()).toContain("Installed");
    expect(wrapper.text()).toContain("Backup");
    expect(wrapper.text()).toContain("Settings");
  });
});
