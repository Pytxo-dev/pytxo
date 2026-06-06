import { themes as prismThemes } from "prism-react-renderer";
import type { Config } from "@docusaurus/types";
import type * as Preset from "@docusaurus/preset-classic";

const config: Config = {
  title: "Pytxo Docs",
  tagline: "Agent hypervisor & telemetry plane",
  favicon: "img/favicon.ico",

  future: {
    v4: true,
  },

  url: "https://pytxo.com",
  baseUrl: "/docs/",
  trailingSlash: false,

  organizationName: "Pytxo-dev",
  projectName: "pytxo",

  onBrokenLinks: "throw",
  onBrokenMarkdownLinks: "warn",

  i18n: {
    defaultLocale: "en",
    locales: ["en"],
  },

  presets: [
    [
      "classic",
      {
        docs: {
          sidebarPath: "./sidebars.ts",
          routeBasePath: "/",
          editUrl: "https://github.com/Pytxo-dev/pytxo/tree/main/apps/docs/",
        },
        blog: false,
        theme: {
          customCss: "./src/css/custom.css",
        },
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    image: "img/logo.png",
    colorMode: {
      defaultMode: "dark",
      disableSwitch: true,
      respectPrefersColorScheme: false,
    },
    navbar: {
      title: "Pytxo Docs",
      logo: {
        alt: "Pytxo",
        src: "img/logo.png",
      },
      items: [
        {
          href: "https://pytxo.com",
          label: "Home",
          position: "left",
        },
        {
          href: "https://pytxo.com/download",
          label: "Download",
          position: "right",
        },
        {
          href: "https://github.com/Pytxo-dev/pytxo",
          label: "GitHub",
          position: "right",
        },
      ],
    },
    footer: {
      style: "dark",
      links: [
        {
          title: "Getting started",
          items: [
            { label: "Introduction", to: "/" },
            { label: "Install", to: "/getting-started/install" },
            { label: "First three-agent run", to: "/getting-started/first-three-agent-run" },
          ],
        },
        {
          title: "Product",
          items: [
            { label: "What is Pytxo?", to: "/concepts/what-is-pytxo" },
            { label: "Three moats", to: "/concepts/three-moats" },
            { label: "Reality Deck", to: "/concepts/reality-deck" },
          ],
        },
        {
          title: "Developers",
          items: [
            { label: "Architecture", to: "/developers/architecture" },
            { label: "CLI reference", to: "/reference/cli" },
            { label: "Contributing", to: "/developers/contributing" },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} Pytxo. MIT.`,
    },
    prism: {
      theme: prismThemes.dracula,
      darkTheme: prismThemes.dracula,
      additionalLanguages: ["bash", "toml", "rust"],
    },
  } satisfies Preset.ThemeConfig,
};

export default config;
