#![deny(unsafe_code)]

//! Test suite for Phase 344: Phonon Studio Non-Linear Soliton Kerr Microcomb Phononic Frequency Comb Engine.
//!
//! Verifies:
//! 1. LLE split-step Fourier solver numerical stability and Hamiltonian energy conservation.
//! 2. Anomalous dispersion requirement D_2 > 0 for supporting localized bright Kerr solitons.
//! 3. Turing pattern roll formation at moderate detuning with periodic azimuthal rolls.
//! 4. Dissipative Kerr soliton formation at red-detuning with sech^2 pulse profile and ps duration.
//! 5. Comb spectrum power calculation in dBm and multi-threshold comb bandwidth metrics.

use phonon_solver::kerr_microcomb::{
    CombSpectrum, LleSplitStepSolver, MicrocombRegime, MicroresonatorParams,
};

#[test]
fn test_lle_split_step_stability_and_energy_conservation() {
    let params = MicroresonatorParams::default_bright_soliton();
    let mut solver = LleSplitStepSolver::new(params);

    // 1. Numerical stability in dissipative drive regime
    for _ in 0..100 {
        solver.step(0.01);
    }
    assert!(
        solver.state.mean_power.is_finite(),
        "Mean power must remain finite"
    );
    assert!(
        solver.state.peak_power.is_finite(),
        "Peak power must remain finite"
    );
    assert!(
        solver.state.mean_power > 0.0,
        "Mean power must be strictly positive"
    );

    // 2. Exact Hamiltonian energy conservation in conservative lossless limit (gamma = 0, F_0 = 0)
    let mut cons_solver = LleSplitStepSolver::new(params);
    cons_solver.set_conservative(true);
    cons_solver.init_soliton(0.0);

    let initial_energy = cons_solver.state.mean_power;
    assert!(initial_energy > 0.1, "Initial state must have non-zero energy");

    for _ in 0..100 {
        cons_solver.step(0.01);
    }

    let final_energy = cons_solver.state.mean_power;
    let rel_diff = (final_energy - initial_energy).abs() / initial_energy;

    assert!(
        rel_diff < 1.0e-8,
        "Split-step Fourier method must conserve energy in conservative limit: rel_diff = {:e}",
        rel_diff
    );
}

#[test]
fn test_anomalous_dispersion_condition_for_bright_solitons() {
    let params_anomalous = MicroresonatorParams::default_bright_soliton();
    let mut solver_anomalous = LleSplitStepSolver::new(params_anomalous);
    solver_anomalous.init_soliton(0.0);

    for _ in 0..150 {
        solver_anomalous.step(0.01);
    }

    // Under anomalous dispersion D_2 > 0, localized bright soliton remains stable
    assert_eq!(
        solver_anomalous.state.regime,
        MicrocombRegime::DissipativeSoliton,
        "Anomalous dispersion must sustain dissipative soliton"
    );
    assert!(
        solver_anomalous.state.peak_power > 1.0,
        "Soliton peak power must remain high under D_2 > 0: got {}",
        solver_anomalous.state.peak_power
    );

    // Under normal dispersion D_2 < 0, bright pulse disperses
    let mut params_normal = MicroresonatorParams::default_bright_soliton();
    params_normal.d2 = -params_normal.d2.abs(); // Normal dispersion
    let mut solver_normal = LleSplitStepSolver::new(params_normal);
    solver_normal.init_soliton(0.0);

    for _ in 0..150 {
        solver_normal.step(0.01);
    }

    assert!(
        solver_normal.state.peak_power < solver_anomalous.state.peak_power * 0.7,
        "Normal dispersion must disperse bright localized pulse: normal peak = {}, anomalous peak = {}",
        solver_normal.state.peak_power,
        solver_anomalous.state.peak_power
    );
}

