use crate::config::{
    get_global_settings_path, get_local_settings_path, Permission, PermissionCategory,
    PermissionType, Settings,
};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Editing { index: usize, input: String },
    Adding { input: String },
    Confirm { message: String, action: ConfirmAction },
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    Delete(usize),
    Save,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsSource {
    Global,
    Local,
}

pub struct App {
    pub global_settings: Settings,
    pub local_settings: Settings,
    pub selected_tab: PermissionType,
    pub selected_source: SettingsSource,
    pub mode: AppMode,
    pub dirty: bool,
    pub should_quit: bool,
    pub tree_state: TreeState,
    pub status_message: Option<String>,
}

pub struct TreeState {
    pub expanded: HashMap<PermissionCategory, bool>,
    pub selected_category: Option<PermissionCategory>,
    pub selected_index: Option<usize>,
    pub flat_index: usize,
}

impl Default for TreeState {
    fn default() -> Self {
        let mut expanded = HashMap::new();
        for cat in &[
            PermissionCategory::Git,
            PermissionCategory::Npm,
            PermissionCategory::GCloud,
            PermissionCategory::FileSystem,
            PermissionCategory::Web,
            PermissionCategory::Python,
            PermissionCategory::Cargo,
            PermissionCategory::Docker,
            PermissionCategory::GitHub,
            PermissionCategory::Other,
        ] {
            expanded.insert(cat.clone(), true);
        }
        TreeState {
            expanded,
            selected_category: None,
            selected_index: None,
            flat_index: 0,
        }
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let global_path = get_global_settings_path();
        let local_path = get_local_settings_path();

        let global_settings = global_path
            .as_ref()
            .map(|p| Settings::load_or_default(p))
            .unwrap_or_default();

        let local_settings = local_path
            .as_ref()
            .map(|p| Settings::load_or_default(p))
            .unwrap_or_default();

        Ok(App {
            global_settings,
            local_settings,
            selected_tab: PermissionType::Allow,
            selected_source: SettingsSource::Global,
            mode: AppMode::Normal,
            dirty: false,
            should_quit: false,
            tree_state: TreeState::default(),
            status_message: None,
        })
    }

    pub fn current_settings(&self) -> &Settings {
        match self.selected_source {
            SettingsSource::Global => &self.global_settings,
            SettingsSource::Local => &self.local_settings,
        }
    }

    pub fn current_settings_mut(&mut self) -> &mut Settings {
        match self.selected_source {
            SettingsSource::Global => &mut self.global_settings,
            SettingsSource::Local => &mut self.local_settings,
        }
    }

    pub fn current_permissions(&self) -> &Vec<String> {
        let settings = self.current_settings();
        match self.selected_tab {
            PermissionType::Allow => &settings.permissions.allow,
            PermissionType::Deny => &settings.permissions.deny,
            PermissionType::Ask => &settings.permissions.ask,
        }
    }

    pub fn current_permissions_mut(&mut self) -> &mut Vec<String> {
        let tab = self.selected_tab.clone();
        let settings = match self.selected_source {
            SettingsSource::Global => &mut self.global_settings,
            SettingsSource::Local => &mut self.local_settings,
        };
        match tab {
            PermissionType::Allow => &mut settings.permissions.allow,
            PermissionType::Deny => &mut settings.permissions.deny,
            PermissionType::Ask => &mut settings.permissions.ask,
        }
    }

    pub fn get_categorized_permissions(&self) -> HashMap<PermissionCategory, Vec<(usize, Permission)>> {
        let permissions = self.current_permissions();
        let mut categorized: HashMap<PermissionCategory, Vec<(usize, Permission)>> = HashMap::new();

        for (idx, raw) in permissions.iter().enumerate() {
            let perm = Permission::parse(raw);
            categorized
                .entry(perm.category.clone())
                .or_default()
                .push((idx, perm));
        }

        categorized
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = match self.selected_tab {
            PermissionType::Allow => PermissionType::Deny,
            PermissionType::Deny => PermissionType::Ask,
            PermissionType::Ask => PermissionType::Allow,
        };
        self.tree_state = TreeState::default();
    }

    pub fn toggle_source(&mut self) {
        self.selected_source = match self.selected_source {
            SettingsSource::Global => SettingsSource::Local,
            SettingsSource::Local => SettingsSource::Global,
        };
        self.tree_state = TreeState::default();
    }

    pub fn toggle_category(&mut self, category: &PermissionCategory) {
        let expanded = self.tree_state.expanded.entry(category.clone()).or_insert(true);
        *expanded = !*expanded;
    }

    pub fn save(&mut self) -> Result<()> {
        match self.selected_source {
            SettingsSource::Global => {
                if let Some(path) = get_global_settings_path() {
                    self.global_settings.save(&path)?;
                }
            }
            SettingsSource::Local => {
                if let Some(path) = get_local_settings_path() {
                    self.local_settings.save(&path)?;
                }
            }
        }
        self.dirty = false;
        self.status_message = Some("Saved!".to_string());
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        let global_path = get_global_settings_path();
        let local_path = get_local_settings_path();

        self.global_settings = global_path
            .as_ref()
            .map(|p| Settings::load_or_default(p))
            .unwrap_or_default();

        self.local_settings = local_path
            .as_ref()
            .map(|p| Settings::load_or_default(p))
            .unwrap_or_default();

        self.dirty = false;
        self.tree_state = TreeState::default();
        self.status_message = Some("Reloaded!".to_string());
        Ok(())
    }

    pub fn add_permission(&mut self, permission: String) {
        self.current_permissions_mut().push(permission);
        self.dirty = true;
    }

    pub fn delete_permission(&mut self, index: usize) {
        let perms = self.current_permissions_mut();
        if index < perms.len() {
            perms.remove(index);
            self.dirty = true;
        }
    }

    pub fn edit_permission(&mut self, index: usize, new_value: String) {
        let perms = self.current_permissions_mut();
        if index < perms.len() {
            perms[index] = new_value;
            self.dirty = true;
        }
    }

    pub fn build_flat_items(&self) -> Vec<FlatItem> {
        let categorized = self.get_categorized_permissions();
        let mut items = Vec::new();

        let categories = [
            PermissionCategory::Git,
            PermissionCategory::Npm,
            PermissionCategory::GCloud,
            PermissionCategory::GitHub,
            PermissionCategory::FileSystem,
            PermissionCategory::Web,
            PermissionCategory::Python,
            PermissionCategory::Cargo,
            PermissionCategory::Docker,
            PermissionCategory::Other,
        ];

        for cat in &categories {
            if let Some(perms) = categorized.get(cat) {
                if !perms.is_empty() {
                    let expanded = *self.tree_state.expanded.get(cat).unwrap_or(&true);
                    items.push(FlatItem::Category {
                        category: cat.clone(),
                        count: perms.len(),
                        expanded,
                    });
                    if expanded {
                        for (idx, perm) in perms {
                            items.push(FlatItem::Permission {
                                index: *idx,
                                permission: perm.clone(),
                            });
                        }
                    }
                }
            }
        }

        items
    }
}

#[derive(Debug, Clone)]
pub enum FlatItem {
    Category {
        category: PermissionCategory,
        count: usize,
        expanded: bool,
    },
    Permission {
        index: usize,
        permission: Permission,
    },
}
