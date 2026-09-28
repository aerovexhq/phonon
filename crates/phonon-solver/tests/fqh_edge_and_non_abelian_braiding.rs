//! Integration tests for FQH edge Luttinger liquid modes, non-Abelian braiding algebra, and trajectory fidelity.

use phonon_models::fqh::{
    c_abs_sq, mat2_dagger, mat2_mul, FqhState, LuttingerEdgeModel, NonAbelianBraidingModel,
};
use phonon_solver::fqh::{BraidOperation, BraidingTrajectorySolver};

#[test]
fn test_laughlin_and_moore_read_edge_parameters() {
    let b_tesla = 3.0;
    let laughlin = LuttingerEdgeModel::laughlin_one_third(b_tesla);
    let moore_read = LuttingerEdgeModel::moore_read_five_halves(b_tesla);

    // Filling factors
    assert_eq!(laughlin.state, FqhState::LaughlinOneThird);
    assert!((laughlin.filling_factor() - 1.0 / 3.0).abs() < 1e-9);
    assert_eq!(moore_read.state, FqhState::MooreReadFiveHalves);
    assert!((moore_read.filling_factor() - 2.5).abs() < 1e-9);

    // Fractional charges: e/3 and e/4
    assert!((laughlin.fractional_charge_ratio() - 1.0 / 3.0).abs() < 1e-9);
    assert!((moore_read.fractional_charge_ratio() - 0.25).abs() < 1e-9);

    // Effective flux quanta: 3 * (h/e) and 4 * (h/e)
    let phi_laughlin = laughlin.effective_flux_quantum_weber();
    let phi_mr = moore_read.effective_flux_quantum_weber();
    assert!(phi_mr > phi_laughlin);
    assert!((phi_mr / phi_laughlin - 4.0 / 3.0).abs() < 1e-9);

    // Conformal central charges: c = 1.0 (Laughlin) and c = 1.5 (Moore-Read: 1 charge + 0.5 Majorana)
    assert_eq!(laughlin.central_charge(), 1.0);
    assert_eq!(moore_read.central_charge(), 1.5);

    // Conformal weights: h = 1/6 for e/3, h = 3/32 for \u{03c3} anyon
    assert!((laughlin.quasiparticle_conformal_weight() - 1.0 / 6.0).abs() < 1e-9);
    assert!((moore_read.quasiparticle_conformal_weight() - 3.0 / 32.0).abs() < 1e-9);

    // Magnetic length at 3 T: l_B = sqrt(hbar / (e B)) approx 14.8 nm
    let l_b = laughlin.magnetic_length_nm();
    assert!(
        l_b > 14.0 && l_b < 15.5,
        "Magnetic length at 3 T should be ~14.8 nm, got {}",
        l_b
    );
}

#[test]
fn test_non_abelian_braiding_matrices_and_non_commutativity() {
    let r12 = NonAbelianBraidingModel::braid_r12();
    let b23 = NonAbelianBraidingModel::braid_b23();

    // Verify unitarity: U * U\u{2020} = I
    let r_dag = mat2_dagger(&r12);
    let r_prod = mat2_mul(&r12, &r_dag);

    assert!((r_prod[0].0 - 1.0).abs() < 1e-12 && r_prod[0].1.abs() < 1e-12);
    assert!(c_abs_sq(r_prod[1]) < 1e-12);
    assert!(c_abs_sq(r_prod[2]) < 1e-12);
    assert!((r_prod[3].0 - 1.0).abs() < 1e-12 && r_prod[3].1.abs() < 1e-12);

    let b_dag = mat2_dagger(&b23);
    let b_prod = mat2_mul(&b23, &b_dag);
    assert!((b_prod[0].0 - 1.0).abs() < 1e-12 && b_prod[0].1.abs() < 1e-12);
    assert!(c_abs_sq(b_prod[1]) < 1e-12);
    assert!(c_abs_sq(b_prod[2]) < 1e-12);
    assert!((b_prod[3].0 - 1.0).abs() < 1e-12 && b_prod[3].1.abs() < 1e-12);

    // Mathematical proof of non-Abelian braiding: [R, B_23] != 0
    let comm_norm = NonAbelianBraidingModel::commutator_norm();
    assert!(
        comm_norm > 0.5,
        "Commutator norm must be strictly positive for non-Abelian anyons: got {}",
        comm_norm
    );

    // Topological entanglement entropy: S_topo = ln(2) approx 0.693147
    let s_topo = NonAbelianBraidingModel::topological_entanglement_entropy();
    assert!((s_topo - 2.0_f64.ln()).abs() < 1e-12);
    assert!((s_topo - std::f64::consts::LN_2).abs() < 1e-12);
}

#[test]
fn test_quantum_trajectory_braiding_sequence_fidelity() {
    let solver = BraidingTrajectorySolver::cryogenic_low_noise();

    // 8-braid sequence
    let sequence = vec![
        BraidOperation::R12,
        BraidOperation::B23,
        BraidOperation::R12,
        BraidOperation::B23,
        BraidOperation::B23Dagger,
        BraidOperation::R12Dagger,
        BraidOperation::B23Dagger,
        BraidOperation::R12Dagger,
    ];

    let result = solver.run_braid_sequence(&sequence);

    assert_eq!(result.gate_count, 8);
    assert!(
        result.fidelity >= 0.999,
        "Braiding fidelity should exceed 99.9%, got {}",
        result.fidelity
    );
    assert!(
        result.purity >= 0.999,
        "State purity should exceed 99.9%, got {}",
        result.purity
    );
    assert!(result.passed_fidelity);

    // Check trace conservation of density matrix: Tr(\u{03c1}) = \u{03c1}00 + \u{03c1}11 = 1.0
    let tr_re = result.final_density_matrix[0].0 + result.final_density_matrix[3].0;
    let tr_im = result.final_density_matrix[0].1 + result.final_density_matrix[3].1;
    assert!((tr_re - 1.0).abs() < 1e-12);
    assert!(tr_im.abs() < 1e-12);
}
