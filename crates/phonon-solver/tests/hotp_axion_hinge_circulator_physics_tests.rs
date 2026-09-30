#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic higher-order
//! axion electrodynamics and chiral quadrupole-hinge polariton circulators.

use phonon_models::hotp_axion_hinge_circulator::HotpAxionHingeCirculatorParams;
use phonon_solver::hotp_axion_hinge_circulator::HotpAxionHingeCirculatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = HotpAxionHingeCirculatorParams::new(
        0.50,   // below 0.80 pi
        0.20,   // below 0.35
        0.05,   // below 0.10
        0.50,   // below 1.0 GHz
        0.50,   // below 1.0 mK
        0.50,   // below 1.0 um
        0.20,   // below 0.5 um
        5.0e3,  // below 1.0e4
    );
    assert_eq!(underflow.axion_angle_theta_pi, 0.80);
    assert_eq!(underflow.quadrupole_polarization_qxy, 0.35);
    assert_eq!(underflow.magnetoelectric_hinge_coupling_alpha, 0.10);
    assert_eq!(underflow.acoustic_hinge_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.hinge_channel_length_um, 1.0);
    assert_eq!(underflow.inter_hinge_separation_um, 0.5);
    assert_eq!(underflow.cavity_resonance_quality_factor, 1.0e4);

    // Test values strictly above physical maximum bounds
    let overflow = HotpAxionHingeCirculatorParams::new(
        1.50,   // above 1.20 pi
        0.80,   // above 0.65
        1.10,   // above 0.95
        20.0,   // above 15.0 GHz
        80.0,   // above 50.0 mK
        30.0,   // above 20.0 um
        15.0,   // above 10.0 um
        8.0e5,  // above 5.0e5
    );
    assert_eq!(overflow.axion_angle_theta_pi, 1.20);
    assert_eq!(overflow.quadrupole_polarization_qxy, 0.65);
    assert_eq!(overflow.magnetoelectric_hinge_coupling_alpha, 0.95);
    assert_eq!(overflow.acoustic_hinge_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.hinge_channel_length_um, 20.0);
    assert_eq!(overflow.inter_hinge_separation_um, 10.0);
    assert_eq!(overflow.cavity_resonance_quality_factor, 5.0e5);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = HotpAxionHingeCirculatorParams::default();
    assert_eq!(params.axion_angle_theta_pi, 1.0);
    assert_eq!(params.quadrupole_polarization_qxy, 0.50);
    assert_eq!(params.magnetoelectric_hinge_coupling_alpha, 0.72);
    assert_eq!(params.acoustic_hinge_frequency_ghz, 5.6);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.hinge_channel_length_um, 6.0);
    assert_eq!(params.inter_hinge_separation_um, 3.5);
    assert_eq!(params.cavity_resonance_quality_factor, 1.1e5);

    let solver = HotpAxionHingeCirculatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.hinge_polariton_transmission_fidelity >= 0.9980,
        "Hinge polariton transmission fidelity must be >= 0.9980, got {:.6}",
        metrics.hinge_polariton_transmission_fidelity
    );
    assert!(
        metrics.higher_order_topological_gap_mhz >= 46.0,
        "Higher-order topological gap must be >= 46.0 MHz, got {:.4} MHz",
        metrics.higher_order_topological_gap_mhz
    );
    assert!(
        metrics.dynamic_non_reciprocal_isolation_db >= 54.0,
        "Dynamic non-reciprocal isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.dynamic_non_reciprocal_isolation_db
    );
    assert!(
        metrics.inter_hinge_crosstalk_isolation_db >= 53.0,
        "Inter-hinge crosstalk isolation must be >= 53.0 dB, got {:.4} dB",
        metrics.inter_hinge_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 15.0,
        "Topological mode dephasing rate must be <= 15.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_magnetoelectric_hinge_coupling_scaling() {
    let mut low_params = HotpAxionHingeCirculatorParams::default();
    low_params.magnetoelectric_hinge_coupling_alpha = 0.15;
    let mut high_params = HotpAxionHingeCirculatorParams::default();
    high_params.magnetoelectric_hinge_coupling_alpha = 0.90;

    let low_solver = HotpAxionHingeCirculatorSolver::new(low_params);
    let high_solver = HotpAxionHingeCirculatorSolver::new(high_params);

    assert!(
        high_solver.compute_hinge_polariton_transmission_fidelity()
            > low_solver.compute_hinge_polariton_transmission_fidelity(),
        "Stronger magnetoelectric coupling must enhance polariton transmission fidelity"
    );
    assert!(
        high_solver.compute_higher_order_topological_gap_mhz()
            > low_solver.compute_higher_order_topological_gap_mhz(),
        "Stronger magnetoelectric coupling must widen higher-order topological gap"
    );
    assert!(
        high_solver.compute_dynamic_non_reciprocal_isolation_db()
            > low_solver.compute_dynamic_non_reciprocal_isolation_db(),
        "Stronger magnetoelectric coupling must improve non-reciprocal circulation isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Stronger magnetoelectric coupling must suppress topological dephasing rate"
    );
}

