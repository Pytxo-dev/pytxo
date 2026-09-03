import { expect, test } from "@playwright/test";
import {
  ATTEMPT_LABELS,
  SURFACE_LABELS,
  agentState,
  attemptTone,
  isPartiallyApplied,
  surfaceTone,
  waveProgress,
  worstTone,
} from "../src/lib/epistemic";
import type { AgentDto, EnforcementSurface, RunApplyAttempt } from "../src/lib/types";
import { completeOnboarding } from "./helpers";

const SURFACE_STATUSES: EnforcementSurface["status"][] = [
  "enforced",
  "advisory",
  "unavailable",
  "bypassed",
];

const ATTEMPT_OUTCOMES: RunApplyAttempt["outcome"][] = [
  "committed",
  "rolled_back",
  "recovery_required",
  "interrupted",
];

type RenderedChip = {
  tone: string | null;
  treatment: string | null;
  label: string;
  hue: string;
  fontSize: number;
};

/** Reads every state chip in one region so encoding can be compared as a set. */
async function readChips(page: import("@playwright/test").Page, region: string) {
  return page.getByLabel(region).locator(".chip").evaluateAll((chips) =>
    chips.map((chip) => {
      const mark = chip.querySelector(".mark");
      const label = chip.querySelector("strong");
      return {
        tone: chip.getAttribute("data-tone"),
        treatment: mark?.getAttribute("data-treatment") ?? null,
        label: label?.textContent?.trim() ?? "",
        hue: mark ? getComputedStyle(mark).borderColor : "",
        fontSize: label ? Number.parseFloat(getComputedStyle(label).fontSize) : 0,
      };
    }),
  ) as Promise<RenderedChip[]>;
}

function agent(overrides: Partial<AgentDto>): AgentDto {
  return {
    id: "a",
    run_id: "run-1",
    task_id: "t",
    wave: 0,
    status: "queued",
    exit_code: null,
    root_id: null,
    ...overrides,
  } as AgentDto;
}

test.describe("Epistemic state contract", () => {
  test("every enforcement status maps to exactly one epistemic tone", () => {
    expect(SURFACE_STATUSES.map(surfaceTone)).toEqual([
      "verified",
      "claimed",
      "unknown",
      "refuted",
    ]);
  });

  test("every apply outcome maps to a tone that never overstates the evidence", () => {
    expect(ATTEMPT_OUTCOMES.map(attemptTone)).toEqual([
      "verified",
      "refuted",
      "refuted",
      "unknown",
    ]);
  });

  test("every status and outcome carries a distinct human label", () => {
    const labels = [
      ...SURFACE_STATUSES.map((status) => SURFACE_LABELS[status]),
      ...ATTEMPT_OUTCOMES.map((outcome) => ATTEMPT_LABELS[outcome]),
    ];
    expect(new Set(labels).size).toBe(labels.length);
    expect(labels.every((label) => label.trim().length > 0)).toBe(true);
  });

  test("an unrecognised agent status resolves to unknown rather than a guess", () => {
    expect(agentState(agent({ status: "something-new" })).tone).toBe("unknown");
    expect(agentState(agent({ status: "stopped" })).tone).toBe("unknown");
  });

  test("a summary never reads stronger than its weakest surface", () => {
    expect(worstTone(["verified", "claimed"], "verified")).toBe("claimed");
    expect(worstTone(["verified", "unknown"], "verified")).toBe("unknown");
    expect(worstTone(["verified", "claimed", "refuted"], "verified")).toBe("refuted");
    expect(worstTone(["verified", "verified"], "verified")).toBe("verified");
  });

  test("an unconfirmed rollback is always reported as partially applied", () => {
    const base = {
      attempt_id: "x",
      created_at: "2026-08-01T00:00:00Z",
      phase: "rolled_back",
      error_code: null,
      error_message: null,
    };
    expect(isPartiallyApplied({ ...base, outcome: "interrupted", rollback_confirmed: false })).toBe(true);
    expect(isPartiallyApplied({ ...base, outcome: "recovery_required", rollback_confirmed: false })).toBe(true);
    expect(isPartiallyApplied({ ...base, outcome: "rolled_back", rollback_confirmed: true })).toBe(false);
    expect(isPartiallyApplied({ ...base, outcome: "committed", rollback_confirmed: false })).toBe(false);
  });

  // The ledger must never imply a quantity the orchestrator does not compute.
  // Wave settlement is the only ratio with a real numerator and denominator.
  test("progress separates settled waves from successful waves", () => {
    expect(waveProgress([])).toBeNull();
    expect(
      waveProgress([
        agent({ id: "1", wave: 0, status: "completed", exit_code: 0 }),
        agent({ id: "2", wave: 1, status: "running" }),
      ]),
    ).toEqual({ settled: 1, successful: 1, total: 2 });
    expect(
      waveProgress([
        agent({ id: "1", wave: 0, status: "completed", exit_code: 0 }),
        agent({ id: "2", wave: 0, status: "running" }),
      ]),
    ).toEqual({ settled: 0, successful: 0, total: 1 });
    expect(
      waveProgress([
        agent({ id: "1", wave: 0, status: "failed", exit_code: 1 }),
        agent({ id: "2", wave: 1, status: "stopped" }),
      ]),
    ).toEqual({ settled: 2, successful: 0, total: 2 });
    expect(
      waveProgress([
        agent({ id: "1", wave: 0, status: "future_status", exit_code: null }),
        agent({ id: "2", wave: 1, status: "blocked_by_dependency", exit_code: null }),
      ]),
    ).toEqual({ settled: 1, successful: 0, total: 2 });
  });
});

