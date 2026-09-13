/** Plain text from recorded PTY chunks, never a terminal emulator.
 * State belongs to one observer and stream, so split escape sequences survive
 * page boundaries without consuming unrelated stderr or changing stored bytes.
 */
export function createRecordedOutputReader() {
  type State = "text" | "escape" | "intermediate" | "csi" | "osc" | "string" | "string-escape";
  const streams = new Map<string, { state: State; stringState: "osc" | "string"; cr: boolean }>();
  return (stream: string, payload: string): string => {
    let cursor = streams.get(stream);
    if (!cursor) { cursor = { state: "text", stringState: "string", cr: false }; streams.set(stream, cursor); }
    let text = "";
    for (const char of payload) {
      const n = char.charCodeAt(0);
      if (n === 0x18 || n === 0x1a) { cursor.state = "text"; continue; }
      if (cursor.state === "string-escape") {
        if (char === "\\") cursor.state = "text";
        else if (n !== 0x1b) cursor.state = cursor.stringState;
        continue;
      }
      if (cursor.state === "osc" || cursor.state === "string") {
        if (n === 0x9c || (n === 7 && cursor.state === "osc")) cursor.state = "text";
        else if (n === 0x1b) { cursor.stringState = cursor.state; cursor.state = "string-escape"; }
        continue;
      }
      if (n === 0x1b) { cursor.state = "escape"; continue; }
      if (cursor.state === "escape") {
        if (char === "[") cursor.state = "csi";
        else if (char === "]") cursor.state = "osc";
        else if (["P", "X", "^", "_"].includes(char)) cursor.state = "string";
        else if (n >= 0x20 && n <= 0x2f) cursor.state = "intermediate";
        else cursor.state = "text";
        continue;
      }
      if (cursor.state === "intermediate") {
        if (n >= 0x30 && n <= 0x7e) cursor.state = "text";
        continue;
      }
      if (cursor.state === "csi") {
        if (n >= 0x40 && n <= 0x7e) cursor.state = "text";
        continue;
      }
      if (n === 0x9b) { cursor.state = "csi"; continue; }
      if (n === 0x9d) { cursor.state = "osc"; continue; }
      if ([0x90, 0x98, 0x9e, 0x9f].includes(n)) { cursor.state = "string"; continue; }
      if (char === "\r") { text += "\n"; cursor.cr = true; continue; }
      if (char === "\n") { if (!cursor.cr) text += char; cursor.cr = false; continue; }
      if (n < 0x20 && char !== "\t" || n >= 0x7f && n <= 0x9f) continue;
      cursor.cr = false;
      text += char;
    }
    return text;
  };
}
