//! Integration tests for Kitaev honeycomb models, Majorana dispersion, and anyon braiding.

use phonon_models::topological::{
    AnyonModelKind, BerryPhaseHolonomy, FibonacciAnyonModel, IsingAnyonModel,
    KitaevHoneycombLattice, KitaevParameters, KitaevPhase,
};

#[test]
fn test_kitaev_honeycomb_dispersion_and_phases() {
    // 1. Isotropic gapless B-phase with broken time-reversal symmetry
    let params_b = KitaevParameters::isotropic(1.0, 0.1);
    assert_eq!(params_b.phase(), KitaevPhase::GaplessB);
    assert_eq!(params_b.chern_number(), 1);

    let lattice = KitaevHoneycombLattice::new(6, 6, params_b);
    assert_eq!(lattice.num_spins(), 72);
    assert_eq!(lattice.num_plaquettes(), 36);

    // Minimum Majorana gap should be non-zero due to kappa != 0
    let gap = lattice.find_majorana_energy_gap(20);
    assert!(
        gap > 0.0,
        "Majorana gap must open under broken TRS: got {}",
        gap
    );

    // Vortex excitation gap should match analytical ~0.1536 J_z
    let v_gap = lattice.vortex_excitation_gap();
    assert!(
        (v_gap - 0.1536).abs() < 1e-4,
        "Vortex gap mismatch: got {}",
        v_gap
    );

    // 4-fold ground state degeneracy on torus (genus = 1)
    assert_eq!(lattice.topological_ground_state_degeneracy(1), 4);
    assert_eq!(lattice.topological_ground_state_degeneracy(2), 16);

    // 2. Gapped A-phases
    let params_az = KitaevParameters {
        j_x: 0.2,
        j_y: 0.2,
        j_z: 1.0,
        kappa: 0.0,
    };
    assert_eq!(params_az.phase(), KitaevPhase::GappedAz);
    assert_eq!(params_az.chern_number(), 0);

    let params_ax = KitaevParameters {
        j_x: 1.0,
        j_y: 0.3,
        j_z: 0.3,
        kappa: 0.0,
    };
    assert_eq!(params_ax.phase(), KitaevPhase::GappedAx);
}

#[test]
fn test_kitaev_toric_code_perturbative_limit() {
    let params = KitaevParameters::toric_code_limit(0.1, 1.0);
    assert_eq!(params.phase(), KitaevPhase::GappedAz);

    // J_eff = J_x^2 * J_y^2 / (16 * J_z^3) = (0.01 * 0.01) / 16 = 1e-4 / 16 = 6.25e-6
    let j_eff = params.toric_code_effective_coupling();
    assert!((j_eff - 6.25e-6).abs() < 1e-9);

    let mut lattice = KitaevHoneycombLattice::new(4, 4, params);
    // Vortex-free ground state
    assert_eq!(lattice.count_vortices(), 0);

    // Introduce a pair of vortices by flipping one z-bond gauge
    lattice.set_z_gauge(1, 1, -1);
    assert_eq!(
        lattice.count_vortices(),
        2,
        "Flipping a link gauge creates a pair of vortices"
    );
}

#[test]
fn test_ising_anyon_fusion_and_braiding() {
    let (d1, d_sigma, d_psi) = IsingAnyonModel::quantum_dimensions();
    assert!((d1 - 1.0).abs() < 1e-12);
    assert!((d_sigma - 2.0_f64.sqrt()).abs() < 1e-12);
    assert!((d_psi - 1.0).abs() < 1e-12);
    assert!((IsingAnyonModel::total_quantum_dimension() - 2.0).abs() < 1e-12);

    // Yang-Baxter equation for braid generators: sigma_1 * sigma_2 * sigma_1 = sigma_2 * sigma_1 * sigma_2
    let (yb_valid, err) = IsingAnyonModel::verify_yang_baxter();
    assert!(yb_valid, "Yang-Baxter relation failed with error {}", err);

    // Topologically protected Phase gate S
    let s_gate = IsingAnyonModel::phase_gate_s();
    assert!((s_gate.m00.0 - 1.0).abs() < 1e-6);
    assert!((s_gate.m11.1 - 1.0).abs() < 1e-6);

    // Topologically protected Hadamard gate H
    let h_gate = IsingAnyonModel::hadamard_gate_h();
    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
    assert!((h_gate.m00.0 - inv_sqrt2).abs() < 1e-6);
    assert!((h_gate.m01.0 - inv_sqrt2).abs() < 1e-6);
    assert!((h_gate.m10.0 - inv_sqrt2).abs() < 1e-6);
    assert!((h_gate.m11.0 - (-inv_sqrt2)).abs() < 1e-6);
}

#[test]
fn test_fibonacci_anyon_universal_braiding() {
    let phi = FibonacciAnyonModel::golden_ratio();
    assert!((phi - 1.6180339887).abs() < 1e-6);

    let (d1, d_tau) = FibonacciAnyonModel::quantum_dimensions();
    assert_eq!(d1, 1.0);
    assert_eq!(d_tau, phi);

    // F-matrix is orthogonal symmetric (F^2 = I)
    let f = FibonacciAnyonModel::f_matrix();
    let f_sq = f.matmul(&f);
    assert!((f_sq.m00.0 - 1.0).abs() < 1e-12);
    assert!(f_sq.m01.0.abs() < 1e-12);
    assert!(f_sq.m10.0.abs() < 1e-12);
    assert!((f_sq.m11.0 - 1.0).abs() < 1e-12);

    // Yang-Baxter equation for Fibonacci anyons
    let (yb_valid, err) = FibonacciAnyonModel::verify_yang_baxter();
    assert!(
        yb_valid,
        "Fibonacci Yang-Baxter relation failed with error {}",
        err
    );

    // Synthesize Hadamard gate
    let (_braid_h, fid) = FibonacciAnyonModel::synthesize_hadamard();
    assert!(
        fid > 0.90,
        "Hadamard braid synthesis fidelity must be high: got {}",
        fid
    );

    // Berry phase path invariance
    let holonomy = BerryPhaseHolonomy::default();
    let (inv_ising, _) = holonomy.verify_path_invariance(AnyonModelKind::Ising, 0.5);
    let (inv_fib, _) = holonomy.verify_path_invariance(AnyonModelKind::Fibonacci, 0.5);
    assert!(inv_ising);
    assert!(inv_fib);
}
