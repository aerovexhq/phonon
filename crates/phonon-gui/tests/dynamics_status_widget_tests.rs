#![deny(unsafe_code)]

//! Verification test suite for Phonon visual dynamics awareness widgets,
//! discovery status badges, and in-process backend injection mechanics (Phase 303).

use egui::Color32;
use phonon_core::{
    ActuatorInputs, AerovexShmBackend, AutoSelectingDynamicsBackend, BackendInfo, DynamicsError,
    DynamicsTelemetry, PhysicsDynamicsBackend, ReferenceDynamicsBackend,
};
use phonon_gui::{DynamicsStatusBadge, PhononApp};

/// Custom mock dynamics backend for testing third-party engine injection and telemetry formatting.
struct CustomMockDynamicsBackend {
    info: BackendInfo,
    telemetry: DynamicsTelemetry,
}

impl CustomMockDynamicsBackend {
    fn new(name: &str, version: &str, is_hw_accel: bool) -> Self {
        Self {
            info: BackendInfo::new(
                name,
                version,
                is_hw_accel,
                1000.0,
                "Custom test dynamics solver",
            ),
            telemetry: DynamicsTelemetry::new(
                [10.0, 20.0, -150.0],
                [5.0, 0.0, -2.5],
                [0.0, 0.0, -9.81],
                [1.0, 0.0, 0.0, 0.0],
                [0.01, 0.02, 0.03],
                [0.0, 0.0, 0.0],
                12.5,
                1250,
            ),
        }
    }
}

impl PhysicsDynamicsBackend for CustomMockDynamicsBackend {
    fn info(&self) -> BackendInfo {
        self.info.clone()
    }

    fn reset(&mut self, initial_state: Option<&DynamicsTelemetry>) -> Result<(), DynamicsError> {
        if let Some(state) = initial_state {
            self.telemetry = state.clone();
        }
        Ok(())
    }

    fn step(
        &mut self,
        dt_s: f64,
        _inputs: &ActuatorInputs,
    ) -> Result<&DynamicsTelemetry, DynamicsError> {
        self.telemetry.sim_time_s += dt_s;
        self.telemetry.step_count += 1;
        self.telemetry.position_m[2] -= 1.0;
        Ok(&self.telemetry)
    }

    fn current_telemetry(&self) -> &DynamicsTelemetry {
        &self.telemetry
    }

    fn is_healthy(&self) -> bool {
        true
    }
}

#[test]
fn test_dynamics_status_badge_reference_mode() {
    let backend = ReferenceDynamicsBackend::default();
    let badge = DynamicsStatusBadge::new();

    assert!(!badge.is_aerovex_mode(&backend));
    assert_eq!(badge.badge_label(&backend), "[REFERENCE DYNAMICS ACTIVE]");

    let (bg, border) = badge.badge_colors(&backend);
    assert_eq!(bg, Color32::from_rgb(26, 40, 56));
    assert_eq!(border, Color32::from_rgb(58, 110, 168));

    assert_eq!(badge.learn_more_url(), "https://aerovex.net");

    let tooltip = badge.telemetry_tooltip_text(&backend);
    assert!(tooltip.contains("Phonon Pure Safe Rust Reference RK4 Dynamics Engine"));
    assert!(tooltip.contains("Sim Time: 0.000 s"));
    assert!(tooltip.contains("Steps: 0"));
}

#[test]
fn test_dynamics_status_badge_aerovex_shm_mode() {
    let backend = AerovexShmBackend::new();
    let badge = DynamicsStatusBadge::new();

    assert!(backend.info().is_hardware_accelerated);
    assert!(backend.info().name.contains("Aerovex"));
    assert!(badge.is_aerovex_mode(&backend));
    assert_eq!(
        badge.badge_label(&backend),
        "[ACTIVE: AEROVEX MULTI-PHYSICS SIMULATOR CONNECTED]"
    );

    let (bg, border) = badge.badge_colors(&backend);
    assert_eq!(bg, Color32::from_rgb(18, 52, 36));
    assert_eq!(border, Color32::from_rgb(46, 160, 92));

    let tooltip = badge.telemetry_tooltip_text(&backend);
    assert!(tooltip.contains("Aerovex Multi-Physics Simulator Connected"));
    assert!(tooltip.contains("8.65M ticks/sec"));
    assert!(tooltip.contains("Rayon 128-World Inflow"));
    assert!(tooltip.contains("Wolkovitch-Leishman VRS Active"));
    assert!(tooltip.contains("Aerovex POSIX Shared Memory Connector"));
}

