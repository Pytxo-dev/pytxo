//! Raw, UI-thread-owned renderer. Never register this child with Tauri or attach
//! a command callback/custom protocol. URL input is not process-start authority.
use crate::ipc_error::{IpcResult, PytxoIpcError};
use crate::local_preview_policy::{can_control_preview, PreviewTarget};
use serde::{Deserialize, Serialize};
use tauri::{Manager, Webview};

fn error(message: impl ToString) -> PytxoIpcError {
    PytxoIpcError::new("local_preview", message.to_string())
}

#[derive(Clone, Copy, Deserialize)]
pub struct PreviewBounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    viewport_width: f64,
    viewport_height: f64,
}
impl PreviewBounds {
    fn valid(self) -> bool {
        [self.x, self.y, self.width, self.height, self.viewport_width, self.viewport_height]
            .iter().all(|v| v.is_finite())
            && self.width >= 80.0 && self.height >= 60.0
            && self.x >= 0.0 && self.y >= 100.0
            && self.x + self.width <= self.viewport_width + 1.0
            && self.y + self.height <= self.viewport_height + 1.0
            // The native child cannot occupy the mission action area.
            && (self.x >= self.viewport_width * 0.5 || self.y >= 380.0)
    }
}

#[derive(Serialize)]
pub struct PreviewInfo {
    id: String,
    url: String,
    storage_verified: bool,
    failure: Option<String>,
    blocked_requests: u32,
}

