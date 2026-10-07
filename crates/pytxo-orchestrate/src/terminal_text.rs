//! Readable text from recorded PTY output, shared by Desktop and the terminal shell.

use pytxo_store::EventRecord;

/// Drop CSI/OSC terminal sequences and control characters from PTY output,
/// keeping line breaks. ConPTY wraps a full line by moving the cursor back onto
/// the last column and re-emitting that character; the repeat after a cursor
/// move is dropped. Mirrors `stripTerminal` in Desktop's `terminal-text.ts`.
pub fn strip_terminal_text(payload: &str) -> String {
    let mut out = String::with_capacity(payload.len());
    let mut chars = payload.chars().peekable();
    let mut after_cursor_move = false;
    while let Some(c) = chars.next() {
        match c {
            '\u{1b}' => match chars.next() {
                // CSI: parameters until a final byte in '@'..='~'.
                Some('[') => {
                    for c in chars.by_ref() {
                        if ('@'..='~').contains(&c) {
                            after_cursor_move = c == 'H';
                            break;
                        }
                    }
                }
                // OSC: until BEL or ST (ESC \).
                Some(']') => {
                    while let Some(c) = chars.next() {
                        if c == '\u{7}' || (c == '\u{1b}' && chars.next_if_eq(&'\\').is_some()) {
                            break;
                        }
                    }
                }
                _ => {}
            },
            '\n' => {
                out.push('\n');
                after_cursor_move = false;
            }
            '\t' => out.push(' '),
            c if c.is_control() => {}
            c => {
                if !(std::mem::take(&mut after_cursor_move) && out.ends_with(c)) {
                    out.push(c);
                }
            }
        }
    }
    out
}

/// Recorded worker output is one event per line. ConPTY continues a
/// soft-wrapped line in a new event that starts by moving the cursor back onto
/// the last column, so that event joins the previous line instead of starting
/// a new one. Mirrors `joinOutputLines` in Desktop's `terminal-text.ts`.
pub fn join_output_lines<'a>(lines: impl IntoIterator<Item = &'a str>) -> String {
    let mut text = String::new();
    for (index, line) in lines.into_iter().enumerate() {
        if index > 0 && !continues_wrapped_line(line) {
            text.push('\n');
        }
        text.push_str(line);
    }
    text
}

/// `ESC [ row ; col H` at the start of a line event.
fn continues_wrapped_line(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("\u{1b}[") else {
        return false;
    };
    let params = rest
        .find(|c: char| !c.is_ascii_digit() && c != ';')
        .map_or(rest, |end| &rest[..end]);
    params.contains(';') && rest[params.len()..].starts_with('H')
}

/// One readable line per meaningful event. Mirrors `eventLines` in Desktop.
pub fn event_lines(events: &[EventRecord]) -> Vec<String> {
    fn flush(output: &mut Vec<&str>, lines: &mut Vec<String>) {
        let text = strip_terminal_text(&join_output_lines(output.drain(..)));
        lines.extend(
            text.split('\n')
                .filter(|line| !line.trim().is_empty())
                .map(|line| line.trim_end().to_string()),
        );
    }
    let mut lines = Vec::new();
    let mut output = Vec::new();
    for event in events {
        if event.kind == "stdout" || event.kind == "stderr" {
            output.push(event.payload.as_str());
            continue;
        }
        flush(&mut output, &mut lines);
        match event.kind.as_str() {
            "verify" => lines.push(format!("$ {}", strip_terminal_text(&event.payload).trim())),
            "verify-ok" => lines.push("✓ Task checks passed".into()),
            "verify-failed" => lines.push("✗ Task checks failed".into()),
            "agent-cancelled" => lines.push("Stopped".into()),
            _ => {}
        }
    }
    flush(&mut output, &mut lines);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(kind: &str, payload: &str) -> EventRecord {
        EventRecord {
            id: 0,
            agent_id: "a".into(),
            ts: chrono::Utc::now(),
            kind: kind.into(),
            payload: payload.into(),
        }
    }

    #[test]
    fn wrapped_conpty_output_reads_as_the_original_line() {
        let wrapped = "ERROR: {\"mes\u{1b}[23;80Hssage\":\"model is not supported with a ChatGPT \u{1b}[23;80H account.\"}";
        assert_eq!(
            strip_terminal_text(wrapped),
            "ERROR: {\"message\":\"model is not supported with a ChatGPT account.\"}"
        );
    }

    #[test]
    fn each_event_is_a_line_unless_conpty_continues_a_wrap() {
        let lines = event_lines(&[
            event("stdout", "\u{1b}]0;npm\u{7}hello"),
            event("stdout", "wor"),
            // The wrap repeats the last character after moving to the last column.
            event("stdout", "\u{1b}[23;80Hrld"),
            event("verify", "node --test"),
            event("verify-ok", ""),
        ]);
        assert_eq!(
            lines,
            ["hello", "world", "$ node --test", "✓ Task checks passed"]
        );
    }
}
