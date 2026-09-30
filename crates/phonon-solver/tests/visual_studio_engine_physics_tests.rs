#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Native Engine & WebAssembly Real-Time Physics Interactive Co-Processor.

use phonon_models::visual_studio_engine::VisualStudioEngineParams;
use phonon_solver::visual_studio_engine::VisualStudioEngineSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = VisualStudioEngineParams::new(
        256.0,  // below 512.0 pixels
        15.0,   // below 30.0 fps
        500.0,  // below 1000.0 nodes
        32.0,   // below 64.0 kb
        -1.0,   // below 0.0 tier
        5.0,    // below 10.0 hz
        128.0,  // below 256.0 pages
        0.5,    // below 1.0 mK
    );
    assert_eq!(underflow.canvas_render_resolution_pixels, 512.0);
    assert_eq!(underflow.target_frame_rate_fps, 30.0);
    assert_eq!(underflow.multi_physics_mesh_nodes, 1000.0);
    assert_eq!(underflow.ipc_buffer_size_kb, 64.0);
    assert_eq!(underflow.realism_tier_active, 0.0);
    assert_eq!(underflow.interactive_parameter_update_rate_hz, 10.0);
    assert_eq!(underflow.wasm_memory_pool_pages, 256.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);

    // Test values strictly above physical maximum bounds
    let overflow = VisualStudioEngineParams::new(
        16384.0,   // above 8192.0 pixels
        300.0,     // above 240.0 fps
        2000000.0, // above 1000000.0 nodes
        32768.0,   // above 16384.0 kb
        8.0,       // above 6.0 tier
        1500.0,    // above 1000.0 hz
        131072.0,  // above 65536.0 pages
        75.0,      // above 50.0 mK
    );
    assert_eq!(overflow.canvas_render_resolution_pixels, 8192.0);
    assert_eq!(overflow.target_frame_rate_fps, 240.0);
    assert_eq!(overflow.multi_physics_mesh_nodes, 1000000.0);
    assert_eq!(overflow.ipc_buffer_size_kb, 16384.0);
    assert_eq!(overflow.realism_tier_active, 6.0);
    assert_eq!(overflow.interactive_parameter_update_rate_hz, 1000.0);
    assert_eq!(overflow.wasm_memory_pool_pages, 65536.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = VisualStudioEngineParams::default();
    assert_eq!(params.canvas_render_resolution_pixels, 2048.0);
    assert_eq!(params.target_frame_rate_fps, 60.0);
    assert_eq!(params.multi_physics_mesh_nodes, 100000.0);
    assert_eq!(params.ipc_buffer_size_kb, 1024.0);
    assert_eq!(params.realism_tier_active, 0.0);
    assert_eq!(params.interactive_parameter_update_rate_hz, 120.0);
    assert_eq!(params.wasm_memory_pool_pages, 4096.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);

    let solver = VisualStudioEngineSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.engine_frame_fidelity >= 0.9980,
        "Engine frame fidelity must be >= 0.9980, got {:.6}",
        metrics.engine_frame_fidelity
    );
    assert!(
        metrics.interactive_state_retention_fraction >= 0.9970,
        "Interactive state retention fraction must be >= 0.9970, got {:.6}",
        metrics.interactive_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_tier_crosstalk_isolation_db >= 55.0,
        "Inter-tier crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_tier_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 12.0,
        "Topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_canvas_resolution_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.canvas_render_resolution_pixels = 1024.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.canvas_render_resolution_pixels = 4096.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_target_frame_rate_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.target_frame_rate_fps = 45.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.target_frame_rate_fps = 144.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_multi_physics_mesh_nodes_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.multi_physics_mesh_nodes = 5000.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.multi_physics_mesh_nodes = 500000.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_ipc_buffer_size_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.ipc_buffer_size_kb = 128.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.ipc_buffer_size_kb = 8192.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_realism_tier_active_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.realism_tier_active = 0.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.realism_tier_active = 5.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_interactive_parameter_update_rate_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.interactive_parameter_update_rate_hz = 30.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.interactive_parameter_update_rate_hz = 500.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_wasm_memory_pool_pages_scaling() {
    let mut low_p = VisualStudioEngineParams::default();
    low_p.wasm_memory_pool_pages = 512.0;
    let mut high_p = VisualStudioEngineParams::default();
    high_p.wasm_memory_pool_pages = 32768.0;

    let low_m = VisualStudioEngineSolver::new(low_p).evaluate_metrics();
    let high_m = VisualStudioEngineSolver::new(high_p).evaluate_metrics();

    assert!(high_m.engine_frame_fidelity > low_m.engine_frame_fidelity);
    assert!(high_m.interactive_state_retention_fraction > low_m.interactive_state_retention_fraction);
    assert!(high_m.topological_protection_gap_mhz > low_m.topological_protection_gap_mhz);
    assert!(high_m.inter_tier_crosstalk_isolation_db > low_m.inter_tier_crosstalk_isolation_db);
    assert!(high_m.topological_mode_dephasing_rate_hz < low_m.topological_mode_dephasing_rate_hz);
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold_p = VisualStudioEngineParams::default();
    cold_p.cryogenic_temperature_mk = 2.0;
    let mut warm_p = VisualStudioEngineParams::default();
    warm_p.cryogenic_temperature_mk = 45.0;

    let cold_m = VisualStudioEngineSolver::new(cold_p).evaluate_metrics();
    let warm_m = VisualStudioEngineSolver::new(warm_p).evaluate_metrics();

    assert!(cold_m.engine_frame_fidelity > warm_m.engine_frame_fidelity);
    assert!(cold_m.interactive_state_retention_fraction > warm_m.interactive_state_retention_fraction);
    assert!(cold_m.topological_protection_gap_mhz > warm_m.topological_protection_gap_mhz);
    assert!(cold_m.inter_tier_crosstalk_isolation_db > warm_m.inter_tier_crosstalk_isolation_db);
    assert!(cold_m.topological_mode_dephasing_rate_hz < warm_m.topological_mode_dephasing_rate_hz);
}
