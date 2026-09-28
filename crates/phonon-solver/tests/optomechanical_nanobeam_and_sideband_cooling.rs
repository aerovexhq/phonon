//! Integration tests for cavity optomechanics, photonic crystal nanobeams, and dynamical sideband cooling.

use phonon_models::optomechanics::{OptomechanicalParams, SidebandCoolingParams};

#[test]
fn test_nanobeam_optomechanical_parameters() {
    let nb = OptomechanicalParams::standard_nanobeam();
    assert_eq!(nb.mechanical_frequency_hz, 5.0e9);
    assert!(
        nb.is_resolved_sideband(),
        "Nanobeam must be in the resolved sideband regime"
    );
    assert!(nb.sideband_resolution_parameter() > 300.0);

    let x_zpf = nb.zero_point_fluctuation_m();
    assert!(
        x_zpf > 1e-16 && x_zpf < 1e-14,
        "x_zpf should be ~ few fm, got {} m",
        x_zpf
    );

    let g_pull = nb.frequency_pull_parameter_rad_per_s_m();
    assert!(
        g_pull > 1e17,
        "Frequency pull G must be large (> 1e17 rad/(s*m))"
    );

    let q_opt = nb.optical_quality_factor();
    assert!(q_opt > 1.0e7, "Optical Q must exceed 1e7");

    let q_mech = nb.mechanical_quality_factor();
    assert!(q_mech >= 1.0e6, "Mechanical Q must be at least 1e6");

    let g_eff = nb.effective_coupling_hz(3000.0);
    assert!(
        g_eff > 5.0e7,
        "Linearized coupling g must exceed 50 MHz at 3000 photons"
    );

    let c = nb.cooperativity(3000.0);
    assert!(
        c > 100.0,
        "Cooperativity C must exceed 100 at 3000 photons, got {}",
        c
    );
}

#[test]
fn test_dynamical_sideband_cooling_to_ground_state() {
    let cooling = SidebandCoolingParams::standard_ground_state_nanobeam();

    // Check optical damping
    let gamma_opt = cooling.optical_damping_hz();
    assert!(
        gamma_opt > 1.0e6,
        "Optical damping Gamma_opt must exceed 1 MHz, got {} Hz",
        gamma_opt
    );

    // Damping enhancement over intrinsic mechanical linewidth
    let damping_ratio = gamma_opt / cooling.system.mechanical_damping_hz;
    assert!(
        damping_ratio > 200.0,
        "Optical damping must exceed mechanical damping by > 200x"
    );

    // Quantum backaction cooling limit
    let n_min = cooling.quantum_backaction_limit();
    assert!(
        n_min < 1.0e-4,
        "In deep resolved sideband regime, n_min should be << 1, got {}",
        n_min
    );

    // Thermal occupancy before cooling at 4 Kelvin
    let n_th = cooling
        .system
        .thermal_phonon_occupancy(cooling.bath_temperature_k);
    assert!(
        n_th > 10.0,
        "Initial thermal phonon occupancy at 4K should be > 10, got {}",
        n_th
    );

    // Cooled phonon occupancy
    let n_bar = cooling.cooled_phonon_occupancy();
    assert!(
        n_bar < 0.1,
        "Ground state cooling requires n_bar < 0.1, got {}",
        n_bar
    );
    assert!(cooling.is_ground_state_cooled());

    // Effective mode temperature
    let t_eff = cooling.effective_temperature_k();
    assert!(
        t_eff < 0.1,
        "Effective mechanical temperature should be < 100 mK, got {} K",
        t_eff
    );
}
