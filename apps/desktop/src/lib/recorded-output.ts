/** Plain text from recorded PTY chunks, never a terminal emulator.
 * State belongs to one observer and stream, so split escape sequences survive
 * page boundaries without consuming unrelated stderr or changing stored bytes.
 */
export function createRecordedOutputChunkReader() {
  type State = "text" | "escape" | "intermediate" | "csi" | "osc" | "string" | "string-escape";
  const streams = new Map<string, { state: State; stringState: "osc" | "string"; cr: boolean; csi: string; last: string; wrap: boolean; width: number; widthKnown: boolean }>();
  return (stream: string, payload: string): RecordedOutputChunk => {
    let cursor = streams.get(stream);
    if (!cursor) { cursor = { state: "text", stringState: "string", cr: false, csi: "", last: "", wrap: false, width: 0, widthKnown: true }; streams.set(stream, cursor); }
    let text = "";
    let continuation = false;
    let newRow = true;
    for (const char of payload) {
      const n = char.charCodeAt(0);
      if (n === 0x18 || n === 0x1a) { cursor.state = "text"; cursor.wrap = false; continue; }
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
        if (char !== "[") cursor.wrap = false;
        if (char === "[") { cursor.state = "csi"; cursor.csi = ""; }
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
        if (n >= 0x40 && n <= 0x7e) {
          // Only unwrap a repeated boundary backed by the preceding ASCII row's
          // full width. Arbitrary repositioning and unknown Unicode cell widths
          // are not enough evidence to delete recorded text.
          const position = /^(\d+);(\d+)$/.exec(cursor.csi);
          if (char !== "m") cursor.wrap = false;
          if (char === "H" && position && Number(position[1]) > 0 && Number(position[2]) > 1 && cursor.widthKnown && Number(position[2]) === cursor.width && cursor.last && cursor.last !== "\n") {
            cursor.wrap = true;
            newRow = true;
          }
          cursor.state = "text";
        } else cursor.csi = cursor.csi.length < 32 ? cursor.csi + char : "invalid";
        continue;
      }
      if (n === 0x9b) { cursor.state = "csi"; cursor.csi = ""; continue; }
      if (n === 0x9d) { cursor.state = "osc"; cursor.wrap = false; continue; }
      if ([0x90, 0x98, 0x9e, 0x9f].includes(n)) { cursor.state = "string"; cursor.wrap = false; continue; }
      if (char === "\r") { text += "\n"; cursor.cr = true; cursor.last = "\n"; cursor.wrap = false; newRow = true; continue; }
      if (char === "\n") { if (!cursor.cr) text += char; cursor.cr = false; cursor.last = "\n"; cursor.wrap = false; newRow = true; continue; }
      if (n < 0x20 && char !== "\t" || n >= 0x7f && n <= 0x9f) { cursor.wrap = false; continue; }
      cursor.cr = false;
      if (newRow) { cursor.width = 0; cursor.widthKnown = true; newRow = false; }
      cursor.width += 1;
      if (n < 0x20 || n > 0x7e) cursor.widthKnown = false;
      if (cursor.wrap) {
        cursor.wrap = false;
        if (char === cursor.last) { if (!text) continuation = true; continue; }
      }
      text += char;
      cursor.last = char;
    }
    return { readable: text, continuation };
  };
}

export interface RecordedOutputChunk { readable: string; continuation: boolean }

export function createRecordedOutputReader() {
  const read = createRecordedOutputChunkReader();
  return (stream: string, payload: string): string => read(stream, payload).readable;
}

/** Presentation only: raw events and their boundaries remain available unchanged. */
export function groupRecordedOutput<T extends RecordedOutputChunk & { id: number; kind: string }>(events: T[]) {
  const rows: { id: number; kind: string; readable: string }[] = [];
  for (const event of events) {
    const previous = rows.at(-1);
    if (previous?.kind === event.kind) {
      const separator = event.continuation || !previous.readable || previous.readable.endsWith("\n") || !event.readable ? "" : "\n";
      previous.readable += separator + event.readable;
    } else rows.push({ id: event.id, kind: event.kind, readable: event.readable });
  }
  return rows;
}
