#![deny(unsafe_code)]

//! Categorized application preferences, configuration persistence, and keybinding management for Phonon Studio.

use crate::actions::ActionId;
use crate::storage::ProjectStorageManager;
use crate::theme::{PhononTheme, ThemePreset};
use std::collections::HashMap;
use std::path::PathBuf;

/// Complete application preferences and customization settings.
#[derive(Debug, Clone, PartialEq)]
pub struct AppPreferences {
    // 1. General
    pub autosave_enabled: bool,
    pub autosave_interval_secs: u32,
    pub default_project_prefix: String,
    pub auto_center_on_load: bool,

    // 2. Canvas & Layout
    pub show_grid: bool,
    pub grid_size: f32,
    pub snap_to_grid: bool,
    pub show_toolbar_run_dc: bool,
    pub show_toolbar_run_transient: bool,
    pub show_toolbar_export_netlist: bool,
    pub show_toolbar_clear_canvas: bool,

    // 3. Theme & Colors
    pub active_theme_preset: ThemePreset,
    pub custom_theme: Option<PhononTheme>,

    // 4. Keybindings (action debug string -> custom shortcut or None for unassigned)
    pub keybind_overrides: HashMap<String, Option<String>>,

    // 5. History & Undo Limits
    pub in_memory_history_limit: usize,
    pub on_disk_history_limit: usize,

    // 6. Thermal & Physics
    pub thermal_ambient_temp_k: f64,
    pub thermal_colormap: String,
    pub thermal_overlay_enabled: bool,

    // 7. Recent Projects
    pub recent_projects: Vec<String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            autosave_enabled: true,
            autosave_interval_secs: 30,
            default_project_prefix: "Untitled".to_string(),
            auto_center_on_load: true,

            show_grid: true,
            grid_size: 20.0,
            snap_to_grid: true,
            show_toolbar_run_dc: true,
            show_toolbar_run_transient: true,
            show_toolbar_export_netlist: true,
            show_toolbar_clear_canvas: true,

            active_theme_preset: ThemePreset::DarkNavy,
            custom_theme: None,

            keybind_overrides: HashMap::new(),

            in_memory_history_limit: 500,
            on_disk_history_limit: 50,

            thermal_ambient_temp_k: 300.0,
            thermal_colormap: "Turbo".to_string(),
            thermal_overlay_enabled: false,

            recent_projects: Vec::new(),
        }
    }
}

impl AppPreferences {
    /// Creates a new `AppPreferences` instance with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the currently active theme (either custom or preset).
    pub fn current_theme(&self) -> PhononTheme {
        if let Some(custom) = &self.custom_theme {
            custom.clone()
        } else {
            self.active_theme_preset.to_theme()
        }
    }

    /// Queries the effective shortcut for a given action, applying overrides over defaults.
    pub fn get_shortcut(&self, action_id: ActionId, default_shortcut: Option<&str>) -> Option<String> {
        let key = format!("{:?}", action_id);
        if let Some(override_opt) = self.keybind_overrides.get(&key) {
            override_opt.clone()
        } else {
            default_shortcut.map(|s| s.to_string())
        }
    }

    /// Sets or unassigns a custom shortcut for a given action.
    pub fn set_shortcut(&mut self, action_id: ActionId, shortcut: Option<String>) {
        let key = format!("{:?}", action_id);
        self.keybind_overrides.insert(key, shortcut);
    }

    /// Clears any override for a given action, restoring its default keybinding.
    pub fn reset_shortcut(&mut self, action_id: ActionId) {
        let key = format!("{:?}", action_id);
        self.keybind_overrides.remove(&key);
    }

    /// Clears all keybinding overrides, restoring default shortcuts across all actions.
    pub fn reset_all_shortcuts(&mut self) {
        self.keybind_overrides.clear();
    }

    /// Clamps numeric limits to valid operational bounds.
    pub fn clamp_limits(&mut self) {
        self.in_memory_history_limit = self.in_memory_history_limit.clamp(10, 5000);
        self.on_disk_history_limit = self.on_disk_history_limit.clamp(0, 500);
        self.grid_size = self.grid_size.clamp(5.0, 100.0);
        self.autosave_interval_secs = self.autosave_interval_secs.clamp(5, 600);
        self.thermal_ambient_temp_k = self.thermal_ambient_temp_k.clamp(100.0, 600.0);
    }

