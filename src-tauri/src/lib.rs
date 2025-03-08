use tauri::{
    menu::{Menu, MenuEvent, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Result, WebviewUrl, WebviewWindowBuilder,
};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            setup(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &App) -> Result<()> {
    setup_window(app)?;
    setup_tray(app)?;
    Ok(())
}

fn setup_window(app: &App) -> Result<()> {
    let win_builder = WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("godot-toolbox")
        .inner_size(440.0, 700.0)
        .closable(false)
        .fullscreen(false)
        .minimizable(false)
        .maximizable(false)
        .decorations(false)
        .visible(false);

    // set transparent title bar only when building for macOS
    #[cfg(target_os = "macos")]
    let win_builder = win_builder.title_bar_style(TitleBarStyle::Transparent);

    let window = win_builder.build().unwrap();

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
            println!("TODO open menu item was clicked");
            // TODO open app
        }
        "quit" => {
            println!("quit menu item was clicked");
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
            println!("system tray received a left click");
            toggle_window_visibility(icon.app_handle());
        }
        // TrayIconEvent::Click { button: MouseButton::Right, button_state: MouseButtonState::Up, .. } => todo!(),
        // TrayIconEvent::DoubleClick { .. } => todo!(),
        // TrayIconEvent::Enter { .. } => todo!(),
        // TrayIconEvent::Move { .. } => todo!(),
        // TrayIconEvent::Leave { .. } => todo!(),
        _ => {}
    }
}

fn toggle_window_visibility(app: &AppHandle) {
    let window = app.get_webview_window("main").unwrap();
    if !window.is_visible().unwrap() {
        window.show().unwrap();
    } else {
        window.hide().unwrap();
    }
}
