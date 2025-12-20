use core::fmt;
use std::{env, path::PathBuf};
use std::{path::Path, process::Command};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct PasswordFile {
    pub relative: PathBuf,
    pub absolute: PathBuf,
}

impl std::fmt::Display for PasswordFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.relative.display())
    }
}

pub fn get_password_files() -> Vec<PasswordFile> {
    let home = env::var("HOME")
        .map(PathBuf::from)
        .expect("No se pudo obtener $HOME");

    let prefix = home.join(".password-store");

    if !prefix.is_dir() {
        eprintln!("No existe {:?}", prefix);
        return Vec::new();
    }

    WalkDir::new(&prefix)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("gpg"))
        .map(|e| {
            let absolute = e.into_path();
            let relative = absolute.strip_prefix(&prefix).unwrap().with_extension("");

            PasswordFile { relative, absolute }
        })
        .collect()
}

pub fn get_password(name: &Path) -> Result<String, String> {
    let output = Command::new("pass")
        .arg("show")
        .arg(name)
        .output()
        .map_err(|e| format!("Failed to run pass: {}", e))?;

    if !output.status.success() {
        return Err("pass failed".into());
    }

    let stdout =
        String::from_utf8(output.stdout).map_err(|_| "Invalid UTF-8 from pass".to_string())?;

    Ok(stdout)
}
