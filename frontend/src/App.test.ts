import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import App from "./App.vue";

describe("App", () => {
  it("mounts and contains app name", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Astro-Up");
  });

  it("displays version string", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toMatch(/v\d+\.\d+\.\d+/);
  });

  it("renders PrimeVue Card component", () => {
    const wrapper = mount(App);
    expect(wrapper.find("[data-pc-name='card']").exists() || wrapper.find(".p-card").exists()).toBe(
      true,
    );
  });
});
