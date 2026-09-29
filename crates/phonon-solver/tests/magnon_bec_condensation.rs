//! Integration tests for YIG Magnon Bose-Einstein Condensation & Four-Magnon Thermalization.

use phonon_models::magnon_bec::YigMagnonFilm;
use phonon_solver::magnon_bec::MagnonGpeSolver;

#[test]
fn test_yig_dispersion_and_effective_mass() {
    let film = YigMagnonFilm::default();
    assert_eq!(film.saturation_magnetization_a_per_m, 1.4e5);
    assert!((film.gilbert_damping - 3e-5).abs() < 1e-8);

    let k_min = film.wavevector_minimum_per_m();
    assert!((k_min - 4.0e6).abs() < 1e-3);

    let w_min = film.spin_wave_frequency_rad_per_s(k_min);
    assert!(
        w_min > 1.0e10,
        "Minimum spin-wave frequency should be > 10 Grad/s, got {:e}",
        w_min
    );

    let e_min = film.minimum_spin_wave_energy_joules();
    assert!(
        e_min > 1.0e-24,
        "Minimum energy should be > 1e-24 J, got {:e}",
        e_min
    );

    let m_star = film.effective_magnon_mass_kg();
    // Effective magnon mass is typically around 10^-31 to 10^-30 kg (~ m_e)
    assert!(
        m_star > 1.0e-32 && m_star < 1.0e-29,
        "Effective mass should be ~10^-31 to 10^-30 kg, got {:e}",
        m_star
    );

    // Four-magnon scattering rate should be ~ 10^8 s^-1
    let gamma_4m = film.four_magnon_scattering_rate_per_s(1.0e24);
    assert!(
        gamma_4m > 5.0e7 && gamma_4m < 5.0e8,
        "4-magnon rate should be ~ 10^8 s^-1, got {:e}",
        gamma_4m
    );
}

#[test]
fn test_magnon_chemical_potential_and_pumping_threshold() {
    let film = YigMagnonFilm::default();

    let h_crit = film.critical_pumping_threshold_a_per_m(4.0e9);
    assert!(
        h_crit > 0.0 && h_crit < 1000.0,
        "Critical threshold should be physically reasonable"
    );

    let e_min = film.minimum_spin_wave_energy_joules();

    // Below threshold (P = 0.5 P_crit): mu_m < E_min
    let mu_sub = film.chemical_potential_joules(0.5);
    assert!(
        mu_sub < e_min,
        "Chemical potential below threshold must be strictly < E_min"
    );
    assert!(
        mu_sub > 0.5 * e_min,
        "Chemical potential should approach E_min"
    );

    // At and above threshold (P >= P_crit): mu_m saturates exactly at E_min
    let mu_crit = film.chemical_potential_joules(1.0);
    assert!(
        (mu_crit - e_min).abs() < 1e-30,
        "Chemical potential at threshold must equal E_min"
    );

    let mu_super = film.chemical_potential_joules(2.5);
    assert!(
        (mu_super - e_min).abs() < 1e-30,
        "Chemical potential above threshold must saturate at E_min"
    );

    // Condensate fraction
    assert_eq!(film.condensate_fraction(0.8), 0.0);
    let f_super = film.condensate_fraction(2.0);
    assert!(
        f_super > 0.40 && f_super < 0.60,
        "At 2x threshold, fraction ~ 0.5, got {:.2}",
        f_super
    );
}

#[test]
fn test_magnon_gpe_steady_state_solver() {
    let film = YigMagnonFilm::default();
    let solver = MagnonGpeSolver::new();

    // Below threshold
    let res_sub = solver.solve_steady_state(&film, 0.7, 64);
    assert_eq!(res_sub.condensate_fraction, 0.0);
    assert_eq!(res_sub.total_condensate_particles, 0.0);

    // Above threshold (P = 2.0 P_crit)
    let res_super = solver.solve_steady_state(&film, 2.0, 64);
    assert!(res_super.condensate_fraction > 0.40);
    assert!(res_super.total_condensate_particles > 0.0);
    assert!(
        res_super.density_profile[32] > 0.0,
        "Center density must be non-zero in BEC"
    );
    assert!(
        res_super.phase_coherence_length_m > 30.0e-6,
        "Coherence length should span cavity"
    );
}