fn authorize(webview: &Webview) -> IpcResult<()> {
    if !can_control_preview(webview.window().label(), webview.label()) {
        return Err(error(
            "Only the main Pytxo workspace can control a preview.",
        ));
    }
    Ok(())
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::{
        cell::{Cell, RefCell},
        collections::HashMap,
        path::PathBuf,
        rc::Rc,
        time::Instant,
    };
    use webview2_com::{
        AcceleratorKeyPressedEventHandler, CoTaskMemPWSTR, Microsoft::Web::WebView2::Win32::*,
        MoveFocusRequestedEventHandler, NavigationCompletedEventHandler,
        NavigationStartingEventHandler, PermissionRequestedEventHandler, ProcessFailedEventHandler,
        WebResourceRequestedEventHandler,
    };
    use windows_core::{w, Interface, PWSTR};
    use wry::{WebViewBuilderExtWindows, WebViewExtWindows};

    struct Preview {
        // Drop the view before its context. Neither leaves the UI thread.
        view: wry::WebView,
        _context: wry::WebContext,
        url: String,
        failure: Rc<RefCell<Option<String>>>,
        lease: Instant,
        revision: u64,
        blocked: Rc<Cell<u32>>,
    }
    thread_local! { static VIEWS: RefCell<HashMap<String, Preview>> = RefCell::new(HashMap::new()); }

    pub fn hide_all() {
        VIEWS.with(|views| {
            for preview in views.borrow().values() {
                let _ = preview.view.set_visible(false);
            }
        });
    }

    pub fn expire() {
        VIEWS.with(|views| {
            for preview in views.borrow().values() {
                if preview.lease.elapsed().as_millis() > 750 {
                    let _ = preview.view.set_visible(false);
                }
            }
        });
    }

    pub fn open(window: &tauri::Window, target: PreviewTarget) -> IpcResult<PreviewInfo> {
        if VIEWS.with(|views| views.borrow().len() >= 4) {
            return Err(error("Four local previews are open. Close one first."));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let directory = window
            .app_handle()
            .path()
            .app_cache_dir()
            .map_err(error)?
            .join("isolated-previews")
            .join(&id);
        std::fs::create_dir_all(&directory).map_err(error)?;
        let expected = std::fs::canonicalize(&directory).map_err(error)?;
        let mut context = wry::WebContext::new(Some(directory));
        let navigation = target.clone();
        let failure = Rc::new(RefCell::new(None));
        let blocked = Rc::new(Cell::new(0u32));
        let navigation_blocked = blocked.clone();
        let view = wry::WebViewBuilder::new_with_web_context(&mut context)
            .with_visible(false)
            .with_focused(false)
            .with_devtools(false)
            .with_hotkeys_zoom(false)
            .with_clipboard(false)
            .with_general_autofill_enabled(false)
            .with_default_context_menus(false)
            .with_browser_accelerator_keys(false)
            .with_browser_extensions_enabled(false)
            .with_incognito(true)
            .with_navigation_handler(move |url| {
                let allowed =
                    tauri::Url::parse(&url).is_ok_and(|url| navigation.allows_navigation(&url));
                if !allowed {
                    navigation_blocked.set(navigation_blocked.get().saturating_add(1));
                }
                allowed
            })
            .with_new_window_req_handler(|_, _| wry::NewWindowResponse::Deny)
            .with_download_started_handler(|_, _| false)
            // Do not set a URL until native policies and actual storage are verified.
            .build_as_child(window)
            .map_err(error)?;
        // SAFETY: all COM objects are used on the thread which created this child.
        // Failing any required native setting drops the still-hidden blank child.
        unsafe {
            let environment: ICoreWebView2Environment7 =
                view.environment().cast().map_err(error)?;
            let mut raw = PWSTR::null();
            environment.UserDataFolder(&mut raw).map_err(error)?;
            let actual = CoTaskMemPWSTR::from(raw).to_string();
            if std::fs::canonicalize(PathBuf::from(actual)).map_err(error)? != expected {
                return Err(error(
                    "Preview storage isolation could not be verified. The page was not opened.",
                ));
            }
            let core = view.webview();
            let settings = core.Settings().map_err(error)?;
            settings.SetAreHostObjectsAllowed(false).map_err(error)?;
            settings.SetIsWebMessageEnabled(false).map_err(error)?;
            let settings4: ICoreWebView2Settings4 = settings.cast().map_err(error)?;
            settings4
                .SetIsPasswordAutosaveEnabled(false)
                .map_err(error)?;
            let mut token = 0;
            let frame_target = target.clone();
            core.add_FrameNavigationStarting(
                &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                    if let Some(args) = args {
                        let mut raw = PWSTR::null();
                        args.Uri(&mut raw)?;
                        let url = CoTaskMemPWSTR::from(raw).to_string();
                        args.SetCancel(
                            !tauri::Url::parse(&url)
                                .is_ok_and(|url| frame_target.allows_navigation(&url)),
                        )?;
                    }
                    Ok(())
                })),
                &mut token,
            )
            .map_err(error)?;
            // Include frame and worker resource requests. No fallback to an older
            // filter which omits workers: an unsupported runtime fails closed.
            let core22: ICoreWebView2_22 = core.cast().map_err(error)?;
            core22
                .AddWebResourceRequestedFilterWithRequestSourceKinds(
                    w!("*"),
                    COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
                    COREWEBVIEW2_WEB_RESOURCE_REQUEST_SOURCE_KINDS_ALL,
                )
                .map_err(error)?;
            let resource_target = target.clone();
            let resources_blocked = blocked.clone();
            let environment = view.environment();
            core.add_WebResourceRequested(
                &WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
                    if let Some(args) = args {
                        let mut raw = PWSTR::null();
                        args.Request()?.Uri(&mut raw)?;
                        let url = CoTaskMemPWSTR::from(raw).to_string();
                        if !tauri::Url::parse(&url)
                            .is_ok_and(|url| resource_target.allows_navigation(&url))
                        {
                            resources_blocked.set(resources_blocked.get().saturating_add(1));
                            let response = environment.CreateWebResourceResponse(
                                None,
                                403,
                                w!("Preview boundary"),
                                w!("Content-Type: text/plain\r\nCache-Control: no-store"),
                            )?;
                            args.SetResponse(&response)?;
                        }
                    }
                    Ok(())
                })),
                &mut token,
            )
            .map_err(error)?;
            core.add_PermissionRequested(
                &PermissionRequestedEventHandler::create(Box::new(|_, args| {
                    if let Some(args) = args {
                        args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                    }
                    Ok(())
                })),
                &mut token,
            )
            .map_err(error)?;
            let failed = failure.clone();
            let controller = view.controller();
            core.add_ProcessFailed(
                &ProcessFailedEventHandler::create(Box::new(move |_, _| {
                    *failed.borrow_mut() =
                        Some("The preview renderer stopped. Close and reopen this preview.".into());
                    controller.SetIsVisible(false)?;
                    Ok(())
                })),
                &mut token,
            )
            .map_err(error)?;
            let load_failure = failure.clone();
            core.add_NavigationCompleted(&NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                if let Some(args) = args {
                    let mut success = Default::default();
                    args.IsSuccess(&mut success)?;
                    if !success.as_bool() {
                        *load_failure.borrow_mut() = Some("The local page did not finish loading. Check the server address or close and reopen the preview.".into());
                    }
                }
                Ok(())
            })), &mut token).map_err(error)?;
            let main_window = window
                .app_handle()
                .get_webview_window("main")
                .ok_or_else(|| error("Main workspace is unavailable."))?;
            let main: Webview = main_window.as_ref().clone();
            let main_tab = main.clone();
            view.controller()
                .add_MoveFocusRequested(
                    &MoveFocusRequestedEventHandler::create(Box::new(move |_, args| {
                        if let Some(args) = args {
                            args.SetHandled(true)?;
                        }
                        let _ = main_tab.set_focus();
                        Ok(())
                    })),
                    &mut token,
                )
                .map_err(error)?;
            view.controller()
                .add_AcceleratorKeyPressed(
                    &AcceleratorKeyPressedEventHandler::create(Box::new(move |_, args| {
                        if let Some(args) = args {
                            let mut key = 0;
                            args.VirtualKey(&mut key)?;
                            if key == 0x75 {
                                // F6: leave the page, without relying on its JavaScript.
                                args.SetHandled(true)?;
                                let _ = main.set_focus();
                            }
                        }
                        Ok(())
                    })),
                    &mut token,
                )
                .map_err(error)?;
        }
        view.load_url(target.url().as_str()).map_err(error)?;
        let url = target.url().to_string();
        VIEWS.with(|views| {
            views.borrow_mut().insert(
                id.clone(),
                Preview {
                    view,
                    _context: context,
                    url: url.clone(),
                    failure,
                    lease: Instant::now(),
                    revision: 0,
                    blocked,
                },
            )
        });
        Ok(PreviewInfo {
            id,
            url,
            storage_verified: true,
            failure: None,
            blocked_requests: 0,
        })
    }

    pub fn sync(
        window: &tauri::Window,
        id: &str,
        bounds: Option<PreviewBounds>,
        revision: u64,
    ) -> IpcResult<PreviewInfo> {
        VIEWS.with(|views| {
            let mut views = views.borrow_mut();
            let preview = views
                .get_mut(id)
                .ok_or_else(|| error("This preview has closed."))?;
            let failure = preview.failure.borrow().clone();
            if revision <= preview.revision {
                return Ok(PreviewInfo {
                    id: id.into(),
                    url: preview.url.clone(),
                    storage_verified: true,
                    failure,
                    blocked_requests: preview.blocked.get(),
                });
            }
            preview.revision = revision;
            if let Some(bounds) = bounds.filter(|_| failure.is_none()) {
                let size = window.inner_size().map_err(error)?;
                // Physical conversion uses the actual native client size, including
                // browser zoom and Windows scaling; never trust a supplied DPR.
                let sx = size.width as f64 / bounds.viewport_width;
                let sy = size.height as f64 / bounds.viewport_height;
                if !(0.25..=8.0).contains(&sx) || !(0.25..=8.0).contains(&sy) {
                    preview.view.set_visible(false).map_err(error)?;
                    return Err(error("Preview geometry is unavailable."));
                }
                preview
                    .view
                    .set_bounds(wry::Rect {
                        position: wry::dpi::PhysicalPosition::new(
                            (bounds.x * sx).round() as i32,
                            (bounds.y * sy).round() as i32,
                        )
                        .into(),
                        size: wry::dpi::PhysicalSize::new(
                            (bounds.width * sx).floor() as u32,
                            (bounds.height * sy).floor() as u32,
                        )
                        .into(),
                    })
                    .map_err(error)?;
                preview.lease = Instant::now();
                preview
                    .view
                    .set_visible(
                        window.is_visible().map_err(error)?
                            && !window.is_minimized().map_err(error)?,
                    )
                    .map_err(error)?;
            } else {
                preview.view.set_visible(false).map_err(error)?;
            }
            Ok(PreviewInfo {
                id: id.into(),
                url: preview.url.clone(),
                storage_verified: true,
                failure,
                blocked_requests: preview.blocked.get(),
            })
        })
    }

    pub fn close(id: &str) {
        VIEWS.with(|views| {
            views.borrow_mut().remove(id);
        });
    }
}

