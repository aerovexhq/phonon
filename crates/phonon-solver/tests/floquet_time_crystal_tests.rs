#![deny(unsafe_code)]

//! Test suite for Floquet-Bloch Quantum Acoustic Discrete Time Crystal (DTC) Simulator.

use std::f64::consts::PI;
use phonon_solver::floquet_time_crystal::{
    EdwardsAndersonOrder, FloquetState, FloquetStateKind, FloquetTimeCrystalParams,
    FloquetUnitaryOperator, RigidityPhaseDiagram, SubharmonicSpectralAnalysis,
};

#[test]
fn test_floquet_unitary_unitarity_and_norm_preservation() {
    let mut params = FloquetTimeCrystalParams::default();
    params.chain_length = 4;
    params.pulse_error_epsilon = 0.08;

    let op = FloquetUnitaryOperator::new(&params);
    let unitarity_error = op.verify_unitarity();
    assert!(
        unitarity_error < 1e-10,
        "Floquet unitary operator deviation from identity: {unitarity_error}"
    );

    // Test state norm preservation
    let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, 4, 1234);
    assert!((state0.norm() - 1.0).abs() < 1e-12);

    let state1 = op.apply_step(&state0);
    assert!(
        (state1.norm() - 1.0).abs() < 1e-12,
        "Norm must be strictly preserved under unitary evolution"
    );

    let state2 = op.apply_step(&state1);
    assert!((state2.norm() - 1.0).abs() < 1e-12);
}

#[test]
fn test_period_doubling_2t_oscillation() {
    let mut params = FloquetTimeCrystalParams::default();
    params.chain_length = 4;
    params.pulse_error_epsilon = 0.05;
    params.damping_gamma = 0.0; // unattenuated test

    let op = FloquetUnitaryOperator::new(&params);
    let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, 4, 42);
    let traj = op.evolve(&state0, 30);

    assert_eq!(traj.cycle_count, 30);
    assert!(
        traj.is_period_doubled(),
        "Stroboscopic magnetization must exhibit robust 2T period doubling"
    );

    // Verify opposite signs for first 10 cycles
    for n in 0..10 {
        let m_even = traj.average_magnetization[2 * n];
        let m_odd = traj.average_magnetization[2 * n + 1];
        assert!(
            m_even * m_odd < 0.0,
            "Cycle {}: M_z(2n T) ({m_even}) and M_z((2n+1) T) ({m_odd}) must have opposite signs",
            n
        );
    }
}

#[test]
fn test_subharmonic_fourier_peak_at_half_drive_frequency() {
    let mut params = FloquetTimeCrystalParams::default();
    params.chain_length = 4;
    params.pulse_error_epsilon = 0.05;

    let op = FloquetUnitaryOperator::new(&params);
    let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, 4, 42);
    let traj = op.evolve(&state0, 40);

    let analysis = SubharmonicSpectralAnalysis::from_trajectory(&traj);
    assert!(
        (analysis.subharmonic_peak_frequency - 0.5).abs() <= 0.03,
        "Subharmonic peak frequency {} must be centered at 0.5 * Omega",
        analysis.subharmonic_peak_frequency
    );
    assert!(
        analysis.subharmonic_fraction >= 0.70,
        "Subharmonic power fraction {} must be >= 0.70",
        analysis.subharmonic_fraction
    );
    assert!(analysis.is_rigidly_locked);
}

#[test]
fn test_spatiotemporal_rigidity_across_nonzero_epsilon() {
    // Test that the peak frequency remains rigidly pinned at 0.5 across non-zero epsilon
    let epsilons = [0.0, 0.05, 0.10];
    for &eps in &epsilons {
        let mut params = FloquetTimeCrystalParams::default();
        params.chain_length = 4;
        params.pulse_error_epsilon = eps;

        let op = FloquetUnitaryOperator::new(&params);
        let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, 4, 42);
        let traj = op.evolve(&state0, 40);
        let analysis = SubharmonicSpectralAnalysis::from_trajectory(&traj);

        assert!(
            (analysis.subharmonic_peak_frequency - 0.5).abs() <= 0.03,
            "For epsilon = {eps}, subharmonic frequency {} must remain locked at 0.5",
            analysis.subharmonic_peak_frequency
        );
        assert!(
            analysis.subharmonic_fraction >= 0.65,
            "For epsilon = {eps}, subharmonic fraction {} must be high",
            analysis.subharmonic_fraction
        );
    }
}

