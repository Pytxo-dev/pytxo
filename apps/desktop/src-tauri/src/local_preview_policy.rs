//! Policy for the approved local preview. This module does not create a webview,
//! navigate, grant IPC, start a server or change the current capability manifest.
//! A preview must use an isolated renderer before these rules are integrated.

#[derive(Debug, Clone)]
pub struct PreviewTarget {
    url: tauri::Url,
}

impl PreviewTarget {
    /// Parse once and show the canonical URL before the user opens it. Errors
    /// deliberately omit the supplied URL, which may contain private query data.
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        if input.len() > 8192 || input.chars().any(char::is_control) {
            return Err("Enter a local HTTP or HTTPS URL without control characters.");
        }
        let url = tauri::Url::parse(input.trim())
            .map_err(|_| "Enter a complete local HTTP or HTTPS URL.")?;
        if !matches!(url.scheme(), "http" | "https") {
            return Err("Local previews support HTTP and HTTPS only.");
        }
        if !url.username().is_empty() || url.password().is_some() {
            return Err("Credentials in preview URLs are not supported.");
        }
        if !matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]")) {
            return Err("Use localhost, 127.0.0.1 or [::1] for a local preview.");
        }
        if url.port() == Some(0) {
            return Err("Choose the port of your running local server.");
        }
        Ok(Self { url })
    }

    pub fn url(&self) -> &tauri::Url {
        &self.url
    }

    /// A redirect, link or script may navigate only within the explicitly opened
    /// scheme/host/port. Another local server requires a new user selection too.
    pub fn allows_navigation(&self, next: &tauri::Url) -> bool {
        matches!(next.scheme(), "http" | "https")
            && next.username().is_empty()
            && next.password().is_none()
            && next.origin() == self.url.origin()
    }
}

/// The preview's containing OS window is insufficient identity: an untrusted
/// child renderer may share it. Control requests must come from the main webview.
pub fn can_control_preview(window_label: &str, webview_label: &str) -> bool {
    window_label == "main" && webview_label == "main"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_explicit_loopback_urls_and_retains_the_selected_route() {
        for input in [
            "http://localhost:5173/",
            "http://127.0.0.1:8080/preview?q=one#route",
            "https://[::1]:8443/",
        ] {
            let target = PreviewTarget::parse(input).unwrap();
            assert_eq!(target.url().as_str(), input);
            assert!(target.allows_navigation(target.url()));
        }
    }

    #[test]
    fn rejects_remote_custom_file_credential_and_malformed_targets() {
        for input in [
            "https://example.com/",
            "http://localhost.evil.test/",
            "http://localhost./",
            "http://0.0.0.0:5173/",
            "http://192.168.1.2/",
            "http://[::ffff:127.0.0.1]/",
            "file:///C:/pytxo/index.html",
            "tauri://localhost/",
            "http://ipc.localhost/",
            "javascript:alert(1)",
            "data:text/html,hello",
            "about:blank",
            "//localhost:5173/",
            "http://user:secret@localhost:5173/",
            "http://user@localhost/",
            "http://localhost:0/",
            "http://local\nhost:5173/",
            "",
        ] {
            assert!(PreviewTarget::parse(input).is_err(), "accepted {input}");
        }
        assert!(PreviewTarget::parse(&format!("http://localhost/{}", "a".repeat(8192))).is_err());
    }

    #[test]
    fn redirects_require_the_exact_selected_origin() {
        let target = PreviewTarget::parse("http://localhost:5173/start").unwrap();
        assert!(target
            .allows_navigation(&tauri::Url::parse("http://localhost:5173/next?q=2#hash").unwrap()));
        for input in [
            "http://localhost:5174/",
            "https://localhost:5173/",
            "http://127.0.0.1:5173/",
            "http://localhost/",
            "https://example.com/",
            "tauri://localhost/",
            "http://ipc.localhost/",
            "about:blank",
            "http://user@localhost:5173/",
            "blob:http://localhost:5173/1234",
        ] {
            assert!(
                !target.allows_navigation(&tauri::Url::parse(input).unwrap()),
                "allowed {input}"
            );
        }
    }

    #[test]
    fn a_child_in_the_main_window_never_inherits_preview_control() {
        assert!(can_control_preview("main", "main"));
        for (window, webview) in [
            ("main", "preview-1"),
            ("preview-1", "main"),
            ("flow", "flow"),
            ("main", ""),
            ("", "main"),
        ] {
            assert!(!can_control_preview(window, webview));
        }
    }
}