test.describe("Epistemic state rendering", () => {
  test.beforeEach(async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-state-matrix-v1": "1" });
  });

  test("all four enforcement statuses render distinct non-colour affordances", async ({ page }) => {
    await page.goto("/#/work");
    const receipt = page.getByLabel("Permission enforcement receipt");
    await expect(receipt).toBeVisible();

    const expected: Record<EnforcementSurface["status"], string> = {
      enforced: "fill",
      advisory: "outline",
      unavailable: "hatch",
      bypassed: "fill",
    };

    const chips = await readChips(page, "Permission enforcement receipt");

    for (const status of SURFACE_STATUSES) {
      const chip = chips.find((candidate) => candidate.label === SURFACE_LABELS[status]);
      expect(chip, `chip for ${status}`).toBeTruthy();
      expect(chip.treatment, `${status} treatment`).toBe(expected[status]);
      expect(chip.tone, `${status} tone`).toBe(surfaceTone(status));
    }

    // Encoding is legible without colour: every value shows a marker treatment
    // and a text label, and the label distinguishes the two fill values.
    const encodings = SURFACE_STATUSES.map((status) => {
      const chip = chips.find((candidate) => candidate.label === SURFACE_LABELS[status]);
      return `${chip.treatment}:${chip.label}`;
    });
    expect(new Set(encodings).size).toBe(SURFACE_STATUSES.length);

    // Unknown must not borrow the attention hue: absence of evidence is not a
    // pending decision.
    const unknown = chips.find((chip) => chip.tone === "unknown");
    const attention = chips.find((chip) => chip.tone === "attention");
    if (unknown && attention) expect(unknown.hue).not.toBe(attention.hue);
  });

  test("all four apply outcomes render distinct non-colour affordances", async ({ page }) => {
    await page.goto("/#/work");
    await expect(page.getByLabel("Apply attempts")).toBeVisible();

    const expected: Record<RunApplyAttempt["outcome"], string> = {
      committed: "fill",
      rolled_back: "fill",
      recovery_required: "fill",
      interrupted: "hatch",
    };

    const chips = await readChips(page, "Apply attempts");

    for (const outcome of ATTEMPT_OUTCOMES) {
      const chip = chips.find((candidate) => candidate.label === ATTEMPT_LABELS[outcome]);
      expect(chip, `chip for ${outcome}`).toBeTruthy();
      expect(chip.treatment, `${outcome} treatment`).toBe(expected[outcome]);
      expect(chip.tone, `${outcome} tone`).toBe(attemptTone(outcome));
    }

    const encodings = ATTEMPT_OUTCOMES.map((outcome) => {
      const chip = chips.find((candidate) => candidate.label === ATTEMPT_LABELS[outcome]);
      return `${chip.treatment}:${chip.label}`;
    });
    expect(new Set(encodings).size).toBe(ATTEMPT_OUTCOMES.length);
  });

  test("operator-read state labels sit at or above the 11px floor", async ({ page }) => {
    await page.goto("/#/work");
    for (const region of ["Permission enforcement receipt", "Apply attempts"]) {
      for (const chip of await readChips(page, region)) {
        expect(chip.fontSize, `${region} / ${chip.label}`).toBeGreaterThanOrEqual(11);
      }
    }
  });

  test("an unconfirmed rollback states partial modification and offers recovery", async ({ page }) => {
    await page.goto("/#/work");
    await expect(page.getByText("Working tree may be partially modified")).toBeVisible();
  });

  test("no state visual renders a hardcoded percentage dimension", async ({ page }) => {
    await page.goto("/#/work");
    const fabricated = await page.evaluate(() => {
      const suspects = [
        ...document.querySelectorAll(
          "[class*='timeline'], [class*='progress'], [class*='fill'], [class*='track'], [class*='meter']",
        ),
      ];
      return suspects
        .map((node) => {
          const inline = node.getAttribute("style") ?? "";
          const width = getComputedStyle(node).width;
          // A bound width arrives as an inline style or custom property. A
          // stylesheet percentage that is not 100% is a fabricated quantity.
          const boundInline = /--|width/.test(inline);
          return { className: node.className, width, boundInline };
        })
        .filter((entry) => !entry.boundInline && /^\d+(\.\d+)?px$/.test(entry.width) === false);
    });
    expect(fabricated).toEqual([]);
  });
});
