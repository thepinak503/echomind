/// Cross-platform abstraction layer for system operations
/// Handles Linux, macOS, and Windows compatibility with comprehensive fallbacks
use crate::error::{EchomindError, Result};
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

pub fn get_config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .ok_or_else(|| {
            EchomindError::ConfigError(
                "Unable to determine config directory for your platform".to_string(),
            )
        })
        .map(|p| p.join("echomind"))
}

pub fn get_cache_dir() -> Result<PathBuf> {
    dirs::cache_dir()
        .ok_or_else(|| {
            EchomindError::ConfigError(
                "Unable to determine cache directory for your platform".to_string(),
            )
        })
        .map(|p| p.join("echomind"))
}

pub fn get_data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .ok_or_else(|| {
            EchomindError::ConfigError(
                "Unable to determine data directory for your platform".to_string(),
            )
        })
        .map(|p| p.join("echomind"))
}

pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .ok_or_else(|| EchomindError::ConfigError("Unable to determine home directory".to_string()))
}

pub fn get_config_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| "C:\\Users\\Public".to_string());
        PathBuf::from(appdata).join("echomind").join("config.toml")
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/Shared".to_string());
        PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("echomind")
            .join("config.toml")
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config)
                .join("echomind")
                .join("config.toml")
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/root".to_string());
            PathBuf::from(home)
                .join(".config")
                .join("echomind")
                .join("config.toml")
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("echomind")
            .join("config.toml")
    }
}

pub mod clipboard {
    use crate::error::Result;

