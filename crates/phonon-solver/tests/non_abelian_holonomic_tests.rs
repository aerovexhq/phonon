#![deny(unsafe_code)]

//! Test suite for Phase 341: Non-Abelian Wilczek-Zee Gauge Connection & Tripartite Acoustic Cavity Co-Simulator.
//!
//! Verifies:
//! - Degenerate dark state orthogonality <D1|D2> = 0 and zero energy H|Da> = 0.
//! - Wilczek-Zee connection anti-Hermiticity A = -A^dagger.
//! - Non-Abelian commutation failure [U(C1), U(C2)] != 0 for non-commuting parameter loops.
//! - Geometric quantum logic gate synthesis fidelity >= 0.99 for Clifford gates (H, S, X, Z) and arbitrary rotations.
//! - Dynamical phase cancellation |E_dyn| < 1e-4 rad guaranteeing pure geometric transformation.

use phonon_solver::non_abelian_holonomic::{
    DarkSubspace, HolonomicGateType, ParameterLoop, TripartiteCavityParams, TripartiteCoSimulator,
    WilczekZeeConnection, WilsonLoopIntegrator,
};
use std::f64::consts::PI;

#[test]
fn test_degenerate_dark_state_orthogonality_and_zero_energy() {
    let test_angles = [
        (0.1, 0.2),
        (PI / 6.0, PI / 4.0),
        (PI / 4.0, PI / 3.0),
        (PI / 3.0, PI / 2.0),
        (PI / 2.0, 3.0 * PI / 4.0),
        (2.0 * PI / 3.0, 4.0 * PI / 3.0),
    ];

    let omega_0_mhz = 50.0;

    for &(theta, phi) in &test_angles {
        // 1. Orthonormality check
        let (overlap, norm1, norm2) = DarkSubspace::check_orthonormality(theta, phi);
        assert!(
            overlap.abs() < 1e-12,
            "Dark states must be strictly orthogonal: got <D1|D2> = {}",
            overlap
        );
        assert!(
            (norm1 - 1.0).abs() < 1e-12,
            "Dark state |D1> must be normalized to unity: got {}",
            norm1
        );
        assert!(
            (norm2 - 1.0).abs() < 1e-12,
            "Dark state |D2> must be normalized to unity: got {}",
            norm2
        );

        // 2. Zero energy check: H |Da> = 0
        let (res1, res2) = DarkSubspace::check_zero_energy(omega_0_mhz, theta, phi);
        assert!(
            res1 < 1e-10,
            "Dark state |D1> energy eigenvalue must be zero: got ||H|D1>|| = {}",
            res1
        );
        assert!(
            res2 < 1e-10,
            "Dark state |D2> energy eigenvalue must be zero: got ||H|D2>|| = {}",
            res2
        );

        // 3. Bright state orthogonality
        let b = DarkSubspace::bright_state(theta, phi);
        let d1 = DarkSubspace::d1(theta, phi);
        let d2 = DarkSubspace::d2(phi);
        let b_d1 = DarkSubspace::inner_product(&b, &d1);
        let b_d2 = DarkSubspace::inner_product(&b, &d2);
        assert!(
            b_d1.abs() < 1e-12,
            "Bright state |B> must be orthogonal to |D1>: got {}",
            b_d1
        );
        assert!(
            b_d2.abs() < 1e-12,
            "Bright state |B> must be orthogonal to |D2>: got {}",
            b_d2
        );
    }
}

#[test]
fn test_wilczek_zee_connection_anti_hermiticity() {
    let test_points = [
        (0.2, 0.4),
        (PI / 4.0, PI / 2.0),
        (PI / 3.0, PI),
        (PI / 2.0, 1.5 * PI),
    ];

    for &(theta, phi) in &test_points {
        let a_theta = WilczekZeeConnection::connection_theta(theta, phi);
        let a_phi = WilczekZeeConnection::connection_phi(theta, phi);

        // A^dagger = -A
        assert!(
            a_theta.is_anti_hermitian(1e-12),
            "Wilczek-Zee connection A_theta must be anti-Hermitian at theta={}, phi={}",
            theta,
            phi
        );
        assert!(
            a_phi.is_anti_hermitian(1e-12),
            "Wilczek-Zee connection A_phi must be anti-Hermitian at theta={}, phi={}",
            theta,
            phi
        );

        // Analytical matrix element verification: (A_theta)_12 = - (A_theta)_21 = -sin(theta) / 2
        let expected_s = -theta.sin() * 0.5;
        assert!(
            (a_theta.data[0][1].re - expected_s).abs() < 1e-12,
            "Matrix element (A_theta)_12 must equal -sin(theta)/2: expected {}, got {}",
            expected_s,
            a_theta.data[0][1].re
        );
        assert!(
            (a_theta.data[1][0].re - (-expected_s)).abs() < 1e-12,
            "Matrix element (A_theta)_21 must equal +sin(theta)/2: expected {}, got {}",
            -expected_s,
            a_theta.data[1][0].re
        );
        assert!(
            a_theta.data[0][0].norm() < 1e-12 && a_theta.data[1][1].norm() < 1e-12,
            "Diagonal elements of A_theta must be zero"
        );
    }
}

