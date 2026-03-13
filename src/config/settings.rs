use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Permissions {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default)]
    pub ask: Vec<String>,
    #[serde(rename = "defaultMode", skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Settings {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(default)]
    pub permissions: Permissions,
    #[serde(flatten)]
    pub other: serde_json::Value,
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read settings from {:?}", path))?;
        let settings: Settings = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse settings from {:?}", path))?;
        Ok(settings)
    }

    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize settings")?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write settings to {:?}", path))?;
        Ok(())
    }

}

pub fn get_user_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

pub fn get_project_settings_path(root: &Path) -> PathBuf {
    root.join(".claude").join("settings.json")
}

pub fn get_local_settings_path(root: &Path) -> PathBuf {
    root.join(".claude").join("settings.local.json")
}

pub fn find_project_root() -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let path = String::from_utf8(output.stdout).ok()?;
    Some(PathBuf::from(path.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert!(settings.permissions.allow.is_empty());
        assert!(settings.permissions.deny.is_empty());
        assert!(settings.permissions.ask.is_empty());
    }

}