#[test]
fn test_turing_pattern_roll_formation() {
    let params = MicroresonatorParams::default_turing_roll();
    let mut solver = LleSplitStepSolver::new(params);
    solver.init_turing(8);

    for _ in 0..100 {
        solver.step(0.01);
    }

    assert_eq!(
        solver.state.regime,
        MicrocombRegime::TuringRolls,
        "State must be classified as Turing pattern rolls"
    );
    assert!(
        solver.state.peak_power > solver.state.mean_power * 1.2,
        "Turing rolls must exhibit significant modulation"
    );
}

#[test]
fn test_dissipative_kerr_soliton_formation_and_sech2_fit() {
    let params = MicroresonatorParams::default_bright_soliton();
    let mut solver = LleSplitStepSolver::new(params);
    solver.init_soliton(0.0);

    for _ in 0..100 {
        solver.step(0.01);
    }

    assert_eq!(
        solver.state.regime,
        MicrocombRegime::DissipativeSoliton,
        "State must settle into dissipative Kerr soliton"
    );

    let spectrum = CombSpectrum::from_state(&solver.state, &params, 1.0);

    // Check sech^2 analytical fit quality
    assert!(
        spectrum.fit_r_squared > 0.85,
        "Soliton pulse profile must fit sech^2 with R^2 > 0.85: got {}",
        spectrum.fit_r_squared
    );

    // Pulse duration tau_FWHM in picoseconds
    assert!(
        spectrum.tau_fwhm_ps > 0.5 && spectrum.tau_fwhm_ps < 2500.0,
        "Temporal pulse duration must be realistic ps: got {} ps",
        spectrum.tau_fwhm_ps
    );
    assert!(
        spectrum.peak_intensity > spectrum.background_intensity * 3.0,
        "Soliton pulse must have strong peak-to-background contrast"
    );
}

#[test]
fn test_comb_spectrum_power_and_bandwidth_metrics() {
    let params = MicroresonatorParams::default_bright_soliton();
    let mut solver = LleSplitStepSolver::new(params);
    solver.init_soliton(0.0);

    for _ in 0..60 {
        solver.step(0.01);
    }

    let spectrum = CombSpectrum::from_state(&solver.state, &params, 1.0);

    // Frequency grid checks
    assert_eq!(
        spectrum.mu.len(),
        params.grid_size,
        "Spectrum must contain N modes"
    );
    assert_eq!(
        spectrum.powers_dbm.len(),
        params.grid_size,
        "Powers vector must contain N entries"
    );
    assert!(
        spectrum.mu.contains(&0),
        "Spectrum must contain pump mode mu = 0"
    );

    // Powers must be finite dBm values
    for &p in &spectrum.powers_dbm {
        assert!(p.is_finite(), "Power in dBm must be finite: got {}", p);
    }

    // Pump mode power check
    let pump_p = spectrum.pump_power_dbm();
    assert!(
        pump_p > -30.0,
        "Pump power must be prominent: got {} dBm",
        pump_p
    );

    // Bandwidth threshold ordering
    assert!(
        spectrum.mode_count_3db <= spectrum.mode_count_10db,
        "3-dB mode count ({}) must be <= 10-dB mode count ({})",
        spectrum.mode_count_3db,
        spectrum.mode_count_10db
    );
    assert!(
        spectrum.mode_count_10db <= spectrum.mode_count_20db,
        "10-dB mode count ({}) must be <= 20-dB mode count ({})",
        spectrum.mode_count_10db,
        spectrum.mode_count_20db
    );
    assert!(
        spectrum.bandwidth_3db_hz <= spectrum.bandwidth_10db_hz,
        "3-dB bandwidth must be <= 10-dB bandwidth"
    );
    assert!(
        spectrum.bandwidth_10db_hz <= spectrum.bandwidth_20db_hz,
        "10-dB bandwidth must be <= 20-dB bandwidth"
    );
    assert_eq!(
        spectrum.repetition_rate_hz, 100.0e6,
        "Repetition rate must match FSR = 100 MHz"
    );
}
