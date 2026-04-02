//! Synward Desktop App - Tauri-based UI
//!
//! Provides a graphical interface for:
//! - Setup wizard
//! - Validation reports
//! - Settings management
//! - System tray integration
//! - File watcher for AI-generated code validation

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager, WindowEvent,
};

mod commands;
mod config;
mod state;
mod watcher;
mod validation_service;
mod auto_fix;

pub use state::AppState;
pub use watcher::{FileWatcher, WatchConfig, WatchEvent};
pub use validation_service::{ValidationService, ValidationResult, ClassifiedError, ErrorClass, SeverityLevel};
pub use auto_fix::{AutoFixService, FixProposal};

/// Run the Tauri application
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state::AppState::default())
        .setup(|app| {
            // Force window to foreground on startup
            if let Some(window) = app.get_webview_window("main") {
                // Show and focus immediately
                window.show().ok();
                window.set_focus().ok();
                window.set_always_on_top(true).ok();
                window.center().ok();

                // Force foreground with delay
                let win = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    win.set_focus().ok();
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    win.set_focus().ok();
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    win.set_always_on_top(false).ok();
                    win.set_focus().ok();
                });
            }

            // Create system tray
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Synward - Code Validation")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                            window.set_always_on_top(true).ok();
                            let win = window.clone();
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                win.set_always_on_top(false).ok();
                            });
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: tauri::tray::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                            window.set_always_on_top(true).ok();
                            let win = window.clone();
                            std::thread::spawn(move || {
                                std::thread::sleep(std::time::Duration::from_millis(500));
                                win.set_always_on_top(false).ok();
                            });
                        }
                    }
                })
                .build(app)?;

            // Handle window close - minimize to tray instead of quitting
            if let Some(window) = app.get_webview_window("main") {
                window.clone().on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        // Prevent close, hide instead
                        api.prevent_close();
                        #[allow(clippy::unnecessary_cast)]
                        {
                            window.hide().ok();
                        }
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::validate_code,
            commands::get_config,
            commands::save_config,
            commands::get_status,
            commands::start_watcher,
            commands::stop_watcher,
            commands::get_errors,
            commands::clear_errors,
            commands::request_fix,
            commands::apply_fix,
            commands::get_watcher_status,
            commands::list_directory,
            commands::read_file,
            commands::get_workspace_root,
            commands::save_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