    #[cfg(target_os = "macos")]
    fn copy_to_clipboard_macos(text: &str) -> Result<()> {
        use std::process::Command;
        Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(text.as_bytes())?;
                child.wait()?;
                Ok(())
            })
            .map_err(|e| crate::error::EchomindError::Other(format!("Clipboard error: {}", e)))?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn read_from_clipboard_macos() -> Result<String> {
        use std::process::Command;
        let output = Command::new("pbpaste").output().map_err(|e| {
            crate::error::EchomindError::Other(format!("Clipboard read error: {}", e))
        })?;
        String::from_utf8(output.stdout).map_err(|e| {
            crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
        })
    }

    #[cfg(target_os = "windows")]
    fn copy_to_clipboard_windows(text: &str) -> Result<()> {
        use std::os::windows::process::CommandExt;
        use std::process::Command;
        Command::new("cmd")
            .args(&["/c", "clip"])
            .creation_flags(0x08000000)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(text.as_bytes())?;
                child.wait()?;
                Ok(())
            })
            .map_err(|e| crate::error::EchomindError::Other(format!("Clipboard error: {}", e)))?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn read_from_clipboard_windows() -> Result<String> {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(&["-Command", "Get-Clipboard"])
            .output()
            .map_err(|e| {
                crate::error::EchomindError::Other(format!("Clipboard read error: {}", e))
            })?;
        String::from_utf8(output.stdout).map_err(|e| {
            crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
        })
    }

    #[cfg(target_os = "linux")]
    fn copy_to_clipboard_linux(text: &str) -> Result<()> {
        use std::process::Command;

        if let Ok(mut clipboard) = Command::new("xclip")
            .args(&["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Ok(_) =
                std::io::Write::write_all(&mut clipboard.stdin.take().unwrap(), text.as_bytes())
            {
                if let Ok(_) = clipboard.wait() {
                    return Ok(());
                }
            }
        }

        if let Ok(mut clipboard) = Command::new("xsel")
            .args(&["--clipboard", "--input"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Ok(_) =
                std::io::Write::write_all(&mut clipboard.stdin.take().unwrap(), text.as_bytes())
            {
                if let Ok(_) = clipboard.wait() {
                    return Ok(());
                }
            }
        }

        if let Ok(mut clipboard) = Command::new("wl-paste")
            .args(&["--type", "text/plain"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Ok(_) =
                std::io::Write::write_all(&mut clipboard.stdin.take().unwrap(), text.as_bytes())
            {
                if let Ok(_) = clipboard.wait() {
                    return Ok(());
                }
            }
        }

        Err(crate::error::EchomindError::Other(
            "No clipboard tool available. Install xclip, xsel, or wl-paste.".to_string(),
        ))
    }

    #[cfg(target_os = "linux")]
    fn read_from_clipboard_linux() -> Result<String> {
        use std::process::Command;

        if let Ok(output) = Command::new("xclip")
            .args(&["-selection", "clipboard", "-o"])
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
                });
            }
        }

        if let Ok(output) = Command::new("xsel")
            .args(&["--clipboard", "--output"])
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
                });
            }
        }

        if let Ok(output) = Command::new("wl-paste")
            .args(&["--type", "text/plain"])
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
                });
            }
        }

        Err(crate::error::EchomindError::Other(
            "No clipboard tool available. Install xclip, xsel, or wl-paste.".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn copy_to_clipboard_fallback(_text: &str) -> Result<()> {
        Err(crate::error::EchomindError::Other(
            "Clipboard not supported on this platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn read_from_clipboard_fallback() -> Result<String> {
        Err(crate::error::EchomindError::Other(
            "Clipboard not supported on this platform".to_string(),
        ))
    }

    pub fn copy_to_clipboard(text: &str) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            copy_to_clipboard_macos(text)
        }
        #[cfg(target_os = "windows")]
        {
            copy_to_clipboard_windows(text)
        }
        #[cfg(target_os = "linux")]
        {
            copy_to_clipboard_linux(text)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            copy_to_clipboard_fallback(text)
        }
    }

    pub fn read_from_clipboard() -> Result<String> {
        #[cfg(target_os = "macos")]
        {
            read_from_clipboard_macos()
        }
        #[cfg(target_os = "windows")]
        {
            read_from_clipboard_windows()
        }
        #[cfg(target_os = "linux")]
        {
            read_from_clipboard_linux()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            read_from_clipboard_fallback()
        }
    }
}

pub mod terminal {
    use crate::error::Result;

    pub fn is_terminal() -> bool {
        use std::io::IsTerminal;
        std::io::stdout().is_terminal()
    }

    pub fn get_terminal_size() -> Result<(u16, u16)> {
        use crossterm::terminal;
        let (cols, rows) = terminal::size().map_err(|e| {
            crate::error::EchomindError::Other(format!("Failed to get terminal size: {}", e))
        })?;
        Ok((cols, rows))
    }

    pub fn enable_raw_mode() -> std::io::Result<()> {
        use crossterm::terminal;
        terminal::enable_raw_mode()
    }

    pub fn disable_raw_mode() -> std::io::Result<()> {
        use crossterm::terminal;
        terminal::disable_raw_mode()
    }

    pub fn clear_screen() -> Result<()> {
        use crossterm::execute;
        use crossterm::terminal::{Clear, ClearType};
        execute!(std::io::stdout(), Clear(ClearType::All)).map_err(|e| {
            crate::error::EchomindError::Other(format!("Failed to clear screen: {}", e))
        })?;
        Ok(())
    }
}

pub mod system {
    pub fn get_platform() -> &'static str {
        #[cfg(target_os = "windows")]
        {
            "windows"
        }
        #[cfg(target_os = "macos")]
        {
            "macos"
        }
        #[cfg(target_os = "linux")]
        {
            "linux"
        }
        #[cfg(target_os = "android")]
        {
            "android"
        }
        #[cfg(target_os = "ios")]
        {
            "ios"
        }
        #[cfg(target_os = "freebsd")]
        {
            "freebsd"
        }
        #[cfg(target_os = "openbsd")]
        {
            "openbsd"
        }
        #[cfg(target_os = "netbsd")]
        {
            "netbsd"
        }
        #[cfg(target_os = "solaris")]
        {
            "solaris"
        }
        #[cfg(target_os = "illumos")]
        {
            "illumos"
        }
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "android",
            target_os = "ios",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "solaris",
            target_os = "illumos"
        )))]
        {
            "unknown"
        }
    }

    pub fn get_architecture() -> &'static str {
        #[cfg(target_arch = "x86_64")]
        {
            "x86_64"
        }
        #[cfg(target_arch = "aarch64")]
        {
            "aarch64"
        }
        #[cfg(target_arch = "arm")]
        {
            "arm"
        }
        #[cfg(target_arch = "armv7")]
        {
            "armv7"
        }
        #[cfg(target_arch = "i686")]
        {
            "i686"
        }
        #[cfg(target_arch = "x86")]
        {
            "x86"
        }
        #[cfg(target_arch = "wasm32")]
        {
            "wasm32"
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "armv7",
            target_arch = "i686",
            target_arch = "x86",
            target_arch = "wasm32"
        )))]
        {
            "unknown"
        }
    }

    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    pub fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    pub fn is_android() -> bool {
        cfg!(target_os = "android")
    }

    pub fn is_ios() -> bool {
        cfg!(target_os = "ios")
    }

    pub fn is_unix() -> bool {
        cfg!(unix)
    }

    pub fn is_bsd() -> bool {
        #[cfg(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd"))]
        {
            true
        }
        #[cfg(not(any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd")))]
        {
            false
        }
    }
}

