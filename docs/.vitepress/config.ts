import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Astro-Up",
  description:
    "Astrophotography software manager for Windows — install, detect, and update imaging software.",
  // Deploy to GitHub Pages at nightwatch-astro.github.io/astro-up/
  // When adding a custom domain later, change base to "/" and set site URL.
  base: "/astro-up/",

  head: [["link", { rel: "icon", href: "/astro-up/favicon.ico" }]],

  themeConfig: {
    logo: "/logo.svg",

    nav: [
      { text: "Docs", link: "/guide/what-is-astro-up" },
      {
        text: "Download",
        link: "https://github.com/nightwatch-astro/astro-up/releases/latest",
      },
    ],

    sidebar: [
      {
        text: "Getting Started",
        items: [
          { text: "What is Astro-Up?", link: "/guide/what-is-astro-up" },
          { text: "Installation", link: "/guide/installation" },
          { text: "Quick Start", link: "/guide/quick-start" },
        ],
      },
      {
        text: "Features",
        items: [
          { text: "Software Catalog", link: "/guide/catalog" },
          { text: "Detection", link: "/guide/detection" },
          { text: "Installing & Updating", link: "/guide/installing" },
          { text: "Backup & Restore", link: "/guide/backup" },
          { text: "Configuration", link: "/guide/configuration" },
        ],
      },
      {
        text: "Guides",
        items: [
          { text: "How It Works", link: "/guide/how-it-works" },
          { text: "Troubleshooting", link: "/guide/troubleshooting" },
        ],
      },
      {
        text: "Reference",
        items: [
          { text: "CLI Commands", link: "/reference/cli" },
          { text: "Configuration File", link: "/reference/config" },
          { text: "Catalog Format", link: "/reference/catalog" },
          { text: "FAQ", link: "/reference/faq" },
        ],
      },
      {
        text: "Contributing",
        items: [
          { text: "Development Setup", link: "/guide/development" },
          { text: "Architecture", link: "/guide/architecture" },
          { text: "Adding Manifests", link: "/guide/adding-manifests" },
          { text: "Silent Installers", link: "/guide/silent-installers" },
          { text: "Lifecycle Testing", link: "/guide/lifecycle-testing" },
        ],
      },
    ],

    socialLinks: [
      {
        icon: "github",
        link: "https://github.com/nightwatch-astro/astro-up",
      },
    ],

    footer: {
      message: "Licensed under Apache-2.0",
      copyright: "Copyright 2024\u2013present Nightwatch Astro",
    },

    search: {
      provider: "local",
    },
  },
});
