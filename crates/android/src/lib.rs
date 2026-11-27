//! Android-only support for portable command-line installs.
//!
//! The CLI keeps its writable state beside the executable in `.wordcraft/`: `cache/`,
//! `config/`, `data/`, `state/`, `tmp/`, and `home/` back the corresponding XDG, temporary,
//! and home directories. `fonts/` holds user fonts, `timezone` can set a time zone, and `hosts` is
//! reserved for future hostname mappings. Existing non-empty XDG, home, temporary, and time-zone
//! variables take precedence; the binary directory is prepended to `PATH`.

use std::path::PathBuf;

/// True only when compiled for Android. Compile-time; no probing.
pub fn is_android() -> bool {
    cfg!(target_os = "android")
}

/// `<dir of the running executable>/.wordcraft`, or None if the executable path is unknown.
pub fn portable_dir() -> Option<PathBuf> {
    #[cfg(target_os = "android")]
    {
        return std::env::current_exe().ok()?.parent().map(|dir| dir.join(".wordcraft"));
    }

    #[cfg(not(target_os = "android"))]
    None
}

/// The temporary directory to use: `TMPDIR` when set, else `<portable>/tmp`, else None.
pub fn temp_dir() -> Option<PathBuf> {
    std::env::var_os("TMPDIR").map(PathBuf::from).or_else(|| portable_dir().map(|dir| dir.join("tmp")))
}

/// The system font folders on Android, in scan order. Empty on every other platform.
pub fn system_font_dirs() -> Vec<PathBuf> {
    #[cfg(target_os = "android")]
    {
        let mut dirs = vec![PathBuf::from("/system/fonts")];
        if let Some(portable) = portable_dir() {
            dirs.push(portable.join("fonts"));
        }
        if let Some(home) = std::env::var_os("HOME") {
            dirs.push(PathBuf::from(home).join("fonts"));
        }
        return dirs;
    }

    #[cfg(not(target_os = "android"))]
    Vec::new()
}

/// Android only: re-exec with the portable environment; a no-op on other platforms.
pub fn prepare() -> std::io::Result<()> {
    #[cfg(target_os = "android")]
    {
        return prepare_android();
    }

    #[cfg(not(target_os = "android"))]
    Ok(())
}

#[cfg(target_os = "android")]
fn prepare_android() -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    if std::env::var_os("WORDCRAFT_ANDROID_ENV").is_some() {
        return Ok(());
    }

    let Some(portable) = portable_dir() else {
        return Ok(());
    };
    let tmp = portable.join("tmp");
    if std::fs::create_dir_all(&tmp).is_err() {
        return Ok(());
    }

    let mut vars = Vec::new();
    for (name, dir) in [
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_CONFIG_DIRS", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_STATE_HOME", "state"),
        ("TMPDIR", "tmp"),
        ("HOME", "home"),
    ] {
        if !has_nonempty_env(name) {
            vars.push((name, portable.join(dir).into_os_string()));
        }
    }

    if !has_nonempty_env("TZDIR") {
        let tzdir = PathBuf::from("/system/usr/share/zoneinfo");
        if tzdir.is_dir() {
            vars.push(("TZDIR", tzdir.into_os_string()));
        }
    }

    if !has_nonempty_env("TZ") {
        let timezone = timezone_setting(&portable).or_else(system_timezone).unwrap_or_else(|| "UTC".to_owned());
        vars.push(("TZ", timezone.into()));
    }

    let executable = std::env::current_exe()?;
    let binary_dir = executable.parent().ok_or_else(|| std::io::Error::other("the executable has no parent directory"))?;
    let mut path = vec![binary_dir.to_path_buf()];
    if let Some(existing) = std::env::var_os("PATH").filter(|value| !value.is_empty()) {
        path.extend(std::env::split_paths(&existing));
    }
    if let Ok(path) = std::env::join_paths(path) {
        vars.push(("PATH", path));
    }

    write_if_missing(&portable.join("timezone"), b"# Set a time zone, for example:\n# Asia/Shanghai\n");
    write_if_missing(&portable.join("hosts"), b"# Add extra hostname mappings here.\n");

    let mut command = Command::new(executable);
    command.args(std::env::args_os().skip(1));
    command.envs(vars);
    command.env("WORDCRAFT_ANDROID_ENV", "1");
    Err(command.exec())
}

#[cfg(target_os = "android")]
fn has_nonempty_env(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| !value.is_empty())
}

#[cfg(target_os = "android")]
fn timezone_setting(portable: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(portable.join("timezone"))
        .ok()?
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
}

#[cfg(target_os = "android")]
fn system_timezone() -> Option<String> {
    let output = std::process::Command::new("getprop").arg("persist.sys.timezone").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let timezone = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!timezone.is_empty()).then_some(timezone)
}

#[cfg(target_os = "android")]
fn write_if_missing(path: &std::path::Path, contents: &[u8]) {
    if !path.exists() {
        let _ = std::fs::write(path, contents);
    }
}

#[cfg(all(test, not(target_os = "android")))]
mod tests {
    #[test]
    fn android_support_is_inert_on_other_platforms() {
        assert!(!super::is_android());
        assert!(super::portable_dir().is_none());
        assert!(super::system_font_dirs().is_empty());
        assert!(super::prepare().is_ok());
    }
}
