use tauri::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::PageLoadEvent,
    AppHandle, Manager, WebviewWindow, WindowEvent,
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

mod autostart;
mod browser;
mod window_state;

// Restore both the native window and the WebView's last focused input.
fn show_main(window: &WebviewWindow) -> tauri::Result<()> {
    window.show()?;
    window.unminimize()?;
    window.set_focus()?;
    let webview: &tauri::Webview = window.as_ref();
    webview.set_focus()
}

// Native shortcuts preserve browser clipboard permissions and the editor's undo stack.
#[cfg(windows)]
fn send_edit_shortcut(window: &WebviewWindow, letter: u8) -> std::io::Result<()> {
    use windows::Win32::UI::{
        Input::KeyboardAndMouse::{
            GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
            KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU,
            VK_RWIN, VK_SHIFT,
        },
        WindowsAndMessaging::GetForegroundWindow,
    };

    if !matches!(letter, b'Z' | b'Y' | b'X' | b'C' | b'V' | b'A') {
        return Err(std::io::Error::other("Unsupported edit shortcut"));
    }
    let hwnd = window
        .hwnd()
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    // All INPUT fields are initialized; scan code/time/extra info remain zero.
    let key_event = |key, flags| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: flags,
                ..Default::default()
            },
        },
    };
    let key = VIRTUAL_KEY(u16::from(letter));
    let release = [
        key_event(key, KEYEVENTF_KEYUP),
        key_event(VK_CONTROL, KEYEVENTF_KEYUP),
    ];
    let inputs = [
        key_event(VK_CONTROL, KEYBD_EVENT_FLAGS::default()),
        key_event(key, KEYBD_EVENT_FLAGS::default()),
        release[0],
        release[1],
    ];

    // Only send to our foreground window, with no held modifiers to distort the command.
    unsafe {
        if GetForegroundWindow() != hwnd
            || [VK_CONTROL, VK_SHIFT, VK_MENU, VK_LWIN, VK_RWIN]
                .iter()
                .any(|key| GetAsyncKeyState(i32::from(key.0)) < 0)
        {
            return Err(std::io::Error::other(
                "Edit shortcut requires main-window focus and released modifiers",
            ));
        }
        let size = std::mem::size_of::<INPUT>() as i32;
        if SendInput(&inputs, size) != inputs.len() as u32 {
            // Release our keys after partial injection; never retry a possibly-applied edit.
            SendInput(&release, size);
            return Err(std::io::Error::other(
                "Windows could not send the edit shortcut",
            ));
        }
    }
    Ok(())
}

// Tray-owned predefined actions target a hidden HWND, so route actions to main explicitly.
fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    if event.id() == "quit" {
        if let Some(window) = app.get_webview_window("main") {
            // In Tauri 2.0, AsRef targets the WebView; placement needs its native host.
            window_state::save(&window.as_ref().window());
        }
        app.exit(0);
        return;
    }
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    let result = match event.id().as_ref() {
        "show" => show_main(&window),
        "hide" => {
            window_state::save(&window.as_ref().window());
            window.hide()
        }
        "minimize" => window.minimize(),
        "center" => window.center(),
        "fullscreen" => {
            // Preserve normal placement before borderless fullscreen changes native geometry.
            window_state::save(&window.as_ref().window());
            window.is_fullscreen().and_then(|full| window.set_fullscreen(!full))
        }
        // Keep the app available while Google completes its sign-out navigation.
        "signOut" => show_main(&window).and_then(|()| {
            window.eval("window.location.href = 'https://mail.google.com/mail/logout?hl=en';")
        }),
        #[cfg(windows)]
        "undo" | "redo" | "cut" | "copy" | "paste" | "selectAll" => {
            let letter = match event.id().as_ref() {
                "undo" => b'Z',
                "redo" => b'Y',
                "cut" => b'X',
                "copy" => b'C',
                "paste" => b'V',
                _ => b'A',
            };
            // Queue after the tray menu closes, then focus before sending the shortcut.
            app.run_on_main_thread(move || {
                if let Err(error) = show_main(&window) {
                    eprintln!("Could not focus task editor: {error}");
                } else if let Err(error) = send_edit_shortcut(&window, letter) {
                    eprintln!("Could not edit task: {error}");
                }
            })
        }
        _ => Ok(()),
    };
    if let Err(error) = result {
        eprintln!("Tray action failed: {error}");
    }
}

// A checked item auto-toggles before its event; commit that state or restore reality.
fn update_autostart(item: &CheckMenuItem<tauri::Wry>) {
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        autostart::set_enabled(&root, item.is_checked()?, &std::env::current_exe()?)?;
        item.set_checked(autostart::is_enabled(&root)?)?;
        item.set_text("Start at login")?;
        Ok(())
    })();
    if let Err(error) = result {
        // Do not leave a successful-looking checkmark after a failed registry write.
        let _ = item.set_checked(autostart::is_enabled(&root).unwrap_or(false));
        let _ = item.set_text("Start at login (update failed)");
        eprintln!("Could not update startup setting: {error}");
    }
}

