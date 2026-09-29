#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for cavity quantum
//! acoustodynamical (cQAD) spin-phonon interfaces and chiral squeezed vacuum synthesizers.

use phonon_models::cavity_acoustodynamical_spin::CavityAcoustodynamicalSpinParams;
use phonon_solver::cavity_acoustodynamical_spin::CavityAcoustodynamicalSpinSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = CavityAcoustodynamicalSpinParams::new(
        0.05,  // below 0.1 mW
        5.0,   // below 10.0 kHz
        0.2,   // below 0.5 MHz
        2.0,   // below 5.0 dB
        2.0,   // below 5.0 mK
        0.5,   // below 1.0 GHz
        0.5,   // below 1.0 Hz
        10.0,  // below 20.0 dB
    );
    assert!((underflow.pump_power_mw - 0.1).abs() < 1e-9);
    assert!((underflow.cavity_decay_rate_khz - 10.0).abs() < 1e-9);
    assert!((underflow.spin_phonon_coupling_mhz - 0.5).abs() < 1e-9);
    assert!((underflow.non_linear_gain_db - 5.0).abs() < 1e-9);
    assert!((underflow.cryogenic_temp_mk - 5.0).abs() < 1e-9);
    assert!((underflow.acoustic_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.spin_dephasing_rate_hz - 1.0).abs() < 1e-9);
    assert!((underflow.chiral_isolation_db - 20.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = CavityAcoustodynamicalSpinParams::new(
        30.0,  // above 20.0 mW
        600.0, // above 500.0 kHz
        35.0,  // above 25.0 MHz
        40.0,  // above 30.0 dB
        150.0, // above 100.0 mK
        20.0,  // above 15.0 GHz
        80.0,  // above 50.0 Hz
        80.0,  // above 60.0 dB
    );
    assert!((overflow.pump_power_mw - 20.0).abs() < 1e-9);
    assert!((overflow.cavity_decay_rate_khz - 500.0).abs() < 1e-9);
    assert!((overflow.spin_phonon_coupling_mhz - 25.0).abs() < 1e-9);
    assert!((overflow.non_linear_gain_db - 30.0).abs() < 1e-9);
    assert!((overflow.cryogenic_temp_mk - 100.0).abs() < 1e-9);
    assert!((overflow.acoustic_frequency_ghz - 15.0).abs() < 1e-9);
    assert!((overflow.spin_dephasing_rate_hz - 50.0).abs() < 1e-9);
    assert!((overflow.chiral_isolation_db - 60.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = CavityAcoustodynamicalSpinParams::default();
    let solver = CavityAcoustodynamicalSpinSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.acoustic_quadrature_squeezing_db >= 12.0,
        "Default acoustic quadrature squeezing must be >= 12.0 dB, got {:.4} dB",
        metrics.acoustic_quadrature_squeezing_db
    );
    assert!(
        metrics.spin_phonon_fidelity >= 0.9970,
        "Default spin-phonon fidelity must be >= 0.9970, got {:.6}",
        metrics.spin_phonon_fidelity
    );
    assert!(
        metrics.spin_coherence_lifetime_ms >= 50.0,
        "Default spin coherence lifetime must be >= 50.0 ms, got {:.4} ms",
        metrics.spin_coherence_lifetime_ms
    );
    assert!(
        metrics.thermal_phonon_occupancy <= 0.05,
        "Default thermal phonon occupancy must be <= 0.05, got {:.6}",
        metrics.thermal_phonon_occupancy
    );
    assert!(
        metrics.purcell_enhancement_factor >= 25.0,
        "Default Purcell enhancement factor must be >= 25.0, got {:.4}",
        metrics.purcell_enhancement_factor
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_pump_power_and_gain_scaling() {
    let base = CavityAcoustodynamicalSpinParams::default();
    let solver_base = CavityAcoustodynamicalSpinSolver::new(base);

    let high_pump = CavityAcoustodynamicalSpinParams::new(
        8.0, // increased from 4.5 mW
        base.cavity_decay_rate_khz,
        base.spin_phonon_coupling_mhz,
        base.non_linear_gain_db,
        base.cryogenic_temp_mk,
        base.acoustic_frequency_ghz,
        base.spin_dephasing_rate_hz,
        base.chiral_isolation_db,
    );
    let solver_high_pump = CavityAcoustodynamicalSpinSolver::new(high_pump);

    let high_gain = CavityAcoustodynamicalSpinParams::new(
        base.pump_power_mw,
        base.cavity_decay_rate_khz,
        base.spin_phonon_coupling_mhz,
        22.0, // increased from 16.5 dB
        base.cryogenic_temp_mk,
        base.acoustic_frequency_ghz,
        base.spin_dephasing_rate_hz,
        base.chiral_isolation_db,
    );
    let solver_high_gain = CavityAcoustodynamicalSpinSolver::new(high_gain);

    let m_base = solver_base.evaluate_metrics();
    let m_pump = solver_high_pump.evaluate_metrics();
    let m_gain = solver_high_gain.evaluate_metrics();

    assert!(
        m_pump.acoustic_quadrature_squeezing_db > m_base.acoustic_quadrature_squeezing_db,
        "Higher pump power must increase acoustic quadrature squeezing"
    );
    assert!(
        m_gain.acoustic_quadrature_squeezing_db > m_base.acoustic_quadrature_squeezing_db,
        "Higher non-linear gain must increase acoustic quadrature squeezing"
    );
}

#[test]
fn test_cavity_decay_rate_scaling() {
    let base = CavityAcoustodynamicalSpinParams::default();
    let solver_base = CavityAcoustodynamicalSpinSolver::new(base);

    let high_decay = CavityAcoustodynamicalSpinParams::new(
        base.pump_power_mw,
        150.0, // increased from 85.0 kHz
        base.spin_phonon_coupling_mhz,
        base.non_linear_gain_db,
        base.cryogenic_temp_mk,
        base.acoustic_frequency_ghz,
        base.spin_dephasing_rate_hz,
        base.chiral_isolation_db,
    );
    let solver_high_decay = CavityAcoustodynamicalSpinSolver::new(high_decay);

    let m_base = solver_base.evaluate_metrics();
    let m_high_decay = solver_high_decay.evaluate_metrics();

    assert!(
        m_high_decay.acoustic_quadrature_squeezing_db < m_base.acoustic_quadrature_squeezing_db,
        "Higher cavity decay rate must degrade acoustic quadrature squeezing"
    );
    assert!(
        m_high_decay.spin_phonon_fidelity < m_base.spin_phonon_fidelity,
        "Higher cavity decay rate must degrade spin-phonon state transfer fidelity"
    );
    assert!(
        m_high_decay.purcell_enhancement_factor < m_base.purcell_enhancement_factor,
        "Higher cavity decay rate must reduce Purcell enhancement factor"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = CavityAcoustodynamicalSpinParams::default();
    let solver_base = CavityAcoustodynamicalSpinSolver::new(base);

    let warm = CavityAcoustodynamicalSpinParams::new(
        base.pump_power_mw,
        base.cavity_decay_rate_khz,
        base.spin_phonon_coupling_mhz,
        base.non_linear_gain_db,
        45.0, // warmed from 20.0 mK
        base.acoustic_frequency_ghz,
        base.spin_dephasing_rate_hz,
        base.chiral_isolation_db,
    );
    let solver_warm = CavityAcoustodynamicalSpinSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_warm.thermal_phonon_occupancy > m_base.thermal_phonon_occupancy,
        "Elevated cryogenic temperature must increase thermal phonon occupancy"
    );
    assert!(
        m_warm.acoustic_quadrature_squeezing_db < m_base.acoustic_quadrature_squeezing_db,
        "Elevated temperature must degrade acoustic quadrature squeezing"
    );
    assert!(
        m_warm.spin_coherence_lifetime_ms < m_base.spin_coherence_lifetime_ms,
        "Elevated temperature must shorten spin coherence lifetime"
    );
    assert!(
        m_warm.spin_phonon_fidelity < m_base.spin_phonon_fidelity,
        "Elevated temperature must degrade spin-phonon state transfer fidelity"
    );
}

#[test]
fn test_spin_dephasing_and_coupling_scaling() {
    let base = CavityAcoustodynamicalSpinParams::default();
    let solver_base = CavityAcoustodynamicalSpinSolver::new(base);

    let high_deph = CavityAcoustodynamicalSpinParams::new(
        base.pump_power_mw,
        base.cavity_decay_rate_khz,
        base.spin_phonon_coupling_mhz,
        base.non_linear_gain_db,
        base.cryogenic_temp_mk,
        base.acoustic_frequency_ghz,
        24.0, // increased from 12.0 Hz
        base.chiral_isolation_db,
    );
    let solver_high_deph = CavityAcoustodynamicalSpinSolver::new(high_deph);

    let high_coupling = CavityAcoustodynamicalSpinParams::new(
        base.pump_power_mw,
        base.cavity_decay_rate_khz,
        9.0, // increased from 5.8 MHz
        base.non_linear_gain_db,
        base.cryogenic_temp_mk,
        base.acoustic_frequency_ghz,
        base.spin_dephasing_rate_hz,
        base.chiral_isolation_db,
    );
    let solver_high_coupling = CavityAcoustodynamicalSpinSolver::new(high_coupling);

    let m_base = solver_base.evaluate_metrics();
    let m_deph = solver_high_deph.evaluate_metrics();
    let m_coup = solver_high_coupling.evaluate_metrics();

    assert!(
        m_deph.spin_coherence_lifetime_ms < m_base.spin_coherence_lifetime_ms,
        "Higher spin dephasing rate must shorten spin coherence lifetime"
    );
    assert!(
        m_deph.spin_phonon_fidelity < m_base.spin_phonon_fidelity,
        "Higher spin dephasing rate must degrade spin-phonon state transfer fidelity"
    );
    assert!(
        m_coup.purcell_enhancement_factor > m_base.purcell_enhancement_factor,
        "Higher spin-phonon coupling must increase Purcell enhancement factor"
    );
    assert!(
        m_coup.spin_phonon_fidelity > m_base.spin_phonon_fidelity,
        "Higher spin-phonon coupling must increase spin-phonon state transfer fidelity"
    );
}

#[test]
fn test_chiral_isolation_scaling() {
    let base = CavityAcoustodynamicalSpinParams::default();
    let solver_base = CavityAcoustodynamicalSpinSolver::new(base);

    let high_iso = CavityAcoustodynamicalSpinParams::new(
        base.pump_power_mw,
        base.cavity_decay_rate_khz,
        base.spin_phonon_coupling_mhz,
        base.non_linear_gain_db,
        base.cryogenic_temp_mk,
        base.acoustic_frequency_ghz,
        base.spin_dephasing_rate_hz,
        48.0, // increased from 38.0 dB
    );
    let solver_high_iso = CavityAcoustodynamicalSpinSolver::new(high_iso);

    let m_base = solver_base.evaluate_metrics();
    let m_iso = solver_high_iso.evaluate_metrics();

    assert!(
        m_iso.spin_phonon_fidelity > m_base.spin_phonon_fidelity,
        "Higher chiral isolation must increase spin-phonon state transfer fidelity"
    );
    assert!(
        m_iso.spin_coherence_lifetime_ms > m_base.spin_coherence_lifetime_ms,
        "Higher chiral isolation must improve spin coherence lifetime"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = CavityAcoustodynamicalSpinParams::default();
    let solver = CavityAcoustodynamicalSpinSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.acoustic_quadrature_squeezing_db >= 12.0);
    assert!(metrics.spin_phonon_fidelity >= 0.9970);
    assert!(metrics.spin_coherence_lifetime_ms >= 50.0);
    assert!(metrics.thermal_phonon_occupancy <= 0.05);
    assert!(metrics.purcell_enhancement_factor >= 25.0);
}
