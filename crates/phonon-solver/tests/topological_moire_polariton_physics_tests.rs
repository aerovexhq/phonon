#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological moire
//! acoustic polaritonic lattices and flat-band phonon superfluidity.

use phonon_models::topological_moire_polariton::TopologicalMoirePolaritonParams;
use phonon_solver::topological_moire_polariton::TopologicalMoirePolaritonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = TopologicalMoirePolaritonParams::new(
        0.1,    // below 0.5 deg
        0.5,    // below 1.0 GHz
        5.0,    // below 10.0 MHz
        20.0,   // below 50.0 nm
        0.1,    // below 0.5 ueV*um^2
        0.2,    // below 1.0 mK
        20.0,   // below 50.0 ps
        1.0e5,  // below 1.0e6
    );
    assert!((underflow.twist_angle_deg - 0.5).abs() < 1e-9);
    assert!((underflow.acoustic_center_freq_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.interlayer_tunneling_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.moire_period_nm - 50.0).abs() < 1e-9);
    assert!((underflow.non_linear_polariton_interaction_uev_um2 - 0.5).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.polariton_lifetime_ps - 50.0).abs() < 1e-9);
    assert!((underflow.acoustic_quality_factor - 1.0e6).abs() < 1e-3);

    // Test values above physical maximum bounds
    let overflow = TopologicalMoirePolaritonParams::new(
        10.0,   // above 5.0 deg
        25.0,   // above 15.0 GHz
        250.0,  // above 150.0 MHz
        800.0,  // above 500.0 nm
        45.0,   // above 20.0 ueV*um^2
        100.0,  // above 50.0 mK
        2500.0, // above 1000.0 ps
        5.0e8,  // above 1.0e8
    );
    assert!((overflow.twist_angle_deg - 5.0).abs() < 1e-9);
    assert!((overflow.acoustic_center_freq_ghz - 15.0).abs() < 1e-9);
    assert!((overflow.interlayer_tunneling_mhz - 150.0).abs() < 1e-9);
    assert!((overflow.moire_period_nm - 500.0).abs() < 1e-9);
    assert!((overflow.non_linear_polariton_interaction_uev_um2 - 20.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.polariton_lifetime_ps - 1000.0).abs() < 1e-9);
    assert!((overflow.acoustic_quality_factor - 1.0e8).abs() < 1e-3);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TopologicalMoirePolaritonParams::default();
    let solver = TopologicalMoirePolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.superfluid_velocity_m_per_s >= 2500.0,
        "Default superfluid velocity must be >= 2500.0 m/s, got {:.4} m/s",
        metrics.superfluid_velocity_m_per_s
    );
    assert!(
        metrics.propagation_loss_db_per_cm <= 0.020,
        "Default propagation loss must be <= 0.020 dB/cm, got {:.4} dB/cm",
        metrics.propagation_loss_db_per_cm
    );
    assert!(
        metrics.condensation_threshold_density <= 5.0e12,
        "Default condensation threshold density must be <= 5.0e12 m^-2, got {:.4e}",
        metrics.condensation_threshold_density
    );
    assert_eq!(
        metrics.chern_number, 1,
        "Default Chern number must be 1, got {}",
        metrics.chern_number
    );
    assert!(
        metrics.flat_band_bandwidth_mhz <= 2.0,
        "Default flat-band bandwidth must be <= 2.0 MHz, got {:.4} MHz",
        metrics.flat_band_bandwidth_mhz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_twist_angle_scaling() {
    let magic = TopologicalMoirePolaritonParams::default();
    let solver_magic = TopologicalMoirePolaritonSolver::new(magic);
    let w_magic = solver_magic.compute_flat_band_bandwidth_mhz();
    let vs_magic = solver_magic.compute_superfluid_velocity_m_per_s();

    // Detuned twist angle (e.g. 1.25 deg)
    let detuned = TopologicalMoirePolaritonParams::new(
        1.25,
        magic.acoustic_center_freq_ghz,
        magic.interlayer_tunneling_mhz,
        magic.moire_period_nm,
        magic.non_linear_polariton_interaction_uev_um2,
        magic.operating_temp_m_k,
        magic.polariton_lifetime_ps,
        magic.acoustic_quality_factor,
    );
    let solver_detuned = TopologicalMoirePolaritonSolver::new(detuned);
    let w_detuned = solver_detuned.compute_flat_band_bandwidth_mhz();
    let vs_detuned = solver_detuned.compute_superfluid_velocity_m_per_s();

    assert!(
        w_detuned > w_magic,
        "Bandwidth must broaden away from magic twist angle: detuned {:.4} MHz vs magic {:.4} MHz",
        w_detuned,
        w_magic
    );
    assert!(
        vs_magic > vs_detuned,
        "Superfluid velocity must decrease away from magic angle: magic {:.4} m/s vs detuned {:.4} m/s",
        vs_magic,
        vs_detuned
    );
}

#[test]
fn test_temperature_degradation() {
    let low_temp = TopologicalMoirePolaritonParams::new(1.08, 4.2, 55.0, 180.0, 5.5, 5.0, 350.0, 2.5e7);
    let high_temp = TopologicalMoirePolaritonParams::new(1.08, 4.2, 55.0, 180.0, 5.5, 30.0, 350.0, 2.5e7);

    let solver_low = TopologicalMoirePolaritonSolver::new(low_temp);
    let solver_high = TopologicalMoirePolaritonSolver::new(high_temp);

    let vs_low = solver_low.compute_superfluid_velocity_m_per_s();
    let vs_high = solver_high.compute_superfluid_velocity_m_per_s();
    assert!(
        vs_low > vs_high,
        "Superfluid velocity must degrade with temperature: low T {:.4} m/s vs high T {:.4} m/s",
        vs_low,
        vs_high
    );

    let loss_low = solver_low.compute_propagation_loss_db_per_cm();
    let loss_high = solver_high.compute_propagation_loss_db_per_cm();
    assert!(
        loss_high > loss_low,
        "Propagation loss must increase with temperature: low T {:.4} dB/cm vs high T {:.4} dB/cm",
        loss_low,
        loss_high
    );

    let nth_low = solver_low.compute_condensation_threshold_density();
    let nth_high = solver_high.compute_condensation_threshold_density();
    assert!(
        nth_high > nth_low,
        "Condensation threshold must increase with temperature: low T {:.4e} vs high T {:.4e}",
        nth_low,
        nth_high
    );

    let w_low = solver_low.compute_flat_band_bandwidth_mhz();
    let w_high = solver_high.compute_flat_band_bandwidth_mhz();
    assert!(
        w_high > w_low,
        "Bandwidth must broaden with temperature: low T {:.4} MHz vs high T {:.4} MHz",
        w_low,
        w_high
    );
}

#[test]
fn test_loss_scaling() {
    let base = TopologicalMoirePolaritonParams::default();
    let solver_base = TopologicalMoirePolaritonSolver::new(base);
    let loss_base = solver_base.compute_propagation_loss_db_per_cm();

    // High quality factor reduces acoustic damping loss
    let high_q = TopologicalMoirePolaritonParams::new(
        base.twist_angle_deg,
        base.acoustic_center_freq_ghz,
        base.interlayer_tunneling_mhz,
        base.moire_period_nm,
        base.non_linear_polariton_interaction_uev_um2,
        base.operating_temp_m_k,
        base.polariton_lifetime_ps,
        8.0e7,
    );
    let solver_high_q = TopologicalMoirePolaritonSolver::new(high_q);
    let loss_high_q = solver_high_q.compute_propagation_loss_db_per_cm();

    assert!(
        loss_base > loss_high_q,
        "Higher Q factor must reduce propagation loss: base {:.5} dB/cm vs high Q {:.5} dB/cm",
        loss_base,
        loss_high_q
    );

    // Low quality factor increases loss
    let low_q = TopologicalMoirePolaritonParams::new(
        base.twist_angle_deg,
        base.acoustic_center_freq_ghz,
        base.interlayer_tunneling_mhz,
        base.moire_period_nm,
        base.non_linear_polariton_interaction_uev_um2,
        base.operating_temp_m_k,
        base.polariton_lifetime_ps,
        5.0e6,
    );
    let solver_low_q = TopologicalMoirePolaritonSolver::new(low_q);
    let loss_low_q = solver_low_q.compute_propagation_loss_db_per_cm();

    assert!(
        loss_low_q > loss_base,
        "Lower Q factor must increase propagation loss: low Q {:.5} dB/cm vs base {:.5} dB/cm",
        loss_low_q,
        loss_base
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let default_params = TopologicalMoirePolaritonParams::default();
    let default_solver = TopologicalMoirePolaritonSolver::new(default_params);
    let default_metrics = default_solver.evaluate_metrics();
    assert!(default_metrics.is_physically_compliant);

    // Test non-linear interaction scaling
    let high_int = TopologicalMoirePolaritonParams::new(
        default_params.twist_angle_deg,
        default_params.acoustic_center_freq_ghz,
        default_params.interlayer_tunneling_mhz,
        default_params.moire_period_nm,
        10.0,
        default_params.operating_temp_m_k,
        default_params.polariton_lifetime_ps,
        default_params.acoustic_quality_factor,
    );
    let solver_high_int = TopologicalMoirePolaritonSolver::new(high_int);
    let vs_high = solver_high_int.compute_superfluid_velocity_m_per_s();
    assert!(
        vs_high > default_metrics.superfluid_velocity_m_per_s,
        "Higher non-linear polariton interaction must increase superfluid velocity: high {:.4} vs base {:.4}",
        vs_high,
        default_metrics.superfluid_velocity_m_per_s
    );
}
