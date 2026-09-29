#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological acoustic
//! higher-order corner mode lasers and non-Hermitian phonon cavities.

use phonon_models::topological_corner_laser::TopologicalCornerLaserParams;
use phonon_solver::topological_corner_laser::TopologicalCornerLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = TopologicalCornerLaserParams::new(
        0.5,    // below 1.0 GHz
        5.0,    // below 10.0 MHz
        1.0,    // below 2.0 MHz
        2.0,    // below 5.0 uW
        0.5,    // below 1.0 MHz
        0.2,    // below 0.5 MHz
        0.5,    // below 1.0 mK
        -1.0,   // below 0.0 %
    );
    assert!((underflow.acoustic_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.inter_cell_hopping_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.intra_cell_hopping_mhz - 2.0).abs() < 1e-9);
    assert!((underflow.optical_pump_power_uw - 5.0).abs() < 1e-9);
    assert!((underflow.non_hermitian_gain_mhz - 1.0).abs() < 1e-9);
    assert!((underflow.acoustic_loss_rate_mhz - 0.5).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.disorder_amplitude_percent - 0.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = TopologicalCornerLaserParams::new(
        20.0,   // above 15.0 GHz
        150.0,  // above 100.0 MHz
        60.0,   // above 40.0 MHz
        150.0,  // above 100.0 uW
        50.0,   // above 30.0 MHz
        20.0,   // above 10.0 MHz
        75.0,   // above 50.0 mK
        20.0,   // above 10.0 %
    );
    assert!((overflow.acoustic_frequency_ghz - 15.0).abs() < 1e-9);
    assert!((overflow.inter_cell_hopping_mhz - 100.0).abs() < 1e-9);
    assert!((overflow.intra_cell_hopping_mhz - 40.0).abs() < 1e-9);
    assert!((overflow.optical_pump_power_uw - 100.0).abs() < 1e-9);
    assert!((overflow.non_hermitian_gain_mhz - 30.0).abs() < 1e-9);
    assert!((overflow.acoustic_loss_rate_mhz - 10.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.disorder_amplitude_percent - 10.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TopologicalCornerLaserParams::default();
    let solver = TopologicalCornerLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.corner_lasing_efficiency >= 0.750,
        "Default corner lasing efficiency must be >= 0.750, got {:.4}",
        metrics.corner_lasing_efficiency
    );
    assert!(
        metrics.threshold_power_uw <= 10.0,
        "Default threshold power must be <= 10.0 uW, got {:.4} uW",
        metrics.threshold_power_uw
    );
    assert!(
        metrics.corner_mode_localization >= 0.920,
        "Default corner mode localization must be >= 0.920, got {:.4}",
        metrics.corner_mode_localization
    );
    assert!(
        metrics.mode_discrimination_db >= 25.0,
        "Default mode discrimination must be >= 25.0 dB, got {:.3} dB",
        metrics.mode_discrimination_db
    );
    assert!(
        metrics.emission_linewidth_khz <= 5.0,
        "Default emission linewidth must be <= 5.0 kHz, got {:.4} kHz",
        metrics.emission_linewidth_khz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_hopping_ratio_scaling() {
    let base = TopologicalCornerLaserParams::default();
    let solver_base = TopologicalCornerLaserSolver::new(base);
    let loc_base = solver_base.compute_corner_mode_localization();

    // Decrease intra-cell hopping to strengthen dimerization and quadrupole topology
    let deeper_topology = TopologicalCornerLaserParams::new(
        base.acoustic_frequency_ghz,
        base.inter_cell_hopping_mhz,
        6.0, // reduced from 12.0 MHz
        base.optical_pump_power_uw,
        base.non_hermitian_gain_mhz,
        base.acoustic_loss_rate_mhz,
        base.operating_temp_m_k,
        base.disorder_amplitude_percent,
    );
    let solver_deep = TopologicalCornerLaserSolver::new(deeper_topology);
    let loc_deep = solver_deep.compute_corner_mode_localization();

    assert!(
        loc_deep >= loc_base,
        "Smaller intra-to-inter hopping ratio must enhance corner mode localization (base: {:.4}, deep: {:.4})",
        loc_base,
        loc_deep
    );
}

#[test]
fn test_gain_scaling() {
    let base = TopologicalCornerLaserParams::default();
    let solver_base = TopologicalCornerLaserSolver::new(base);
    let eff_base = solver_base.compute_corner_lasing_efficiency();
    let pth_base = solver_base.compute_threshold_power_uw();
    let md_base = solver_base.compute_mode_discrimination_db();

    // Increase non-Hermitian localized gain
    let higher_gain = TopologicalCornerLaserParams::new(
        base.acoustic_frequency_ghz,
        base.inter_cell_hopping_mhz,
        base.intra_cell_hopping_mhz,
        base.optical_pump_power_uw,
        22.0, // increased from 15.0 MHz
        base.acoustic_loss_rate_mhz,
        base.operating_temp_m_k,
        base.disorder_amplitude_percent,
    );
    let solver_high = TopologicalCornerLaserSolver::new(higher_gain);
    let eff_high = solver_high.compute_corner_lasing_efficiency();
    let pth_high = solver_high.compute_threshold_power_uw();
    let md_high = solver_high.compute_mode_discrimination_db();

    assert!(
        eff_high >= eff_base,
        "Higher non-Hermitian gain must increase lasing slope efficiency (base: {:.4}, high: {:.4})",
        eff_base,
        eff_high
    );
    assert!(
        pth_high <= pth_base,
        "Higher non-Hermitian gain must reduce threshold power (base: {:.4}, high: {:.4})",
        pth_base,
        pth_high
    );
    assert!(
        md_high >= md_base,
        "Higher non-Hermitian gain must increase mode discrimination (base: {:.3}, high: {:.3})",
        md_base,
        md_high
    );
}

#[test]
fn test_disorder_degradation() {
    let base = TopologicalCornerLaserParams::default();
    let solver_base = TopologicalCornerLaserSolver::new(base);
    let eff_base = solver_base.compute_corner_lasing_efficiency();
    let pth_base = solver_base.compute_threshold_power_uw();
    let loc_base = solver_base.compute_corner_mode_localization();
    let md_base = solver_base.compute_mode_discrimination_db();
    let lw_base = solver_base.compute_emission_linewidth_khz();

    // Elevated metamaterial fabrication disorder amplitude (5.0% vs 2.0%)
    let disordered = TopologicalCornerLaserParams::new(
        base.acoustic_frequency_ghz,
        base.inter_cell_hopping_mhz,
        base.intra_cell_hopping_mhz,
        base.optical_pump_power_uw,
        base.non_hermitian_gain_mhz,
        base.acoustic_loss_rate_mhz,
        base.operating_temp_m_k,
        5.0, // increased disorder
    );
    let solver_disordered = TopologicalCornerLaserSolver::new(disordered);
    let eff_disorder = solver_disordered.compute_corner_lasing_efficiency();
    let pth_disorder = solver_disordered.compute_threshold_power_uw();
    let loc_disorder = solver_disordered.compute_corner_mode_localization();
    let md_disorder = solver_disordered.compute_mode_discrimination_db();
    let lw_disorder = solver_disordered.compute_emission_linewidth_khz();

    assert!(
        eff_disorder <= eff_base,
        "Higher disorder must degrade lasing efficiency (base: {:.4}, disordered: {:.4})",
        eff_base,
        eff_disorder
    );
    assert!(
        pth_disorder >= pth_base,
        "Higher disorder must increase threshold power (base: {:.4}, disordered: {:.4})",
        pth_base,
        pth_disorder
    );
    assert!(
        loc_disorder <= loc_base,
        "Higher disorder must degrade corner mode localization (base: {:.4}, disordered: {:.4})",
        loc_base,
        loc_disorder
    );
    assert!(
        md_disorder <= md_base,
        "Higher disorder must degrade mode discrimination (base: {:.3}, disordered: {:.3})",
        md_base,
        md_disorder
    );
    assert!(
        lw_disorder >= lw_base,
        "Higher disorder must broaden emission linewidth (base: {:.4}, disordered: {:.4})",
        lw_base,
        lw_disorder
    );
}

#[test]
fn test_temperature_degradation() {
    let base = TopologicalCornerLaserParams::default();
    let solver_base = TopologicalCornerLaserSolver::new(base);
    let eff_base = solver_base.compute_corner_lasing_efficiency();
    let pth_base = solver_base.compute_threshold_power_uw();
    let md_base = solver_base.compute_mode_discrimination_db();
    let lw_base = solver_base.compute_emission_linewidth_khz();

    // Warmer cryogenic operating temperature (25.0 mK vs 15.0 mK)
    let warmer = TopologicalCornerLaserParams::new(
        base.acoustic_frequency_ghz,
        base.inter_cell_hopping_mhz,
        base.intra_cell_hopping_mhz,
        base.optical_pump_power_uw,
        base.non_hermitian_gain_mhz,
        base.acoustic_loss_rate_mhz,
        25.0, // elevated temperature
        base.disorder_amplitude_percent,
    );
    let solver_warmer = TopologicalCornerLaserSolver::new(warmer);
    let eff_warm = solver_warmer.compute_corner_lasing_efficiency();
    let pth_warm = solver_warmer.compute_threshold_power_uw();
    let md_warm = solver_warmer.compute_mode_discrimination_db();
    let lw_warm = solver_warmer.compute_emission_linewidth_khz();

    assert!(
        eff_warm <= eff_base,
        "Warmer temperature must decrease lasing efficiency (base: {:.4}, warmer: {:.4})",
        eff_base,
        eff_warm
    );
    assert!(
        pth_warm >= pth_base,
        "Warmer temperature must increase threshold power (base: {:.4}, warmer: {:.4})",
        pth_base,
        pth_warm
    );
    assert!(
        md_warm <= md_base,
        "Warmer temperature must degrade mode discrimination (base: {:.3}, warmer: {:.3})",
        md_base,
        md_warm
    );
    assert!(
        lw_warm >= lw_base,
        "Warmer temperature must broaden emission linewidth (base: {:.4}, warmer: {:.4})",
        lw_base,
        lw_warm
    );
}

#[test]
fn test_physical_compliance() {
    let params = TopologicalCornerLaserParams::default();
    let solver = TopologicalCornerLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.corner_lasing_efficiency >= 0.750);
    assert!(metrics.threshold_power_uw <= 10.0);
    assert!(metrics.corner_mode_localization >= 0.920);
    assert!(metrics.mode_discrimination_db >= 25.0);
    assert!(metrics.emission_linewidth_khz <= 5.0);
}