    /// Adds a project name or path to the top of the recent projects list, avoiding duplicates.
    pub fn add_recent_project(&mut self, name: &str) {
        let clean = name.trim().to_string();
        if clean.is_empty() {
            return;
        }
        self.recent_projects.retain(|p| p != &clean);
        self.recent_projects.insert(0, clean);
        if self.recent_projects.len() > 10 {
            self.recent_projects.truncate(10);
        }
    }

    /// Clears the recent projects history.
    pub fn clear_recent_projects(&mut self) {
        self.recent_projects.clear();
    }

    /// Serializes preferences into human-readable configuration text.
    pub fn to_config_str(&self) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str("# Phonon Studio Preferences Configuration\n");
        out.push_str(&format!("autosave_enabled: {}\n", self.autosave_enabled));
        out.push_str(&format!("autosave_interval_secs: {}\n", self.autosave_interval_secs));
        out.push_str(&format!("default_project_prefix: \"{}\"\n", self.default_project_prefix));
        out.push_str(&format!("auto_center_on_load: {}\n", self.auto_center_on_load));
        out.push_str(&format!("show_grid: {}\n", self.show_grid));
        out.push_str(&format!("grid_size: {}\n", self.grid_size));
        out.push_str(&format!("snap_to_grid: {}\n", self.snap_to_grid));
        out.push_str(&format!("show_toolbar_run_dc: {}\n", self.show_toolbar_run_dc));
        out.push_str(&format!("show_toolbar_run_transient: {}\n", self.show_toolbar_run_transient));
        out.push_str(&format!("show_toolbar_export_netlist: {}\n", self.show_toolbar_export_netlist));
        out.push_str(&format!("show_toolbar_clear_canvas: {}\n", self.show_toolbar_clear_canvas));
        out.push_str(&format!("active_theme_preset: \"{}\"\n", self.active_theme_preset.name()));
        out.push_str(&format!("in_memory_history_limit: {}\n", self.in_memory_history_limit));
        out.push_str(&format!("on_disk_history_limit: {}\n", self.on_disk_history_limit));
        out.push_str(&format!("thermal_ambient_temp_k: {}\n", self.thermal_ambient_temp_k));
        out.push_str(&format!("thermal_colormap: \"{}\"\n", self.thermal_colormap));
        out.push_str(&format!("thermal_overlay_enabled: {}\n", self.thermal_overlay_enabled));

        for proj in &self.recent_projects {
            out.push_str(&format!("recent_project: \"{}\"\n", proj));
        }

        if let Some(custom) = &self.custom_theme {
            out.push_str("custom_theme_yaml: |\n");
            for line in custom.to_yaml_string().lines() {
                out.push_str(&format!("  {}\n", line));
            }
        }

        for (action, shortcut_opt) in &self.keybind_overrides {
            match shortcut_opt {
                Some(s) => out.push_str(&format!("keybind: {} = \"{}\"\n", action, s)),
                None => out.push_str(&format!("keybind: {} = none\n", action)),
            }
        }

