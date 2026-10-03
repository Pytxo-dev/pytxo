import { expect, test } from "@playwright/test";
import { createRecordedOutputChunkReader, createRecordedOutputReader, groupRecordedOutput } from "../src/lib/recorded-output";

test("recorded text removes formatting across chunks without mixing streams", () => {
  const read = createRecordedOutputReader();
  expect(read("stdout", "before\x1b[")).toBe("before");
  expect(read("stderr", "real failure")).toBe("real failure");
  expect(read("stdout", "31mfailed\x1b[0m\x1b[23;80H")).toBe("failed");
  expect(read("stdout", "日本語 λ 🚀\tkept")).toBe("日本語 λ 🚀\tkept");
});

test("OSC links clipboard titles and terminal strings never become output or actions", () => {
  const read = createRecordedOutputReader();
  expect(read("out", "\x1b]8;;https://example.invalid\x1b\\label\x1b]8;;\x1b\\")).toBe("label");
  expect(read("out", "\x1b]52;c;clipboard-data")).toBe("");
  expect(read("out", "\x07visible\x1bPprivate control")).toBe("visible");
  expect(read("out", "\x1b")).toBe("");
  expect(read("out", "\\after")).toBe("after");
  expect(read("out", "\x1b]0;title\x07\x1b(Btext")).toBe("text");
});

test("normalizes CRLF across events and retains progress text without cursor emulation", () => {
  const read = createRecordedOutputReader();
  expect(read("out", "one\r")).toBe("one\n");
  expect(read("out", "\ntwo\rthree\b\x07\x00\n")).toBe("two\nthree\n");
  expect(read("out", "<script>text only</script>")).toBe("<script>text only</script>");
});

test("C1 sequences and cancellation preserve subsequent readable data", () => {
  const read = createRecordedOutputReader();
  expect(read("out", "\x9b32mcolor\x9b0m\x9dtitle\x9cplain")).toBe("colorplain");
  expect(read("out", "\x1b]unterminated\x18recovered")).toBe("recovered");
  const otherObserver = createRecordedOutputReader();
  read("out", "\x1b[");
  expect(otherObserver("out", "31m literal")).toBe("31m literal");
});

test("ConPTY soft wraps join without changing the raw event payloads", () => {
  const read = createRecordedOutputChunkReader();
  const prefix = " ".repeat(68);
  const raw = [prefix + "risk is foun", "\x1b[23;80Hnd.", "bookkeeper", "repeat", "together"];
  const events = raw.map((payload, id) => ({ id, kind: "stdout", payload, ...read("stdout", payload) }));
  expect(groupRecordedOutput(events)).toEqual([{ id: 0, kind: "stdout", readable: prefix + "risk is found.\nbookkeeper\nrepeat\ntogether" }]);
  expect(events.map(event => event.payload)).toEqual(raw);
});

test("split ConPTY cursor sequences survive record and page boundaries", () => {
  for (const split of [0, 1, 2, 5, 8, 9]) {
    const read = createRecordedOutputChunkReader();
    const cursor = "\x1b[23;80H";
    const prefix = " ".repeat(76);
    const payloads = [prefix + "foun" + cursor.slice(0, split), cursor.slice(split) + "nd."];
    const events = payloads.map((payload, id) => ({ id, kind: "stdout", ...read("stdout", payload) }));
    expect(groupRecordedOutput(events)[0].readable).toBe(prefix + "found.");
  }
});

test("output streams retain order and cannot borrow another stream's repeated character", () => {
  const read = createRecordedOutputChunkReader();
  const events = [
    { id: 1, kind: "stdout", payload: " ".repeat(76) + "foun" },
    { id: 2, kind: "stderr", payload: "warning" },
    { id: 3, kind: "stdout", payload: "\x1b[23;80Hnd." },
    { id: 4, kind: "stderr", payload: "\x1b[1;1Hgenuine" },
  ].map(event => ({ ...event, ...read(event.kind, event.payload) }));
  expect(groupRecordedOutput(events).map(row => [row.kind, row.readable])).toEqual([
    ["stdout", " ".repeat(76) + "foun"], ["stderr", "warning"], ["stdout", "d."], ["stderr", "genuine"],
  ]);
});

test("home, relative movement, colors, and plain repeats do not trigger wrap deduplication", () => {
  for (const control of ["", "\x1b[H", "\x1b[1;1H", "\x1b[2C", "\x1b[31m"]) {
    const read = createRecordedOutputReader();
    expect(read("stdout", "foun" + control + "nd.")).toBe("founnd.");
  }
});

test("cursor movement without the matching repeated character is not a continuation", () => {
  const read = createRecordedOutputChunkReader();
  const events = ["first", "\x1b[2;2Hsecond"].map((payload, id) => ({ id, kind: "stdout", ...read("stdout", payload) }));
  expect(groupRecordedOutput(events)[0].readable).toBe("first\nsecond");
});

test("a new cursor operation cancels a pending ConPTY wrap", () => {
  for (const control of ["\x1b[H", "\x1b[2C", "\x1b]0;title\x07", "\b"]) {
    const read = createRecordedOutputChunkReader();
    const prefix = " ".repeat(76);
    const events = [prefix + "foun\x1b[23;80H", control + "nd."].map((payload, id) => ({ id, kind: "stdout", ...read("stdout", payload) }));
    expect(groupRecordedOutput(events)[0].readable).toBe(prefix + "foun\nnd.");
  }
});

test("a matching letter alone does not prove the cursor is at a wrapped row boundary", () => {
  const read = createRecordedOutputChunkReader();
  const events = ["first", "\x1b[2;2Htree"].map((payload, id) => ({ id, kind: "stdout", ...read("stdout", payload) }));
  expect(groupRecordedOutput(events)[0].readable).toBe("first\ntree");
});