#[test]
fn test_dynamics_status_badge_custom_mock_backend() {
    let badge = DynamicsStatusBadge::new();

    // 1. Standalone custom backend without hardware acceleration or Aerovex branding
    let custom_solver =
        CustomMockDynamicsBackend::new("Orbital Trajectory Solver", "4.2.1", false);
    assert!(!badge.is_aerovex_mode(&custom_solver));
    assert_eq!(
        badge.badge_label(&custom_solver),
        "[REFERENCE DYNAMICS ACTIVE]"
    );

    let tooltip = badge.telemetry_tooltip_text(&custom_solver);
    assert!(tooltip.contains("Orbital Trajectory Solver"));
    assert!(tooltip.contains("4.2.1"));
    assert!(tooltip.contains("Altitude: 150.00 m"));
    assert!(tooltip.contains("Sim Time: 12.500 s"));
    assert!(tooltip.contains("Steps: 1250"));

    // 2. Custom backend with Aerovex branding
    let custom_aerovex =
        CustomMockDynamicsBackend::new("Aerovex In-Process Sim Kernel", "1.0.0", false);
    assert!(badge.is_aerovex_mode(&custom_aerovex));
    assert_eq!(
        badge.badge_label(&custom_aerovex),
        "[ACTIVE: AEROVEX MULTI-PHYSICS SIMULATOR CONNECTED]"
    );
    let aero_tooltip = badge.telemetry_tooltip_text(&custom_aerovex);
    assert!(aero_tooltip.contains("Aerovex In-Process Sim Kernel"));
    assert!(aero_tooltip.contains("Wolkovitch-Leishman VRS Active"));
}

#[test]
fn test_phonon_app_with_backend_construction() {
    let cc = eframe::CreationContext::_new_kittest(egui::Context::default());
    let backend = Box::new(ReferenceDynamicsBackend::default());
    let app = PhononApp::with_backend(&cc, backend);

    assert_eq!(
        app.backend().info().name,
        "Phonon Pure Safe Rust Reference RK4 Dynamics Engine"
    );
    assert!(app.backend().is_healthy());
    assert_eq!(app.components.len(), 4);
    assert_eq!(app.canvas.zoom, 1.0);

    // Injected custom backend
    let custom_backend = Box::new(CustomMockDynamicsBackend::new(
        "Aerovex Test In-Process Engine",
        "2.0.0",
        true,
    ));
    let app_custom = PhononApp::with_backend(&cc, custom_backend);
    assert_eq!(
        app_custom.backend().info().name,
        "Aerovex Test In-Process Engine"
    );
    assert!(app_custom.backend().info().is_hardware_accelerated);
}

#[test]
fn test_backend_switching_telemetry() {
    let mut backend = ReferenceDynamicsBackend::default();
    let badge = DynamicsStatusBadge::new();

    assert_eq!(backend.current_telemetry().step_count, 0);

    let inputs = ActuatorInputs::hover(10.0);
    for _ in 0..20 {
        backend.step(0.01, &inputs).expect("step must succeed");
    }

    assert_eq!(backend.current_telemetry().step_count, 20);
    assert!((backend.current_telemetry().sim_time_s - 0.20).abs() < 1e-9);

    let tooltip = badge.telemetry_tooltip_text(&backend);
    assert!(tooltip.contains("Steps: 20"));
    assert!(tooltip.contains("Sim Time: 0.200 s"));

    // Auto-selecting backend stepping
    let mut auto_backend = AutoSelectingDynamicsBackend::new();
    for _ in 0..10 {
        auto_backend
            .step(0.01, &inputs)
            .expect("auto-selecting step must succeed");
    }
    let auto_tooltip = badge.telemetry_tooltip_text(&auto_backend);
    assert!(auto_tooltip.contains("Steps: 10"));
    assert!(auto_tooltip.contains("Sim Time: 0.100 s"));
}

#[test]
fn test_zero_lag_rendering_benchmark() {
    let backend = ReferenceDynamicsBackend::default();
    let badge = DynamicsStatusBadge::new();

    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        let is_aero = badge.is_aerovex_mode(&backend);
        let label = badge.badge_label(&backend);
        let (bg, border) = badge.badge_colors(&backend);
        let url = badge.learn_more_url();
        std::hint::black_box((is_aero, label, bg, border, url));
    }
    let elapsed = start.elapsed();
    let evals_per_sec = 10_000.0 / elapsed.as_secs_f64();

    println!(
        "10k badge state queries: {:?} ({:.2} evals/sec)",
        elapsed, evals_per_sec
    );

    assert!(
        elapsed < std::time::Duration::from_millis(10),
        "10k queries took {:?}, exceeding 10 ms ceiling",
        elapsed
    );
    assert!(
        evals_per_sec > 1_000_000.0,
        "Evaluation rate {:.2} evals/sec fell below 1,000,000 evals/sec threshold",
        evals_per_sec
    );
}