        out
    }

    /// Deserializes preferences from configuration text.
    pub fn from_config_str(content: &str) -> Self {
        let mut prefs = Self::default();
        let mut custom_yaml_lines = Vec::new();
        let mut in_custom_yaml = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if in_custom_yaml {
                if line.starts_with("  ") {
                    custom_yaml_lines.push(&line[2..]);
                    continue;
                } else {
                    in_custom_yaml = false;
                }
            }

            if let Some((key, val)) = trimmed.split_once(':') {
                let key = key.trim();
                let mut val = val.trim();
                if (val.starts_with('"') && val.ends_with('"'))
                    || (val.starts_with('\'') && val.ends_with('\''))
                {
                    if val.len() >= 2 {
                        val = &val[1..val.len() - 1];
                    }
                }

                match key {
                    "autosave_enabled" => prefs.autosave_enabled = val.parse().unwrap_or(true),
                    "autosave_interval_secs" => prefs.autosave_interval_secs = val.parse().unwrap_or(30),
                    "default_project_prefix" => prefs.default_project_prefix = val.to_string(),
                    "auto_center_on_load" => prefs.auto_center_on_load = val.parse().unwrap_or(true),
                    "show_grid" => prefs.show_grid = val.parse().unwrap_or(true),
                    "grid_size" => prefs.grid_size = val.parse().unwrap_or(20.0),
                    "snap_to_grid" => prefs.snap_to_grid = val.parse().unwrap_or(true),
                    "show_toolbar_run_dc" => prefs.show_toolbar_run_dc = val.parse().unwrap_or(true),
                    "show_toolbar_run_transient" => prefs.show_toolbar_run_transient = val.parse().unwrap_or(true),
                    "show_toolbar_export_netlist" => prefs.show_toolbar_export_netlist = val.parse().unwrap_or(true),
                    "show_toolbar_clear_canvas" => prefs.show_toolbar_clear_canvas = val.parse().unwrap_or(true),
                    "active_theme_preset" => {
                        if let Some(preset) = ThemePreset::from_name(val) {
                            prefs.active_theme_preset = preset;
                        }
                    }
                    "in_memory_history_limit" => prefs.in_memory_history_limit = val.parse().unwrap_or(500),
                    "on_disk_history_limit" => prefs.on_disk_history_limit = val.parse().unwrap_or(50),
                    "thermal_ambient_temp_k" => prefs.thermal_ambient_temp_k = val.parse().unwrap_or(300.0),
                    "thermal_colormap" => prefs.thermal_colormap = val.to_string(),
                    "thermal_overlay_enabled" => prefs.thermal_overlay_enabled = val.parse().unwrap_or(false),
                    "recent_project" => {
                        let clean = val.to_string();
                        if !clean.is_empty() && !prefs.recent_projects.contains(&clean) {
                            prefs.recent_projects.push(clean);
                        }
                    }
                    "custom_theme_yaml" => in_custom_yaml = true,
                    "keybind" => {
                        if let Some((act, sc)) = val.split_once('=') {
                            let act = act.trim().to_string();
                            let sc = sc.trim();
                            let clean_sc = if sc.eq_ignore_ascii_case("none") {
                                None
                            } else {
                                let s = sc.trim_matches('"').trim_matches('\'');
                                Some(s.to_string())
                            };
                            prefs.keybind_overrides.insert(act, clean_sc);
                        }
                    }
                    _ => {}
                }
            }
        }

        if !custom_yaml_lines.is_empty() {
            let joined = custom_yaml_lines.join("\n");
            if let Ok(theme) = PhononTheme::from_yaml_str(&joined) {
                prefs.custom_theme = Some(theme);
            }
        }

        prefs.clamp_limits();
        prefs
    }

    /// Canonical path for storing preferences on desktop (`~/.phonon/preferences.phn`).
    pub fn preferences_path() -> PathBuf {
        #[cfg(target_arch = "wasm32")]
        {
            PathBuf::from("/preferences.phn")
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(home) = std::env::var_os("HOME") {
                PathBuf::from(home).join(".phonon").join("preferences.phn")
            } else {
                std::env::temp_dir().join("phonon").join("preferences.phn")
            }
        }
    }

    /// Saves preferences to persistent storage on disk or default target.
    pub fn save(&self) -> Result<(), String> {
        self.save_to_disk()
    }

    /// Writes preferences to persistent storage on disk.
    pub fn save_to_disk(&self) -> Result<(), String> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = Self::preferences_path();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = self.to_config_str();
            std::fs::write(&path, content).map_err(|e| e.to_string())
        }
    }

    /// Reads preferences from persistent storage on disk, or defaults if not found.
    pub fn load_from_disk() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = Self::preferences_path();
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    return Self::from_config_str(&content);
                }
            }
            Self::default()
        }
    }

    /// Saves preferences via ProjectStorageManager for cross-platform support.
    pub fn save_to_storage(&self, manager: &mut ProjectStorageManager) -> Result<(), String> {
        let content = self.to_config_str();
        #[cfg(target_arch = "wasm32")]
        {
            manager.save_raw("__preferences__", content.as_bytes())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (manager, &content);
            self.save_to_disk()
        }
    }

    /// Loads preferences via ProjectStorageManager for cross-platform support.
    pub fn load_from_storage(manager: &ProjectStorageManager) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(bytes) = manager.load_raw("__preferences__") {
                if let Ok(content) = std::str::from_utf8(&bytes) {
                    return Self::from_config_str(content);
                }
            }
            Self::default()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = manager;
            Self::load_from_disk()
        }
    }
}
