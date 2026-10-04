#![deny(unsafe_code)]

//! Comprehensive test suite for Phonon Studio Categorized Preferences Dialog & Theme Engine.
//!
//! Validates:
//! 1. Default AppPreferences values across all categorized configuration sections.
//! 2. Keybinding customization: get, set, override, reset, and reset all.
//! 3. History depth limits clamping: in_memory_history_limit (10..=5000), on_disk_history_limit (0..=500).
//! 4. Theme presets: DarkNavy, LightPaper, HighContrastOled, MonokaiPro, CyberpunkNeon.
//! 5. Zero-dependency YAML theme serialization and deserialization roundtrip.
//! 6. Hex color conversion utilities (#RRGGBB, #RRGGBBAA, #RGB).
//! 7. PreferencesDialog navigation tabs, search filter, and shortcut editing state.
//! 8. PreferencesDialog headless egui render pass across all 6 tabs and sub-modals.

use egui::{Color32, Context};
use phonon_gui::actions::{ActionId, ActionRegistry};
use phonon_gui::preferences::AppPreferences;
use phonon_gui::schematic::HistoryStack;
use phonon_gui::thermal::Colormap;
use phonon_gui::theme::{color32_to_hex, hex_to_color32, PhononTheme, ThemePreset};
use phonon_gui::widgets::{PreferencesDialog, PreferencesTab};

#[test]
fn test_default_app_preferences_values() {
    let prefs = AppPreferences::default();

    // 1. General
    assert!(prefs.autosave_enabled);
    assert_eq!(prefs.autosave_interval_secs, 30);
    assert_eq!(prefs.default_project_prefix, "Untitled");
    assert!(prefs.auto_center_on_load);

    // 2. Canvas & Layout
    assert!(prefs.show_grid);
    assert_eq!(prefs.grid_size, 20.0);
    assert!(prefs.snap_to_grid);
    assert!(prefs.show_toolbar_run_dc);
    assert!(prefs.show_toolbar_run_transient);
    assert!(prefs.show_toolbar_export_netlist);
    assert!(prefs.show_toolbar_clear_canvas);

    // 3. Theme
    assert_eq!(prefs.active_theme_preset, ThemePreset::DarkNavy);
    assert!(prefs.custom_theme.is_none());

    // 4. Keybindings
    assert!(prefs.keybind_overrides.is_empty());

    // 5. History Limits
    assert_eq!(prefs.in_memory_history_limit, 500);
    assert_eq!(prefs.on_disk_history_limit, 50);

    // 6. Thermal
    assert_eq!(prefs.thermal_ambient_temp_k, 300.0);
    assert_eq!(prefs.thermal_colormap, "Turbo");
    assert!(!prefs.thermal_overlay_enabled);
}

#[test]
fn test_preferences_config_string_roundtrip() {
    let mut prefs = AppPreferences::default();
    prefs.autosave_interval_secs = 45;
    prefs.default_project_prefix = "SuperCircuit".to_string();
    prefs.grid_size = 25.0;
    prefs.active_theme_preset = ThemePreset::MonokaiPro;
    prefs.in_memory_history_limit = 1000;
    prefs.on_disk_history_limit = 100;
    prefs.thermal_ambient_temp_k = 320.5;
    prefs.set_shortcut(ActionId::Undo, Some("Ctrl+Shift+Z".to_string()));
    prefs.set_shortcut(ActionId::SaveProject, None);

    let config_text = prefs.to_config_str();
    let loaded = AppPreferences::from_config_str(&config_text);

    assert_eq!(loaded.autosave_enabled, prefs.autosave_enabled);
    assert_eq!(loaded.autosave_interval_secs, 45);
    assert_eq!(loaded.default_project_prefix, "SuperCircuit");
    assert_eq!(loaded.grid_size, 25.0);
    assert_eq!(loaded.active_theme_preset, ThemePreset::MonokaiPro);
    assert_eq!(loaded.in_memory_history_limit, 1000);
    assert_eq!(loaded.on_disk_history_limit, 100);
    assert!((loaded.thermal_ambient_temp_k - 320.5).abs() < 1e-4);

    assert_eq!(
        loaded.get_shortcut(ActionId::Undo, Some("Ctrl+Z")),
        Some("Ctrl+Shift+Z".to_string())
    );
    assert_eq!(
        loaded.get_shortcut(ActionId::SaveProject, Some("Ctrl+S")),
        None
    );
    assert_eq!(
        loaded.get_shortcut(ActionId::NewProject, Some("Ctrl+N")),
        Some("Ctrl+N".to_string())
    );
}

