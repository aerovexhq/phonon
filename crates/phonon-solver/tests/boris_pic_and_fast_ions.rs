//! Tests for Boris Particle-in-Cell (PIC) kinetic fast-ion orbit tracker,
//! energy conservation, and trapped banana orbit classification.

use phonon_models::plasma::{
    KineticParticle, PlasmaSpecies, SolovevEquilibrium, TokamakGeometry, DT_ALPHA_ENERGY_JOULES,
};
use phonon_solver::plasma::BorisPicTracker;

#[test]
fn test_boris_pic_energy_conservation_in_pure_magnetic_field() {
    // In a pure magnetic field (E = 0), Boris algorithm must conserve kinetic energy exactly
    let tracker = BorisPicTracker::new(1.0e-11); // 10 ps

    let mut particle = KineticParticle::new(
        PlasmaSpecies::Deuteron,
        [1.0, 0.0, 0.0],
        [1.0e5, 5.0e5, 2.0e5], // 3D velocity
    );

    let initial_energy = particle.kinetic_energy_joules();
    let b_field = [0.0, 0.0, 3.0]; // Uniform 3 T along Z
    let e_field = [0.0, 0.0, 0.0];

    for _ in 0..10_000 {
        tracker.step_particle(&mut particle, e_field, b_field);
    }

    let final_energy = particle.kinetic_energy_joules();
    let relative_energy_drift = (final_energy - initial_energy).abs() / initial_energy;

    assert!(
        relative_energy_drift < 1e-10,
        "Boris integrator must conserve kinetic energy with relative error < 1e-10 (got {:e})",
        relative_energy_drift
    );
}

#[test]
fn test_fast_alpha_particle_banana_orbit_tracking() {
    let tracker = BorisPicTracker::new(1.0e-9);

    let iter = TokamakGeometry::iter_baseline();
    let solovev = SolovevEquilibrium::from_geometry_and_current(iter);
    let r0 = iter.major_radius_r0;
    let a = iter.minor_radius_a;

    let field_fn = |pos: [f64; 3]| -> ([f64; 3], [f64; 3]) {
        let r = (pos[0].powi(2) + pos[1].powi(2)).sqrt().max(0.1);
        let z = pos[2];
        let b_cyl = solovev.magnetic_field_at(r, z);
        let phi = pos[1].atan2(pos[0]);
        let bx = b_cyl[0] * phi.cos() - b_cyl[1] * phi.sin();
        let by = b_cyl[0] * phi.sin() + b_cyl[1] * phi.cos();
        let bz = b_cyl[2];
        ([0.0, 0.0, 0.0], [bx, by, bz])
    };

    // Alpha particle born at 3.52 MeV in fusion
    let v_alpha = (2.0 * DT_ALPHA_ENERGY_JOULES / 6.6446573357e-27).sqrt();

    // Particle with pitch angle near 45 degrees
    let v_par = 0.5 * v_alpha;
    let v_perp = (v_alpha.powi(2) - v_par.powi(2)).sqrt();
    let particle_trapped = KineticParticle::new(
        PlasmaSpecies::AlphaParticle,
        [r0 + 0.3 * a, 0.0, 0.0],
        [v_perp, v_par, 0.0],
    );

    let report_trapped = tracker.track_orbit(
        particle_trapped,
        field_fn,
        1000,
        r0 - 1.2 * a,
        r0 + 1.2 * a,
        1.5 * a,
    );

    assert!(report_trapped.is_confined);
    assert!(report_trapped.relative_energy_drift < 1e-7);

    // Passing particle
    let particle_passing = KineticParticle::new(
        PlasmaSpecies::AlphaParticle,
        [r0 + 0.2 * a, 0.0, 0.0],
        [0.1 * v_alpha, 0.99 * v_alpha, 0.0],
    );

    let report_passing = tracker.track_orbit(
        particle_passing,
        field_fn,
        1000,
        r0 - 1.2 * a,
        r0 + 1.2 * a,
        1.5 * a,
    );

    assert!(report_passing.is_confined);
    assert!(report_passing.relative_energy_drift < 1e-7);
}

#[test]
fn test_ensemble_parallel_tracking() {
    let tracker = BorisPicTracker::new(1.0e-9);
    let r0 = 1.85; // SPARC
    let b0 = 12.2;

    let field_fn = |pos: [f64; 3]| -> ([f64; 3], [f64; 3]) {
        let r = (pos[0].powi(2) + pos[1].powi(2)).sqrt().max(0.1);
        let b_phi = b0 * (r0 / r);
        let phi = pos[1].atan2(pos[0]);
        let bx = -b_phi * phi.sin();
        let by = b_phi * phi.cos();
        ([0.0, 0.0, 0.0], [bx, by, 0.0])
    };

    let mut particles = Vec::with_capacity(16);
    for i in 0..16 {
        let frac = (i as f64) / 16.0;
        let v = 1.0e6 + frac * 2.0e6;
        particles.push(KineticParticle::new(
            PlasmaSpecies::Deuteron,
            [r0 + 0.1, 0.0, 0.0],
            [0.0, v, 0.1 * v],
        ));
    }

    let reports =
        tracker.track_ensemble_parallel(&particles, field_fn, 1000, r0 - 0.6, r0 + 0.6, 1.0);

    assert_eq!(reports.len(), 16);
    for r in &reports {
        assert!(r.is_confined);
        assert!(r.relative_energy_drift < 1e-7);
    }
}
