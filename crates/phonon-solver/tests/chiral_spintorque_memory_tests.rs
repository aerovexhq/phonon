#![deny(unsafe_code)]

//! Integration test suite for Phase 443:
//! Chiral Phonon-Magnon Spin-Torque Acoustic Memory & Cryogenic Superconducting Spintronic Crossbar.

use phonon_solver::chiral_spintorque_memory::{
    AcousticSpinTorqueParams, AcousticSpinTorqueSolver, ChiralSpinTorqueMemoryProcessor,
    MemoryCellState, PolaritonWritingHeadParams, PolaritonWritingHeadSolver,
    SpintronicCrossbarParams, SpintronicCrossbarSolver,
};

#[test]
fn test_chiral_acoustic_spin_torque_switching_and_thermal_stability() {
    let mut params = AcousticSpinTorqueParams::default();
    params.acoustic_strain_amplitude = 1.8e-4;
    params.phonon_chirality_sign = 1.0;

    let solver = AcousticSpinTorqueSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // 1. Verify sub-nanosecond magnetization switching latency
    assert!(
        metrics.switching_latency_ns <= 1.0,
        "Switching latency {:.3} ns must be <= 1.0 ns",
        metrics.switching_latency_ns
    );

    // 2. Verify critical strain amplitude <= 2.5e-4
    assert!(
        metrics.critical_strain_amplitude <= 2.5e-4,
        "Critical strain {:.2e} must be <= 2.5e-4",
        metrics.critical_strain_amplitude
    );

    // 3. Verify non-volatile thermal stability factor Delta >= 60.0
    assert!(
        metrics.thermal_stability_factor >= 60.0,
        "Thermal stability factor {:.1} must be >= 60.0",
        metrics.thermal_stability_factor
    );

    // 4. Verify low write energy dissipation <= 15.0 fJ
    assert!(
        metrics.write_energy_fj <= 15.0,
        "Write energy {:.2} fJ must be <= 15.0 fJ",
        metrics.write_energy_fj
    );

    // 5. Verify deterministic switching aligned with phonon chirality
    assert_eq!(metrics.final_magnetization_mz, 1.0);

    // Test reverse chirality switching (-1.0)
    params.phonon_chirality_sign = -1.0;
    let rev_solver = AcousticSpinTorqueSolver::new(params);
    let rev_metrics = rev_solver.evaluate_metrics();
    assert_eq!(rev_metrics.final_magnetization_mz, -1.0);

    // 6. Verify 3D magnetization vector trajectory
    let traj = solver.compute_trajectory(32);
    assert_eq!(traj.len(), 32);
    let initial = &traj[0];
    let final_pt = &traj[traj.len() - 1];
    assert!((initial.m_z - (-1.0)).abs() < 0.1);
    assert!((final_pt.m_z - 1.0).abs() < 0.1);
}

#[test]
fn test_polariton_writing_head_focusing_and_directionality() {
    let params = PolaritonWritingHeadParams::default();
    let solver = PolaritonWritingHeadSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Verify directional write isolation >= 30.0 dB
    assert!(
        metrics.directional_isolation_db >= 30.0,
        "Directional isolation {:.1} dB must be >= 30.0 dB",
        metrics.directional_isolation_db
    );

    // 2. Verify sub-45nm focal spot size FWHM <= 45.0 nm
    assert!(
        metrics.focal_spot_fwhm_nm <= 45.0,
        "Focal spot size {:.1} nm must be <= 45.0 nm",
        metrics.focal_spot_fwhm_nm
    );

    // 3. Verify forward write efficiency >= 85.0%
    assert!(
        metrics.forward_efficiency_percent >= 85.0,
        "Forward efficiency {:.1}% must be >= 85.0%",
        metrics.forward_efficiency_percent
    );

    // 4. Verify dissipation per bit <= 15.0 fJ
    assert!(
        metrics.dissipation_per_bit_fj <= 15.0,
        "Dissipation {:.2} fJ must be <= 15.0 fJ",
        metrics.dissipation_per_bit_fj
    );

    // 5. Verify spatial strain profile computation
    let profile = solver.compute_spatial_strain_profile(35);
    assert_eq!(profile.len(), 35);

    // 6. Verify isolation spectrum computation
    let spectrum = solver.compute_isolation_spectrum(30);
    assert_eq!(spectrum.len(), 30);
}

#[test]
fn test_spintronic_crossbar_readout_and_crosstalk() {
    let mut params = SpintronicCrossbarParams::default();
    params.num_rows = 8;
    params.num_cols = 8;
    params.selected_row = 2;
    params.selected_col = 3;

    let mut solver = SpintronicCrossbarSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // 1. Verify TMR ratio >= 180.0%
    assert!(
        metrics.measured_tmr_percent >= 180.0,
        "TMR ratio {:.1}% must be >= 180.0%",
        metrics.measured_tmr_percent
    );

    // 2. Verify crossbar line crosstalk isolation >= 35.0 dB
    assert!(
        metrics.crosstalk_isolation_db >= 35.0,
        "Crosstalk isolation {:.1} dB must be >= 35.0 dB",
        metrics.crosstalk_isolation_db
    );

    // 3. Verify fast readout latency <= 2.0 ns
    assert!(
        metrics.readout_latency_ns <= 2.0,
        "Readout latency {:.2} ns must be <= 2.0 ns",
        metrics.readout_latency_ns
    );

    // 4. Verify SQUID readout SNR >= 22.0 dB
    assert!(
        metrics.readout_snr_db >= 22.0,
        "Readout SNR {:.1} dB must be >= 22.0 dB",
        metrics.readout_snr_db
    );

    // 5. Test addressing and bit flipping
    solver.set_cell(2, 3, MemoryCellState::AntiParallelState1);
    assert_eq!(solver.get_cell(2, 3), MemoryCellState::AntiParallelState1);
    assert!(solver.get_cell(2, 3).is_logic_one());

    solver.set_cell(2, 3, MemoryCellState::ParallelState0);
    assert_eq!(solver.get_cell(2, 3), MemoryCellState::ParallelState0);
    assert!(!solver.get_cell(2, 3).is_logic_one());

    // 6. Verify SQUID readout trace computation
    let trace = solver.compute_readout_trace(30);
    assert_eq!(trace.len(), 30);
}

#[test]
fn test_chiral_spintorque_memory_10_point_audit_all_passed() {
    let processor = ChiralSpinTorqueMemoryProcessor::new();
    let audit = processor.audit_memory_system();

    assert!(audit.sub_ns_switching_latency_passed, "Audit 1 failed");
    assert!(audit.threshold_critical_strain_passed, "Audit 2 failed");
    assert!(audit.non_volatile_thermal_stability_passed, "Audit 3 failed");
    assert!(audit.polariton_directional_isolation_passed, "Audit 4 failed");
    assert!(audit.low_write_dissipation_passed, "Audit 5 failed");
    assert!(audit.sub_45nm_focal_spot_passed, "Audit 6 failed");
    assert!(audit.high_tmr_ratio_passed, "Audit 7 failed");
    assert!(audit.crossbar_crosstalk_isolation_passed, "Audit 8 failed");
    assert!(audit.fast_readout_latency_passed, "Audit 9 failed");
    assert!(audit.squid_readout_snr_passed, "Audit 10 failed");

    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
