#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological phononic
//! Floquet-Majorana braiding processors and non-Abelian topological logic.

use phonon_models::floquet_majorana_braiding_processor::FloquetMajoranaBraidingProcessorParams;
use phonon_solver::floquet_majorana_braiding_processor::FloquetMajoranaBraidingProcessorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = FloquetMajoranaBraidingProcessorParams::new(
        0.5,  // below 1.0 GHz
        5.0,  // below 10.0 MHz
        0.05, // below 0.1 rad
        2.0,  // below 5.0 um
        2.0,  // below 5.0 MHz
        0.1,  // below 0.5 kHz
        0.5,  // below 1.0 mK
        1,    // below 3
    );
    assert!((underflow.floquet_drive_freq_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.floquet_modulation_amplitude_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.synthetic_gauge_flux_rad - 0.1).abs() < 1e-9);
    assert!((underflow.phononic_waveguide_length_um - 5.0).abs() < 1e-9);
    assert!((underflow.majorana_coupling_gap_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.acoustic_loss_rate_khz - 0.5).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert_eq!(underflow.braiding_nodes_count, 3);

    // Test values above physical maximum bounds
    let overflow = FloquetMajoranaBraidingProcessorParams::new(
        25.0,  // above 20.0 GHz
        200.0, // above 150.0 MHz
        5.0,   // above 3.14159 rad
        150.0, // above 100.0 um
        100.0, // above 80.0 MHz
        70.0,  // above 50.0 kHz
        70.0,  // above 50.0 mK
        20,    // above 12
    );
    assert!((overflow.floquet_drive_freq_ghz - 20.0).abs() < 1e-9);
    assert!((overflow.floquet_modulation_amplitude_mhz - 150.0).abs() < 1e-9);
    assert!((overflow.synthetic_gauge_flux_rad - 3.14159).abs() < 1e-9);
    assert!((overflow.phononic_waveguide_length_um - 100.0).abs() < 1e-9);
    assert!((overflow.majorana_coupling_gap_mhz - 80.0).abs() < 1e-9);
    assert!((overflow.acoustic_loss_rate_khz - 50.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert_eq!(overflow.braiding_nodes_count, 12);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetMajoranaBraidingProcessorParams::default();
    let solver = FloquetMajoranaBraidingProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_gate_fidelity >= 0.9980,
        "Default braiding gate fidelity must be >= 0.9980, got {:.5}",
        metrics.braiding_gate_fidelity
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 15.0,
        "Default topological protection gap must be >= 15.0 MHz, got {:.3} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.operation_latency_ns <= 150.0,
        "Default operation latency must be <= 150.0 ns, got {:.3} ns",
        metrics.operation_latency_ns
    );
    assert!(
        metrics.edge_state_isolation_db >= 40.0,
        "Default edge state isolation must be >= 40.0 dB, got {:.3} dB",
        metrics.edge_state_isolation_db
    );
    assert!(
        metrics.non_abelian_state_purity >= 0.9950,
        "Default non-Abelian state purity must be >= 0.9950, got {:.5}",
        metrics.non_abelian_state_purity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_modulation_scaling() {
    let base = FloquetMajoranaBraidingProcessorParams::default();
    let solver_base = FloquetMajoranaBraidingProcessorSolver::new(base);
    let gap_base = solver_base.compute_topological_protection_gap_mhz();

    let high_mod = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        120.0, // increased modulation amplitude from 60.0 to 120.0 MHz
        base.synthetic_gauge_flux_rad,
        base.phononic_waveguide_length_um,
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.braiding_nodes_count,
    );
    let solver_high = FloquetMajoranaBraidingProcessorSolver::new(high_mod);
    let gap_high = solver_high.compute_topological_protection_gap_mhz();

    assert!(
        gap_high > gap_base,
        "Increasing modulation amplitude must increase topological protection gap: base={:.3} MHz, high={:.3} MHz",
        gap_base,
        gap_high
    );
}

#[test]
fn test_flux_scaling() {
    let base = FloquetMajoranaBraidingProcessorParams::default();
    let low_flux_params = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        100.0, // elevated drive to operate above clamp floor
        1.0,   // flux = 1.0 rad
        base.phononic_waveguide_length_um,
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.braiding_nodes_count,
    );
    let high_flux_params = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        100.0,
        3.0, // flux = 3.0 rad (approaching pi)
        base.phononic_waveguide_length_um,
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.braiding_nodes_count,
    );

    let solver_low = FloquetMajoranaBraidingProcessorSolver::new(low_flux_params);
    let solver_high = FloquetMajoranaBraidingProcessorSolver::new(high_flux_params);

    let gap_low = solver_low.compute_topological_protection_gap_mhz();
    let gap_high = solver_high.compute_topological_protection_gap_mhz();

    assert!(
        gap_high > gap_low,
        "Increasing synthetic gauge flux towards pi must increase topological protection gap: low={:.3} MHz, high={:.3} MHz",
        gap_low,
        gap_high
    );
}

