import { test } from "@playwright/test";
import { checkTerminalText } from "../src/lib/terminal-text";

test("worker output keeps readable lines from wrapped PTY text", () => {
  checkTerminalText();
});
