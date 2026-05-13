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
        use std::process::Command;
        Command::new("cmd")
            .args(["/c", "clip"])
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
            .args(["-Command", "Get-Clipboard"])
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

        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if std::io::Write::write_all(&mut child.stdin.take().unwrap(), text.as_bytes()).is_ok()
                && child.wait().is_ok()
            {
                return Ok(());
            }
        }

        if let Ok(mut child) = Command::new("xsel")
            .args(["--clipboard", "--input"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if std::io::Write::write_all(&mut child.stdin.take().unwrap(), text.as_bytes()).is_ok()
                && child.wait().is_ok()
            {
                return Ok(());
            }
        }

        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if std::io::Write::write_all(&mut child.stdin.take().unwrap(), text.as_bytes()).is_ok()
                && child.wait().is_ok()
            {
                return Ok(());
            }
        }

        Err(crate::error::EchomindError::Other(
            "No clipboard tool available. Install xclip, xsel, or wl-clipboard.".to_string(),
        ))
    }

    #[cfg(target_os = "linux")]
    fn read_from_clipboard_linux() -> Result<String> {
        use std::process::Command;

        if let Ok(output) = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
                });
            }
        }

        if let Ok(output) = Command::new("xsel")
            .args(["--clipboard", "--output"])
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
                });
            }
        }

        if let Ok(output) = Command::new("wl-paste")
            .args(["--type", "text/plain"])
            .output()
        {
            if !output.stdout.is_empty() {
                return String::from_utf8(output.stdout).map_err(|e| {
                    crate::error::EchomindError::Other(format!("Clipboard parse error: {}", e))
                });
            }
        }

        Err(crate::error::EchomindError::Other(
            "No clipboard tool available. Install xclip, xsel, or wl-clipboard.".to_string(),
        ))
    }

    pub fn copy_to_clipboard(text: &str) -> Result<()> {
        #[cfg(target_os = "macos")]
        { copy_to_clipboard_macos(text) }
        #[cfg(target_os = "windows")]
        { copy_to_clipboard_windows(text) }
        #[cfg(target_os = "linux")]
        { copy_to_clipboard_linux(text) }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { Err(crate::error::EchomindError::Other("Clipboard not supported on this platform".into())) }
    }

    pub fn read_from_clipboard() -> Result<String> {
        #[cfg(target_os = "macos")]
        { read_from_clipboard_macos() }
        #[cfg(target_os = "windows")]
        { read_from_clipboard_windows() }
        #[cfg(target_os = "linux")]
        { read_from_clipboard_linux() }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { Err(crate::error::EchomindError::Other("Clipboard not supported on this platform".into())) }
    }
}