// Keep common actions at the top level and advanced actions in compact submenus.
fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let window = app.get_webview_window("main")
        .ok_or_else(|| std::io::Error::other("Main window is missing"))?;
    browser::configure(&window)?;
    // Reading startup state must not enable it or prevent the rest of the app from opening.
    let startup_state = autostart::is_enabled(&RegKey::predef(HKEY_CURRENT_USER));
    let startup = CheckMenuItem::with_id(
        app, "autostart", "Start at login", startup_state.is_ok(),
        startup_state.unwrap_or_else(|error| {
            eprintln!("Could not read startup setting: {error}");
            false
        }), None::<&str>,
    )?;
    let menu = Menu::with_items(app, &[
        &MenuItem::with_id(app, "show", "Show / Restore", true, None::<&str>)?,
        &MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?,
        &MenuItem::with_id(app, "center", "Center", true, None::<&str>)?,
        &PredefinedMenuItem::separator(app)?,
        &startup,
        &PredefinedMenuItem::separator(app)?,
        &Submenu::with_items(app, "Edit", true, &[
            &MenuItem::with_id(app, "undo", "Undo", true, None::<&str>)?,
            &MenuItem::with_id(app, "redo", "Redo", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "cut", "Cut", true, None::<&str>)?,
            &MenuItem::with_id(app, "copy", "Copy", true, None::<&str>)?,
            &MenuItem::with_id(app, "paste", "Paste", true, None::<&str>)?,
            &MenuItem::with_id(app, "selectAll", "Select All", true, None::<&str>)?,
        ])?,
        &Submenu::with_items(app, "Window", true, &[
            &MenuItem::with_id(app, "minimize", "Minimize", true, None::<&str>)?,
            &MenuItem::with_id(app, "fullscreen", "Toggle Full Screen", true, None::<&str>)?,
        ])?,
        &Submenu::with_items(app, "More", true, &[
            &PredefinedMenuItem::about(app, None, Default::default())?,
            &MenuItem::with_id(app, "signOut", "Sign Out", true, None::<&str>)?,
        ])?,
        &PredefinedMenuItem::separator(app)?,
        &MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?,
    ])?;
    let icon = app.default_window_icon().cloned()
        .ok_or_else(|| std::io::Error::other("Application icon is missing"))?;
    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("Google Tasks")
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(move |app, event| {
            // Startup stays native-only; the remote page receives no registry permission.
            if event.id() == "autostart" {
                update_autostart(&startup);
            } else {
                on_menu_event(app, event);
            }
        })
        .on_tray_icon_event(|tray, event| {
            // A left click always restores instead of toggling twice on a double-click.
            if matches!(event, TrayIconEvent::Click {
                button: MouseButton::Left, button_state: MouseButtonState::Up, ..
            }) {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    if let Err(error) = show_main(&window) {
                        eprintln!("Could not restore task window: {error}");
                    }
                }
            }
        })
        .build(app)?;
    // Restore only after tray setup so the hidden-on-create window is always recoverable.
    window_state::restore(&window.as_ref().window());
    window.show()?;
    Ok(())
}

// Google navigates from /embed/ to /embed/list/<id>; both are trusted embed routes.
fn is_tasks_embed_url(url: &tauri::Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("tasks.google.com")
        && url.path().starts_with("/embed/")
}

// Theme only Tasks; keep native window lifecycle and tray actions outside remote IPC.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(setup_tray)
        .on_page_load(|webview, payload| {
            let url = payload.url();
            // Never alter Google sign-in pages or other destinations.
            if matches!(payload.event(), PageLoadEvent::Finished)
                && is_tasks_embed_url(url)
            {
                if let Err(error) = webview.eval(include_str!("theme.js")) {
                    eprintln!("Could not inject Google Tasks theme: {error}");
                }
            }
        })
        .on_window_event(|window, event| {
            // Closing must not strand a frameless app; Quit in the tray exits explicitly.
            if let WindowEvent::CloseRequested { api, .. } = event {
                window_state::save(window);
                api.prevent_close();
                if let Err(error) = window.hide() {
                    eprintln!("Could not hide task window: {error}");
                }
            } else if matches!(event, WindowEvent::Focused(false)) {
                // Save after interaction, not on every pixel of a move/resize operation.
                window_state::save(window);
            }
        })
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::is_tasks_embed_url;

    // Exercise the real outer injection guard, including the observed regression URL.
    #[test]
    fn tasks_embed_routes_are_scoped_to_the_exact_https_host() {
        for address in [
            "https://tasks.google.com/embed/?authuser=0",
            "https://tasks.google.com/embed/list/~default",
            "https://tasks.google.com/embed/list/another-list?authuser=1",
        ] {
            assert!(is_tasks_embed_url(&tauri::Url::parse(address).unwrap()));
        }
        for address in [
            "http://tasks.google.com/embed/",
            "https://tasks.google.com.evil.example/embed/list/~default",
            "https://accounts.google.com/",
            "https://tasks.google.com/tasks/",
            "https://tasks.google.com/embed-evil/",
        ] {
            assert!(!is_tasks_embed_url(&tauri::Url::parse(address).unwrap()));
        }
    }

    // Windows fetch IPC supplies only the Origin header, not /embed/list/<id>.
    #[test]
    fn drag_capability_accepts_the_ipc_origin_but_no_other_host() {
        let capability: serde_json::Value =
            serde_json::from_str(include_str!("../capabilities/tasks-drag.json")).unwrap();
        // This pinned Tauri type implements FromStr, not Serde deserialization.
        let patterns: Vec<tauri::utils::acl::RemoteUrlPattern> = capability["remote"]["urls"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pattern| pattern.as_str().unwrap().parse().unwrap())
            .collect();
        for address in [
            "https://tasks.google.com/",
            "https://tasks.google.com/embed/list/~default",
        ] {
            let url = tauri::Url::parse(address).unwrap();
            assert!(patterns.iter().any(|pattern| pattern.test(&url)));
        }
        for address in [
            "http://tasks.google.com/",
            "https://accounts.google.com/",
            "https://tasks.google.com.evil.example/",
        ] {
            let url = tauri::Url::parse(address).unwrap();
            assert!(!patterns.iter().any(|pattern| pattern.test(&url)));
        }
        assert_eq!(
            capability["permissions"],
            serde_json::json!(["core:window:allow-start-dragging"])
        );
    }
}