#[test]
fn test_axion_angle_theta_scaling() {
    let mut tuned_params = HotpAxionHingeCirculatorParams::default();
    tuned_params.axion_angle_theta_pi = 1.0;
    let mut detuned_params = HotpAxionHingeCirculatorParams::default();
    detuned_params.axion_angle_theta_pi = 0.82;

    let tuned_solver = HotpAxionHingeCirculatorSolver::new(tuned_params);
    let detuned_solver = HotpAxionHingeCirculatorSolver::new(detuned_params);

    assert!(
        tuned_solver.compute_higher_order_topological_gap_mhz()
            > detuned_solver.compute_higher_order_topological_gap_mhz(),
        "Exact axion angle quantization theta=pi must maximize higher-order topological gap"
    );
    assert!(
        tuned_solver.compute_hinge_polariton_transmission_fidelity()
            > detuned_solver.compute_hinge_polariton_transmission_fidelity(),
        "Quantized axion angle theta=pi must maximize polariton transmission fidelity"
    );
    assert!(
        tuned_solver.compute_dynamic_non_reciprocal_isolation_db()
            > detuned_solver.compute_dynamic_non_reciprocal_isolation_db(),
        "Quantized axion angle theta=pi must enhance dynamic non-reciprocal isolation"
    );
}

#[test]
fn test_quadrupole_polarization_scaling() {
    let mut quantized_params = HotpAxionHingeCirculatorParams::default();
    quantized_params.quadrupole_polarization_qxy = 0.50;
    let mut detuned_params = HotpAxionHingeCirculatorParams::default();
    detuned_params.quadrupole_polarization_qxy = 0.38;

    let quantized_solver = HotpAxionHingeCirculatorSolver::new(quantized_params);
    let detuned_solver = HotpAxionHingeCirculatorSolver::new(detuned_params);

    assert!(
        quantized_solver.compute_higher_order_topological_gap_mhz()
            > detuned_solver.compute_higher_order_topological_gap_mhz(),
        "Exact bulk quadrupole quantization Q_xy=0.5 must maximize higher-order topological gap"
    );
    assert!(
        quantized_solver.compute_inter_hinge_crosstalk_isolation_db()
            > detuned_solver.compute_inter_hinge_crosstalk_isolation_db(),
        "Exact bulk quadrupole quantization Q_xy=0.5 must maximize inter-hinge crosstalk isolation"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = HotpAxionHingeCirculatorParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = HotpAxionHingeCirculatorParams::default();
    high_temp.cryogenic_temperature_mk = 45.0;

    let low_solver = HotpAxionHingeCirculatorSolver::new(low_temp);
    let high_solver = HotpAxionHingeCirculatorSolver::new(high_temp);

    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Sub-kelvin refrigeration must suppress topological mode dephasing rate"
    );
    assert!(
        low_solver.compute_hinge_polariton_transmission_fidelity()
            > high_solver.compute_hinge_polariton_transmission_fidelity(),
        "Lower temperature must enhance hinge polariton transmission fidelity"
    );
}

