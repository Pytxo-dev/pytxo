import type { Meta, StoryObj } from "@storybook/svelte-vite";
import SetupWizard from "./SetupWizard.svelte";

const meta = { title: "Desktop 2/Onboarding", component: SetupWizard, args: { onComplete: () => {}, onWorkspaceSelected: () => {} } } satisfies Meta<typeof SetupWizard>;
export default meta;
type Story = StoryObj<typeof meta>;
export const Welcome: Story = {};
