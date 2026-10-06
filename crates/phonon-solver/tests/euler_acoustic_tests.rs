#![deny(unsafe_code)]

use phonon_solver::euler_acoustic::{
    solve_real_symmetric_3x3, EulerEdgeTransportEngine, EulerLatticeSolver, EulerParams,
    EulerPhase, RibbonParams,
};
use std::f64::consts::PI;

#[test]
fn test_real_symmetric_hamiltonian_and_eigenvectors() {
    let mat = [
        [4.0, 1.0, -2.0],
        [1.0, 3.0, 0.5],
        [-2.0, 0.5, 5.0],
    ];

    let (evals, evecs) = solve_real_symmetric_3x3(mat);

    // Eigenvalues must be sorted ascending
    assert!(evals[0] <= evals[1]);
    assert!(evals[1] <= evals[2]);

    // Check orthonormality of real frame vectors: u_i . u_j = delta_ij
    for i in 0..3 {
        for j in 0..3 {
            let dot = evecs[i][0] * evecs[j][0]
                + evecs[i][1] * evecs[j][1]
                + evecs[i][2] * evecs[j][2];
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!(
                (dot - expected).abs() < 1e-5,
                "Orthogonality error for i={}, j={}: got {:.6}",
                i, j, dot
            );
        }
    }

    // Verify matrix reconstruction: sum_k lambda_k u_k,i u_k,j = A_ij
    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0;
            for k in 0..3 {
                sum += evals[k] * evecs[k][i] * evecs[k][j];
            }
            assert!(
                (sum - mat[i][j]).abs() < 1e-4,
                "Matrix reconstruction error at ({}, {}): got {:.5}, expected {:.5}",
                i, j, sum, mat[i][j]
            );
        }
    }
}

#[test]
fn test_topological_euler_phase_classification() {
    let params = EulerParams {
        hopping_ta_hz: 300.0,
        coupling_t12_hz: 400.0,
        coupling_t13_hz: 350.0,
        coupling_t23_hz: 350.0,
        mass_m1_hz: 200.0,
        mass_m2_hz: -200.0,
        mass_m3_hz: 0.0,
        delta_12_hz: 150.0,
        ..Default::default()
    };
    let solver = EulerLatticeSolver::new(params);

    // Classification must be TopologicalEuler
    assert_eq!(solver.phase, EulerPhase::TopologicalEuler);

    // Quantized Euler invariant chi must be 1
    assert_eq!(
        solver.quantized_euler_class, 1,
        "Quantized Euler class must be 1 in topological phase"
    );
    assert_eq!(solver.patch_euler_invariant, 1);

    // Bulk bandgaps must be non-zero
    assert!(
        solver.bulk_gap_1_hz > 50.0,
        "Bulk gap 1 must be > 50 Hz, got {:.1} Hz",
        solver.bulk_gap_1_hz
    );
    assert!(
        solver.bulk_gap_2_hz > 50.0,
        "Bulk gap 2 must be > 50 Hz, got {:.1} Hz",
        solver.bulk_gap_2_hz
    );
}

#[test]
fn test_trivial_and_nodal_phases() {
    // Trivial configuration: vanishing dimerization and small mass differences
    let params_trivial = EulerParams {
        mass_m1_hz: 0.0,
        mass_m2_hz: 0.0,
        mass_m3_hz: 0.0,
        delta_12_hz: 0.0,
        ..Default::default()
    };
    let solver_trivial = EulerLatticeSolver::new(params_trivial);

    assert_ne!(solver_trivial.phase, EulerPhase::TopologicalEuler);
    assert_eq!(solver_trivial.quantized_euler_class, 0);
}

#[test]
fn test_domain_wall_edge_state_confinement() {
    let params = RibbonParams {
        lattice_params: EulerParams::default(),
        num_cells_y: 20,
        is_domain_wall: true,
        defect_disorder_hz: 0.0,
    };
    let engine = EulerEdgeTransportEngine::new(params);

    // Edge localization ratio must be >= 80%
    assert!(
        engine.metrics.edge_confinement_pct >= 80.0,
        "Domain wall edge confinement must be >= 80%, got {:.2}%",
        engine.metrics.edge_confinement_pct
    );

    // Domain wall transmission efficiency must be high (>= -0.5 dB)
    assert!(
        engine.metrics.transmission_efficiency_db >= -0.5,
        "Transmission efficiency must be >= -0.5 dB, got {:.2} dB",
        engine.metrics.transmission_efficiency_db
    );

    // Bulk isolation must be >= 25.0 dB
    assert!(
        engine.metrics.bulk_isolation_db >= 25.0,
        "Bulk isolation must be >= 25 dB, got {:.2} dB",
        engine.metrics.bulk_isolation_db
    );

    // Selected edge mode profile check
    assert_eq!(
        engine.selected_edge_mode.spatial_intensity_profile.len(),
        20
    );
}

#[test]
fn test_non_abelian_frame_rotation() {
    let params = RibbonParams {
        lattice_params: EulerParams::default(),
        ..Default::default()
    };
    let engine = EulerEdgeTransportEngine::new(params);

    // Non-Abelian frame rotation angle must equal pi in the topological Euler phase
    assert!(
        (engine.metrics.frame_rotation_angle_rad - PI).abs() < 1e-4,
        "Frame rotation angle must be pi rad, got {:.4} rad",
        engine.metrics.frame_rotation_angle_rad
    );
}
