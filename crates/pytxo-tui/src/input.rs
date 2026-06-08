use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

/// Returns true for Enter and Windows `\r` / `\n` character events.
pub fn is_submit_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Enter | KeyCode::Char('\r') | KeyCode::Char('\n')
    )
}

/// Returns true for Esc and q (cancel / quit).
pub fn is_cancel_key(code: KeyCode) -> bool {
    matches!(code, KeyCode::Esc | KeyCode::Char('q'))
}

/// Accept key events for interactive controls (Windows may emit Release for Enter).
pub fn accepts_key_event(key: &KeyEvent) -> bool {
    if key.kind == KeyEventKind::Press {
        return true;
    }
    key.kind == KeyEventKind::Release && is_submit_key(key.code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};

    fn key(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            kind,
            modifiers: crossterm::event::KeyModifiers::empty(),
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn submit_keys_include_enter_and_cr() {
        assert!(is_submit_key(KeyCode::Enter));
        assert!(is_submit_key(KeyCode::Char('\r')));
        assert!(is_submit_key(KeyCode::Char('\n')));
        assert!(!is_submit_key(KeyCode::Char('a')));
    }

    #[test]
    fn accepts_release_for_submit_on_windows() {
        let enter_release = key(KeyCode::Enter, KeyEventKind::Release);
        assert!(accepts_key_event(&enter_release));
        let a_release = key(KeyCode::Char('a'), KeyEventKind::Release);
        assert!(!accepts_key_event(&a_release));
    }
}
