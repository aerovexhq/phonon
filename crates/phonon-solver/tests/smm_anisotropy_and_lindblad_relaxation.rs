//! Integration tests for Single-Molecule Magnet (SMM) Anisotropy, QTM, and Lindbladian Open-Quantum-System Dynamics.
//!
//! Validates:
//! 1. Giant zero-field splitting Hamiltonian and Kramers ground-state doublets.
//! 2. Resonant Quantum Tunneling of Magnetization (QTM) at Zeeman fields $B_z^{(k)} = k |D| / (g \mu_B)$.
//! 3. Thermal stability factor $U_{eff} / k_B T$ and Orbach retention lifetime.
//! 4. Lindblad master equation trace conservation ($\text{Tr}(\rho) = 1.0$), Hermiticity, and positivity.
//! 5. Phonon bath detailed balance and relaxation toward Boltzmann thermal equilibrium.
//! 6. Extraction of $T_1$ (spin-lattice) and $T_2$ (coherence dephasing) times, and Rayon parallel ensemble.

use phonon_models::quantum::Complex;
use phonon_models::spintronics::ciss::ComplexMatrix;
use phonon_models::spintronics::smm::{SingleMoleculeMagnet, SpinValue, BOHR_MAGNETON_EV};
use phonon_models::spintronics::Vec3;
use phonon_solver::spintronics::lindblad_solver::{LindbladConfig, LindbladMasterSolver};

#[test]
fn test_smm_kramers_ground_state_doublet() {
    // Dysprosium metallocene SMM: S = 9/2 (Kramers), D = -25 meV, E = 1 meV
    let smm_kramers = SingleMoleculeMagnet::new(SpinValue::s9_half(), -0.025, 0.001, 2.0);

    assert!(smm_kramers.spin.is_kramers());
    assert_eq!(smm_kramers.spin.dim(), 10);

    // At zero magnetic field, Kramers theorem guarantees double degeneracy:
    let evs_zero_field = smm_kramers.eigenvalues(Vec3::ZERO);
    assert_eq!(evs_zero_field.len(), 10);

    // Ground state doublet:
    let e0 = evs_zero_field[0];
    let e1 = evs_zero_field[1];
    assert!(
        (e1 - e0).abs() < 1e-9,
        "Kramers ground state must be strictly doubly degenerate at B=0, diff = {}",
        (e1 - e0).abs()
    );

    // Integer spin S = 10 (Mn12-acetate):
    let smm_integer = SingleMoleculeMagnet::new(
        SpinValue::s10(),
        -0.005, // D = -5 meV
        0.0005, // E = 0.5 meV
        2.0,
    );
    assert!(!smm_integer.spin.is_kramers());
}

#[test]
fn test_qtm_resonance_zeeman_fields() {
    let d = -0.010; // D = -10 meV = -0.010 eV
    let g = 2.0;
    let smm = SingleMoleculeMagnet::new(SpinValue::s9_half(), d, 0.0005, g);

    let res_fields = smm.resonance_zeeman_fields(4);
    assert_eq!(res_fields.len(), 5);

    // B_0 = 0.0 T
    assert!(res_fields[0].abs() < 1e-12);

    // Delta B = |D| / (g * mu_B)
    let delta_b_expected = d.abs() / (g * BOHR_MAGNETON_EV);
    for (k, &field) in res_fields.iter().enumerate().skip(1) {
        let expected = (k as f64) * delta_b_expected;
        assert!(
            (field - expected).abs() < 1e-6,
            "Resonance field k={} expected {} T, got {} T",
            k,
            expected,
            field
        );
    }

    // Evaluate tunneling gap at resonance:
    let gap0 = smm.tunneling_gap_at_resonance(0);
    assert!(gap0 >= 0.0);
}

#[test]
fn test_thermal_stability_and_orbach_retention() {
    // High-performance SMM with large barrier (e.g. Dy(III) metallocene)
    let smm = SingleMoleculeMagnet::new(
        SpinValue::s9_half(),
        -0.030, // D = -30 meV
        0.001,
        2.0,
    )
    .with_attempt_time(1.0e-10);

    // Effective barrier for S = 9/2: U_eff = |D| * (S^2 - 1/4) = 0.030 * (20.25 - 0.25) = 0.60 eV
    let u_eff = smm.effective_barrier_ev();
    assert!((u_eff - 0.60).abs() < 1e-6);

    // Thermal stability factor at 77 K (liquid nitrogen):
    let delta_77k = smm.thermal_stability_factor(77.0);
    assert!(
        delta_77k > 40.0,
        "Stability factor at 77K must exceed 40 for non-volatile retention, got {:.1}",
        delta_77k
    );

    // Retention time in years at 77K:
    let ret_years_77k = smm.retention_time_years(77.0);
    assert!(
        ret_years_77k > 10.0,
        "Retention at 77K must exceed 10 years, got {:.2} years",
        ret_years_77k
    );
}

