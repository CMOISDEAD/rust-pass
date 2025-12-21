use regex::Regex;
use std::collections::HashMap;
use std::process::Command;
use std::{env, path::PathBuf};
use walkdir::WalkDir;

pub fn get_password_files() -> HashMap<String, String> {
    let home = env::var("HOME")
        .map(PathBuf::from)
        .expect("No se pudo obtener $HOME");

    let prefix = home.join(".password-store");

    let mut map = HashMap::new();

    if !prefix.is_dir() {
        eprintln!("No existe {:?}", prefix);
        return map;
    }

    for entry in WalkDir::new(&prefix)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("gpg"))
    {
        let absolute = entry.into_path();
        let relative = absolute
            .strip_prefix(&prefix)
            .unwrap()
            .with_extension("");

        let key = relative.to_string_lossy().to_string();

        map.insert(key.clone(), key);
    }

    map
}

pub fn get_password_content(name: &String) -> Result<String, String> {
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

pub enum ParsedPassword {
    Raw(String),
    KeyValue(HashMap<String, String>),
}

pub fn parse_kv(content: &str) -> ParsedPassword {
    let re =
        Regex::new(r"(?m)^\s*(?P<key>[\w\s.-]+)\s*:\s*(?P<value>.+)\s*$").expect("invalid regex");

    if !re.is_match(content) {
        return ParsedPassword::Raw(content.trim().to_string());
    }

    let mut map = HashMap::new();

    for caps in re.captures_iter(content) {
        let key = caps.name("key").unwrap().as_str().to_string();
        let value = caps.name("value").unwrap().as_str().to_string();

        map.insert(key, value);
    }

    ParsedPassword::KeyValue(map)
}
