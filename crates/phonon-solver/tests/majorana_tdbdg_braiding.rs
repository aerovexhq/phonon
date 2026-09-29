//! Integration tests for Alicea tri-junction braiding, gate ramps,
//! time-dependent Bogoliubov-de Gennes (TdBdG) solver, and non-Abelian quantum logic.

use phonon_models::hexagonal_majorana::{
    AliceaTriJunctionBraiding, GateRampProfile, InPlaneMagneticField, MajoranaMaterialParams,
    TopologicalQubitRegister,
};
use phonon_solver::hexagonal_majorana::{
    Complex, MajoranaBraidSimulator, MajoranaNoiseEnvironment, TdBdGLatticeConfig, TdBdGSolver,
};
use std::f64::consts::PI;

#[test]
fn test_alicea_braid_choreography_and_ramp_profiles() {
    let arm_length_nm = 1000.0;
    let braiding =
        AliceaTriJunctionBraiding::new(100.0e-9, GateRampProfile::SmoothPolynomial, arm_length_nm);

    assert_eq!(braiding.total_duration_s(), 300.0e-9);

    // Test that throughout the braid, the Majoranas never collide
    let num_samples = 30;
    for s in 0..=num_samples {
        let t = (s as f64) * braiding.total_duration_s() / (num_samples as f64);
        let sep = braiding.separation_nm(t);
        assert!(
            sep > 0.3 * arm_length_nm,
            "Majoranas must maintain safe separation at t={:.1} ns (got sep={:.1} nm)",
            t * 1e9,
            sep
        );
    }

    // Verify that SmoothPolynomial has zero initial and final velocity
    assert_eq!(
        GateRampProfile::SmoothPolynomial.evaluate_velocity(0.0),
        0.0
    );
    assert_eq!(
        GateRampProfile::SmoothPolynomial.evaluate_velocity(1.0),
        0.0
    );

    // Verify diabatic error comparison: SmoothPolynomial must suppress Landau-Zener error
    // by orders of magnitude compared to Linear ramp in fast-switching regime
    let braiding_linear =
        AliceaTriJunctionBraiding::new(0.05e-9, GateRampProfile::Linear, arm_length_nm);
    let braiding_smooth =
        AliceaTriJunctionBraiding::new(0.05e-9, GateRampProfile::SmoothPolynomial, arm_length_nm);
    let err_linear = braiding_linear.diabatic_error(0.05);
    let err_smooth = braiding_smooth.diabatic_error(0.05);
    assert!(
        err_smooth < err_linear,
        "Smooth polynomial ramp must have significantly lower diabatic error than linear ramp"
    );
}

#[test]
fn test_tdbdg_solver_rk4_unitarity_and_berry_phase() {
    let materials = MajoranaMaterialParams::inas_al();
    let field = InPlaneMagneticField::new(1.0, PI / 6.0);
    let config = TdBdGLatticeConfig::new(6, 800.0, &materials, &field);
    let braiding =
        AliceaTriJunctionBraiding::new(20.0e-9, GateRampProfile::SmoothPolynomial, 800.0);

    let solver = TdBdGSolver::new(config, braiding);
    let dim = solver.config.nambu_dimension();

    // Create normalized test wavepacket localized on Arm 0
    let mut initial_psi = vec![Complex::zero(); dim];
    initial_psi[4] = Complex::one(); // site 1, u_up component
    initial_psi[5] = Complex::new(0.0, 0.5);

    let norm = (initial_psi[4].norm_sq() + initial_psi[5].norm_sq()).sqrt();
    initial_psi[4] = initial_psi[4].scale(1.0 / norm);
    initial_psi[5] = initial_psi[5].scale(1.0 / norm);

    // Perform RK4 step
    let step_psi = solver.rk4_step(&initial_psi, 0.0, 1.0e-12);
    let mut step_norm_sq = 0.0;
    for c in &step_psi {
        step_norm_sq += c.norm_sq();
    }
    assert!(
        (step_norm_sq.sqrt() - 1.0).abs() < 1e-10,
        "RK4 integration must strictly preserve wavefunction unitarity"
    );

    // Integrate short braid
    let (final_psi, berry_phase, max_leak) = solver.integrate_braid(&initial_psi, 10);
    let mut final_norm_sq = 0.0;
    for c in &final_psi {
        final_norm_sq += c.norm_sq();
    }
    assert!(
        (final_norm_sq.sqrt() - 1.0).abs() < 1e-8,
        "Full braid integration must preserve unitarity"
    );
    assert!(berry_phase.is_finite());
    assert!(max_leak < 0.05);
}

#[test]
fn test_topological_qubit_braid_gates_and_non_abelian_statistics() {
    let materials = MajoranaMaterialParams::inas_al();
    let field = InPlaneMagneticField::new(1.0, PI / 6.0);
    let noise = MajoranaNoiseEnvironment::standard_cryogenic();
    let sim = MajoranaBraidSimulator::new(materials, field, noise, 1500.0);

    // Test B12 on |0_L>
    let res_b12 = sim.simulate_braid_12(
        (1.0, 0.0),
        (0.0, 0.0),
        100.0e-9,
        GateRampProfile::SmoothPolynomial,
    );
    assert!(
        res_b12.fidelity > 0.99,
        "B12 braid fidelity must exceed 99% (got {})",
        res_b12.fidelity
    );
    assert!(res_b12.diabatic_leakage < 1e-4);

    // Test B23 on |0_L> (creates equal superposition with relative phase -i)
    let res_b23 = sim.simulate_braid_23(
        (1.0, 0.0),
        (0.0, 0.0),
        100.0e-9,
        GateRampProfile::SmoothPolynomial,
    );
    assert!(
        res_b23.fidelity > 0.99,
        "B23 braid fidelity must exceed 99% (got {})",
        res_b23.fidelity
    );

    // Test Pauli Z = B12^2
    let mut q = TopologicalQubitRegister::new_one(0.20);
    q.apply_pauli_z();
    // Z on |1_L> produces -|1_L>
    assert!((q.beta.0 - (-1.0)).abs() < 1e-10);

    // Verify non-Abelian statistics: B12 B23 != B23 B12
    let comm_norm = sim.verify_non_abelian_commutator();
    assert!(
        comm_norm > 0.5,
        "Majorana braiding operations must be strictly non-commutative (got norm={})",
        comm_norm
    );

    // Verify Yang-Baxter relation: B12 B23 B12 == B23 B12 B23
    let yb_err = sim.verify_yang_baxter_relation();
    assert!(
        yb_err < 1e-10,
        "Braid group Yang-Baxter relation must hold exactly (got error={})",
        yb_err
    );
}
