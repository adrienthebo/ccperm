use crate::config::{
    find_project_root, get_local_settings_path, get_project_settings_path,
    get_user_settings_path, Permission, PermissionCategory, PermissionType, Settings,
};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tui_textarea::TextArea;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Editing { index: usize },
    Adding,
    Confirm { message: String, action: ConfirmAction },
    Moving {
        index: usize,
        permission: String,
        destinations: Vec<SettingsSource>,
        selected: usize,
    },
    Changing {
        index: usize,
        permission: String,
        destinations: Vec<PermissionType>,
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
    pub user_baseline: Settings,
    pub project_baseline: Settings,
    pub local_baseline: Settings,
    pub project_root: Option<PathBuf>,
    pub selected_tab: PermissionType,
    pub selected_source: SettingsSource,
    pub mode: AppMode,
    pub dirty: HashSet<SettingsSource>,
    pub should_quit: bool,
    pub tree_state: TreeState,
    pub status_message: Option<String>,
    pub textarea: Option<TextArea<'static>>,
}

pub struct TreeState {
    pub expanded: HashMap<PermissionCategory, bool>,
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
            PermissionCategory::Go,
            PermissionCategory::GitHub,
            PermissionCategory::Mcp,
            PermissionCategory::Skill,
            PermissionCategory::SlashCommand,
            PermissionCategory::Other,
        ] {
            expanded.insert(cat.clone(), true);
        }
        TreeState {
            expanded,
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
            user_baseline: user_settings.clone(),
            project_baseline: project_settings.clone(),
            local_baseline: local_settings.clone(),
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
            textarea: None,
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

    pub fn source_changes(&self, source: SettingsSource) -> (usize, usize) {
        let (current, baseline) = match source {
            SettingsSource::User => (&self.user_settings, &self.user_baseline),
            SettingsSource::Project => (&self.project_settings, &self.project_baseline),
            SettingsSource::Local => (&self.local_settings, &self.local_baseline),
        };
        let mut added = 0;
        let mut removed = 0;
        for (cur, base) in [
            (&current.permissions.allow, &baseline.permissions.allow),
            (&current.permissions.deny, &baseline.permissions.deny),
            (&current.permissions.ask, &baseline.permissions.ask),
        ] {
            added += cur.iter().filter(|p| !base.contains(p)).count();
            removed += base.iter().filter(|p| !cur.contains(p)).count();
        }
        (added, removed)
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
        self.user_baseline = self.user_settings.clone();
        self.project_baseline = self.project_settings.clone();
        self.local_baseline = self.local_settings.clone();
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

        self.user_baseline = self.user_settings.clone();
        self.project_baseline = self.project_settings.clone();
        self.local_baseline = self.local_settings.clone();
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

    pub fn sort_permissions(&mut self) {
        let perms = self.current_permissions_mut();
        perms.sort();
        self.dirty.insert(self.selected_source);
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

    pub fn change_permission_type(&mut self, index: usize, destination: PermissionType) {
        let perm = {
            let perms = self.current_permissions_mut();
            if index >= perms.len() {
                return;
            }
            perms.remove(index)
        };
        let settings = self.current_settings_mut();
        match destination {
            PermissionType::Allow => settings.permissions.allow.push(perm),
            PermissionType::Deny => settings.permissions.deny.push(perm),
            PermissionType::Ask => settings.permissions.ask.push(perm),
        };
        self.dirty.insert(self.selected_source);
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
            PermissionCategory::Go,
            PermissionCategory::Mcp,
            PermissionCategory::Skill,
            PermissionCategory::SlashCommand,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DuplicateKind {
    Duplicate {
        source: SettingsSource,
    },
    Overrides {
        source: SettingsSource,
        permission_type: PermissionType,
    },
    OverriddenBy {
        source: SettingsSource,
        permission_type: PermissionType,
    },
}

fn type_restrictiveness(t: &PermissionType) -> u8 {
    match t {
        PermissionType::Allow => 0,
        PermissionType::Ask => 1,
        PermissionType::Deny => 2,
    }
}

fn source_precedence(s: &SettingsSource) -> u8 {
    match s {
        SettingsSource::User => 0,
        SettingsSource::Project => 1,
        SettingsSource::Local => 2,
    }
}

impl App {
    pub fn detect_duplicates(&self) -> HashMap<String, Vec<DuplicateKind>> {
        let current = self.current_permissions();
        let mut result: HashMap<String, Vec<DuplicateKind>> = HashMap::new();

        // Intra-array: same string appears more than once in this array
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for perm in current {
            *counts.entry(perm.as_str()).or_default() += 1;
        }
        for (perm, count) in &counts {
            if *count > 1 {
                result
                    .entry(perm.to_string())
                    .or_default()
                    .push(DuplicateKind::Duplicate {
                        source: self.selected_source,
                    });
            }
        }

        // Cross-source
        let other_sources: Vec<(SettingsSource, &Settings)> = [
            (SettingsSource::User, &self.user_settings),
            (SettingsSource::Project, &self.project_settings),
            (SettingsSource::Local, &self.local_settings),
        ]
        .into_iter()
        .filter(|(s, _)| *s != self.selected_source)
        .collect();

        let current_type = &self.selected_tab;
        let unique_perms: HashSet<&String> = current.iter().collect();

        for perm in &unique_perms {
            for (source, settings) in &other_sources {
                for (ptype, arr) in [
                    (PermissionType::Allow, &settings.permissions.allow),
                    (PermissionType::Deny, &settings.permissions.deny),
                    (PermissionType::Ask, &settings.permissions.ask),
                ] {
                    if !arr.contains(perm) {
                        continue;
                    }

                    let kind = if ptype == *current_type {
                        DuplicateKind::Duplicate { source: *source }
                    } else {
                        let current_wins =
                            type_restrictiveness(current_type) > type_restrictiveness(&ptype)
                            || (type_restrictiveness(current_type) == type_restrictiveness(&ptype)
                                && source_precedence(&self.selected_source) > source_precedence(source));

                        if current_wins {
                            DuplicateKind::Overrides {
                                source: *source,
                                permission_type: ptype,
                            }
                        } else {
                            DuplicateKind::OverriddenBy {
                                source: *source,
                                permission_type: ptype,
                            }
                        }
                    };

                    result.entry(perm.to_string()).or_default().push(kind);
                }
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;

    struct TestApp {
        app: App,
    }

    impl TestApp {
        fn new() -> Self {
            Self {
                app: App {
                    user_settings: Settings::default(),
                    project_settings: Settings::default(),
                    local_settings: Settings::default(),
                    user_baseline: Settings::default(),
                    project_baseline: Settings::default(),
                    local_baseline: Settings::default(),
                    project_root: None,
                    selected_tab: PermissionType::Allow,
                    selected_source: SettingsSource::User,
                    mode: AppMode::Normal,
                    dirty: HashSet::new(),
                    should_quit: false,
                    tree_state: TreeState::default(),
                    status_message: None,
                    textarea: None,
                },
            }
        }

        fn perms(mut self, source: SettingsSource, ptype: PermissionType, perms: Vec<&str>) -> Self {
            let vec: Vec<String> = perms.into_iter().map(String::from).collect();
            let settings = match source {
                SettingsSource::User => &mut self.app.user_settings,
                SettingsSource::Project => &mut self.app.project_settings,
                SettingsSource::Local => &mut self.app.local_settings,
            };
            match ptype {
                PermissionType::Allow => settings.permissions.allow = vec,
                PermissionType::Deny => settings.permissions.deny = vec,
                PermissionType::Ask => settings.permissions.ask = vec,
            };
            self
        }

        fn viewing(mut self, source: SettingsSource, ptype: PermissionType) -> Self {
            self.app.selected_source = source;
            self.app.selected_tab = ptype;
            self
        }

        fn build(self) -> App {
            self.app
        }
    }

    // --- Duplicate detection ---

    #[test]
    fn no_duplicates() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Allow, vec!["Bash(npm install)"])
            .build();
        assert!(app.detect_duplicates().is_empty());
    }

    #[test]
    fn intra_array_duplicate() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)", "Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::User,
        }));
    }

    #[test]
    fn cross_source_same_type_is_duplicate() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Allow, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::Project,
        }));
    }

    #[test]
    fn cross_source_same_type_deny_is_duplicate() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Deny, vec!["Bash(git push)"])
            .perms(SettingsSource::Local, PermissionType::Deny, vec!["Bash(git push)"])
            .viewing(SettingsSource::User, PermissionType::Deny)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::Local,
        }));
    }

    #[test]
    fn cross_source_same_type_ask_is_duplicate() {
        let app = TestApp::new()
            .perms(SettingsSource::Project, PermissionType::Ask, vec!["Bash(git push)"])
            .perms(SettingsSource::Local, PermissionType::Ask, vec!["Bash(git push)"])
            .viewing(SettingsSource::Project, PermissionType::Ask)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::Local,
        }));
    }

    // --- Type restrictiveness: Deny > Ask > Allow ---

    #[test]
    fn allow_overridden_by_deny() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Deny, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Deny,
        }));
    }

    #[test]
    fn allow_overridden_by_ask() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Ask, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Ask,
        }));
    }

    #[test]
    fn ask_overridden_by_deny() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Ask, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Deny, vec!["Bash(git push)"])
            .viewing(SettingsSource::User, PermissionType::Ask)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Deny,
        }));
    }

    #[test]
    fn deny_overrides_allow() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Deny, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Allow, vec!["Bash(git push)"])
            .viewing(SettingsSource::User, PermissionType::Deny)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Overrides {
            source: SettingsSource::Project,
            permission_type: PermissionType::Allow,
        }));
    }

    #[test]
    fn deny_overrides_ask() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Deny, vec!["Bash(git push)"])
            .perms(SettingsSource::Local, PermissionType::Ask, vec!["Bash(git push)"])
            .viewing(SettingsSource::User, PermissionType::Deny)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Overrides {
            source: SettingsSource::Local,
            permission_type: PermissionType::Ask,
        }));
    }

    #[test]
    fn ask_overrides_allow() {
        let app = TestApp::new()
            .perms(SettingsSource::Project, PermissionType::Ask, vec!["Bash(git push)"])
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .viewing(SettingsSource::Project, PermissionType::Ask)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Overrides {
            source: SettingsSource::User,
            permission_type: PermissionType::Allow,
        }));
    }

    // --- Source precedence (tiebreaker when same restrictiveness) ---
    // This shouldn't happen in practice (same type = Duplicate), but
    // if types had equal restrictiveness from different categories it would matter.
    // We don't have that case, so source precedence only applies as a theoretical tiebreaker.

    // --- Viewing from different sources ---

    #[test]
    fn viewing_project_allow_overridden_by_local_deny() {
        let app = TestApp::new()
            .perms(SettingsSource::Project, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Local, PermissionType::Deny, vec!["Bash(git push)"])
            .viewing(SettingsSource::Project, PermissionType::Allow)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Local,
            permission_type: PermissionType::Deny,
        }));
    }

    #[test]
    fn viewing_local_deny_overrides_user_allow() {
        let app = TestApp::new()
            .perms(SettingsSource::Local, PermissionType::Deny, vec!["Bash(git push)"])
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .viewing(SettingsSource::Local, PermissionType::Deny)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Overrides {
            source: SettingsSource::User,
            permission_type: PermissionType::Allow,
        }));
    }

    #[test]
    fn viewing_local_ask_overrides_project_allow() {
        let app = TestApp::new()
            .perms(SettingsSource::Local, PermissionType::Ask, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Allow, vec!["Bash(git push)"])
            .viewing(SettingsSource::Local, PermissionType::Ask)
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Overrides {
            source: SettingsSource::Project,
            permission_type: PermissionType::Allow,
        }));
    }

    // --- Compound cases ---

    #[test]
    fn intra_dup_and_cross_source_override() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)", "Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Deny, vec!["Bash(git push)"])
            .perms(SettingsSource::Local, PermissionType::Allow, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::User,
        }));
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Deny,
        }));
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::Local,
        }));
    }

    #[test]
    fn same_source_has_dup_and_override() {
        // Project has permission in both Allow and Deny; viewing User/Allow
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Deny, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert_eq!(kinds.len(), 2);
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::Project,
        }));
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Deny,
        }));
    }

    #[test]
    fn intra_duplicates_dont_multiply_cross_source() {
        // 3 copies in User/Allow + conflict in Project/Deny
        // should produce 1 Duplicate + 1 OverriddenBy, not 3 OverriddenBy
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec![
                "Bash(git push)", "Bash(git push)", "Bash(git push)",
            ])
            .perms(SettingsSource::Project, PermissionType::Deny, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert_eq!(kinds.len(), 2);
        assert!(kinds.contains(&DuplicateKind::Duplicate {
            source: SettingsSource::User,
        }));
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Deny,
        }));
    }

    #[test]
    fn multiple_conflicts_across_all_sources() {
        let app = TestApp::new()
            .perms(SettingsSource::User, PermissionType::Allow, vec!["Bash(git push)"])
            .perms(SettingsSource::Project, PermissionType::Ask, vec!["Bash(git push)"])
            .perms(SettingsSource::Local, PermissionType::Deny, vec!["Bash(git push)"])
            .build();
        let dups = app.detect_duplicates();
        let kinds = &dups["Bash(git push)"];
        assert_eq!(kinds.len(), 2);
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Project,
            permission_type: PermissionType::Ask,
        }));
        assert!(kinds.contains(&DuplicateKind::OverriddenBy {
            source: SettingsSource::Local,
            permission_type: PermissionType::Deny,
        }));
    }
}
