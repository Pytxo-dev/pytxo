import type { Meta, StoryObj } from "@storybook/svelte-vite";
import { createDesktopBackend } from "../../lib/desktop-backend";
import FlowScreen from "./FlowScreen.svelte";

const meta = { title: "Desktop 2/Flow states", component: FlowScreen, args: { backend: createDesktopBackend(), domains: [{ domain_id: "dom-pytxo", repo_root: "C:/dev/pytxo" }] } } satisfies Meta<typeof FlowScreen>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Draft: Story = { args: { previewState: "draft" } };
export const VoiceRecording: Story = { args: { previewState: "recording" } };
export const VoicePaused: Story = { args: { previewState: "paused" } };
export const VoiceTranscribing: Story = { args: { previewState: "transcribing" } };
export const VoiceCancelled: Story = { args: { previewState: "cancelled" } };
export const VoiceFailed: Story = { args: { previewState: "failed" } };
export const VoiceUncertainCorrection: Story = { args: { previewState: "uncertain" } };
export const Planning: Story = { args: { previewState: "planning" } };
export const Ready: Story = { args: { previewState: "ready" } };
export const Blocked: Story = { args: { previewState: "blocked" } };
export const Dispatched: Story = { args: { previewState: "dispatched" } };

export const RetainedDraftMissingWorkspace: Story = {
  args: {
    draft: { mission: "Fix src/parser.rs in the original repository.", source: "text", domainId: "removed-workspace", adeId: "codex" },
    preferredDomainId: "dom-pytxo",
  },
  play: async ({ canvasElement }) => {
    const workspace = canvasElement.querySelector<HTMLSelectElement>('select[aria-label="Project"]');
    const build = Array.from(canvasElement.querySelectorAll("button")).find(button => button.textContent?.trim() === "Build plan");
    if (!workspace || workspace.value !== "" || !build?.disabled || !canvasElement.textContent?.includes("The draft's workspace is no longer available")) {
      throw new Error("An unavailable draft workspace must require explicit selection, without falling back to the current folder.");
    }
  },
};
