import { test, expect } from "@playwright/test";

test.describe("Pytxo Desktop shell", () => {
  test("setup wizard renders on fresh profile", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.removeItem("pytxo-deck-setup-v1");
      localStorage.removeItem("pytxo-deck-tabs-v1");
    });
    await page.goto("/");
    await expect(page.getByText("Welcome to Pytxo Desktop")).toBeVisible();
    await expect(page.getByRole("button", { name: "Get started" })).toBeVisible();
    await expect(page.getByLabel("Setup progress")).toBeVisible();
  });

  test("setup skip-all reaches ready step", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.removeItem("pytxo-deck-setup-v1");
      localStorage.removeItem("pytxo-deck-tabs-v1");
    });
    await page.goto("/");
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await expect(page.getByRole("heading", { name: "You're ready" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Enter Pytxo Desktop" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Back" })).toBeVisible();
  });

  test("desktop 2 shell opens in operations and navigates primary routes", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("pytxo-deck-setup-v1", "complete");
      localStorage.removeItem("pytxo-deck-tabs-v1");
    });
    await page.goto("/");
    await expect(page.getByText("Pytxo Desktop")).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Operations" })).toBeVisible();
    await page.getByRole("link", { name: "Flow" }).click();
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();
    await page.getByRole("link", { name: "Integrations" }).click();
    await expect(page.getByRole("heading", { name: "Integrations" })).toBeVisible();
  });

  test("legacy workspace tabs migrate to recents without deletion", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("pytxo-deck-setup-v1", "complete");
      localStorage.setItem(
        "pytxo-deck-tabs-v1",
        JSON.stringify({
          tabs: [
            {
              id: "tab-1",
              domainId: "/tmp/demo",
              label: "demo",
              projectId: null,
              selectedRunId: null,
              selectedAgentId: null,
              dispatchRepo: "",
            },
          ],
          activeTabId: "tab-1",
        }),
      );
    });
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
    await expect(page.getByText("demo", { exact: true })).toBeVisible();
    const migration = await page.evaluate(() => ({
      legacy: localStorage.getItem("pytxo-deck-tabs-v1"),
      recents: localStorage.getItem("pytxo-desktop-recents-v2"),
    }));
    expect(migration.legacy).not.toBeNull();
    expect(migration.recents).toContain("demo");
  });

  test("legacy shell remains available for one rollback release", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("pytxo-deck-setup-v1", "complete");
      localStorage.setItem("desktop_shell_v1", "true");
    });
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Open workspaces" })).toBeVisible();
  });

  test("text Flow requires preview before mock dispatch", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    await page.getByLabel("Flow mission").fill("Make the desktop shell production ready");
    await expect(page.getByText("Waiting for a mission")).toBeVisible();
    await page.getByRole("button", { name: /Build plan/ }).click();
    await expect(page.getByText("Ready for review")).toBeVisible();
    await page.getByRole("button", { name: /Dispatch Flow/ }).click();
    await expect(page.getByRole("button", { name: /Dispatched run-preview/ })).toBeVisible();
  });

  test("Voice capture produces an editable Flow mission with the preview backend", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    await expect(page.getByLabel("Voice input device")).toBeEnabled();
    await page.getByRole("button", { name: "Start Voice" }).click();
    await expect(page.getByRole("button", { name: "Finish recording" })).toBeVisible();
    await page.getByRole("button", { name: "Pause" }).click();
    await expect(page.getByRole("button", { name: "Resume" })).toBeVisible();
    await page.getByRole("button", { name: "Resume" }).click();
    await page.getByRole("button", { name: "Finish recording" }).click();
    await expect(page.getByRole("button", { name: "Transcribing…" })).toBeVisible();
    await expect(page.getByLabel("Flow mission")).toHaveValue("Create a production-ready Flow mission");
  });

  test("Voice supports press-and-hold capture", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    const voice = page.getByTestId("voice-capture");
    const box = await voice.boundingBox();
    expect(box).not.toBeNull();
    await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
    await page.mouse.down();
    await page.waitForTimeout(400);
    await page.mouse.move(20, 20);
    await page.mouse.up();
    await expect(page.getByLabel("Flow mission")).toHaveValue("Create a production-ready Flow mission");
  });

  test("Voice pointer cancellation releases capture without retaining audio", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    const voice = page.getByTestId("voice-capture");
    await voice.dispatchEvent("pointerdown", { button: 0, pointerId: 7 });
    await voice.dispatchEvent("pointercancel", { button: 0, pointerId: 7 });
    await expect(page.getByText("Voice capture cancelled · no audio retained")).toBeVisible();
  });

  test("leaving Flow cancels active microphone capture", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await page.getByRole("link", { name: "Operations" }).click();
    await page.getByRole("link", { name: "Flow" }).click();
    await page.getByRole("button", { name: "Start Voice" }).click();
    await expect(page.getByRole("button", { name: "Finish recording" })).toBeVisible();
  });

  test("Voice transcription can be cancelled without retaining audio", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await page.getByRole("button", { name: "Finish recording" }).click();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible();
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(page.getByText("Voice capture cancelled · no audio retained")).toBeVisible();
  });

  test("Voice exposes device loss and uncertain transcript correction", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/flow");
    await page.getByLabel("Voice input device").selectOption("Disconnected microphone");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await expect(page.getByText("Voice capture failed · choose another input device")).toBeVisible();

    await page.getByLabel("Voice input device").selectOption("Studio microphone");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await page.getByRole("button", { name: "Finish recording" }).click();
    const correction = page.getByLabel("Correct uncertain segment 1");
    await expect(correction).toBeVisible();
    await correction.fill("Update the desktop approval flow");
    await expect(page.getByLabel("Flow mission")).toHaveValue("Update the desktop approval flow");
  });

  test("current and legacy deep links route through the same shell", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/");
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("pytxo-deep-link", { detail: "pytxo://flow" })));
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("pytxo-deep-link", { detail: "pytxo-deck://settings" })));
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  });

  test("approval resolution, focus routes, and restart persistence work", async ({ page }) => {
    await page.addInitScript(() => localStorage.setItem("pytxo-deck-setup-v1", "complete"));
    await page.goto("/#/approvals");
    await expect(page.getByText("14 files changed · permission profile Orbit")).toBeVisible();
    await expect(page.getByText("Review required")).toBeVisible();
    await expect(page.getByText("No secrets or permission escalation detected.")).toHaveCount(0);
    await page.getByRole("button", { name: /Approve & flush/ }).click();
    await expect(page.getByRole("heading", { name: "Inbox clear" })).toBeVisible();
    await page.goto("/#/topology-focus");
    await expect(page.getByRole("heading", { name: "Topology Focus" })).toBeVisible();
    await page.goto("/#/run-review");
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
    await page.getByRole("link", { name: "Settings" }).click();
    await page.reload();
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  });
});
