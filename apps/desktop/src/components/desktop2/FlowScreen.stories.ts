import type { Meta, StoryObj } from "@storybook/svelte-vite";
import { createDesktopBackend } from "../../lib/desktop-backend";
import FlowScreen from "./FlowScreen.svelte";

const meta = { title: "Desktop 2/Flow states", component: FlowScreen, args: { backend: createDesktopBackend(), domains: [{ repo_root: "C:/dev/pytxo" }] } } satisfies Meta<typeof FlowScreen>;
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
