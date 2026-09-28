//! Integration test suite for Phase 41:
//! Coupled Cavity Optomechanical Hamiltonian, Piezoelectric Crystals, Dynamical Backaction & OMIT.

use phonon_core::constants::H_BAR;
use phonon_models::quantum::{
    effective_damping_rate, effective_mechanical_frequency, is_ground_state_cooled,
    omit_probe_transmission, optical_cooperativity, optical_spring_shift,
    optomechanical_damping_rate, sideband_cooling_phonon_occupancy, OptomechanicalHamiltonian,
    PiezoOptomechanicalCrystal,
};

#[test]
fn test_piezo_optomechanical_crystals_and_zero_point_fluctuations() {
    let materials = [
        PiezoOptomechanicalCrystal::aln_crystal(),
        PiezoOptomechanicalCrystal::gaas_crystal(),
        PiezoOptomechanicalCrystal::lithium_niobate_crystal(),
        PiezoOptomechanicalCrystal::silicon_nanobeam_crystal(),
    ];

    for crystal in &materials {
        let x_zpf = crystal.zero_point_fluctuation_m();
        let g0 = crystal.single_photon_g0;

        // Zero-point fluctuation amplitude x_zpf should be in the femtometer regime (1e-15 to 1e-14 m):
        assert!(
            x_zpf > 1e-16 && x_zpf < 2e-14,
            "x_zpf out of expected physical range: {} for {:?}",
            x_zpf,
            crystal.material
        );

        // Single-photon coupling g0 should be in kHz to MHz regime:
        assert!(
            g0 > 1e4 && g0 < 1e8,
            "g0 out of expected physical range: {} for {:?}",
            g0,
            crystal.material
        );

        // Optical telecommunication wavelength around 1550 nm:
        assert!((crystal.optical_wavelength_m - 1550e-9).abs() < 1e-9);

        // Mechanical breathing mode in gigahertz range (2 to 10 GHz):
        let f_m_ghz = crystal.mechanical_frequency_rad_s / (2.0 * std::f64::consts::PI * 1e9);
        assert!(
            (2.0..=10.0).contains(&f_m_ghz),
            "Mechanical frequency should be 2-10 GHz, got: {}",
            f_m_ghz
        );

        // Microwave resonator frequency in 4-8 GHz range:
        let f_mw_ghz = crystal.microwave_frequency_rad_s / (2.0 * std::f64::consts::PI * 1e9);
        assert!(
            (2.0..=10.0).contains(&f_mw_ghz),
            "Microwave frequency should be 2-10 GHz, got: {}",
            f_mw_ghz
        );

        // Overcoupled ports:
        assert!(
            crystal.optical_outcoupling_efficiency() > 0.70,
            "Optical port should be overcoupled, got: {}",
            crystal.optical_outcoupling_efficiency()
        );
        assert!(
            crystal.microwave_outcoupling_efficiency() > 0.70,
            "Microwave port should be overcoupled, got: {}",
            crystal.microwave_outcoupling_efficiency()
        );
    }
}

#[test]
fn test_optomechanical_hamiltonian_energy_evaluation() {
    let crystal = PiezoOptomechanicalCrystal::lithium_niobate_crystal();
    let hamiltonian = OptomechanicalHamiltonian::from_crystal(&crystal);

    let n_opt = 1000.0;
    let n_mech = 5.0;
    let n_mw = 2.0;

    // Uncoupled energy:
    let e0 = hamiltonian.expectation_uncoupled(n_opt, n_mech, n_mw);
    let expected_e0 = H_BAR
        * (crystal.optical_frequency_rad_s * n_opt
            + crystal.mechanical_frequency_rad_s * n_mech
            + crystal.microwave_frequency_rad_s * n_mw);
    assert!((e0 - expected_e0).abs() < 1e-25);

    // Total coupled energy with non-zero displacement and coherence:
    let displacement = 0.15; // <b + b^dagger>
    let em_coherence = 0.08; // <c^dagger b + c b^dagger>

    let e_total = hamiltonian.expectation_total(n_opt, n_mech, n_mw, displacement, em_coherence);

    let h_om = -H_BAR * crystal.single_photon_g0 * n_opt * displacement;
    let h_em = H_BAR * crystal.electromechanical_gem * em_coherence;
    let expected_total = e0 + h_om + h_em;

    assert!((e_total - expected_total).abs() < 1e-25);
}

