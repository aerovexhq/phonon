#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological acoustic chiral
//! skyrmion-lattice transducers and non-reciprocal magnon-polaron interconnects.

use phonon_models::chiral_skyrmion_magnon_polaron::ChiralSkyrmionMagnonPolaronParams;
use phonon_solver::chiral_skyrmion_magnon_polaron::ChiralSkyrmionMagnonPolaronSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralSkyrmionMagnonPolaronParams::new(
        0.10,   // below 0.50 mJ/m^2
        5.0,    // below 20.0 ppm
        1.0,    // below 5.0 MHz
        10.0,   // below 30.0 nm
        0.0001, // below 0.001
        0.5,    // below 1.0 GHz
        0.2,    // below 1.0 mK
        0.5,    // below 2.0 nm
    );
    assert_eq!(underflow.dmi_exchange_strength_mj_m2, 0.50);
    assert_eq!(underflow.acoustic_strain_drive_amplitude_ppm, 20.0);
    assert_eq!(underflow.magnon_polaron_coupling_mhz, 5.0);
    assert_eq!(underflow.skyrmion_lattice_constant_nm, 30.0);
    assert_eq!(underflow.gilbert_damping_alpha, 0.001);
    assert_eq!(underflow.acoustic_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.heterostructure_thickness_nm, 2.0);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralSkyrmionMagnonPolaronParams::new(
        10.0,  // above 6.0 mJ/m^2
        1000.0,// above 600.0 ppm
        150.0, // above 100.0 MHz
        500.0, // above 250.0 nm
        0.10,  // above 0.05
        25.0,  // above 15.0 GHz
        100.0, // above 50.0 mK
        80.0,  // above 50.0 nm
    );
    assert_eq!(overflow.dmi_exchange_strength_mj_m2, 6.0);
    assert_eq!(overflow.acoustic_strain_drive_amplitude_ppm, 600.0);
    assert_eq!(overflow.magnon_polaron_coupling_mhz, 100.0);
    assert_eq!(overflow.skyrmion_lattice_constant_nm, 250.0);
    assert_eq!(overflow.gilbert_damping_alpha, 0.05);
    assert_eq!(overflow.acoustic_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.heterostructure_thickness_nm, 50.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralSkyrmionMagnonPolaronParams::default();
    let solver = ChiralSkyrmionMagnonPolaronSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.topological_hall_angle_deg >= 18.0,
        "Default topological Hall angle must be >= 18.0 deg, got {:.4} deg",
        metrics.topological_hall_angle_deg
    );
    assert!(
        metrics.magnon_polaron_transfer_fidelity >= 0.9970,
        "Default magnon-polaron transfer fidelity must be >= 0.9970, got {:.6}",
        metrics.magnon_polaron_transfer_fidelity
    );
    assert!(
        metrics.non_reciprocal_acoustic_isolation_db >= 48.0,
        "Default non-reciprocal acoustic isolation must be >= 48.0 dB, got {:.4} dB",
        metrics.non_reciprocal_acoustic_isolation_db
    );
    assert!(
        metrics.skyrmion_drift_velocity_mps >= 180.0,
        "Default skyrmion drift velocity must be >= 180.0 m/s, got {:.4} m/s",
        metrics.skyrmion_drift_velocity_mps
    );
    assert!(
        metrics.topological_charge_stability_ratio >= 0.990,
        "Default topological charge stability ratio must be >= 0.990, got {:.6}",
        metrics.topological_charge_stability_ratio
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_dmi_exchange_strength_scaling() {
    let base = ChiralSkyrmionMagnonPolaronParams::default();
    let solver_base = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut enhanced = base;
    enhanced.dmi_exchange_strength_mj_m2 = 5.2;
    let solver_enhanced = ChiralSkyrmionMagnonPolaronSolver::new(enhanced);

    let hall_base = solver_base.compute_topological_hall_angle_deg();
    let hall_enhanced = solver_enhanced.compute_topological_hall_angle_deg();
    assert!(
        hall_enhanced > hall_base,
        "Higher DMI must increase topological Hall angle: enhanced={:.4}, base={:.4}",
        hall_enhanced, hall_base
    );

    let iso_base = solver_base.compute_non_reciprocal_acoustic_isolation_db();
    let iso_enhanced = solver_enhanced.compute_non_reciprocal_acoustic_isolation_db();
    assert!(
        iso_enhanced > iso_base,
        "Higher DMI must enhance non-reciprocal acoustic isolation: enhanced={:.4}, base={:.4}",
        iso_enhanced, iso_base
    );

    let stab_base = solver_base.compute_topological_charge_stability_ratio();
    let stab_enhanced = solver_enhanced.compute_topological_charge_stability_ratio();
    assert!(
        stab_enhanced >= stab_base,
        "Higher DMI must improve topological charge stability: enhanced={:.6}, base={:.6}",
        stab_enhanced, stab_base
    );
}

#[test]
fn test_acoustic_strain_drive_scaling() {
    let base = ChiralSkyrmionMagnonPolaronParams::default();
    let solver_base = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut high_drive = base;
    high_drive.acoustic_strain_drive_amplitude_ppm = 450.0;
    let solver_high = ChiralSkyrmionMagnonPolaronSolver::new(high_drive);

    let vel_base = solver_base.compute_skyrmion_drift_velocity_mps();
    let vel_high = solver_high.compute_skyrmion_drift_velocity_mps();
    assert!(
        vel_high > vel_base,
        "Higher strain drive amplitude must increase skyrmion drift velocity: high={:.4}, base={:.4}",
        vel_high, vel_base
    );

    let hall_base = solver_base.compute_topological_hall_angle_deg();
    let hall_high = solver_high.compute_topological_hall_angle_deg();
    assert!(
        hall_high > hall_base,
        "Higher strain drive must increase topological Hall deflection: high={:.4}, base={:.4}",
        hall_high, hall_base
    );
}

#[test]
fn test_magnon_polaron_coupling_scaling() {
    let base = ChiralSkyrmionMagnonPolaronParams::default();
    let solver_base = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut high_coupling = base;
    high_coupling.magnon_polaron_coupling_mhz = 80.0;
    let solver_high = ChiralSkyrmionMagnonPolaronSolver::new(high_coupling);

    let fidelity_base = solver_base.compute_magnon_polaron_transfer_fidelity();
    let fidelity_high = solver_high.compute_magnon_polaron_transfer_fidelity();
    assert!(
        fidelity_high > fidelity_base,
        "Higher magnon-polaron coupling must enhance transfer fidelity: high={:.6}, base={:.6}",
        fidelity_high, fidelity_base
    );

    let iso_base = solver_base.compute_non_reciprocal_acoustic_isolation_db();
    let iso_high = solver_high.compute_non_reciprocal_acoustic_isolation_db();
    assert!(
        iso_high > iso_base,
        "Higher magnon-polaron coupling must enhance non-reciprocal isolation: high={:.4}, base={:.4}",
        iso_high, iso_base
    );
}

#[test]
fn test_skyrmion_lattice_constant_scaling() {
    let base = ChiralSkyrmionMagnonPolaronParams::default();
    let solver_base = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut dense_lattice = base;
    dense_lattice.skyrmion_lattice_constant_nm = 45.0; // denser than default 80.0 nm
    let solver_dense = ChiralSkyrmionMagnonPolaronSolver::new(dense_lattice);

    let hall_base = solver_base.compute_topological_hall_angle_deg();
    let hall_dense = solver_dense.compute_topological_hall_angle_deg();
    assert!(
        hall_dense > hall_base,
        "Denser skyrmion lattice must increase topological Hall angle: dense={:.4}, base={:.4}",
        hall_dense, hall_base
    );

    let vel_base = solver_base.compute_skyrmion_drift_velocity_mps();
    let vel_dense = solver_dense.compute_skyrmion_drift_velocity_mps();
    assert!(
        vel_dense > vel_base,
        "Denser skyrmion lattice must increase drift velocity: dense={:.4}, base={:.4}",
        vel_dense, vel_base
    );
}

#[test]
fn test_gilbert_damping_scaling() {
    let base = ChiralSkyrmionMagnonPolaronParams::default();
    let solver_base = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut high_damping = base;
    high_damping.gilbert_damping_alpha = 0.040; // higher than default 0.012
    let solver_damped = ChiralSkyrmionMagnonPolaronSolver::new(high_damping);

    let vel_base = solver_base.compute_skyrmion_drift_velocity_mps();
    let vel_damped = solver_damped.compute_skyrmion_drift_velocity_mps();
    assert!(
        vel_damped < vel_base,
        "Higher Gilbert damping must reduce skyrmion drift velocity: damped={:.4}, base={:.4}",
        vel_damped, vel_base
    );

    let hall_base = solver_base.compute_topological_hall_angle_deg();
    let hall_damped = solver_damped.compute_topological_hall_angle_deg();
    assert!(
        hall_damped < hall_base,
        "Higher Gilbert damping must reduce topological Hall angle: damped={:.4}, base={:.4}",
        hall_damped, hall_base
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = ChiralSkyrmionMagnonPolaronParams::default();
    let solver_base = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut warm = base;
    warm.cryogenic_temperature_mk = 45.0; // higher than default 15.0 mK
    let solver_warm = ChiralSkyrmionMagnonPolaronSolver::new(warm);

    let fidelity_base = solver_base.compute_magnon_polaron_transfer_fidelity();
    let fidelity_warm = solver_warm.compute_magnon_polaron_transfer_fidelity();
    assert!(
        fidelity_warm < fidelity_base,
        "Higher temperature must reduce magnon-polaron transfer fidelity: warm={:.6}, base={:.6}",
        fidelity_warm, fidelity_base
    );

    let iso_base = solver_base.compute_non_reciprocal_acoustic_isolation_db();
    let iso_warm = solver_warm.compute_non_reciprocal_acoustic_isolation_db();
    assert!(
        iso_warm < iso_base,
        "Higher temperature must reduce non-reciprocal isolation: warm={:.4}, base={:.4}",
        iso_warm, iso_base
    );
}

#[test]
fn test_acoustic_frequency_resonance() {
    let base = ChiralSkyrmionMagnonPolaronParams::default(); // default frequency 5.5 GHz
    let solver_res = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut detuned = base;
    detuned.acoustic_frequency_ghz = 13.5;
    let solver_detuned = ChiralSkyrmionMagnonPolaronSolver::new(detuned);

    let fidelity_res = solver_res.compute_magnon_polaron_transfer_fidelity();
    let fidelity_detuned = solver_detuned.compute_magnon_polaron_transfer_fidelity();
    assert!(
        fidelity_res > fidelity_detuned,
        "Resonant acoustic driving must yield higher fidelity than detuned: res={:.6}, detuned={:.6}",
        fidelity_res, fidelity_detuned
    );
}

#[test]
fn test_heterostructure_thickness_optimality() {
    let base = ChiralSkyrmionMagnonPolaronParams::default(); // default thickness 12.0 nm
    let solver_opt = ChiralSkyrmionMagnonPolaronSolver::new(base);

    let mut off_center = base;
    off_center.heterostructure_thickness_nm = 45.0;
    let solver_off = ChiralSkyrmionMagnonPolaronSolver::new(off_center);

    let stab_opt = solver_opt.compute_topological_charge_stability_ratio();
    let stab_off = solver_off.compute_topological_charge_stability_ratio();
    assert!(
        stab_opt > stab_off,
        "Optimal thickness (12 nm) must provide higher charge stability: opt={:.6}, off={:.6}",
        stab_opt, stab_off
    );
}