pub fn on_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    #[cfg(windows)]
    if window.label() == "main"
        && matches!(
            event,
            tauri::WindowEvent::CloseRequested { .. }
                | tauri::WindowEvent::Resized(_)
                | tauri::WindowEvent::ScaleFactorChanged { .. }
        )
    {
        native::hide_all();
    }
    #[cfg(not(windows))]
    let _ = (window, event);
}

pub fn start_expiry(app: tauri::AppHandle) {
    #[cfg(windows)]
    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            if app.run_on_main_thread(native::expire).is_err() {
                break;
            }
        }
    });
    #[cfg(not(windows))]
    let _ = app;
}

async fn on_ui<T: Send + 'static>(
    webview: Webview,
    action: impl FnOnce(tauri::Window) -> IpcResult<T> + Send + 'static,
) -> IpcResult<T> {
    authorize(&webview)?;
    let window = webview.window();
    let (tx, rx) = tokio::sync::oneshot::channel();
    webview
        .app_handle()
        .run_on_main_thread(move || {
            let _ = tx.send(action(window));
        })
        .map_err(error)?;
    rx.await.map_err(error)?
}

#[tauri::command]
pub async fn local_preview_open(webview: Webview, url: String) -> IpcResult<PreviewInfo> {
    authorize(&webview)?;
    let target = PreviewTarget::parse(&url).map_err(error)?;
    on_ui(webview, move |window| {
        #[cfg(windows)]
        {
            native::open(&window, target)
        }
        #[cfg(not(windows))]
        {
            let _ = (window, target);
            Err(error("Local previews currently require Windows Desktop."))
        }
    })
    .await
}