#[test]
fn test_keybinding_customization_and_resets() {
    let mut prefs = AppPreferences::default();

    // Default shortcut queried when no override exists
    assert_eq!(
        prefs.get_shortcut(ActionId::Undo, Some("Ctrl+Z")),
        Some("Ctrl+Z".to_string())
    );
    assert_eq!(
        prefs.get_shortcut(ActionId::ClearCanvas, None),
        None
    );

    // Set custom shortcut override
    prefs.set_shortcut(ActionId::Undo, Some("Alt+Backspace".to_string()));
    assert_eq!(
        prefs.get_shortcut(ActionId::Undo, Some("Ctrl+Z")),
        Some("Alt+Backspace".to_string())
    );

    // Unassign shortcut (override to None)
    prefs.set_shortcut(ActionId::Duplicate, None);
    assert_eq!(
        prefs.get_shortcut(ActionId::Duplicate, Some("Ctrl+D")),
        None
    );

    // Reset single shortcut
    prefs.reset_shortcut(ActionId::Undo);
    assert_eq!(
        prefs.get_shortcut(ActionId::Undo, Some("Ctrl+Z")),
        Some("Ctrl+Z".to_string())
    );
    // Duplicate is still unassigned
    assert_eq!(
        prefs.get_shortcut(ActionId::Duplicate, Some("Ctrl+D")),
        None
    );

    // Reset all shortcuts
    prefs.reset_all_shortcuts();
    assert!(prefs.keybind_overrides.is_empty());
    assert_eq!(
        prefs.get_shortcut(ActionId::Duplicate, Some("Ctrl+D")),
        Some("Ctrl+D".to_string())
    );
}

#[test]
fn test_history_depth_limits_and_bounds_clamping() {
    let mut prefs = AppPreferences::default();

    // in_memory_history_limit bounds: 10..=5000
    prefs.in_memory_history_limit = 5;
    prefs.clamp_limits();
    assert_eq!(prefs.in_memory_history_limit, 10);

    prefs.in_memory_history_limit = 10000;
    prefs.clamp_limits();
    assert_eq!(prefs.in_memory_history_limit, 5000);

    prefs.in_memory_history_limit = 250;
    prefs.clamp_limits();
    assert_eq!(prefs.in_memory_history_limit, 250);

    // on_disk_history_limit bounds: 0..=500
    prefs.on_disk_history_limit = 9999;
    prefs.clamp_limits();
    assert_eq!(prefs.on_disk_history_limit, 500);

    prefs.on_disk_history_limit = 0;
    prefs.clamp_limits();
    assert_eq!(prefs.on_disk_history_limit, 0);

    // Grid size bounds: 5.0..=100.0
    prefs.grid_size = 1.0;
    prefs.clamp_limits();
    assert_eq!(prefs.grid_size, 5.0);

    prefs.grid_size = 200.0;
    prefs.clamp_limits();
    assert_eq!(prefs.grid_size, 100.0);

    // Autosave interval bounds: 5..=600
    prefs.autosave_interval_secs = 1;
    prefs.clamp_limits();
    assert_eq!(prefs.autosave_interval_secs, 5);

    prefs.autosave_interval_secs = 1200;
    prefs.clamp_limits();
    assert_eq!(prefs.autosave_interval_secs, 600);

    // Thermal ambient bounds: 100.0..=600.0
    prefs.thermal_ambient_temp_k = 50.0;
    prefs.clamp_limits();
    assert_eq!(prefs.thermal_ambient_temp_k, 100.0);

    prefs.thermal_ambient_temp_k = 800.0;
    prefs.clamp_limits();
    assert_eq!(prefs.thermal_ambient_temp_k, 600.0);
}