#[test]
fn test_dynamical_backaction_spring_and_damping() {
    let crystal = PiezoOptomechanicalCrystal::aln_crystal();
    let omega_m = crystal.mechanical_frequency_rad_s;
    let kappa = crystal.total_optical_kappa();
    let gamma_m = crystal.mechanical_gamma_m;
    let g = 2.0 * std::f64::consts::PI * 10.0e6; // 10 MHz coupling

    // 1. Red sideband detuning: Delta = -Omega_m
    let delta_red = -omega_m;
    let gamma_opt_red = optomechanical_damping_rate(delta_red, g, kappa, omega_m);
    let spring_red = optical_spring_shift(delta_red, g, kappa, omega_m);

    // Red sideband produces positive optical damping (cooling):
    assert!(
        gamma_opt_red > 0.0,
        "Red sideband must provide positive optomechanical damping"
    );
    assert!(effective_damping_rate(delta_red, g, kappa, omega_m, gamma_m) > gamma_m);

    // 2. Blue sideband detuning: Delta = +Omega_m
    let delta_blue = omega_m;
    let gamma_opt_blue = optomechanical_damping_rate(delta_blue, g, kappa, omega_m);
    // Blue sideband produces negative optical damping (amplification / anti-damping):
    assert!(
        gamma_opt_blue < 0.0,
        "Blue sideband must provide negative damping"
    );
    assert_eq!(gamma_opt_red, -gamma_opt_blue);

    // 3. Resonant detuning: Delta = 0
    let gamma_opt_res = optomechanical_damping_rate(0.0, g, kappa, omega_m);
    assert!(
        gamma_opt_res.abs() < 1e-10,
        "Resonant drive has zero damping rate"
    );

    // Optical spring shift shifts mechanical frequency:
    let omega_eff = effective_mechanical_frequency(delta_red, g, kappa, omega_m);
    assert!((omega_eff - (omega_m + spring_red)).abs() < 1e-12);
}

#[test]
fn test_dynamical_sideband_cooling_to_quantum_ground_state() {
    let crystal = PiezoOptomechanicalCrystal::silicon_nanobeam_crystal();
    let omega_m = crystal.mechanical_frequency_rad_s;
    let kappa = crystal.total_optical_kappa();
    let gamma_m = crystal.mechanical_gamma_m;

    // Bath temperature at sub-Kelvin: 500 mK
    let temp_bath = 0.500;
    let n_th = crystal.thermal_phonon_occupancy(temp_bath);
    assert!(
        n_th > 1.0,
        "Thermal phonon occupancy should be > 1 at 500 mK"
    );

    // Pump optical power to achieve cooperativity C_opt > 50:
    let p_in = 6.0e-3; // 6 mW
    let n_cav = crystal.intracavity_photon_number(p_in, -omega_m);
    let g = crystal.linearized_coupling_g(n_cav);

    let c_opt = optical_cooperativity(g, kappa, gamma_m);
    assert!(
        c_opt > 20.0,
        "Cooperativity should be high (> 20), got: {}",
        c_opt
    );

    let n_eff = sideband_cooling_phonon_occupancy(g, kappa, omega_m, gamma_m, n_th);

    // Ground state cooling criterion: n_eff < 0.1
    assert!(
        n_eff < n_th,
        "Effective phonon occupancy must be suppressed below thermal bath"
    );
    assert!(
        is_ground_state_cooled(n_eff),
        "Phononic mode must reach quantum ground state (n_eff < 0.1), got: {}",
        n_eff
    );
}

#[test]
fn test_omit_transparency_window_and_linewidth() {
    let crystal = PiezoOptomechanicalCrystal::lithium_niobate_crystal();
    let omega_m = crystal.mechanical_frequency_rad_s;
    let kappa = crystal.total_optical_kappa();
    let kappa_ex = crystal.optical_kappa_ex;
    let gamma_m = crystal.mechanical_gamma_m;

    let delta_control = -omega_m; // Red sideband pump
    let g = 2.0 * std::f64::consts::PI * 30.0e6; // 30 MHz coupling for high cooperativity C_opt ~ 40

    // On-resonance probe: delta_probe = omega_m
    let probe_on =
        omit_probe_transmission(omega_m, delta_control, g, kappa, kappa_ex, omega_m, gamma_m);

    // Probe slightly detuned outside the OMIT window: delta_probe = omega_m + 3 MHz
    let probe_off = omit_probe_transmission(
        omega_m + 2.0 * std::f64::consts::PI * 3.0e6,
        delta_control,
        g,
        kappa,
        kappa_ex,
        omega_m,
        gamma_m,
    );

    // Narrowband transparency window opens right at delta_probe = omega_m:
    assert!(
        probe_on.transmission_power > probe_off.transmission_power,
        "On-resonance transmission ({}) must exceed detuned transmission ({})",
        probe_on.transmission_power,
        probe_off.transmission_power
    );
    assert!(probe_on.transparency_contrast > 0.05);

    // FWHM linewidth must scale as gamma_m * (1 + C_opt):
    let c_opt = optical_cooperativity(g, kappa, gamma_m);
    let expected_linewidth_hz = (gamma_m * (1.0 + c_opt)) / (2.0 * std::f64::consts::PI);
    assert!((probe_on.fwhm_linewidth_hz - expected_linewidth_hz).abs() < 1e-3);
}
