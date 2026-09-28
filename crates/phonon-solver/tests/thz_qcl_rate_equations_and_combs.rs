//! Integration test suite for Phase 42:
//! Multi-Level Coupled Rate Equations, Threshold Current Density, Temperature Roll-Off,
//! and Terahertz Frequency Comb Generation via Four-Wave Mixing.

use phonon_models::quantum::ThzPolaritonicWaveguide;
use phonon_solver::quantum::{ThzFrequencyCombEngine, ThzQclRateEquationSolver};

#[test]
fn test_rate_equations_threshold_and_l_i_curve() {
    let mm_waveguide = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
    let solver = ThzQclRateEquationSolver::standard_3_2_thz(mm_waveguide);

    let temp_k = 50.0; // 50 K cryogenic operation
    let j_th = solver.threshold_current_density_at_temperature(temp_k);

    assert!(
        (100.0..=300.0).contains(&j_th),
        "Threshold Jth at 50 K out of expected range: {} A/cm^2",
        j_th
    );

    // 1. Below threshold:
    let state_sub = solver.solve_steady_state(j_th * 0.90, temp_k);
    assert!(!state_sub.is_lasing);
    assert_eq!(state_sub.optical_power_watts, 0.0);
    assert_eq!(state_sub.photon_density_m3, 0.0);

    // 2. At threshold:
    let state_th = solver.solve_steady_state(j_th, temp_k);
    assert!(state_th.optical_power_watts.abs() < 1e-4);

    // 3. Above threshold L-I curve linearity:
    let drive_multipliers = [1.2, 1.4, 1.6, 1.8, 2.0];
    let mut prev_power = 0.0;

    for &mult in &drive_multipliers {
        let drive_j = j_th * mult;
        let state = solver.solve_steady_state(drive_j, temp_k);

        assert!(state.is_lasing);
        assert!(state.optical_power_mw > prev_power);
        assert!(state.photon_density_m3 > 0.0);

        // Clamped inversion density above threshold:
        let inv = state.inversion_density_m3;
        assert!(
            inv > 5e19 && inv < 5e21,
            "Clamped inversion density out of physical range: {} m^-3",
            inv
        );

        prev_power = state.optical_power_mw;
    }

    // At 2.0x threshold, output power should be in the tens of milliwatts:
    assert!(
        prev_power > 10.0,
        "Optical output power at 2x threshold should exceed 10 mW, got: {} mW",
        prev_power
    );
}

#[test]
fn test_temperature_roll_off_and_max_operating_temperature() {
    let mm = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
    let solver = ThzQclRateEquationSolver::standard_3_2_thz(mm);

    // Verify J_th(T) strictly monotonically increases with temperature (thermal back-filling):
    let temps = [10.0, 50.0, 100.0, 150.0, 180.0, 200.0];
    let mut prev_j_th = 0.0;

    for &t in &temps {
        let j_th = solver.threshold_current_density_at_temperature(t);
        assert!(
            j_th > prev_j_th,
            "Threshold current density must monotonically increase with temperature: J_th({}) = {} <= {}",
            t,
            j_th,
            prev_j_th
        );
        prev_j_th = j_th;
    }

    // Maximum operating temperature T_max:
    let t_max = solver.max_operating_temperature_kelvin();
    assert!(
        t_max > 200.0,
        "Modern resonant LO-phonon THz QCL must achieve T_max > 200 K, got: {} K",
        t_max
    );

    // At T > T_max, laser cannot reach threshold within max current:
    let over_t = t_max + 20.0;
    let j_th_over = solver.threshold_current_density_at_temperature(over_t);
    assert!(
        j_th_over > solver.max_alignable_current_density_a_cm2,
        "Above T_max, J_th must exceed max alignable current density"
    );
}

#[test]
fn test_transient_pulse_turn_on_dynamics() {
    let mm = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
    let solver = ThzQclRateEquationSolver::standard_3_2_thz(mm);

    let j_th = solver.threshold_current_density_at_temperature(50.0);
    let j_pulse = j_th * 1.8; // 1.8x threshold pulse
    let pulse_dur = 100.0e-12; // 100 picoseconds
    let dt = 0.5e-12; // 0.5 ps time step

    let trajectory = solver.solve_transient_pulse(j_pulse, pulse_dur, dt, 50.0);

    assert!(trajectory.len() >= 100);

    // 1. Initial state at t = 0: optical power is zero
    assert_eq!(trajectory[0].optical_power_watts, 0.0);

    // 2. Carrier density n3 builds up prior to stimulated emission turn-on:
    let peak_n3 = trajectory
        .iter()
        .map(|s| s.n3)
        .fold(0.0f64, |a, b| a.max(b));
    assert!(
        peak_n3 > 1e20,
        "n3 should build up to > 1e20 m^-3: got {}",
        peak_n3
    );

    // 3. Steady-state lasing is reached by the end of the 100 ps pulse:
    let end_state = trajectory.last().unwrap();
    assert!(
        end_state.optical_power_watts > 1e-3,
        "Steady state power should exceed 1 mW: got {} W",
        end_state.optical_power_watts
    );
}

#[test]
fn test_frequency_comb_generation_and_phase_locking() {
    let center_freq = 3.2e12; // 3.2 THz
    let l_cav = 2.5e-3; // 2.5 mm cavity length
    let n_g = 3.82; // Group index
    let gain_bw = 600.0e9; // 600 GHz gain bandwidth
    let dipole_z = 3.5e-9; // 3.5 nm

    let comb_engine = ThzFrequencyCombEngine::new(center_freq, l_cav, n_g, gain_bw, dipole_z);

    // 1. Giant third-order susceptibility chi^(3) ~ 1e-14 - 1e-12 m^2/V^2:
    let chi_3 = comb_engine.chi_3_susceptibility_m2_v2;
    assert!(
        chi_3 > 1e-15 && chi_3 < 1e-11,
        "Active region chi^(3) out of expected range: {:.2e} m^2/V^2",
        chi_3
    );

    // 2. Equidistant repetition frequency in 10 - 25 GHz microwave band:
    let f_rep_ghz = comb_engine.repetition_frequency_hz / 1e9;
    assert!(
        (10.0..=25.0).contains(&f_rep_ghz),
        "Comb repetition rate should be 10 - 25 GHz, got: {} GHz",
        f_rep_ghz
    );

    // 3. Sub-kilohertz intermode beat note linewidth (< 1000 Hz):
    assert!(
        comb_engine.beat_note_linewidth_hz < 1000.0,
        "Beat note linewidth must be sub-kHz, got: {} Hz",
        comb_engine.beat_note_linewidth_hz
    );

    // 4. Modal spectrum generation:
    let modes = comb_engine.generate_comb_spectrum();
    assert!(modes.len() >= 20);

    // Equidistant mode spacing: nu_{m+1} - nu_m = f_rep
    for i in 1..modes.len() {
        let spacing = modes[i].frequency_hz - modes[i - 1].frequency_hz;
        let diff = (spacing - comb_engine.repetition_frequency_hz).abs();
        assert!(
            diff < 1e3,
            "Comb mode spacing not equidistant: {} vs {}",
            spacing,
            comb_engine.repetition_frequency_hz
        );
    }
}
