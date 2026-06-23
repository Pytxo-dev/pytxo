import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebars: SidebarsConfig = {
  docsSidebar: [
    {
      type: "doc",
      id: "intro",
      label: "Introduction",
    },
    {
      type: "category",
      label: "Getting started",
      items: [
        "getting-started/install",
        "getting-started/folder-trust",
        "getting-started/hypervisor-shell",
        "getting-started/first-three-agent-run",
        "getting-started/testing-ade-clis",
        "getting-started/mcp-from-cursor",
      ],
    },
    {
      type: "category",
      label: "Concepts",
      items: [
        "concepts/what-is-pytxo",
        "concepts/three-moats",
        "concepts/signal-core",
        "concepts/blast-shield",
        "concepts/race-shield",
        "concepts/execution-domains",
        "concepts/modular-projects",
        "concepts/fleet-runs",
        "concepts/galaxy-approvals",
        "concepts/reality-deck",
      ],
    },
    {
      type: "category",
      label: "Reference",
      items: [
        "reference/cli",
        "reference/pytxo-toml",
        "reference/providers-byok",
        "reference/models",
        "reference/permission-tiers",
      ],
    },
  ],
};

export default sidebars;
