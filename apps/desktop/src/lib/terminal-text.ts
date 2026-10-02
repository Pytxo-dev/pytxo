import type { EventDto } from "./types";

/**
 * Plain text from PTY output: drops CSI/OSC sequences and control characters.
 * ConPTY wraps a full line by moving the cursor back onto the last column and
 * re-emitting that character, so the repeat after a cursor move is dropped.
 * Mirrors `strip_terminal_sequences` in the Desktop IPC.
 */
export function stripTerminal(text: string): string {
  let out = "";
  let afterCursorMove = false;
  for (let i = 0; i < text.length; i += 1) {
    const c = text[i];
    if (c === "\u001b") {
      const kind = text[i + 1];
      i += 1;
      if (kind === "[") {
        while (i + 1 < text.length) {
          i += 1;
          if (text[i] >= "@" && text[i] <= "~") { afterCursorMove = text[i] === "H"; break; }
        }
      } else if (kind === "]") {
        while (i + 1 < text.length) {
          i += 1;
          if (text[i] === "\u0007") break;
          if (text[i] === "\u001b" && text[i + 1] === "\\") { i += 1; break; }
        }
      }
      continue;
    }
    if (c === "\n") { out += "\n"; afterCursorMove = false; continue; }
    if (c === "\t") { out += " "; continue; }
    if (c < " " || c === "\u007f") continue;
    if (afterCursorMove && out.endsWith(c)) { afterCursorMove = false; continue; }
    afterCursorMove = false;
    out += c;
  }
  return out;
}

/**
 * Recorded worker output is one event per line. ConPTY continues a soft-wrapped
 * line in a new event that starts by moving the cursor back onto the last
 * column, so that event joins the previous line. Mirrors `join_output_lines`.
 */
export function joinOutputLines(lines: string[]): string {
  return lines.map((line, index) => (index > 0 && !/^\u001b\[\d*;\d*H/.test(line) ? `\n${line}` : line)).join("");
}

/** One readable line per meaningful event. Mirrors `event_lines` in Core. */
export function eventLines(events: EventDto[]): string[] {
  const lines: string[] = [];
  let output: string[] = [];
  const flush = () => {
    for (const line of stripTerminal(joinOutputLines(output)).split("\n")) if (line.trim()) lines.push(line.trimEnd());
    output = [];
  };
  for (const event of events) {
    if (event.kind === "stdout" || event.kind === "stderr") { output.push(event.payload); continue; }
    flush();
    if (event.kind === "verify") lines.push(`$ ${stripTerminal(event.payload).trim()}`);
    else if (event.kind === "verify-ok") lines.push("✓ Task checks passed");
    else if (event.kind === "verify-failed") lines.push("✗ Task checks failed");
    else if (event.kind === "agent-cancelled") lines.push("Stopped");
  }
  flush();
  return lines;
}

/** Self-check against the captured shape of a wrapped Codex PTY error. */
export function checkTerminalText(): void {
  const wrapped = stripTerminal("ERROR: {\"mes\u001b[23;80Hssage\":\"model is not supported with a ChatGPT \u001b[23;80H account.\"}");
  if (wrapped !== "ERROR: {\"message\":\"model is not supported with a ChatGPT account.\"}") throw new Error(wrapped);
  const lines = eventLines([
    { id: 1, agent_id: "a", kind: "stdout", payload: "\u001b]0;npm\u0007hello", ts: "" },
    { id: 2, agent_id: "a", kind: "stdout", payload: "wor", ts: "" },
    { id: 4, agent_id: "a", kind: "stdout", payload: "\u001b[23;80Hrld", ts: "" },
    { id: 3, agent_id: "a", kind: "verify-ok", payload: "", ts: "" },
  ]);
  if (lines.join("|") !== "hello|world|✓ Task checks passed") throw new Error(lines.join("|"));
}
