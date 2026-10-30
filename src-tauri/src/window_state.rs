use serde::{Deserialize, Serialize};
use std::io;
use tauri::Window;
use windows::Win32::{
    Foundation::RECT,
    UI::WindowsAndMessaging::{GetWindowPlacement, SetWindowPlacement, SW_SHOWNORMAL, WINDOWPLACEMENT},
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

// Keep geometry separate from the Windows startup Run entry and version its format.
const SETTINGS_KEY: &str = r"Software\GoogleTasksDesktop";
const PLACEMENT_VALUE: &str = "WindowPlacementV1";

// Native normal-placement coordinates remain scalars for stable JSON serialization.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Bounds {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Bounds {
    // Negative monitor coordinates are valid; reject empty or overflowing dimensions.
    fn valid(&self) -> bool {
        let width = i64::from(self.right) - i64::from(self.left);
        let height = i64::from(self.bottom) - i64::from(self.top);
        (1..=i64::from(i32::MAX)).contains(&width)
            && (1..=i64::from(i32::MAX)).contains(&height)
    }
}

// Missing settings mean first launch; malformed settings must not prevent opening.
fn load(root: &RegKey) -> io::Result<Option<Bounds>> {
    let json = match root.open_subkey(SETTINGS_KEY)
        .and_then(|key| key.get_value::<String, _>(PLACEMENT_VALUE))
    {
        Ok(json) => json,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let bounds: Bounds = serde_json::from_str(&json)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    if !bounds.valid() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid saved window bounds"));
    }
    Ok(Some(bounds))
}

// A single JSON registry value avoids a partially written set of four coordinates.
fn store(root: &RegKey, bounds: &Bounds) -> io::Result<()> {
    if !bounds.valid() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid window bounds"));
    }
    let json = serde_json::to_string(bounds).map_err(io::Error::other)?;
    let (key, _) = root.create_subkey(SETTINGS_KEY)?;
    key.set_value(PLACEMENT_VALUE, &json)
}

// Use native normal placement so minimizing/maximizing never saves a collapsed size.
pub fn save(window: &Window) {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        // Ignore hidden startup events and fullscreen so they cannot replace normal bounds.
        if !window.is_visible()? || window.is_fullscreen()? {
            return Ok(());
        }
        let mut placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        // The initialized native structure and live window handle match the Win32 ABI.
        unsafe { GetWindowPlacement(window.hwnd()?, &mut placement)?; }
        let rect = placement.rcNormalPosition;
        store(&RegKey::predef(HKEY_CURRENT_USER), &Bounds {
            left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom,
        })?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Could not save window placement: {error}");
    }
}

// Get/SetWindowPlacement share coordinates and let Windows recover fully offscreen bounds.
pub fn restore(window: &Window) {
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let Some(bounds) = load(&RegKey::predef(HKEY_CURRENT_USER))? else {
            return Ok(());
        };
        let placement = WINDOWPLACEMENT {
            length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
            showCmd: SW_SHOWNORMAL.0 as u32,
            rcNormalPosition: RECT {
                left: bounds.left, top: bounds.top, right: bounds.right, bottom: bounds.bottom,
            },
            ..Default::default()
        };
        // Restore normal geometry, never a minimized startup window or a stale fullscreen size.
        unsafe { SetWindowPlacement(window.hwnd()?, &placement)?; }
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("Could not restore window placement: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winreg::enums::KEY_SET_VALUE;

    // Persist real native-coordinate data in a sandbox, including a monitor left of primary.
    #[test]
    fn bounds_roundtrip_rejects_invalid_data_without_overwriting_good_state() {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let sandbox = format!(r"Software\GoogleTasksDesktopTests\placement-{}", std::process::id());
        let (root, _) = user.create_subkey(&sandbox).unwrap();
        assert_eq!(load(&root).unwrap(), None);
        let bounds = Bounds { left: -1400, top: 50, right: -600, bottom: 650 };
        store(&root, &bounds).unwrap();
        assert_eq!(load(&root).unwrap(), Some(bounds));
        let invalid = Bounds { left: 10, top: 0, right: 10, bottom: 600 };
        assert!(store(&root, &invalid).is_err());
        assert_eq!(load(&root).unwrap().unwrap().left, -1400);
        let overflow = Bounds { left: i32::MIN, top: 0, right: i32::MAX, bottom: 1 };
        assert!(!overflow.valid());
        let key = root.open_subkey_with_flags(SETTINGS_KEY, KEY_SET_VALUE).unwrap();
        key.set_value(PLACEMENT_VALUE, &"invalid JSON").unwrap();
        assert!(load(&root).is_err());
        drop(key);
        drop(root);
        user.delete_subkey_all(sandbox).unwrap();
    }
}
