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

/// One readable line per meaningful event; worker output is split into its
/// lines, joined across PTY chunk boundaries. Mirrors `eventLines` in Desktop.
pub fn event_lines(events: &[EventRecord]) -> Vec<String> {
    let mut lines = Vec::new();
    let mut pending = String::new();
    let flush = |pending: &mut String, lines: &mut Vec<String>| {
        if !pending.trim().is_empty() {
            lines.push(pending.trim_end().to_string());
        }
        pending.clear();
    };
    for event in events {
        if event.kind == "stdout" || event.kind == "stderr" {
            let text = strip_terminal_text(&event.payload);
            let mut parts = text.split('\n');
            pending.push_str(parts.next().unwrap_or_default());
            for part in parts {
                flush(&mut pending, &mut lines);
                pending.push_str(part);
            }
            continue;
        }
        flush(&mut pending, &mut lines);
        match event.kind.as_str() {
            "verify" => lines.push(format!("$ {}", strip_terminal_text(&event.payload).trim())),
            "verify-ok" => lines.push("✓ Task checks passed".into()),
            "verify-failed" => lines.push("✗ Task checks failed".into()),
            "agent-cancelled" => lines.push("Stopped".into()),
            _ => {}
        }
    }
    flush(&mut pending, &mut lines);
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
    fn lines_join_across_chunks_and_mark_checks() {
        let lines = event_lines(&[
            event("stdout", "\u{1b}]0;npm\u{7}hello\r\nwor"),
            event("stdout", "ld\n"),
            event("verify", "node --test"),
            event("verify-ok", ""),
        ]);
        assert_eq!(
            lines,
            ["hello", "world", "$ node --test", "✓ Task checks passed"]
        );
    }
}
