use std::{ffi::OsString, io, os::windows::ffi::OsStrExt, path::Path};
use winreg::{enums::KEY_SET_VALUE, RegKey};

// A per-user Run value needs no elevation and never changes other startup entries.
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "GoogleTasksDesktop";

// Pass the root explicitly so regression tests use an isolated registry subtree.
pub fn is_enabled(root: &RegKey) -> io::Result<bool> {
    match root
        .open_subkey(RUN_KEY)
        .and_then(|key| key.get_value::<OsString, _>(RUN_VALUE))
    {
        Ok(command) => Ok(!command.is_empty()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

// Quote paths with spaces without a shell, preserving native Windows Unicode paths.
fn startup_command(executable: &Path) -> io::Result<OsString> {
    if !executable.is_absolute() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Startup path must be absolute"));
    }
    let mut command = OsString::from("\"");
    command.push(executable.as_os_str());
    command.push("\"");
    // Windows Run commands are limited to 260 UTF-16 code units, including quotes.
    if command.encode_wide().count() > 260 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Startup path exceeds the Windows Run limit"));
    }
    Ok(command)
}

// Only a user toggle writes the registry; disabling an absent value is idempotent.
pub fn set_enabled(root: &RegKey, enabled: bool, executable: &Path) -> io::Result<()> {
    if enabled {
        let command = startup_command(executable)?;
        let (key, _) = root.create_subkey(RUN_KEY)?;
        return key.set_value(RUN_VALUE, &command);
    }
    match root
        .open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        .and_then(|key| key.delete_value(RUN_VALUE))
    {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};

    // Exercise actual registry persistence without touching the real Windows Run key.
    #[test]
    fn startup_roundtrip_is_quoted_scoped_and_idempotent() {
        let user = RegKey::predef(HKEY_CURRENT_USER);
        let sandbox = format!(r"Software\GoogleTasksDesktopTests\{}", std::process::id());
        let (root, _) = user.create_subkey(&sandbox).unwrap();
        let executable = Path::new(r"C:\Apps\測試 Folder\Google Tasks.exe");
        assert!(!is_enabled(&root).unwrap());
        set_enabled(&root, false, executable).unwrap();
        set_enabled(&root, true, executable).unwrap();
        assert!(is_enabled(&root).unwrap());
        // This sandbox handle also creates the unrelated value checked below.
        let key = root.open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_SET_VALUE).unwrap();
        let command: OsString = key.get_value(RUN_VALUE).unwrap();
        assert_eq!(command, OsString::from(r#""C:\Apps\測試 Folder\Google Tasks.exe""#));
        // Preserve unrelated values when toggling our entry off.
        key.set_value("UnrelatedApp", &"keep").unwrap();
        set_enabled(&root, false, executable).unwrap();
        set_enabled(&root, false, executable).unwrap();
        assert!(!is_enabled(&root).unwrap());
        assert_eq!(key.get_value::<String, _>("UnrelatedApp").unwrap(), "keep");
        assert!(startup_command(Path::new("relative.exe")).is_err());
        assert!(startup_command(Path::new(&format!(r"C:\{}.exe", "a".repeat(260)))).is_err());
        drop(key);
        drop(root);
        user.delete_subkey_all(sandbox).unwrap();
    }
}
