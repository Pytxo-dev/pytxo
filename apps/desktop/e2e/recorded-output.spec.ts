import { expect, test } from "@playwright/test";
import { createRecordedOutputReader } from "../src/lib/recorded-output";

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
