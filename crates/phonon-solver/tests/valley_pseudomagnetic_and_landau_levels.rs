//! Integration tests for acoustic pseudomagnetic gauge fields,
//! relativistic pseudo-Landau levels, and valley-polarized zero modes.

use phonon_models::valley_acoustic::{StrainGaugeParams, ValleyIndex};
use phonon_solver::valley_acoustic::ValleyLandauLevelSolver;

#[test]
fn test_valley_pseudomagnetic_field_above_100_tesla() {
    let params = StrainGaugeParams::new(1.0e-9, 3000.0, 3.5, 8.0e6);
    let bk = params.pseudomagnetic_field_tesla(ValleyIndex::ValleyK);
    let bk_prime = params.pseudomagnetic_field_tesla(ValleyIndex::ValleyKPrime);

    assert!(
        bk >= 100.0,
        "Pseudomagnetic field in Valley K must exceed 100 T, got {:.2} T",
        bk
    );
    assert!(
        bk_prime <= -100.0,
        "Pseudomagnetic field in Valley K' must be opposite and <= -100 T, got {:.2} T",
        bk_prime
    );
    assert_eq!(
        bk, -bk_prime,
        "Time-reversal symmetry requires B_ps(K) = -B_ps(K')"
    );

    let lb = params.magnetic_length_m();
    assert!(
        lb < 3.0e-9,
        "Magnetic confinement length must be sub-3 nm, got {:.2e} m",
        lb
    );
}

#[test]
fn test_relativistic_pseudo_landau_level_dispersion() {
    let params = StrainGaugeParams::new(1.0e-9, 3000.0, 3.5, 8.0e6);
    let solver = ValleyLandauLevelSolver::new(params);

    let spectrum = solver.solve_spectrum(4);
    // Find n = 0, n = 1, n = 4 levels
    let (_, omega_0, e_0) = spectrum.iter().find(|(n, _, _)| *n == 0).unwrap();
    let (_, _omega_1, e_1) = spectrum.iter().find(|(n, _, _)| *n == 1).unwrap();
    let (_, _omega_4, e_4) = spectrum.iter().find(|(n, _, _)| *n == 4).unwrap();

    assert_eq!(*omega_0, 0.0);
    assert_eq!(*e_0, 0.0);

    // E_n scales as sqrt(n), so E_4 / E_1 must be sqrt(4) = 2.0
    let ratio = e_4 / e_1;
    assert!(
        (ratio - 2.0).abs() < 1e-6,
        "Relativistic pseudo-Landau levels must scale as sqrt(n), expected 2.0, got {:.6}",
        ratio
    );

    let gap_mev = solver.solve_landau_gap_mev();
    assert!(
        gap_mev > 1.0,
        "Acoustic pseudo-Landau level spacing Delta E_10 must exceed 1.0 meV, got {:.2} meV",
        gap_mev
    );
}

#[test]
fn test_zero_mode_sublattice_valley_polarization() {
    let params = StrainGaugeParams::new(1.0e-9, 3000.0, 3.5, 8.0e6);
    let solver = ValleyLandauLevelSolver::new(params);

    // n = 0 zero-mode:
    let (pa_k, pb_k) = solver.solve_sublattice_polarization(0, ValleyIndex::ValleyK);
    assert_eq!(pa_k, 1.0);
    assert_eq!(pb_k, 0.0);

    let (pa_kp, pb_kp) = solver.solve_sublattice_polarization(0, ValleyIndex::ValleyKPrime);
    assert_eq!(pa_kp, 0.0);
    assert_eq!(pb_kp, 1.0);

    // n = 1 excited mode: equal sublattice distribution
    let (pa_1, pb_1) = solver.solve_sublattice_polarization(1, ValleyIndex::ValleyK);
    assert_eq!(pa_1, 0.5);
    assert_eq!(pb_1, 0.5);
}