#[test]
fn test_non_abelian_commutation_failure() {
    // Loop 1: latitude loop around z-axis (solid angle pi/2)
    let loop1 = ParameterLoop::circular_loop_z(PI / 3.0, 200);

    // Loop 2: spherical circle around x-axis (solid angle pi/2)
    let loop2 = ParameterLoop::spherical_circle([1.0, 0.0, 0.0], PI / 3.0, 200);

    let (commutator, residual_norm) =
        WilsonLoopIntegrator::evaluate_non_abelian_commutation(&loop1, &loop2);

    // The commutator [U(C1), U(C2)] must be strictly non-zero
    assert!(
        residual_norm > 0.01,
        "Wilczek-Zee holonomies must fail to commute ([U(C1), U(C2)] != 0): got residual norm {}",
        residual_norm
    );

    // Also verify connection generator non-commutation [A_theta, A_phi] != 0
    let gen_comm = WilczekZeeConnection::connection_commutator(PI / 3.0, PI / 4.0);
    let gen_norm = gen_comm.frobenius_norm();
    assert!(
        gen_norm > 1e-6,
        "Connection generators must not commute: got ||[A_th, A_ph]|| = {}",
        gen_norm
    );
    assert_ne!(commutator.frobenius_norm(), 0.0);
}

#[test]
fn test_geometric_gate_synthesis_fidelity_for_clifford_gates() {
    let standard_gates = HolonomicGateType::standard_gates();

    for gate in standard_gates {
        let result = WilsonLoopIntegrator::synthesize_gate(gate);

        assert!(
            result.process_fidelity >= 0.99,
            "Gate synthesis fidelity for {:?} must be >= 0.99: got {}",
            gate,
            result.process_fidelity
        );

        assert!(
            result.holonomy_matrix.is_unitary(1e-6),
            "Synthesized holonomy matrix for {:?} must be unitary: diff norm {}",
            gate,
            result.holonomy_matrix.dagger().mul(&result.holonomy_matrix).sub(&phonon_solver::non_abelian_holonomic::HolonomicComplexMatrix2x2::identity()).frobenius_norm()
        );
    }

    // Additional arbitrary rotation tests
    let rot_z = WilsonLoopIntegrator::synthesize_gate(HolonomicGateType::RotationZ(PI / 2.0));
    assert!(
        rot_z.process_fidelity >= 0.99,
        "R_z(pi/2) fidelity must be >= 0.99: got {}",
        rot_z.process_fidelity
    );

    let rot_x = WilsonLoopIntegrator::synthesize_gate(HolonomicGateType::RotationX(PI / 2.0));
    assert!(
        rot_x.process_fidelity >= 0.99,
        "R_x(pi/2) fidelity must be >= 0.99: got {}",
        rot_x.process_fidelity
    );
}

#[test]
fn test_dynamical_phase_cancellation() {
    let params = TripartiteCavityParams::default();
    let sim = TripartiteCoSimulator::new(params);

    for gate in HolonomicGateType::standard_gates() {
        let trajectory = sim.simulate_trajectory(gate, 150);

        // Dynamical phase error must be < 1e-4 rad
        assert!(
            trajectory.final_dynamical_phase_error.abs() < 1e-4,
            "Dynamical phase must cancel to < 1e-4 rad for {:?}: got {}",
            gate,
            trajectory.final_dynamical_phase_error
        );

        // Process fidelity in co-simulation must be >= 0.99
        assert!(
            trajectory.final_gate_fidelity >= 0.99,
            "Co-simulation fidelity for {:?} must be >= 0.99: got {}",
            gate,
            trajectory.final_gate_fidelity
        );

        // Dark state purity must remain high (> 0.999)
        for &purity in &trajectory.dark_state_purity {
            assert!(
                purity >= 0.999,
                "Dark state purity must remain >= 0.999: got {}",
                purity
            );
        }

        // Lossy excited state population must remain zero
        for &pop in &trajectory.excited_population {
            assert_eq!(
                pop, 0.0,
                "Excited state population must remain exactly 0.0 in dark subspace"
            );
        }

        // Cavity loss decoupling must be positive in dB
        assert!(
            trajectory.cavity_loss_decoupling_db > 20.0,
            "Cavity loss decoupling should exceed 20 dB: got {}",
            trajectory.cavity_loss_decoupling_db
        );
    }
}