pub mod fs {
    use crate::error::Result;
    #[cfg(target_os = "linux")]
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    pub fn create_parent_dirs(path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }

    pub fn expand_tilde(path: &str) -> Result<String> {
        if path.starts_with('~') {
            let home = super::get_home_dir()?;
            Ok(path.replacen('~', home.to_str().unwrap_or("~"), 1))
        } else {
            Ok(path.to_string())
        }
    }

    #[cfg(target_os = "linux")]
    pub fn set_executable(path: &Path) -> Result<()> {
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)?;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    pub fn set_executable(_path: &Path) -> Result<()> {
        Ok(())
    }
}

pub mod audio {
    #[cfg(feature = "voice")]
    pub fn is_voice_available() -> bool {
        true
    }

    #[cfg(not(feature = "voice"))]
    pub fn is_voice_available() -> bool {
        false
    }

    #[cfg(all(feature = "voice", target_os = "windows"))]
    pub fn get_audio_backend() -> &'static str {
        "wasapi"
    }

    #[cfg(all(feature = "voice", target_os = "macos"))]
    pub fn get_audio_backend() -> &'static str {
        "coreaudio"
    }

    #[cfg(all(feature = "voice", target_os = "linux"))]
    pub fn get_audio_backend() -> &'static str {
        if std::env::var("PULSE_SERVER").is_ok() {
            "pulseaudio"
        } else if std::env::var("JACK_DEFAULT_SERVER").is_ok() {
            "jack"
        } else {
            "alsa"
        }
    }

    #[cfg(all(feature = "voice", target_os = "android"))]
    pub fn get_audio_backend() -> &'static str {
        "opensles"
    }

    #[cfg(all(feature = "voice", target_os = "ios"))]
    pub fn get_audio_backend() -> &'static str {
        "audiounit"
    }

    #[cfg(not(feature = "voice"))]
    pub fn get_audio_backend() -> &'static str {
        "unavailable"
    }
}

pub mod network {
    use std::time::Duration;

    pub fn get_recommended_timeout() -> Duration {
        Duration::from_secs(30)
    }

    pub fn get_recommended_pool_size() -> usize {
        #[cfg(target_os = "windows")]
        {
            4
        }
        #[cfg(target_os = "macos")]
        {
            8
        }
        #[cfg(target_os = "linux")]
        {
            8
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            4
        }
    }

    pub fn supports_keep_alive() -> bool {
        true
    }

    pub fn get_user_agent() -> String {
        format!(
            "Echomind/{} ({}) Rust/{}",
            env!("CARGO_PKG_VERSION"),
            super::system::get_platform(),
            std::env::consts::OS
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::system;

    #[test]
    fn test_platform_detection() {
        let platform = system::get_platform();
        assert!(!platform.is_empty());
        println!("Detected platform: {}", platform);
    }

    #[test]
    fn test_architecture_detection() {
        let arch = system::get_architecture();
        assert!(!arch.is_empty());
        println!("Detected architecture: {}", arch);
    }

    #[test]
    fn test_config_dir() {
        let config_dir = get_config_dir();
        assert!(config_dir.is_ok());
        println!("Config directory: {:?}", config_dir);
    }

    #[test]
    fn test_config_path() {
        let config_path = get_config_path();
        println!("Config path: {:?}", config_path);
        assert!(config_path.to_string_lossy().contains("echomind"));
    }
}
