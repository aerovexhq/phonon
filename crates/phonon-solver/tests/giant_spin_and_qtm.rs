//! Integration tests for Giant Spin Hamiltonian, crystal fields,
//! and resonant quantum tunneling of magnetization (QTM).

use phonon_models::stno::GiantSpinParams;
use phonon_solver::stno::GiantSpinSolver;

#[test]
fn test_fe8_molecular_magnet_parameters() {
    let fe8 = GiantSpinParams::fe8_molecular_magnet();
    assert_eq!(fe8.spin_s, 10.0);
    assert_eq!(fe8.hilbert_dim(), 21);

    let barrier_k = fe8.classical_barrier_k();
    assert!(
        (barrier_k - 27.5).abs() < 1e-4,
        "Fe8 classical barrier should be 27.5 K, got {}",
        barrier_k
    );

    let tb_est = fe8.blocking_temperature_estimate_k();
    assert!(
        tb_est > 0.5 && tb_est < 2.0,
        "Blocking temperature should be ~1.1 K, got {}",
        tb_est
    );

    // Resonant QTM magnetic fields
    let bz_0 = fe8.resonant_field_tesla(0);
    assert_eq!(bz_0, 0.0);

    let bz_1 = fe8.resonant_field_tesla(1);
    assert!(
        bz_1 > 0.15 && bz_1 < 0.25,
        "Step k=1 resonant field should be ~0.205 T, got {}",
        bz_1
    );
}

#[test]
fn test_fe8_eigensolver_and_qtm_splitting() {
    let fe8 = GiantSpinParams::fe8_molecular_magnet();

    // At zero magnetic field, rhombic anisotropy E mixes m = +-10, creating tunnel splitting
    let sol = GiantSpinSolver::solve(&fe8, 0.0, 0.0);

    assert_eq!(sol.dim, 21);
    assert_eq!(sol.eigenenergies_k.len(), 21);

    // Check monotonic sorting
    for i in 0..20 {
        assert!(sol.eigenenergies_k[i] <= sol.eigenenergies_k[i + 1]);
    }

    // Ground state splitting should be non-zero due to transverse E term
    let delta_0 = sol.ground_state_splitting_k;
    assert!(
        delta_0 > 1.0e-12 && delta_0 < 1.0e-2,
        "Fe8 ground state tunnel splitting should be physical, got {} K",
        delta_0
    );

    assert!(sol.tunneling_frequency_hz > 0.0);
}

#[test]
fn test_transverse_field_qtm_enhancement() {
    let fe8 = GiantSpinParams::fe8_molecular_magnet();

    // Transverse magnetic field B_x strongly enhances QTM tunnel splitting
    let sol_zero = GiantSpinSolver::solve(&fe8, 0.0, 0.0);
    let sol_trans = GiantSpinSolver::solve(&fe8, 0.0, 0.2); // 0.2 T transverse field

    assert!(
        sol_trans.ground_state_splitting_k > sol_zero.ground_state_splitting_k,
        "Transverse field must enhance QTM tunnel splitting: zero {} K, trans {} K",
        sol_zero.ground_state_splitting_k,
        sol_trans.ground_state_splitting_k
    );
}
