//! Integration tests for dynamically modulated Josephson junctions,
//! parametric amplification gain (>= 15.0 dB), modulation contrast (>= 20.0 dB),
//! and sub-picosecond switching timescale (<= 0.5 ps).

use phonon_models::chiral_phonon_sc::{
    ChiralPhononDriveParams, DynamicJosephsonParams, TransientPairingParams,
};
use phonon_solver::chiral_phonon_sc::ParametricJosephsonSolver;

#[test]
fn test_parametric_josephson_amplification_and_switching() {
    let drive = ChiralPhononDriveParams {
        phonon_frequency_thz: 20.0,
        normalized_amplitude_q0: 2.4,
        nonlinear_coupling_g12_mev: 18.0,
        effective_born_charge: 3.2,
        pulse_duration_ps: 0.40,
    };
    let pairing = TransientPairingParams::default();
    let josephson = DynamicJosephsonParams {
        equilibrium_critical_current_a: 1.5e-5,
        junction_capacitance_f: 1.0e-14,
        modulation_depth: 0.60,
    };

    let solver = ParametricJosephsonSolver::new(josephson, pairing, drive);
    let metrics = solver.solve_dynamic_josephson();

    // Verify plasma frequency in GHz range (nominal 100 - 1000 GHz)
    assert!(
        metrics.plasma_frequency_ghz > 50.0,
        "Plasma frequency {:.1} GHz should be > 50 GHz",
        metrics.plasma_frequency_ghz
    );

    // Parametric amplification gain must be >= 15.0 dB
    assert!(
        metrics.parametric_gain_db >= 15.0,
        "Parametric gain {:.2} dB must be >= 15.0 dB",
        metrics.parametric_gain_db
    );

    // Modulation extinction contrast must be >= 20.0 dB
    assert!(
        metrics.modulation_contrast_db >= 20.0,
        "Modulation contrast {:.2} dB must be >= 20.0 dB",
        metrics.modulation_contrast_db
    );

    // Ultrafast switching timescale must be <= 0.50 ps
    assert!(
        metrics.modulation_time_ps <= 0.50,
        "Modulation switching time {:.3} ps must be <= 0.50 ps",
        metrics.modulation_time_ps
    );
}

#[test]
fn test_parametric_gain_spectrum_and_switching_trajectory() {
    let drive = ChiralPhononDriveParams {
        phonon_frequency_thz: 20.0,
        normalized_amplitude_q0: 2.2,
        nonlinear_coupling_g12_mev: 18.0,
        effective_born_charge: 3.2,
        pulse_duration_ps: 0.35,
    };
    let pairing = TransientPairingParams::default();
    let josephson = DynamicJosephsonParams {
        equilibrium_critical_current_a: 1.0e-5,
        junction_capacitance_f: 8.0e-15,
        modulation_depth: 0.55,
    };

    let solver = ParametricJosephsonSolver::new(josephson, pairing, drive);
    let metrics = solver.solve_dynamic_josephson();

    // On-resonance gain
    let gain_res_db = solver.solve_gain_spectrum_db(0.0, 10.0);
    assert!(
        (gain_res_db - metrics.parametric_gain_db).abs() < 1e-4,
        "Resonant gain {:.2} dB should match metrics gain {:.2} dB",
        gain_res_db,
        metrics.parametric_gain_db
    );

    // Off-resonance gain should roll off
    let gain_detuned_db = solver.solve_gain_spectrum_db(20.0, 10.0);
    assert!(
        gain_detuned_db < gain_res_db - 3.0,
        "Detuned gain {:.2} dB should be significantly lower than resonant gain {:.2} dB",
        gain_detuned_db,
        gain_res_db
    );

    // Switching trajectory
    let trajectory = solver.solve_critical_current_trajectory(50);
    assert_eq!(trajectory.len(), 50);

    let (t_start, ic_start) = trajectory.first().copied().unwrap();
    let (t_end, ic_end) = trajectory.last().copied().unwrap();
    let mid_idx = trajectory.len() / 2;
    let (_t_mid, ic_mid) = trajectory[mid_idx];

    assert!((t_start - 0.0).abs() < 1e-9);
    assert!((t_end - drive.pulse_duration_ps).abs() < 1e-9);
    assert!(
        ic_mid > ic_start,
        "Mid-pulse critical current {:.2} uA must exceed baseline {:.2} uA",
        ic_mid,
        ic_start
    );
    assert!(
        ic_mid > ic_end,
        "Mid-pulse critical current {:.2} uA must exceed tail {:.2} uA",
        ic_mid,
        ic_end
    );
}