#[test]
fn test_mbl_stabilization_vs_clean_thermalization() {
    // With MBL disorder W = 2.0 * J: stable oscillations
    let mut params_dtc = FloquetTimeCrystalParams::preset_stable_dtc();
    params_dtc.chain_length = 4;
    params_dtc.pulse_error_epsilon = 0.08;
    let op_dtc = FloquetUnitaryOperator::new(&params_dtc);
    let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, 4, 42);
    let traj_dtc = op_dtc.evolve(&state0, 40);
    let spectral_dtc = SubharmonicSpectralAnalysis::from_trajectory(&traj_dtc);
    let ea_dtc = EdwardsAndersonOrder::compute(&traj_dtc);

    // Clean thermal regime W = 0: fast dephasing
    let mut params_clean = FloquetTimeCrystalParams::preset_thermal_ergodic();
    params_clean.chain_length = 4;
    params_clean.pulse_error_epsilon = 0.08;
    let op_clean = FloquetUnitaryOperator::new(&params_clean);
    let traj_clean = op_clean.evolve(&state0, 40);
    let spectral_clean = SubharmonicSpectralAnalysis::from_trajectory(&traj_clean);
    let ea_clean = EdwardsAndersonOrder::compute(&traj_clean);

    assert!(
        spectral_dtc.subharmonic_fraction > spectral_clean.subharmonic_fraction,
        "MBL disorder must preserve subharmonic peak power fraction (DTC: {}, Clean: {})",
        spectral_dtc.subharmonic_fraction,
        spectral_clean.subharmonic_fraction
    );
    assert!(
        spectral_dtc.is_rigidly_locked,
        "DTC must be rigidly locked at 0.5 * Omega"
    );
    assert!(
        ea_dtc.asymptotic_order > ea_clean.asymptotic_order,
        "MBL disorder must preserve late-time order (DTC: {}, Clean: {})",
        ea_dtc.asymptotic_order,
        ea_clean.asymptotic_order
    );
}

#[test]
fn test_edwards_anderson_order_parameter() {
    let mut params = FloquetTimeCrystalParams::default();
    params.chain_length = 4;
    params.pulse_error_epsilon = 0.05;

    let op = FloquetUnitaryOperator::new(&params);
    let state0 = FloquetState::from_kind(FloquetStateKind::AllUp, 4, 42);
    let traj = op.evolve(&state0, 30);

    let ea = EdwardsAndersonOrder::compute(&traj);
    assert_eq!(ea.correlations.len(), 30);
    assert!(
        ea.asymptotic_order > 0.20,
        "Edwards-Anderson order {} must be non-zero in DTC phase",
        ea.asymptotic_order
    );
    assert!(ea.is_ordered);
}

#[test]
fn test_rigidity_phase_diagram_plateau() {
    let mut params = FloquetTimeCrystalParams::default();
    params.chain_length = 4;
    let diagram = RigidityPhaseDiagram::compute(&params, 21, 30);

    assert_eq!(diagram.epsilons.len(), 21);
    assert_eq!(diagram.subharmonic_intensities.len(), 21);
    assert!(
        diagram.plateau_width >= 0.15,
        "DTC plateau width {} must be >= 0.15",
        diagram.plateau_width
    );
    assert!(diagram.is_dtc_phase);
}

#[test]
fn test_pi_quasienergy_pairing_gap() {
    let mut params = FloquetTimeCrystalParams::default();
    params.chain_length = 4;
    params.pulse_error_epsilon = 0.05;

    let op = FloquetUnitaryOperator::new(&params);
    let pairing_gap = op.compute_pi_quasienergy_pairing_gap();

    assert!(
        (pairing_gap - PI).abs() < 0.25,
        "Mean quasi-energy pairing gap {pairing_gap} rad must be near pi ({PI})"
    );
}
