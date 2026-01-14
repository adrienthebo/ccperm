use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

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

    pub fn merge_local(&mut self, local: &Settings) {
        // Local settings override global settings
        for rule in &local.permissions.allow {
            if !self.permissions.allow.contains(rule) {
                self.permissions.allow.push(rule.clone());
            }
        }
        for rule in &local.permissions.deny {
            if !self.permissions.deny.contains(rule) {
                self.permissions.deny.push(rule.clone());
            }
        }
        for rule in &local.permissions.ask {
            if !self.permissions.ask.contains(rule) {
                self.permissions.ask.push(rule.clone());
            }
        }
        if local.permissions.default_mode.is_some() {
            self.permissions.default_mode = local.permissions.default_mode.clone();
        }
    }
}

pub fn get_global_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.json"))
}

pub fn get_local_settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude").join("settings.local.json"))
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

    #[test]
    fn test_merge_local() {
        let mut global = Settings::default();
        global.permissions.allow.push("Bash(npm install)".to_string());

        let mut local = Settings::default();
        local.permissions.allow.push("Bash(cargo build)".to_string());

        global.merge_local(&local);
        assert_eq!(global.permissions.allow.len(), 2);
    }
}
