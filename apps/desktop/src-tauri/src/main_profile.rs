//! Work around locked tauri-runtime 2.11.1 dropping WindowConfig.data_directory
//! in WebviewAttributes::from. Only explicitly configured main profiles use this
//! path; default application startup and native capabilities are unchanged.
use std::path::{Component, Path};
use tauri::utils::config::{Config, WindowConfig};

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

pub fn prepare(config: &mut Config) -> Result<Option<WindowConfig>, &'static str> {
    let Some(main) = config
        .app
        .windows
        .iter_mut()
        .find(|window| window.label == "main" && window.create)
    else {
        return Ok(None);
    };
    let Some(directory) = &main.data_directory else {
        return Ok(None);
    };
    if !safe_relative(directory) {
        return Err("Configured main browser profile must be a safe relative path.");
    }
    let saved = main.clone();
    main.create = false;
    Ok(Some(saved))
}

pub fn create(app: &tauri::App, config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
    let relative = config
        .data_directory
        .as_ref()
        .ok_or("Missing configured main browser profile")?;
    if !safe_relative(relative) {
        return Err("Unsafe configured main browser profile".into());
    }
    let directory = app.path().local_data_dir()?.join("main").join(relative);
    tauri::WebviewWindowBuilder::from_config(app, config)?
        .data_directory(directory)
        .build()?;
    Ok(())
}

use tauri::Manager;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_explicit_safe_main_profiles_change_creation() {
        let mut config = Config::default();
        config.app.windows.push(WindowConfig {
            label: "main".into(),
            ..Default::default()
        });
        assert!(prepare(&mut config).unwrap().is_none());
        assert!(config.app.windows[0].create);
        for unsafe_path in ["../profile", "C:\\profile", "\\\\server\\profile", ""] {
            config.app.windows[0].data_directory = Some(unsafe_path.into());
            assert!(prepare(&mut config).is_err());
            assert!(config.app.windows[0].create);
        }
        config.app.windows[0].data_directory = Some("pytxo-validation/profile".into());
        let main = prepare(&mut config).unwrap().unwrap();
        assert!(main.create);
        assert!(!config.app.windows[0].create);
        assert_eq!(
            main.data_directory.unwrap(),
            Path::new("pytxo-validation/profile")
        );
    }
}
