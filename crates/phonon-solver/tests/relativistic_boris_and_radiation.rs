//! Integration tests for relativistic Boris leapfrog pusher, betatron oscillations,
//! and synchrotron X-ray radiation.

use phonon_models::wakefield::{BetatronRadiation, PlasmaChannelParams};
use phonon_solver::wakefield::{BorisPusher, RelativisticParticle, SPEED_OF_LIGHT};

#[test]
fn test_relativistic_particle_properties() {
    let p = RelativisticParticle::electron_with_energy_mev([0.0, 0.0, 0.0], 250.0);
    assert!((p.kinetic_energy_mev() - 250.0).abs() < 1e-4);
    assert!(p.gamma() > 450.0);

    let v = p.velocity();
    assert!(v[2] > 0.999 * SPEED_OF_LIGHT);
    assert!(v[2] < SPEED_OF_LIGHT);
}

#[test]
fn test_boris_symplectic_drift_and_cyclotron() {
    // 1. Vacuum drift: velocity should remain invariant
    let mut p_drift = RelativisticParticle::electron_with_energy_mev([0.0, 0.0, 0.0], 50.0);
    let initial_uz = p_drift.proper_velocity[2];
    for _ in 0..100 {
        BorisPusher::step(
            &mut p_drift,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            1.0e-15,
            false,
        );
    }
    assert_eq!(p_drift.proper_velocity[2], initial_uz);
    assert!(p_drift.position[2] > 0.0);

    // 2. Cyclotron gyration in magnetic field B = [0, 0, 2.0] T: energy strictly conserved
    let mut p_mag = RelativisticParticle::electron([0.0, 0.0, 0.0], [1.0e8, 0.0, 0.0]);
    let initial_energy = p_mag.kinetic_energy_mev();
    let dt = 1.0e-14;

    for _ in 0..500 {
        BorisPusher::step(&mut p_mag, [0.0, 0.0, 0.0], [0.0, 0.0, 2.0], dt, false);
    }

    let final_energy = p_mag.kinetic_energy_mev();
    let rel_err = ((final_energy - initial_energy) / initial_energy).abs();
    assert!(
        rel_err < 1e-6,
        "Boris pusher should conserve energy in pure magnetic field to < 1e-6, got {}",
        rel_err
    );
}

#[test]
fn test_betatron_synchrotron_radiation_and_damping() {
    let plasma = PlasmaChannelParams::standard_underdense();
    let betatron = BetatronRadiation::new(plasma);

    let gamma = 1000.0; // ~500 MeV electron
    let r_beta = 1.0e-6; // 1 um betatron amplitude

    let omega_b = betatron.betatron_frequency_rad_per_s(gamma);
    assert!(omega_b > 1.0e12 && omega_b < 5.0e12);

    let k_beta = betatron.betatron_strength_parameter(gamma, r_beta);
    assert!(k_beta > 0.0);

    let hbar_omega_c = betatron.critical_photon_energy_kev(gamma, r_beta);
    assert!(
        hbar_omega_c > 1.0 && hbar_omega_c < 50.0,
        "Synchrotron critical photon energy should be in hard X-ray range (1-50 keV), got {} keV",
        hbar_omega_c
    );

    let p_rad = betatron.radiated_power_watts(gamma, r_beta);
    assert!(p_rad > 0.0);

    let damping_rate = betatron.radiation_damping_rate_per_s(gamma, r_beta);
    assert!(damping_rate > 0.0);

    // Test Boris pusher with radiation reaction enabled under strong transverse electric field
    let mut p_damped = RelativisticParticle::electron_with_energy_mev([1.0e-6, 0.0, 0.0], 500.0);
    let _initial_e = p_damped.kinetic_energy_mev();
    let e_field = [1.0e11, 0.0, 0.0]; // 100 GV/m transverse field
    for _ in 0..100 {
        BorisPusher::step(&mut p_damped, e_field, [0.0, 0.0, 0.0], 1.0e-15, true);
    }
    assert!(p_damped.kinetic_energy_mev() > 0.0);
}
