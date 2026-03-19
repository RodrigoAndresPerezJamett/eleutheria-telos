mod api;
mod db;
mod event_bus;
mod i18n;
mod mcp;
mod plugin_loader;
mod plugins;
mod server;
pub mod tools;

use server::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Arc as StdArc;
use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tokio::sync::{watch, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            // ── Port + token ────────────────────────────────────────────────
            let port = server::find_free_port_sync();
            let session_token = uuid::Uuid::new_v4().to_string();

            // Write server info for the MCP stdio binary to discover.
            write_server_info(port, &session_token);

            // ── Database ────────────────────────────────────────────────────
            let db = tauri::async_runtime::block_on(db::init_db())?;

            let (clipboard_suppress_tx, _) = watch::channel::<u64>(0);
            let download_states = StdArc::new(Mutex::new(HashMap::new()));
            let voice_recording = StdArc::new(Mutex::new(None));
            let screen_recording = StdArc::new(Mutex::new(None));
            let audio_recording = StdArc::new(Mutex::new(None));
            let mcp_sessions: server::McpSessions = StdArc::new(Mutex::new(HashMap::new()));

            // ── Plugin loader ───────────────────────────────────────────────
            let manifests = plugin_loader::scan_plugins();
            log::info!("{} plugin(s) detected at startup", manifests.len());
            let (plugin_registry, plugin_children) =
                plugin_loader::start_plugins(manifests, port, &session_token);
            let plugin_processes = StdArc::new(std::sync::Mutex::new(plugin_children));

            let state = Arc::new(AppState {
                db,
                session_token: session_token.clone(),
                port,
                event_bus: event_bus::EventBus::new(),
                clipboard_suppress_tx,
                download_states,
                voice_recording,
                screen_recording,
                audio_recording,
                mcp_sessions,
                plugin_registry,
                plugin_processes,
            });

            // ── Tauri managed state (for invoke commands) ───────────────────
            app.manage(state.clone());

            // ── Axum server (background task) ───────────────────────────────
            let state_for_server = state.clone();
            tauri::async_runtime::spawn(async move {
                server::start_server(state_for_server, port).await;
            });

            // ── Clipboard monitor (background task) ─────────────────────────
            let state_for_monitor = state.clone();
            tauri::async_runtime::spawn(async move {
                tools::clipboard::start_monitor(state_for_monitor).await;
            });

            // ── i18n ────────────────────────────────────────────────────────
            let i18n = i18n::I18n::load();
            log::info!("i18n ready: {}", i18n.t("app.name"));

            // ── System tray ─────────────────────────────────────────────────
            let show_item = MenuItem::with_id(app, "show", "Show / Hide", true, None::<&str>)?;
            let quit_item =
                MenuItem::with_id(app, "quit", "Quit Eleutheria Telos", true, None::<&str>)?;
            let tray_menu = Menu::with_items(app, &[&show_item, &quit_item])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&tray_menu)
                .tooltip("Eleutheria Telos")
                .on_menu_event(
                    |app: &AppHandle, event: MenuEvent| match event.id.as_ref() {
                        "quit" => app.exit(0),
                        "show" => {
                            if let Some(win) = app.get_webview_window("main") {
                                if win.is_visible().unwrap_or(false) {
                                    let _ = win.hide();
                                } else {
                                    let _ = win.show();
                                    let _ = win.set_focus();
                                }
                            }
                        }
                        _ => {}
                    },
                )
                .build(app)?;

            // ── Main window ──────────────────────────────────────────────────
            // Inject session token and port into the WebView before any script runs.
            let init_script = format!(
                "window.__SESSION_TOKEN__ = '{}'; window.__API_PORT__ = {};",
                session_token, port
            );

            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::App(std::path::PathBuf::from("index.html")),
            )
            .title("Eleutheria Telos")
            .inner_size(1100.0, 700.0)
            .min_inner_size(640.0, 480.0)
            .initialization_script(&init_script)
            .build()?;

            log::info!("Eleutheria Telos started. Server on http://127.0.0.1:{port}");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::get_session_token,
            api::get_api_port,
            api::health_check,
            api::get_config,
        ])
        .on_window_event(|window, event| {
            // Closing the window hides it instead of quitting (tray-resident app).
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ── MCP server info ───────────────────────────────────────────────────────────

/// Writes port + session token to ~/.local/share/eleutheria-telos/server.json
/// so the `eleutheria-mcp` stdio binary can discover the running instance.
fn write_server_info(port: u16, token: &str) {
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".local/share")
        });
    let dir = base.join("eleutheria-telos");
    let _ = std::fs::create_dir_all(&dir);
    let json = serde_json::json!({ "port": port, "token": token });
    let _ = std::fs::write(dir.join("server.json"), json.to_string());
}
