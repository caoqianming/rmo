#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod metrics;
mod state;

use crate::config::Config;
use crate::metrics::{MetricsCollector, MetricsSnapshot};
use crate::state::AppState;
use std::thread;
use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::window::Color;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, Position, State, WebviewWindow};

#[tauri::command]
fn get_config(state: State<'_, AppState>) -> Config {
    state.config.lock().expect("config lock poisoned").clone()
}

#[tauri::command]
fn get_metrics_snapshot(state: State<'_, AppState>) -> MetricsSnapshot {
    state
        .latest_snapshot
        .lock()
        .expect("snapshot lock poisoned")
        .clone()
}

#[tauri::command]
fn update_config(config: Config, state: State<'_, AppState>, app: AppHandle) -> Result<Config, String> {
    let mut next_config = config;
    next_config.clamp();

    if !next_config.any_metric_enabled() {
        return Err("At least one metric must stay enabled.".into());
    }

    next_config.save();

    {
        let mut stored_config = state.config.lock().map_err(|_| "Config state unavailable")?;
        *stored_config = next_config.clone();
    }

    let snapshot = {
        let mut collector = state
            .collector
            .lock()
            .map_err(|_| "Collector state unavailable")?;
        collector.refresh(next_config.show_network, next_config.show_disk_io)
    };

    {
        let mut latest_snapshot = state
            .latest_snapshot
            .lock()
            .map_err(|_| "Snapshot state unavailable")?;
        *latest_snapshot = snapshot.clone();
    }

    let _ = app.emit("metrics://updated", &snapshot);

    Ok(next_config)
}

#[tauri::command]
fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn resize_window_to_content(
    width: f64,
    height: f64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window missing".to_string())?;

    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width, height)))
        .map_err(|error| error.to_string())?;

    let auto_positioning = *state
        .auto_positioning
        .lock()
        .map_err(|_| "Auto positioning state unavailable".to_string())?;

    if auto_positioning {
        position_window(&window).map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn show_main_window(app: AppHandle) -> Result<(), String> {
    show_window(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_auto_positioning(enabled: bool, state: State<'_, AppState>) -> Result<(), String> {
    let mut auto_positioning = state
        .auto_positioning
        .lock()
        .map_err(|_| "Auto positioning state unavailable".to_string())?;
    *auto_positioning = enabled;

    Ok(())
}

fn show_window(app: &AppHandle) -> tauri::Result<()> {
    let window = app.get_webview_window("main").expect("main window missing");
    let state = app.state::<AppState>();
    let auto_positioning = *state.auto_positioning.lock().expect("auto positioning lock poisoned");

    if auto_positioning {
        position_window(&window)?;
    }

    window.show()?;
    window.set_focus()?;

    Ok(())
}

fn tray_icon_image(app: &AppHandle) -> tauri::Result<Image<'static>> {
    if let Some(icon) = app.default_window_icon() {
        return Ok(icon.clone().to_owned());
    }

    Image::from_path("icons/icon.ico").map(|icon| icon.to_owned())
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let tray_exit = MenuItem::with_id(app, "tray_exit", "退出", true, None::<&str>)?;
    let tray_menu = Menu::with_items(app, &[&tray_exit])?;
    let tray_icon = tray_icon_image(app)?;

    TrayIconBuilder::with_id("main-tray")
        .icon(tray_icon)
        .tooltip("rmo")
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_menu_event({
            move |app, event| match event.id.as_ref() {
                "tray_exit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

fn position_window(window: &WebviewWindow) -> tauri::Result<()> {
    let Some(monitor) = window.current_monitor()? else {
        return Ok(());
    };

    let scale_factor = monitor.scale_factor();
    let monitor_size = monitor.size().to_logical::<f64>(scale_factor);
    let window_size = window.outer_size()?.to_logical::<f64>(scale_factor);
    let margin = 12.0;

    let x = (monitor_size.width - window_size.width - margin).max(margin);
    let y = (monitor_size.height - window_size.height - margin).max(margin);

    window.set_position(Position::Physical(PhysicalPosition::new(
        x.round() as i32,
        y.round() as i32,
    )))?;

    Ok(())
}

fn spawn_refresh_loop(app: AppHandle) {
    thread::spawn(move || loop {
        let state = app.state::<AppState>();
        let current_config = state.config.lock().expect("config lock poisoned").clone();
        let snapshot = {
            let mut collector = state.collector.lock().expect("collector lock poisoned");
            collector.refresh(current_config.show_network, current_config.show_disk_io)
        };

        {
            let mut latest_snapshot = state
                .latest_snapshot
                .lock()
                .expect("snapshot lock poisoned");
            *latest_snapshot = snapshot.clone();
        }

        let _ = app.emit("metrics://updated", &snapshot);

        thread::sleep(Duration::from_secs_f64(current_config.refresh_interval_secs));
    });
}

fn main() {
    let initial_config = Config::load();
    let mut collector = MetricsCollector::new();
    let initial_snapshot = collector.refresh(
        initial_config.show_network,
        initial_config.show_disk_io,
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let app = app.clone();
            let _ = show_window(&app);
        }))
        .manage(AppState::new(
            initial_config,
            collector,
            initial_snapshot,
        ))
        .invoke_handler(tauri::generate_handler![
            get_config,
            get_metrics_snapshot,
            update_config,
            exit_app,
            resize_window_to_content,
            show_main_window,
            set_auto_positioning
        ])
        .setup(|app| {
            let window = app.get_webview_window("main").expect("main window missing");
            window.set_background_color(Some(Color(0, 0, 0, 0)))?;
            position_window(&window)?;
            build_tray(app.handle())?;
            spawn_refresh_loop(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running rmo");
}
