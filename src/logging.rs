use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn sanitize_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let ok = ch.is_ascii_alphanumeric() || "-_ .#@+".contains(ch);
        out.push(if ok { ch } else { '_' });
    }
    out.trim().to_string()
}

pub fn logs_base_dir() -> Result<PathBuf> {
    let base = dirs::data_local_dir().context("Could not resolve data dir")?;
    Ok(base.join("hexrust").join("logs"))
}

pub fn log_path(server: &str, buffer: &str) -> Result<PathBuf> {
    let base = logs_base_dir()?;
    let srv = sanitize_component(server);
    let buf = sanitize_component(buffer);
    Ok(base.join(srv).join(format!("{buf}.log")))
}

pub fn append_line(server: &str, buffer: &str, line: &str) -> Result<()> {
    let path = log_path(server, buffer)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create dir {}", parent.display()))?;
    }
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(f, "{line}").context("write log line")?;
    Ok(())
}

pub fn read_all(server: &str, buffer: &str) -> Result<String> {
    let path = log_path(server, buffer)?;
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))
}

pub fn log_exists(server: &str, buffer: &str) -> Result<bool> {
    Ok(log_path(server, buffer)?.exists())
}

pub fn log_path_display(server: &str, buffer: &str) -> Result<String> {
    Ok(log_path(server, buffer)?.display().to_string())
}
