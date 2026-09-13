import type { Meta, StoryObj } from "@storybook/svelte-vite";
import DesktopShell from "./DesktopShell.svelte";
import type { AppRoute } from "../../lib/navigation.svelte";

const meta = { title: "Desktop 2/Screens", component: DesktopShell, parameters: { layout: "fullscreen" } } satisfies Meta<typeof DesktopShell>;
export default meta;
type Story = StoryObj<typeof meta>;
const screen = (routeOverride: AppRoute, viewport: string): Story => ({ args: { routeOverride }, parameters: { viewport: { defaultViewport: viewport } } });

export const Work1600 = screen("work", "desktop1600");
export const Work1280 = screen("work", "desktop1280");
export const Work960 = screen("work", "compact960");
export const WorkPlan1600 = screen("flow", "desktop1600");
export const WorkPlan1280 = screen("flow", "desktop1280");
export const WorkPlan960 = screen("flow", "compact960");
export const WorkReview1600 = screen("run-review", "desktop1600");
export const WorkReview1280 = screen("run-review", "desktop1280");
export const WorkReview960 = screen("run-review", "compact960");
export const History1600 = screen("history", "desktop1600");
export const History1280 = screen("history", "desktop1280");
export const History960 = screen("history", "compact960");
export const Setup1600 = screen("setup", "desktop1600");
export const Setup1280 = screen("setup", "desktop1280");
export const Setup960 = screen("setup", "compact960");
export const Approvals1600 = screen("approvals", "desktop1600");
export const Approvals1280 = screen("approvals", "desktop1280");
export const Approvals960 = screen("approvals", "compact960");
export const WorkLoading: Story = {
  args: { routeOverride: "work", previewState: "loading" },
  play: async ({ canvasElement }) => {
    const root = canvasElement.ownerDocument.documentElement;
    const prior = root.hasAttribute("data-force-reduced-motion");
    const loader = canvasElement.querySelector(".loading-state div");
    if (!loader) throw new Error("Work loading indicator is missing");
    try {
      root.setAttribute("data-force-reduced-motion", "");
      if (getComputedStyle(loader).animationName !== "none" || loader.getAnimations().length !== 0) {
        throw new Error("Reduced motion must stop the loading animation, not accelerate it");
      }
    } finally {
      root.toggleAttribute("data-force-reduced-motion", prior);
    }
  },
};
export const WorkEmpty: Story = { args: { routeOverride: "work", previewState: "empty" } };
export const WorkOffline: Story = { args: { routeOverride: "work", previewState: "offline" } };
export const WorkError: Story = { args: { routeOverride: "work", previewState: "error" } };
export const HistoryEmpty: Story = { args: { routeOverride: "history", previewState: "empty" } };
export const ApprovalsEmpty: Story = { args: { routeOverride: "approvals", previewState: "empty" } };
export const SetupWorkspacesEmpty: Story = { args: { routeOverride: "workspaces", previewState: "empty" } };
export const SetupAgentsEmpty: Story = { args: { routeOverride: "agents", previewState: "empty" } };

export const WorkSidebarCollapsed: Story = { args: { routeOverride: "work", collapsedOverride: true } };
export const HistorySidebarCollapsed: Story = { args: { routeOverride: "history", collapsedOverride: true } };
export const SetupSidebarCollapsed: Story = { args: { routeOverride: "setup", collapsedOverride: true } };
