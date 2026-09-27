//! Integration tests for Atomistic Molecular Dynamics: Velocity-Verlet symplectic integration,
//! NVE energy conservation, and NVT Berendsen canonical thermalization.

use phonon_models::{
    InteratomicPotential, LennardJonesPotential, MdAtom, MolecularDynamicsSolver, MorsePotential,
};

#[test]
fn test_argon_dimer_nve_symplectic_energy_conservation() {
    let lj = LennardJonesPotential::argon();
    let mass_ar = 39.948 * 1.66053906660e-27; // kg

    // Argon equilibrium distance r0 = 2^(1/6) * sigma ~ 3.816 A
    let r0 = lj.sigma_m * 2.0_f64.powf(1.0 / 6.0);

    let atoms = vec![
        MdAtom::new(0, "Ar", mass_ar, [0.0, 0.0, 0.0]),
        // Displace slightly by 4% to initiate harmonic oscillation
        MdAtom::new(1, "Ar", mass_ar, [r0 * 1.04, 0.0, 0.0]),
    ];

    let mut solver = MolecularDynamicsSolver::new(
        atoms,
        InteratomicPotential::LennardJones(lj),
        [10e-9, 10e-9, 10e-9],
    );

    let dt = 1e-15; // 1 fs
    let e_kin_init = solver.kinetic_energy();
    let e_pot_init = solver.compute_forces();
    let e_tot_init = e_kin_init + e_pot_init;

    // Run for 1000 steps (1 picosecond)
    for _ in 0..1000 {
        solver.step_nve(dt);
    }

    let e_kin_final = solver.kinetic_energy();
    let e_pot_final = solver.compute_forces();
    let e_tot_final = e_kin_final + e_pot_final;

    let delta_e = (e_tot_final - e_tot_init).abs();
    let rel_drift = delta_e / e_tot_init.abs();

    assert!(
        rel_drift < 1e-3,
        "NVE Velocity-Verlet must conserve total energy within 0.1%: initial={}, final={}, drift={:e}",
        e_tot_init,
        e_tot_final,
        rel_drift
    );
}

#[test]
fn test_nvt_berendsen_canonical_thermalization() {
    let morse = MorsePotential::carbon_covalent();
    let mass_c = 12.011 * 1.66053906660e-27;

    let atoms = vec![
        MdAtom::new(0, "C", mass_c, [0.0, 0.0, 0.0]),
        MdAtom::new(1, "C", mass_c, [morse.re_m, 0.0, 0.0]),
    ];

    let mut solver = MolecularDynamicsSolver::new(
        atoms,
        InteratomicPotential::Morse(morse),
        [10e-9, 10e-9, 10e-9],
    );

    // Initialize with cold thermal velocities (10 K)
    solver.initialize_thermal_velocities(10.0, 12345);
    let t_init = solver.instantaneous_temperature();
    assert!(t_init > 0.0 && t_init < 50.0);

    // Thermostat towards target temperature of 300 K
    let target_t = 300.0;
    let dt = 0.5e-15; // 0.5 fs
    let tau = 25e-15; // 25 fs coupling time

    for _ in 0..400 {
        solver.step_nvt(dt, target_t, tau);
    }

    let final_t = solver.instantaneous_temperature();
    assert!(
        (final_t - target_t).abs() < 60.0,
        "Berendsen thermostat must equilibrate system near 300K: got {}",
        final_t
    );
}

#[test]
fn test_carbon_vdw_interlayer_potential() {
    let vdw = LennardJonesPotential::carbon_vdw();

    // At equilibrium distance r0 = 2^(1/6) * sigma ~ 3.816 A
    let r_eq = vdw.sigma_m * 2.0_f64.powf(1.0 / 6.0);
    let f_at_eq = vdw.force_scalar(r_eq);
    assert!(
        f_at_eq.abs() < 1e-12,
        "Force at equilibrium separation must be zero, got {}",
        f_at_eq
    );

    // Attractive at larger distance
    let f_attractive = vdw.force_scalar(r_eq * 1.2);
    assert!(
        f_attractive < 0.0,
        "Force must be attractive (negative) beyond equilibrium, got {}",
        f_attractive
    );

    // Strongly repulsive when compressed
    let f_repulsive = vdw.force_scalar(r_eq * 0.9);
    assert!(
        f_repulsive > 0.0,
        "Force must be repulsive (positive) under compression, got {}",
        f_repulsive
    );
}
