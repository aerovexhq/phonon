#![deny(unsafe_code)]

//! Analytical verification and cold-boot startup latency benchmarks for Phonon Studio.
//!
//! Validates:
//! 1. `default_native_options` defaults to `Renderer::Glow` (fast-path OpenGL/EGL).
//! 2. `PHONON_RENDERER=wgpu` environment variable cleanly switches backend to `Renderer::Wgpu`.
//! 3. `follow_system_theme` is false and default theme is dark (`Theme::Dark`).
//! 4. `PhononApp` cold initialization latency benchmark asserting initialization under 5 ms.

use std::time::Instant;
use egui::Theme;
use phonon_gui::{
    default_boot_theme_config, default_native_options, default_theme, determine_boot_renderer,
    follow_system_theme, PhononApp,
};

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn test_default_native_options_uses_renderer_glow() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Ensure test environment does not have PHONON_RENDERER set
    std::env::remove_var("PHONON_RENDERER");

    let renderer = determine_boot_renderer();
    assert_eq!(
        renderer,
        eframe::Renderer::Glow,
        "Default boot renderer must be Glow for fast OpenGL/EGL initialization"
    );

    let opts = default_native_options();
    assert_eq!(
        opts.renderer,
        eframe::Renderer::Glow,
        "default_native_options must configure Renderer::Glow"
    );
}

#[test]
fn test_phonon_renderer_wgpu_override() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Test uppercase and lowercase wgpu override
    std::env::set_var("PHONON_RENDERER", "wgpu");
    assert_eq!(
        determine_boot_renderer(),
        eframe::Renderer::Wgpu,
        "PHONON_RENDERER=wgpu must select Renderer::Wgpu"
    );

    let opts_wgpu = default_native_options();
    assert_eq!(
        opts_wgpu.renderer,
        eframe::Renderer::Wgpu,
        "default_native_options must reflect wgpu override"
    );

    std::env::set_var("PHONON_RENDERER", "WGPU");
    assert_eq!(
        determine_boot_renderer(),
        eframe::Renderer::Wgpu,
        "PHONON_RENDERER=WGPU must select Renderer::Wgpu"
    );

    // Revert environment variable for clean isolation
    std::env::remove_var("PHONON_RENDERER");
    assert_eq!(
        determine_boot_renderer(),
        eframe::Renderer::Glow,
        "Removing PHONON_RENDERER must revert to Renderer::Glow"
    );
}

#[test]
fn test_follow_system_theme_is_false_and_theme_is_dark() {
    assert!(
        !follow_system_theme(),
        "follow_system_theme must be false to avoid slow D-Bus/X11 theme queries"
    );
    assert_eq!(
        default_theme(),
        Theme::Dark,
        "default_theme must be Theme::Dark for visual CAD consistency"
    );

    let cfg = default_boot_theme_config();
    assert!(
        !cfg.follow_system_theme,
        "BootThemeConfig.follow_system_theme must be false"
    );
    assert_eq!(
        cfg.default_theme,
        Theme::Dark,
        "BootThemeConfig.default_theme must be Theme::Dark"
    );

    let app = PhononApp::default();
    assert!(
        !app.boot_theme_config.follow_system_theme,
        "PhononApp.boot_theme_config.follow_system_theme must be false"
    );
    assert_eq!(
        app.boot_theme_config.default_theme,
        Theme::Dark,
        "PhononApp.boot_theme_config.default_theme must be Theme::Dark"
    );
}

#[test]
fn test_phonon_app_cold_initialization_latency_benchmark() {
    let iterations = 100;
    let mut total_duration = std::time::Duration::ZERO;

    for _ in 0..iterations {
        let start = Instant::now();
        let app = PhononApp::default();
        let elapsed = start.elapsed();
        total_duration += elapsed;

        assert!(!app.components.is_empty());
        assert!(!app.wires.is_empty());
        assert_eq!(app.history.undo_depth(), 0);
    }

    let avg_latency = total_duration / (iterations as u32);
    let avg_ms = avg_latency.as_secs_f64() * 1000.0;
    println!(
        "\nPhononApp Cold Boot Initialization Benchmark ({} cycles):",
        iterations
    );
    println!("  Average Latency: {:.4} ms ({:.2} us)", avg_ms, avg_latency.as_micros() as f64);

    assert!(
        avg_ms < 5.0,
        "PhononApp cold initialization latency must be strictly sub-5ms (measured {:.3} ms)",
        avg_ms
    );
}
