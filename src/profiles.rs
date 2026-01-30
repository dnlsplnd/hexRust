use crate::model::IrcConfig;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub tls: bool,
    pub nick: String,
    pub initial_channel: String,

    // ZNC (optional)
    // If set, hexRust will build an IRC PASS string like:
    //   username/network:password  (or username:password if network is empty)
    pub znc_username: Option<String>,
    pub znc_network: Option<String>,
    pub znc_password: Option<String>,

    // SASL (optional)
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProfilesFile {
    pub profiles: Vec<Profile>,
}


fn build_znc_pass(user: Option<&str>, network: Option<&str>, pass: Option<&str>) -> Option<String> {
    let user = user.map(str::trim).filter(|s| !s.is_empty());
    let pass = pass.map(str::trim).filter(|s| !s.is_empty());
    if user.is_none() || pass.is_none() {
        return None;
    }
    let user = user.unwrap();
    let pass = pass.unwrap();

    let net = network.map(str::trim).filter(|s| !s.is_empty());
    if let Some(net) = net {
        Some(format!("{user}/{net}:{pass}"))
    } else {
        Some(format!("{user}:{pass}"))
    }
}

impl Profile {
    pub fn to_irc_config(&self) -> IrcConfig {
        let server_password = build_znc_pass(
            self.znc_username.as_deref(),
            self.znc_network.as_deref(),
            self.znc_password.as_deref(),
        );

        IrcConfig {
            server: self.server.clone(),
            port: self.port,
            tls: self.tls,
            nick: self.nick.clone(),
            initial_channel: self.initial_channel.clone(),
            server_password,
            sasl_username: self.sasl_username.clone(),
            sasl_password: self.sasl_password.clone(),
        }
    }
}

pub fn default_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Could not resolve config directory")?;
    Ok(base.join("hexrust").join("profiles.toml"))
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create dir {}", parent.display()))?;
    }
    Ok(())
}

pub fn load(path: &Path) -> Result<ProfilesFile> {
    if !path.exists() {
        return Ok(ProfilesFile::default());
    }
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let pf: ProfilesFile = toml::from_str(&text).context("parse profiles.toml")?;
    Ok(pf)
}

pub fn save(path: &Path, pf: &ProfilesFile) -> Result<()> {
    ensure_parent_dir(path)?;
    let text = toml::to_string_pretty(pf).context("serialize profiles.toml")?;
    fs::write(path, text).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn seed_example_if_empty(path: &Path) -> Result<()> {
    let mut pf = load(path)?;
    if !pf.profiles.is_empty() {
        return Ok(());
    }
    pf.profiles.push(Profile {
        name: "Libera (example)".to_string(),
        server: "irc.libera.chat".to_string(),
        port: 6697,
        tls: true,
        nick: "hexrust".to_string(),
        initial_channel: "#hexrust".to_string(),
        znc_username: None,
        znc_network: None,
        znc_password: None,
        sasl_username: None,
        sasl_password: None,
    });
    save(path, &pf)?;
    Ok(())
}
