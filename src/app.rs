use crate::config::{
    find_project_root, get_local_settings_path, get_project_settings_path,
    get_user_settings_path, Permission, PermissionCategory, PermissionType, Settings,
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Editing { index: usize, input: String },
    Adding { input: String },
    Confirm { message: String, action: ConfirmAction },
    Moving {
        index: usize,
        permission: String,
        destinations: Vec<SettingsSource>,
        selected: usize,
    },
    Help,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    Delete(usize),
    Save,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsSource {
    User,
    Project,
    Local,
}

impl SettingsSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::User => "User",
            Self::Project => "Project",
            Self::Local => "Local",
        }
    }
}

pub struct App {
    pub user_settings: Settings,
    pub project_settings: Settings,
    pub local_settings: Settings,
    pub project_root: Option<PathBuf>,
    pub selected_tab: PermissionType,
    pub selected_source: SettingsSource,
    pub mode: AppMode,
    pub dirty: HashSet<SettingsSource>,
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
        let user_settings = get_user_settings_path()
            .map(|p| Settings::load_or_default(&p))
            .unwrap_or_default();

        let project_root = find_project_root();

        let (project_settings, local_settings) = match &project_root {
            Some(root) => (
                Settings::load_or_default(&get_project_settings_path(root)),
                Settings::load_or_default(&get_local_settings_path(root)),
            ),
            None => (Settings::default(), Settings::default()),
        };

        Ok(App {
            user_settings,
            project_settings,
            local_settings,
            project_root,
            selected_tab: PermissionType::Allow,
            selected_source: SettingsSource::User,
            mode: AppMode::Normal,
            dirty: HashSet::new(),
            should_quit: false,
            tree_state: TreeState::default(),
            status_message: None,
        })
    }

    pub fn current_settings(&self) -> &Settings {
        match self.selected_source {
            SettingsSource::User => &self.user_settings,
            SettingsSource::Project => &self.project_settings,
            SettingsSource::Local => &self.local_settings,
        }
    }

    pub fn current_settings_mut(&mut self) -> &mut Settings {
        match self.selected_source {
            SettingsSource::User => &mut self.user_settings,
            SettingsSource::Project => &mut self.project_settings,
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
            SettingsSource::User => &mut self.user_settings,
            SettingsSource::Project => &mut self.project_settings,
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

    pub fn set_source(&mut self, source: SettingsSource) {
        if source != SettingsSource::User && self.project_root.is_none() {
            self.status_message = Some("Not in a git repository".to_string());
            return;
        }
        self.selected_source = source;
        self.tree_state = TreeState::default();
    }

    pub fn toggle_category(&mut self, category: &PermissionCategory) {
        let expanded = self.tree_state.expanded.entry(category.clone()).or_insert(true);
        *expanded = !*expanded;
    }

    pub fn save(&mut self) -> Result<()> {
        let dirty_sources: Vec<SettingsSource> = self.dirty.iter().copied().collect();
        for source in dirty_sources {
            match source {
                SettingsSource::User => {
                    if let Some(path) = get_user_settings_path() {
                        self.user_settings.save(&path)?;
                    }
                }
                SettingsSource::Project => {
                    if let Some(ref root) = self.project_root {
                        self.project_settings.save(&get_project_settings_path(root))?;
                    }
                }
                SettingsSource::Local => {
                    if let Some(ref root) = self.project_root {
                        self.local_settings.save(&get_local_settings_path(root))?;
                    }
                }
            }
        }
        self.dirty.clear();
        self.status_message = Some("Saved!".to_string());
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.user_settings = get_user_settings_path()
            .map(|p| Settings::load_or_default(&p))
            .unwrap_or_default();

        if let Some(ref root) = self.project_root {
            self.project_settings = Settings::load_or_default(&get_project_settings_path(root));
            self.local_settings = Settings::load_or_default(&get_local_settings_path(root));
        } else {
            self.project_settings = Settings::default();
            self.local_settings = Settings::default();
        }

        self.dirty.clear();
        self.tree_state = TreeState::default();
        self.status_message = Some("Reloaded!".to_string());
        Ok(())
    }

    pub fn add_permission(&mut self, permission: String) {
        self.current_permissions_mut().push(permission);
        self.dirty.insert(self.selected_source);
    }

    pub fn delete_permission(&mut self, index: usize) {
        let perms = self.current_permissions_mut();
        if index < perms.len() {
            perms.remove(index);
            self.dirty.insert(self.selected_source);
        }
    }

    pub fn edit_permission(&mut self, index: usize, new_value: String) {
        let perms = self.current_permissions_mut();
        if index < perms.len() {
            perms[index] = new_value;
            self.dirty.insert(self.selected_source);
        }
    }

    pub fn move_permission(&mut self, index: usize, destination: SettingsSource) {
        let perm = {
            let perms = self.current_permissions_mut();
            if index >= perms.len() {
                return;
            }
            perms.remove(index)
        };
        let tab = self.selected_tab.clone();
        let dest_settings = match destination {
            SettingsSource::User => &mut self.user_settings,
            SettingsSource::Project => &mut self.project_settings,
            SettingsSource::Local => &mut self.local_settings,
        };
        match tab {
            PermissionType::Allow => dest_settings.permissions.allow.push(perm),
            PermissionType::Deny => dest_settings.permissions.deny.push(perm),
            PermissionType::Ask => dest_settings.permissions.ask.push(perm),
        };
        self.dirty.insert(self.selected_source);
        self.dirty.insert(destination);
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