#[tauri::command]
pub async fn local_preview_sync(
    webview: Webview,
    id: String,
    bounds: Option<PreviewBounds>,
    revision: u64,
) -> IpcResult<PreviewInfo> {
    on_ui(webview, move |window| {
        let bounds = bounds.filter(|bounds| bounds.valid());
        #[cfg(windows)]
        {
            native::sync(&window, &id, bounds, revision)
        }
        #[cfg(not(windows))]
        {
            let _ = (window, id, bounds, revision);
            Err(error("Local previews currently require Windows Desktop."))
        }
    })
    .await
}

#[tauri::command]
pub async fn local_preview_close(webview: Webview, id: String) -> IpcResult<()> {
    on_ui(webview, move |_| {
        #[cfg(windows)]
        native::close(&id);
        #[cfg(not(windows))]
        let _ = id;
        Ok(())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_bounds_reserve_mission_controls_and_reject_invalid_rectangles() {
        let valid = PreviewBounds {
            x: 820.0,
            y: 280.0,
            width: 440.0,
            height: 500.0,
            viewport_width: 1280.0,
            viewport_height: 800.0,
        };
        assert!(valid.valid());
        assert!(PreviewBounds {
            x: 80.0,
            y: 430.0,
            width: 1180.0,
            height: 350.0,
            ..valid
        }
        .valid());
        for invalid in [
            PreviewBounds { x: 40.0, ..valid },
            PreviewBounds { y: 20.0, ..valid },
            PreviewBounds {
                width: 600.0,
                ..valid
            },
            PreviewBounds {
                height: f64::NAN,
                ..valid
            },
            PreviewBounds {
                viewport_width: 0.0,
                ..valid
            },
        ] {
            assert!(!invalid.valid());
        }
    }
}
