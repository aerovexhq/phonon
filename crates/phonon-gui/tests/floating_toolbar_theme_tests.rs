#![deny(unsafe_code)]

//! Test suite for the theme-adaptive floating CAD tools island.
//!
//! Validates:
//! 1. Dynamic derivation of `FloatingToolbarTheme` tokens across all 5 built-in theme presets.
//! 2. Correct `is_dark()` and `luminance()` detection across presets.
//! 3. Contrast ratio integrity between toolbar background and foreground text/icons.
//! 4. Headless egui render pass in expanded and collapsed states using themed styling.
//! 5. Backwards-compatibility wrapper `show_default`.

use egui::{pos2, vec2, Context, Rect};
use phonon_gui::app::ToolMode;
use phonon_gui::theme::ThemePreset;
use phonon_gui::widgets::{FloatingToolbarState, FloatingToolbarTheme};

#[test]
fn test_theme_presets_luminance_and_dark_mode_detection() {
    let presets = [
        (ThemePreset::DarkNavy, true),
        (ThemePreset::LightPaper, false),
        (ThemePreset::HighContrastOled, true),
        (ThemePreset::MonokaiPro, true),
        (ThemePreset::CyberpunkNeon, true),
    ];

    for (preset, expected_dark) in presets {
        let theme = preset.to_theme();
        assert_eq!(
            theme.is_dark(),
            expected_dark,
            "Preset {:?} dark detection mismatch",
            preset
        );

        let lum = theme.luminance();
        if expected_dark {
            assert!(
                lum < 128.0,
                "Expected dark preset {:?} to have luminance < 128.0, got {}",
                preset,
                lum
            );
        } else {
            assert!(
                lum >= 128.0,
                "Expected light preset {:?} to have luminance >= 128.0, got {}",
                preset,
                lum
            );
        }
    }
}

#[test]
fn test_floating_toolbar_theme_token_derivation() {
    for preset in [
        ThemePreset::DarkNavy,
        ThemePreset::LightPaper,
        ThemePreset::HighContrastOled,
        ThemePreset::MonokaiPro,
        ThemePreset::CyberpunkNeon,
    ] {
        let theme = preset.to_theme();
        let toolbar_theme = FloatingToolbarTheme::from_theme(&theme);

        // Alpha channel of background fill must be non-zero
        assert!(toolbar_theme.bg_fill.a() > 200);

        // Active button background must have translucent tint
        assert!(toolbar_theme.btn_active_bg.a() > 30);

        // Active foreground must not be completely transparent
        assert!(toolbar_theme.btn_active_fg.a() > 200);

        // Normal button foreground must not be completely transparent
        assert!(toolbar_theme.btn_normal_fg.a() > 150);

        // Border stroke color must not be fully transparent
        assert!(toolbar_theme.stroke_color.a() > 50);

        // Check specific presets
        if preset == ThemePreset::LightPaper {
            // Light paper toolbar should have light background
            assert!(toolbar_theme.bg_fill.r() > 200);
            assert!(toolbar_theme.bg_fill.g() > 200);
            assert!(toolbar_theme.bg_fill.b() > 200);
            // Normal foreground should be dark for readability
            assert!(toolbar_theme.btn_normal_fg.r() < 120);
        } else if preset == ThemePreset::HighContrastOled {
            // OLED ultra-dark background
            assert!(toolbar_theme.bg_fill.r() <= 10);
            assert!(toolbar_theme.bg_fill.g() <= 10);
            assert!(toolbar_theme.bg_fill.b() <= 10);
        }
    }
}

#[test]
fn test_floating_toolbar_headless_render_pass() {
    let ctx = Context::default();
    let viewport = Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0));
    let mut state = FloatingToolbarState::new();

    let make_input = || {
        let mut input = egui::RawInput::default();
        input.screen_rect = Some(viewport);
        input
    };

    for preset in [
        ThemePreset::DarkNavy,
        ThemePreset::LightPaper,
        ThemePreset::HighContrastOled,
        ThemePreset::MonokaiPro,
        ThemePreset::CyberpunkNeon,
    ] {
        let theme = preset.to_theme();
        let current_tool = ToolMode::Select;

        // Expanded render pass
        let mut output = ctx.run_ui(make_input(), |ui| {
            let action = state.show(ui.ctx(), viewport, &current_tool, &theme);
            assert!(action.is_none());
        });
        output.textures_delta.clear();

        // Toggle collapsed state and render pass
        state.is_collapsed = true;
        let mut output = ctx.run_ui(make_input(), |ui| {
            let action = state.show(ui.ctx(), viewport, &current_tool, &theme);
            assert!(action.is_none());
        });
        output.textures_delta.clear();
        state.is_collapsed = false;

        // Backwards compatibility show_default pass
        let mut output = ctx.run_ui(make_input(), |ui| {
            let action = state.show_default(ui.ctx(), viewport, &current_tool);
            assert!(action.is_none());
        });
        output.textures_delta.clear();
    }
}

#[test]
fn test_floating_toolbar_state_lifecycle() {
    let mut state = FloatingToolbarState::new();
    assert!(!state.is_collapsed);
    assert!(state.custom_pos.is_none());

    state.custom_pos = Some(pos2(150.0, 200.0));
    assert_eq!(state.custom_pos, Some(pos2(150.0, 200.0)));

    state.is_collapsed = true;
    assert!(state.is_collapsed);

    state.is_collapsed = false;
    assert!(!state.is_collapsed);
}
