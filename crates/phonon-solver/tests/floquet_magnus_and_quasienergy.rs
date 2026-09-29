//! Integration tests for Floquet-Magnus expansion, light-induced mass gap,
//! and unitary Floquet propagator.

use phonon_models::floquet_topological::{FloquetDiracMaterial, FloquetDriveParams};
use phonon_solver::floquet_topological::FloquetMagnusSolver;

#[test]
fn test_floquet_magnus_gap_opening_above_50mev() {
    let material = FloquetDiracMaterial::default();
    // 1.0 eV photon energy, 2.5e8 V/m electric field, RCP (sigma = +1.0)
    let drive = FloquetDriveParams::new_cw(1.0, 2.5e8, 1.0);
    let solver = FloquetMagnusSolver::new(material, drive);

    let gap_mev = solver.solve_dynamic_gap_mev();
    assert!(
        gap_mev >= 50.0,
        "Dynamic gap must be >= 50 meV, got {:.2} meV",
        gap_mev
    );

    let band = solver.solve_quasienergies(0.0, 0.0);
    assert_eq!(band.chern_number, 1);
    assert!(band.dynamic_gap_ev >= 0.050);
    assert!((band.quasienergy_plus_ev - band.dynamic_gap_ev * 0.5).abs() < 1e-6);
    assert!((band.quasienergy_minus_ev + band.dynamic_gap_ev * 0.5).abs() < 1e-6);
}

#[test]
fn test_floquet_quadratic_field_scaling() {
    let material = FloquetDiracMaterial::default();
    let drive1 = FloquetDriveParams::new_cw(1.2, 2.0e8, 1.0);
    let drive2 = FloquetDriveParams::new_cw(1.2, 4.0e8, 1.0); // 2x field -> 4x gap

    let solver1 = FloquetMagnusSolver::new(material, drive1);
    let solver2 = FloquetMagnusSolver::new(material, drive2);

    let gap1 = solver1.solve_dynamic_gap_mev();
    let gap2 = solver2.solve_dynamic_gap_mev();

    let ratio = gap2 / gap1;
    assert!(
        (ratio - 4.0).abs() < 1e-4,
        "Gap must scale quadratically with field E0, expected 4.0x, got {:.4}x",
        ratio
    );
}

#[test]
fn test_floquet_inverse_cubic_frequency_scaling() {
    let material = FloquetDiracMaterial::default();
    let e0 = 2.5e8;
    let drive1 = FloquetDriveParams::new_cw(1.0, e0, 1.0);
    let drive2 = FloquetDriveParams::new_cw(2.0, e0, 1.0); // 2x frequency -> (1/2)^3 = 1/8x gap

    let solver1 = FloquetMagnusSolver::new(material, drive1);
    let solver2 = FloquetMagnusSolver::new(material, drive2);

    let gap1 = solver1.solve_dynamic_gap_mev();
    let gap2 = solver2.solve_dynamic_gap_mev();

    let ratio = gap1 / gap2;
    assert!(
        (ratio - 8.0).abs() < 1e-3,
        "Gap must scale inversely cubic with frequency Omega, expected 8.0x, got {:.4}x",
        ratio
    );
}

#[test]
fn test_chirality_reversal_and_linear_polarization() {
    let material = FloquetDiracMaterial::default();
    let drive_rcp = FloquetDriveParams::new_cw(1.0, 2.5e8, 1.0);
    let drive_lcp = FloquetDriveParams::new_cw(1.0, 2.5e8, -1.0);
    let drive_lp = FloquetDriveParams::new_cw(1.0, 2.5e8, 0.0);

    let solver_rcp = FloquetMagnusSolver::new(material, drive_rcp);
    let solver_lcp = FloquetMagnusSolver::new(material, drive_lcp);
    let solver_lp = FloquetMagnusSolver::new(material, drive_lp);

    let band_rcp = solver_rcp.solve_quasienergies(0.0, 0.0);
    let band_lcp = solver_lcp.solve_quasienergies(0.0, 0.0);
    let band_lp = solver_lp.solve_quasienergies(0.0, 0.0);

    assert_eq!(band_rcp.chern_number, 1);
    assert_eq!(band_lcp.chern_number, -1);
    assert_eq!(band_lp.chern_number, 0);

    assert!(band_rcp.floquet_mass_ev > 0.0);
    assert!(band_lcp.floquet_mass_ev < 0.0);
    assert_eq!(band_lp.floquet_mass_ev, 0.0);
    assert_eq!(band_lp.dynamic_gap_ev, 0.0);
}

#[test]
fn test_unitary_propagator_conservation_and_quasienergies() {
    let material = FloquetDiracMaterial::default();
    let drive = FloquetDriveParams::new_cw(1.0, 2.5e8, 1.0);
    let solver = FloquetMagnusSolver::new(material, drive);

    // Integrate over 64 time slices
    let u = solver.compute_one_period_propagator(64, 0.0, 0.0);
    let err = solver.unitarity_error(u);
    assert!(
        err < 1e-12,
        "Propagator unitarity error must be < 1e-12, got {:.2e}",
        err
    );

    let (eps_plus, eps_minus) = solver.extract_quasienergies(u);
    let analytical_gap = solver.solve_dynamic_gap_mev() / 1000.0;
    let numerical_gap = eps_plus - eps_minus;

    assert!(
        (numerical_gap - analytical_gap).abs() / analytical_gap < 0.05,
        "Propagator quasi-energy gap {:.4} eV must match Magnus gap {:.4} eV within 5%",
        numerical_gap,
        analytical_gap
    );
}

#[test]
fn test_berry_curvature_and_topological_distribution() {
    let material = FloquetDiracMaterial::default();
    let drive = FloquetDriveParams::new_cw(1.0, 2.5e8, 1.0);
    let solver = FloquetMagnusSolver::new(material, drive);

    let omega_0 = solver.solve_berry_curvature(0.0, 0.0);
    assert!(
        omega_0 < 0.0,
        "Berry curvature at Dirac point must be negative for C = +1 band, got {:.2e}",
        omega_0
    );

    // Check decay away from Dirac point
    let omega_p = solver.solve_berry_curvature(1.0e-25, 0.0);
    assert!(
        omega_p.abs() < omega_0.abs(),
        "Berry curvature must decay with momentum"
    );
}