#[test]
fn test_temperature_degradation() {
    let base = FloquetMajoranaBraidingProcessorParams::default();
    let low_temp = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        base.floquet_modulation_amplitude_mhz,
        base.synthetic_gauge_flux_rad,
        base.phononic_waveguide_length_um,
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        5.0, // 5.0 mK
        base.braiding_nodes_count,
    );
    let high_temp = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        base.floquet_modulation_amplitude_mhz,
        base.synthetic_gauge_flux_rad,
        base.phononic_waveguide_length_um,
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        25.0, // 25.0 mK
        base.braiding_nodes_count,
    );

    let solver_low = FloquetMajoranaBraidingProcessorSolver::new(low_temp);
    let solver_high = FloquetMajoranaBraidingProcessorSolver::new(high_temp);

    let fid_low = solver_low.compute_braiding_gate_fidelity();
    let fid_high = solver_high.compute_braiding_gate_fidelity();
    assert!(
        fid_low > fid_high,
        "Higher temperature must degrade braiding gate fidelity: low_temp={:.5}, high_temp={:.5}",
        fid_low,
        fid_high
    );

    let purity_low = solver_low.compute_non_abelian_state_purity();
    let purity_high = solver_high.compute_non_abelian_state_purity();
    assert!(
        purity_low > purity_high,
        "Higher temperature must degrade non-Abelian quantum state purity: low_temp={:.5}, high_temp={:.5}",
        purity_low,
        purity_high
    );

    let iso_low = solver_low.compute_edge_state_isolation_db();
    let iso_high = solver_high.compute_edge_state_isolation_db();
    assert!(
        iso_low > iso_high,
        "Higher temperature must degrade edge state isolation: low_temp={:.3} dB, high_temp={:.3} dB",
        iso_low,
        iso_high
    );
}

#[test]
fn test_waveguide_length_latency_scaling() {
    let base = FloquetMajoranaBraidingProcessorParams::default();
    let short_wg = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        base.floquet_modulation_amplitude_mhz,
        base.synthetic_gauge_flux_rad,
        10.0, // 10.0 um
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.braiding_nodes_count,
    );
    let long_wg = FloquetMajoranaBraidingProcessorParams::new(
        base.floquet_drive_freq_ghz,
        base.floquet_modulation_amplitude_mhz,
        base.synthetic_gauge_flux_rad,
        35.0, // 35.0 um
        base.majorana_coupling_gap_mhz,
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.braiding_nodes_count,
    );

    let solver_short = FloquetMajoranaBraidingProcessorSolver::new(short_wg);
    let solver_long = FloquetMajoranaBraidingProcessorSolver::new(long_wg);

    let tau_short = solver_short.compute_operation_latency_ns();
    let tau_long = solver_long.compute_operation_latency_ns();

    assert!(
        tau_long > tau_short,
        "Longer waveguide must increase braiding operation latency: short={:.3} ns, long={:.3} ns",
        tau_short,
        tau_long
    );
}

#[test]
fn test_physical_compliance() {
    let params = FloquetMajoranaBraidingProcessorParams::default();
    let solver = FloquetMajoranaBraidingProcessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
}
