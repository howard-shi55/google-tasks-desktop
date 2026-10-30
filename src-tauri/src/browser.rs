use tauri::WebviewWindow;
use webview2_com::{
    AcceleratorKeyPressedEventHandler,
    Microsoft::Web::WebView2::Win32::ICoreWebView2Settings3,
};
use windows::{core::Interface, Win32::UI::Input::KeyboardAndMouse::{GetKeyState, VK_MENU}};

// Browser function keys are disabled; Alt+F4 remains the app's close-to-tray action.
fn block_function_key(key: u32, alt_down: bool) -> bool {
    // Win32 virtual-key codes: F1..F24 = 0x70..0x87, F4 = 0x73.
    (0x70..=0x87).contains(&key) && !(key == 0x73 && alt_down)
}

// Apply once to the native WebView, including login pages and subsequent navigation.
pub fn configure(window: &WebviewWindow) -> tauri::Result<()> {
    window.with_webview(|webview| {
        let result = (|| -> windows::core::Result<()> {
            // COM access stays on the WebView's UI thread with valid controller references.
            unsafe {
                let controller = webview.controller();
                let settings = controller.CoreWebView2()?.Settings()?;
                settings.SetAreDefaultContextMenusEnabled(false)?;
                settings.SetAreDevToolsEnabled(false)?;
                settings.SetIsZoomControlEnabled(false)?;
                settings.cast::<ICoreWebView2Settings3>()?
                    .SetAreBrowserAcceleratorKeysEnabled(false)?;

                // Native interception runs before page scripts and preserves editing keys.
                let handler = AcceleratorKeyPressedEventHandler::create(Box::new(|_, args| {
                    if let Some(args) = args {
                        let mut key = 0;
                        args.VirtualKey(&mut key)?;
                        if block_function_key(key, GetKeyState(i32::from(VK_MENU.0)) < 0) {
                            args.SetHandled(true)?;
                        }
                    }
                    Ok(())
                }));
                let mut token = Default::default();
                controller.add_AcceleratorKeyPressed(&handler, &mut token)?;
            }
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("Could not disable browser commands: {error}");
        }
    })
}

#[cfg(test)]
mod tests {
    use super::block_function_key;

    // Check the whole function-key range without capturing normal app/typing keys.
    #[test]
    fn function_keys_are_blocked_but_close_and_editing_are_preserved() {
        for key in 0x70..=0x87 {
            assert!(block_function_key(key, false));
            assert_eq!(block_function_key(key, true), key != 0x73);
        }
        for key in [0x41, 0x43, 0x56, 0x58, 0x5A, 0x25, 0x0D, 0x1B] {
            assert!(!block_function_key(key, false));
            assert!(!block_function_key(key, true));
        }
    }
}
