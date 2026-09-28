#![deny(unsafe_code)]

use phonon_models::cqed::{DispersiveCqedSystem, MicrowaveCavity, PurcellFilter, TransmonParams};
use phonon_solver::cqed::TransmonSpectrumSolver;

#[test]
fn test_transmon_semi_classical_parameters_and_dispersion() {
    let transmon = TransmonParams::standard_5ghz();

    assert_eq!(transmon.ej_ghz, 15.0);
    assert_eq!(transmon.ec_ghz, 0.25);
    assert_eq!(transmon.ej_over_ec(), 60.0);

    // Transition frequency ~ sqrt(8 * 15 * 0.25) - 0.25 = sqrt(30) - 0.25 ~ 5.477 - 0.25 = 5.227 GHz
    let omega_01 = transmon.omega_01_ghz();
    assert!(omega_01 > 5.0 && omega_01 < 5.5);

    // Anharmonicity ~ -E_C = -0.25 GHz
    let alpha = transmon.anharmonicity_ghz();
    assert_eq!(alpha, -0.25);

    // Charge dispersion should be exponentially suppressed (< 100 kHz)
    let eps_0 = transmon.charge_dispersion_epsilon_khz();
    assert!(
        eps_0 < 100.0,
        "Charge dispersion must be exponentially suppressed, got {} kHz",
        eps_0
    );

    // Critical current and capacitance
    assert!(transmon.critical_current_na > 10.0);
    assert!(transmon.total_capacitance_ff > 50.0);
    assert!(transmon.josephson_inductance_nh > 5.0);
}

#[test]
fn test_transmon_exact_numerical_eigensolver() {
    let transmon = TransmonParams::standard_5ghz();
    let solver = TransmonSpectrumSolver::new(12);
    let sol = solver.solve(&transmon);

    // Compare numerical omega_01 with semi-classical analytical formula
    let analytical_w01 = transmon.omega_01_ghz();
    let diff = (sol.omega_01_ghz - analytical_w01).abs();
    assert!(
        diff < 0.15,
        "Numerical w01 ({}) must closely match analytical ({})",
        sol.omega_01_ghz,
        analytical_w01
    );

    // Anharmonicity alpha = w12 - w01 should be close to -E_C = -0.25 GHz
    assert!(
        (sol.anharmonicity_ghz - (-0.25)).abs() < 0.05,
        "Numerical anharmonicity ({}) must be close to -E_C (-0.25)",
        sol.anharmonicity_ghz
    );

    // Charge matrix element |<0| n |1>| ~ (E_J / (32 E_C))^0.25 ~ (60/32)^0.25 ~ 1.17
    assert!(sol.n01_matrix_element > 0.8 && sol.n01_matrix_element < 1.5);
}

#[test]
fn test_purcell_filter_qubit_lifetime_enhancement() {
    let transmon = TransmonParams::standard_5ghz();
    let cavity = MicrowaveCavity::standard_7ghz();
    let cqed = DispersiveCqedSystem::new(transmon, cavity, 75.0); // g = 75 MHz
    let filter = PurcellFilter::standard_7ghz();

    // Raw Purcell rate without filter
    let raw_rate = filter.unfiltered_purcell_rate_khz(&cqed);
    let raw_t1 = filter.unfiltered_purcell_t1_us(&cqed);

    // Filtered Purcell rate with bandpass filter
    let filtered_rate = filter.filtered_purcell_rate_khz(&cqed);
    let filtered_t1 = filter.filtered_purcell_t1_us(&cqed);

    assert!(raw_rate > 0.0);
    assert!(filtered_rate < raw_rate);

    // Lifetime enhancement should exceed 50x
    let enhancement = filtered_t1 / raw_t1;
    assert!(
        enhancement > 50.0,
        "Purcell filter should enhance T1 by > 50x, got {}x (raw T1 = {} us, filtered T1 = {} us)",
        enhancement,
        raw_t1,
        filtered_t1
    );

    // Filter insertion loss at cavity frequency should be low (< 0.5 dB)
    let il = filter.insertion_loss_db_at_cavity(cavity.resonance_frequency_ghz);
    assert!(
        il < 0.5,
        "Insertion loss at cavity should be < 0.5 dB, got {}",
        il
    );
}