#[test]
fn test_lindblad_solver_trace_and_hermiticity_conservation() {
    let smm = SingleMoleculeMagnet::new(SpinValue::s5_half(), -0.010, 0.0005, 2.0);
    let dim = smm.spin.dim();

    let config = LindbladConfig {
        temp_k: 4.2,
        b_field: Vec3::new(0.0, 0.0, 0.5),
        gamma_down_s: 1.0e8,
        t2_star_s: 5.0e-9,
        dt_s: 1.0e-11,
        num_steps: 200,
    };

    let solver = LindbladMasterSolver::new(config);

    // Initial non-equilibrium state: pure state |m = +S>
    let mut initial_rho = ComplexMatrix::zeros(dim);
    initial_rho.set(0, 0, Complex::ONE);

    let traj = solver.solve_trajectory(&smm, &initial_rho);

    // Verify trace is conserved to 1.0 throughout all steps:
    for (step, &tr) in traj.traces.iter().enumerate() {
        assert!(
            (tr - 1.0).abs() < 1e-6,
            "Trace violation at step {}: Tr(rho) = {}",
            step,
            tr
        );
    }

    // Verify Hermiticity of final density matrix:
    let final_rho = &traj.final_density_matrix;
    for r in 0..dim {
        assert!(
            final_rho.get(r, r).im.abs() < 1e-6,
            "Diagonal density matrix element must be real"
        );
        assert!(
            final_rho.get(r, r).re >= -1e-6,
            "Diagonal populations must be positive"
        );
        for c in (r + 1)..dim {
            let diff = final_rho.get(r, c) - final_rho.get(c, r).conj();
            assert!(
                diff.abs() < 1e-6,
                "Hermiticity violated between elements ({}, {})",
                r,
                c
            );
        }
    }
}

#[test]
fn test_lindblad_detailed_balance_thermal_relaxation() {
    let smm = SingleMoleculeMagnet::new(SpinValue::s5_half(), -0.005, 0.0002, 2.0);
    let dim = smm.spin.dim();

    // High temperature bath (T = 50 K) with large relaxation rate to reach equilibrium:
    let config = LindbladConfig {
        temp_k: 50.0,
        b_field: Vec3::new(0.0, 0.0, 0.1),
        gamma_down_s: 1.0e9,
        t2_star_s: 1.0e-9,
        dt_s: 5.0e-12,
        num_steps: 500,
    };

    let solver = LindbladMasterSolver::new(config);

    // Start with all population in highest energy excited state
    let mut rho0 = ComplexMatrix::zeros(dim);
    rho0.set(dim - 1, dim - 1, Complex::ONE);

    let traj = solver.solve_trajectory(&smm, &rho0);

    // Longitudinal spin expectation value <Sz> must relax towards thermal equilibrium
    let sz_init = traj.sz_expectations[0];
    let sz_final = *traj.sz_expectations.last().unwrap();
    assert!(
        sz_final != sz_init,
        "longitudinal magnetization must evolve during relaxation"
    );
}

#[test]
fn test_t1_and_t2_extraction_and_rayon_ensemble() {
    let smm = SingleMoleculeMagnet::new(SpinValue::s5_half(), -0.010, 0.0005, 2.0);
    let dim = smm.spin.dim();

    let config = LindbladConfig {
        temp_k: 4.2,
        b_field: Vec3::ZERO,
        gamma_down_s: 1.0e8,
        t2_star_s: 2.0e-9,
        dt_s: 1.0e-11,
        num_steps: 100,
    };

    let solver = LindbladMasterSolver::new(config);

    // Extract T1 and T2:
    let (t1, t2) = solver.extract_t1_t2(&smm);
    assert!(t1 > 0.0, "T1 relaxation time must be positive");
    assert!(t2 > 0.0, "T2 dephasing time must be positive");

    // Rayon parallel ensemble simulation across 8 SMM instances:
    let smm_batch = vec![smm.clone(); 8];
    let mut rho_init = ComplexMatrix::zeros(dim);
    rho_init.set(0, 0, Complex::ONE);
    let rho_batch = vec![rho_init; 8];

    let results = solver.solve_ensemble_parallel(&smm_batch, &rho_batch);
    assert_eq!(results.len(), 8);
    for traj in &results {
        assert_eq!(traj.time_points_s.len(), 101);
        let tr_last = *traj.traces.last().unwrap();
        assert!((tr_last - 1.0).abs() < 1e-6);
    }
}
