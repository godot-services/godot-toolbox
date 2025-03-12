use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Result, WebviewUrl, WebviewWindowBuilder, WindowEvent,
};

const WINDOW_ID: &'static str = "main";
const WINDOW_WIDTH: f64 = 440.0;
const WINDOW_HEIGHT: f64 = 700.0;
const DEFAULT_SCREEN_MARGIN: f64 = 10.0;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let is_dev = true;
    if is_dev {
        dev_run();
        return;
    }
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .on_tray_icon_event(|tray_handle, event| {
            tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
        })
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet]);

    // keeps the app out of the dock on mac
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dev_run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .on_tray_icon_event(|tray_handle, event| {
            tauri_plugin_positioner::on_tray_event(tray_handle.app_handle(), &event);
        })
        .setup(|app| {
            let b = WebviewWindowBuilder::new(app, WINDOW_ID, WebviewUrl::default())
                .title("godot-toolbox")
                .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
                .closable(false)
                .fullscreen(false)
                .minimizable(false)
                .maximizable(false)
                .decorations(false)
                .skip_taskbar(true)
                .always_on_top(true)
                .focused(true);

            let b = if let Some(monitor) = app.primary_monitor().unwrap() {
                let size = monitor.size();
                let taskbar_height = get_taskbar_height(app.app_handle())?;
                let x = f64::from(size.width) - WINDOW_WIDTH - DEFAULT_SCREEN_MARGIN;
                let y =
                    f64::from(size.height) - taskbar_height - WINDOW_HEIGHT - DEFAULT_SCREEN_MARGIN;
                b.position(x, y)
            } else {
                b
            };

            let _ = b.build();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet]);

    // keeps the app out of the dock on mac
    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Accessory);

    app.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &App) -> Result<()> {
    let open_item = MenuItem::with_id(app, "open", "Open Toolbox", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit Toolbox", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &quit_item])?;
    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .menu(&menu)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;
    Ok(())
}

fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id.as_ref() {
        "open" => {
            toggle_window(app);
        }
        "quit" => {
            app.exit(0);
        }
        _ => {
            println!("menu item {:?} not handled", event.id);
        }
    }
}

fn on_tray_icon_event(icon: &TrayIcon, event: TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            toggle_window(icon.app_handle());
        }
        _ => {}
    }
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_ID) {
        if window.is_visible().is_ok_and(|is_visible| is_visible) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    } else {
        let _ = setup_window(app);
    }
}

fn setup_window(app: &AppHandle) -> Result<()> {
    let win_builder = WebviewWindowBuilder::new(app, WINDOW_ID, WebviewUrl::default())
        .title("godot-toolbox")
        .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
        .closable(false)
        .fullscreen(false)
        .minimizable(false)
        .maximizable(false)
        .decorations(false)
        .skip_taskbar(true)
        .always_on_top(true)
        .focused(true);

    let win_builder = if let Some(monitor) = app.primary_monitor().unwrap() {
        let size = monitor.size();
        let taskbar_height = get_taskbar_height(app)?;
        let x = f64::from(size.width) - WINDOW_WIDTH - DEFAULT_SCREEN_MARGIN;
        let y = f64::from(size.height) - taskbar_height - WINDOW_HEIGHT - DEFAULT_SCREEN_MARGIN;
        win_builder.position(x, y)
    } else {
        win_builder
    };

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

    let window = win_builder.build().unwrap();
    if !window.is_focused().is_ok_and(|is_focused| is_focused) {
        let _ = window.set_focus();
    }
    let window_handler = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Focused(focused) if !focused => {
            let _ = window_handler.hide();
        }
        _ => {}
    });

    // set background color only when building for macOS
    #[cfg(target_os = "macos")]
    {
        use cocoa::appkit::{NSColor, NSWindow};
        use cocoa::base::{id, nil};

        let ns_window = window.ns_window().unwrap() as id;
        unsafe {
            let bg_color = NSColor::colorWithRed_green_blue_alpha_(
                nil,
                50.0 / 255.0,
                158.0 / 255.0,
                163.5 / 255.0,
                1.0,
            );
            ns_window.setBackgroundColor_(bg_color);
        }
    }

    Ok(())
}

fn get_taskbar_height(app: &AppHandle) -> Result<f64> {
    // FIXME window is briefly visible
    let window =
        WebviewWindowBuilder::new(app, "taskbar-check", WebviewUrl::App("index.html".into()))
            .maximized(true)
            .transparent(true)
            .decorations(false)
            .skip_taskbar(true)
            .visible(false)
            .build()?;
    let window_height = f64::from(window.inner_size()?.height);
    let monitor_height = get_primary_monitor_height(app)?;
    window.close()?;
    Ok(monitor_height - window_height)
}

fn get_primary_monitor_height(app: &AppHandle) -> Result<f64> {
    let monitor = app.primary_monitor()?.unwrap();
    let size = monitor.size();
    Ok(f64::from(size.height))
}
