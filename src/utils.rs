use std::{
    env,
    io::Write,
    process::{Command, Stdio},
};

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let session = env::var("XDG_SESSION_TYPE").unwrap_or_default();

    let clipboard_cmd = match session.as_str() {
        "wayland" => "wl-copy",
        "x11" | "" => "xclip",
        other => {
            return Err(format!("Unsupported session type: {}", other));
        }
    };

    let mut child = Command::new(clipboard_cmd)
        .args(if clipboard_cmd == "xclip" {
            &["-selection", "clipboard"][..]
        } else {
            &[]
        })
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {}: {}", clipboard_cmd, e))?;

    child
        .stdin
        .as_mut()
        .ok_or("Failed to open stdin")?
        .write_all(text.as_bytes())
        .map_err(|e| format!("Failed to write to stdin: {}", e))?;

    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for process: {}", e))?;

    if !status.success() {
        return Err(format!("{} exited with error", clipboard_cmd));
    }

    Ok(())
}