#[test]
fn test_inter_hinge_separation_scaling() {
    let mut close_hinges = HotpAxionHingeCirculatorParams::default();
    close_hinges.inter_hinge_separation_um = 0.8;
    let mut distant_hinges = HotpAxionHingeCirculatorParams::default();
    distant_hinges.inter_hinge_separation_um = 8.5;

    let close_solver = HotpAxionHingeCirculatorSolver::new(close_hinges);
    let distant_solver = HotpAxionHingeCirculatorSolver::new(distant_hinges);

    assert!(
        distant_solver.compute_inter_hinge_crosstalk_isolation_db()
            > close_solver.compute_inter_hinge_crosstalk_isolation_db(),
        "Increased inter-hinge separation must improve crosstalk isolation"
    );
}

#[test]
fn test_cavity_quality_factor_scaling() {
    let mut low_q = HotpAxionHingeCirculatorParams::default();
    low_q.cavity_resonance_quality_factor = 2.0e4;
    let mut high_q = HotpAxionHingeCirculatorParams::default();
    high_q.cavity_resonance_quality_factor = 4.5e5;

    let low_solver = HotpAxionHingeCirculatorSolver::new(low_q);
    let high_solver = HotpAxionHingeCirculatorSolver::new(high_q);

    assert!(
        high_q_solver_fidelity(high_solver, low_solver),
        "Higher cavity Q must improve polariton transmission fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher cavity Q must suppress topological dephasing rate"
    );
}

fn high_q_solver_fidelity(
    high: HotpAxionHingeCirculatorSolver,
    low: HotpAxionHingeCirculatorSolver,
) -> bool {
    high.compute_hinge_polariton_transmission_fidelity()
        > low.compute_hinge_polariton_transmission_fidelity()
}

#[test]
fn test_acoustic_frequency_scaling() {
    let mut low_f = HotpAxionHingeCirculatorParams::default();
    low_f.acoustic_hinge_frequency_ghz = 1.5;
    let mut high_f = HotpAxionHingeCirculatorParams::default();
    high_f.acoustic_hinge_frequency_ghz = 12.0;

    let low_solver = HotpAxionHingeCirculatorSolver::new(low_f);
    let high_solver = HotpAxionHingeCirculatorSolver::new(high_f);

    assert!(
        high_solver.compute_higher_order_topological_gap_mhz()
            > low_solver.compute_higher_order_topological_gap_mhz(),
        "Higher acoustic hinge frequency must expand higher-order topological gap"
    );
    assert!(
        high_solver.compute_dynamic_non_reciprocal_isolation_db()
            > low_solver.compute_dynamic_non_reciprocal_isolation_db(),
        "Higher acoustic hinge frequency must improve dynamic non-reciprocal isolation"
    );
}

#[test]
fn test_hinge_channel_length_scaling() {
    let mut short_channel = HotpAxionHingeCirculatorParams::default();
    short_channel.hinge_channel_length_um = 1.8;
    let mut long_channel = HotpAxionHingeCirculatorParams::default();
    long_channel.hinge_channel_length_um = 16.0;

    let short_solver = HotpAxionHingeCirculatorSolver::new(short_channel);
    let long_solver = HotpAxionHingeCirculatorSolver::new(long_channel);

    assert!(
        long_solver.compute_dynamic_non_reciprocal_isolation_db()
            > short_solver.compute_dynamic_non_reciprocal_isolation_db(),
        "Longer hinge channel length must enhance non-reciprocal circulation isolation"
    );
}
