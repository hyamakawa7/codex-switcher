//! Linux desktop startup preferences.
use serde::Serialize;

#[derive(Serialize)]
pub struct StartupSettings {
    launch_at_login: bool,
    start_hidden: bool,
}

#[cfg(target_os = "linux")]
pub mod linux {
    use anyhow::{Context, Result};
    use std::{
        fs,
        io::ErrorKind,
        path::{Path, PathBuf},
    };

    pub fn autostart_file() -> Result<PathBuf> {
        let config = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
            .context("Could not find config directory")?;
        Ok(config.join("autostart/codex-switcher.desktop"))
    }

    pub fn registered(path: &Path) -> Result<bool> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        };
        let mut in_entry = false;
        let mut application = false;
        let mut exec = false;
        let mut enabled = true;
        for line in content.lines().map(str::trim) {
            if line.starts_with('[') {
                in_entry = line == "[Desktop Entry]";
                continue;
            }
            if !in_entry || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "Type" => application = value.trim() == "Application",
                    "Exec" => exec = !value.trim().is_empty(),
                    "Hidden" if value.trim() == "true" => enabled = false,
                    "X-GNOME-Autostart-enabled" if value.trim() == "false" => enabled = false,
                    _ => {}
                }
            }
        }
        Ok(application && exec && enabled)
    }

    pub fn register(path: &Path, executable: &Path, enabled: bool) -> Result<()> {
        if !enabled {
            // A user entry also disables any system-wide entry with the same name.
            return atomic_write(
                path,
                b"[Desktop Entry]\nType=Application\nName=Codex Switcher\nHidden=true\n",
            );
        }
        let executable = executable
            .to_str()
            .context("Executable path is not UTF-8")?;
        anyhow::ensure!(
            !executable.contains(['\n', '\r']),
            "Invalid executable path"
        );
        // Exec quoting is applied before desktop-entry string escaping.
        let mut quoted = String::new();
        for ch in executable.chars() {
            match ch {
                '\\' | '"' | '`' | '$' => {
                    quoted.push('\\');
                    quoted.push(ch);
                }
                '%' => quoted.push_str("%%"),
                _ => quoted.push(ch),
            }
        }
        let quoted = quoted.replace('\\', "\\\\").replace('\t', "\\t");
        atomic_write(path, format!("[Desktop Entry]\nType=Application\nName=Codex Switcher\nExec=\"{quoted}\" --autostart\nIcon=codex-switcher\nTerminal=false\nX-GNOME-Autostart-enabled=true\n").as_bytes())
    }

    pub fn atomic_write(path: &Path, content: &[u8]) -> Result<()> {
        use std::{io::Write, os::unix::fs::OpenOptionsExt};
        let parent = path.parent().context("Missing parent directory")?;
        fs::create_dir_all(parent)?;
        let temporary = parent.join(format!(".codex-switcher-{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(content)?;
            file.sync_all()?;
            fs::rename(&temporary, path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }

    pub fn manual_second_launch(args: &[String]) -> bool {
        !args.iter().skip(1).any(|arg| arg == "--autostart")
    }
}

#[tauri::command]
pub fn get_startup_settings() -> Result<Option<StartupSettings>, String> {
    #[cfg(target_os = "linux")]
    {
        let settings = crate::auth::load_app_settings().map_err(|e| e.to_string())?;
        let path = linux::autostart_file().map_err(|e| e.to_string())?;
        Ok(Some(StartupSettings {
            launch_at_login: linux::registered(&path).map_err(|e| e.to_string())?,
            start_hidden: settings.start_hidden,
        }))
    }
    #[cfg(not(target_os = "linux"))]
    Ok(None)
}

#[tauri::command]
pub fn set_launch_at_login(enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        let path = linux::autostart_file().map_err(|e| e.to_string())?;
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        linux::register(&path, &executable, enabled).map_err(|e| e.to_string())?;
        Ok(enabled)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = enabled;
        Err("Only supported on Linux".into())
    }
}

#[tauri::command]
pub fn set_start_hidden(enabled: bool) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        let mut settings = crate::auth::load_app_settings().map_err(|e| e.to_string())?;
        settings.start_hidden = enabled;
        crate::auth::save_app_settings(&settings).map_err(|e| e.to_string())?;
        Ok(enabled)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = enabled;
        Err("Only supported on Linux".into())
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::linux::*;
    use std::{fs, path::Path};
    struct Temp(std::path::PathBuf);
    impl Temp {
        fn new() -> Self {
            Self(std::env::temp_dir().join(format!("switcher-test-{}", uuid::Uuid::new_v4())))
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn all_four_preferences_are_independent_and_registration_is_repeatable() {
        let temp = Temp::new();
        let path = temp.0.join("autostart/test.desktop");
        assert!(!registered(&path).unwrap());
        for enabled in [true, true, false, false, true] {
            for hidden in [false, true] {
                register(&path, Path::new("/usr/bin/codex-switcher"), enabled).unwrap();
                assert_eq!(registered(&path).unwrap(), enabled);
                let settings = crate::types::AppSettings {
                    start_hidden: hidden,
                    ..Default::default()
                };
                let json = serde_json::to_string(&settings).unwrap();
                assert_eq!(
                    serde_json::from_str::<crate::types::AppSettings>(&json)
                        .unwrap()
                        .start_hidden,
                    hidden
                );
            }
        }
    }
    #[test]
    fn read_and_write_failures_are_reported() {
        let temp = Temp::new();
        fs::create_dir_all(&temp.0).unwrap();
        assert!(registered(&temp.0).is_err());
        assert!(register(&temp.0, Path::new("/app"), true).is_err());
        let file = temp.0.join("file");
        fs::write(&file, "original").unwrap();
        assert!(register(&file.join("bad"), Path::new("/app"), false).is_err());
        assert_eq!(fs::read_to_string(&file).unwrap(), "original");
    }
    #[test]
    fn desktop_flags_and_exec_escaping() {
        let temp = Temp::new();
        let path = temp.0.join("test.desktop");
        register(&path, Path::new("/a b/%x$`\"\\/app"), true).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("%%x"));
        assert!(content.contains("\\\\$"));
        assert!(content.contains(" --autostart\n"));
        fs::write(&path, format!("{content}Hidden=true\n")).unwrap();
        assert!(!registered(&path).unwrap());
        fs::write(&path, format!("{content}X-GNOME-Autostart-enabled=false\n")).unwrap();
        assert!(!registered(&path).unwrap());
    }
    #[test]
    fn second_launch_only_restores_for_manual_invocations() {
        assert!(manual_second_launch(&["app".into()]));
        assert!(!manual_second_launch(&["app".into(), "--autostart".into()]));
        assert!(manual_second_launch(&["app".into(), "--other".into()]));
    }
}
