#![deny(unsafe_code)]

use phonon_models::cqed::{DispersiveCqedSystem, MicrowaveCavity, TransmonParams};

#[test]
fn test_dispersive_regime_validity_and_detuning() {
    let transmon = TransmonParams::standard_5ghz();
    let cavity = MicrowaveCavity::standard_7ghz();
    let cqed = DispersiveCqedSystem::new(transmon, cavity, 70.0);

    // Detuning Delta = w01 - wr ~ 5.23 - 7.0 = -1.77 GHz
    let delta = cqed.detuning_ghz();
    assert!(delta < -1.5 && delta > -2.0);

    // Dispersive condition: |Delta| > 5 * g
    assert!(cqed.is_dispersive());

    // Critical photon number n_crit = Delta^2 / (4 * g^2)
    let n_crit = cqed.critical_photon_number();
    assert!(
        n_crit > 100.0,
        "Critical photon number should be > 100, got {}",
        n_crit
    );
}

#[test]
fn test_multilevel_dispersive_shift_chi() {
    let transmon = TransmonParams::standard_5ghz();
    let cavity = MicrowaveCavity::standard_7ghz();
    let cqed = DispersiveCqedSystem::new(transmon, cavity, 80.0); // g = 80 MHz

    let chi = cqed.dispersive_shift_chi_mhz();
    // chi = - (g^2 * alpha) / (Delta * (Delta + alpha))
    // With g = 0.08 GHz, alpha = -0.25 GHz, Delta = -1.77 GHz:
    // chi is negative, magnitude ~ 1-3 MHz
    assert!(chi < 0.0, "Transmon chi is typically negative");
    assert!(
        chi.abs() > 0.3 && chi.abs() < 5.0,
        "Dispersive shift chi magnitude should be ~ 0.3-5 MHz, got {} MHz",
        chi
    );

    // Cavity frequency pull 2 * |chi|
    let pull = cqed.cavity_frequency_pull_mhz();
    assert_eq!(pull, 2.0 * chi.abs());

    // State-dependent cavity frequencies
    let f_ground = cqed.cavity_freq_ground_ghz();
    let f_excited = cqed.cavity_freq_excited_ghz();
    assert!(f_ground != f_excited);
    assert!((f_ground - f_excited).abs() > 5.0e-4);
}

#[test]
fn test_ac_stark_shift_and_shot_noise_dephasing() {
    let transmon = TransmonParams::standard_5ghz();
    let cavity = MicrowaveCavity::standard_7ghz();
    let cqed = DispersiveCqedSystem::new(transmon, cavity, 70.0);

    // AC Stark shift for n_bar = 5 photons
    let stark_5 = cqed.ac_stark_shift_mhz(5.0);
    let stark_1 = cqed.ac_stark_shift_mhz(1.0);
    assert!((stark_5 - 5.0 * stark_1).abs() < 1e-6);

    // Photon shot noise dephasing
    let gamma_phi = cqed.photon_shot_noise_dephasing_khz(2.0);
    assert!(gamma_phi > 0.0);
}