#[test]
fn test_theme_presets_and_lookup() {
    let presets = ThemePreset::all();
    assert_eq!(presets.len(), 5);
    assert_eq!(
        presets,
        &[
            ThemePreset::DarkNavy,
            ThemePreset::LightPaper,
            ThemePreset::HighContrastOled,
            ThemePreset::MonokaiPro,
            ThemePreset::CyberpunkNeon,
        ]
    );

    // Display names
    assert_eq!(ThemePreset::DarkNavy.name(), "Dark Navy");
    assert_eq!(ThemePreset::LightPaper.name(), "Light Paper");
    assert_eq!(ThemePreset::HighContrastOled.name(), "High Contrast OLED");
    assert_eq!(ThemePreset::MonokaiPro.name(), "Monokai Pro");
    assert_eq!(ThemePreset::CyberpunkNeon.name(), "Cyberpunk Neon");

    // Case-insensitive / alias lookups
    assert_eq!(ThemePreset::from_name("dark navy"), Some(ThemePreset::DarkNavy));
    assert_eq!(ThemePreset::from_name("DarkNavy"), Some(ThemePreset::DarkNavy));
    assert_eq!(ThemePreset::from_name("navy"), Some(ThemePreset::DarkNavy));
    assert_eq!(ThemePreset::from_name("light paper"), Some(ThemePreset::LightPaper));
    assert_eq!(ThemePreset::from_name("paper"), Some(ThemePreset::LightPaper));
    assert_eq!(ThemePreset::from_name("oled"), Some(ThemePreset::HighContrastOled));
    assert_eq!(ThemePreset::from_name("monokai"), Some(ThemePreset::MonokaiPro));
    assert_eq!(ThemePreset::from_name("cyberpunk"), Some(ThemePreset::CyberpunkNeon));
    assert_eq!(ThemePreset::from_name("neon"), Some(ThemePreset::CyberpunkNeon));
    assert_eq!(ThemePreset::from_name("unknown_theme"), None);

    // Verify distinct canvas backgrounds
    assert_eq!(ThemePreset::HighContrastOled.to_theme().canvas_bg, Color32::from_rgb(0, 0, 0));
    assert_eq!(ThemePreset::LightPaper.to_theme().canvas_bg, Color32::from_rgb(248, 249, 250));
    assert_eq!(ThemePreset::DarkNavy.to_theme().canvas_bg, Color32::from_rgb(16, 24, 38));

    // Active theme in AppPreferences
    let mut prefs = AppPreferences::default();
    assert_eq!(prefs.current_theme().name, "Dark Navy");
    prefs.active_theme_preset = ThemePreset::CyberpunkNeon;
    assert_eq!(prefs.current_theme().name, "Cyberpunk Neon");
}

#[test]
fn test_hex_conversion_utilities() {
    let color = Color32::from_rgb(255, 128, 64);
    let hex = color32_to_hex(color);
    assert_eq!(hex, "#FF8040");

    let parsed = hex_to_color32(&hex).expect("Parsing 6-digit hex failed");
    assert_eq!(parsed, color);

    // With and without leading '#'
    assert_eq!(hex_to_color32("FF8040"), Some(color));

    // 8-character RGBA hex with full alpha
    let rgba_hex_full = "#FF8040FF";
    let parsed_full = hex_to_color32(rgba_hex_full).expect("Parsing 8-digit full alpha hex failed");
    assert_eq!(parsed_full.r(), 255);
    assert_eq!(parsed_full.g(), 128);
    assert_eq!(parsed_full.b(), 64);
    assert_eq!(parsed_full.a(), 255);

    // 8-character RGBA hex with semi-transparency (egui stores premultiplied RGB)
    let rgba_hex_half = "#FF804080";
    let parsed_half = hex_to_color32(rgba_hex_half).expect("Parsing 8-digit half alpha hex failed");
    assert_eq!(parsed_half.a(), 128);
    assert_eq!(parsed_half.to_srgba_unmultiplied(), [255, 128, 64, 128]);

    // 3-character RGB shorthand hex
    let short_hex = "#F80";
    let parsed_short = hex_to_color32(short_hex).expect("Parsing 3-digit hex failed");
    assert_eq!(parsed_short.r(), 255);
    assert_eq!(parsed_short.g(), 136);
    assert_eq!(parsed_short.b(), 0);

    // Invalid hex inputs
    assert_eq!(hex_to_color32(""), None);
    assert_eq!(hex_to_color32("#12"), None);
    assert_eq!(hex_to_color32("#12345"), None);
    assert_eq!(hex_to_color32("ZZZZZZ"), None);
}

#[test]
fn test_theme_yaml_roundtrip_all_presets() {
    for preset in ThemePreset::all() {
        let theme = preset.to_theme();
        let yaml_str = theme.to_yaml_string();

        let parsed = PhononTheme::from_yaml_str(&yaml_str)
            .unwrap_or_else(|err| panic!("YAML parse failed for preset {}: {}", preset.name(), err));

        assert_eq!(parsed.name, theme.name);
        assert_eq!(parsed.canvas_bg, theme.canvas_bg);
        assert_eq!(parsed.grid_dot, theme.grid_dot);
        assert_eq!(parsed.grid_line, theme.grid_line);
        assert_eq!(parsed.wire_normal, theme.wire_normal);
        assert_eq!(parsed.wire_selected, theme.wire_selected);
        assert_eq!(parsed.component_body, theme.component_body);
        assert_eq!(parsed.component_stroke, theme.component_stroke);
        assert_eq!(parsed.component_selected, theme.component_selected);
        assert_eq!(parsed.pin_normal, theme.pin_normal);
        assert_eq!(parsed.pin_connected, theme.pin_connected);
        assert_eq!(parsed.accent_primary, theme.accent_primary);
        assert_eq!(parsed.text_primary, theme.text_primary);
        assert_eq!(parsed.text_secondary, theme.text_secondary);
        assert_eq!(parsed.voltage_badge, theme.voltage_badge);
        assert_eq!(parsed, theme);
    }
}

