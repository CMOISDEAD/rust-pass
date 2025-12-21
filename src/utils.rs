use std::process::{Command, Stdio};
use std::io::Write;


pub fn copy_to_clipboard(text: &String) -> Result<(), String> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    child
        .stdin
        .as_mut()
        .ok_or("Failed to open stdin")?
        .write_all(text.as_bytes())
        .map_err(|e| e.to_string())?;

    child.wait().map_err(|e| e.to_string())?;

    Ok(())
}
