import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PrimeVue from "primevue/config";
import ToastService from "primevue/toastservice";
import App from "./App.vue";

// Mock Tauri event API
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock window.matchMedia for jsdom (used by useTheme)
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
  return mount(App, {
    global: {
      plugins: [PrimeVue, ToastService],
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
});