#[test]
fn test_preferences_dialog_navigation_and_state() {
    let mut dialog = PreferencesDialog::new();
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, PreferencesTab::General);
    assert!(dialog.keybind_search.is_empty());
    assert!(dialog.editing_action.is_none());

    // Tab display names
    assert_eq!(PreferencesTab::General.display_name(), "General");
    assert_eq!(PreferencesTab::CanvasLayout.display_name(), "Canvas & Layout");
    assert_eq!(PreferencesTab::ThemeColors.display_name(), "Theme & Colors");
    assert_eq!(PreferencesTab::Keybindings.display_name(), "Keybindings");
    assert_eq!(PreferencesTab::HistoryUndo.display_name(), "History & Undo");
    assert_eq!(PreferencesTab::ThermalPhysics.display_name(), "Thermal & Physics");

    // Open specific tab
    dialog.open(Some(PreferencesTab::Keybindings));
    assert!(dialog.is_open);
    assert_eq!(dialog.active_tab, PreferencesTab::Keybindings);

    // Search query buffer
    dialog.keybind_search = "undo".to_string();
    assert_eq!(dialog.keybind_search, "undo");

    // Shortcut editing state
    dialog.editing_action = Some(ActionId::RotateClockwise);
    dialog.shortcut_input_buffer = "Shift+R".to_string();
    assert_eq!(dialog.editing_action, Some(ActionId::RotateClockwise));
    assert_eq!(dialog.shortcut_input_buffer, "Shift+R");

    // Close resets transient state
    dialog.close();
    assert!(!dialog.is_open);
    assert!(dialog.editing_action.is_none());
    assert!(!dialog.show_yaml_import_modal);
    assert!(!dialog.show_yaml_export_modal);
}

#[test]
fn test_preferences_dialog_headless_egui_render_pass_all_tabs() {
    let ctx = Context::default();
    let registry = ActionRegistry::default();
    let mut history = HistoryStack::default();
    let mut preferences = AppPreferences::default();
    let mut thermal_colormap = Colormap::Turbo;
    let mut dialog = PreferencesDialog::new();

    let tabs = [
        PreferencesTab::General,
        PreferencesTab::CanvasLayout,
        PreferencesTab::ThemeColors,
        PreferencesTab::Keybindings,
        PreferencesTab::HistoryUndo,
        PreferencesTab::ThermalPhysics,
    ];

    let make_input = || {
        let mut input = egui::RawInput::default();
        input.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        ));
        input
    };

    for tab in tabs {
        dialog.open(Some(tab));
        let mut executed = false;

        let mut output = ctx.run_ui(make_input(), |ui| {
            let _changed = dialog.show(
                ui.ctx(),
                &mut preferences,
                &registry,
                &mut history,
                &mut thermal_colormap,
            );
            executed = true;
        });
        output.textures_delta.clear();

        assert!(executed, "Render pass must complete for tab {:?}", tab);
        assert!(dialog.is_open);
    }

    // Keybindings search filter render pass
    dialog.active_tab = PreferencesTab::Keybindings;
    dialog.keybind_search = "zoom".to_string();
    let mut output = ctx.run_ui(make_input(), |ui| {
        dialog.show(
            ui.ctx(),
            &mut preferences,
            &registry,
            &mut history,
            &mut thermal_colormap,
        );
    });
    output.textures_delta.clear();

    // Sub-modal render passes (YAML Import & Export)
    dialog.show_yaml_import_modal = true;
    dialog.yaml_import_buffer = ThemePreset::MonokaiPro.to_theme().to_yaml_string();
    let mut output = ctx.run_ui(make_input(), |ui| {
        dialog.show(
            ui.ctx(),
            &mut preferences,
            &registry,
            &mut history,
            &mut thermal_colormap,
        );
    });
    output.textures_delta.clear();
    assert!(dialog.show_yaml_import_modal);

    dialog.show_yaml_import_modal = false;
    dialog.show_yaml_export_modal = true;
    dialog.yaml_export_buffer = ThemePreset::CyberpunkNeon.to_theme().to_yaml_string();
    let mut output = ctx.run_ui(make_input(), |ui| {
        dialog.show(
            ui.ctx(),
            &mut preferences,
            &registry,
            &mut history,
            &mut thermal_colormap,
        );
    });
    output.textures_delta.clear();
    assert!(dialog.show_yaml_export_modal);

    dialog.close();
    assert!(!dialog.is_open);
}
