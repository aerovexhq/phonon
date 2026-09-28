use phonon_models::diamond_nv::{NvCenterConfig, NV_AXES};
use phonon_solver::diamond_nv::{NvEigenResult, NvHamiltonianSolver};

#[test]
fn test_zero_field_splitting_and_strain() {
    let config = NvCenterConfig::default();
    let solver = NvHamiltonianSolver::new(config.clone());

    // At B = 0, eigenvalues are 0, D - E, D + E
    let eigen: NvEigenResult = solver.solve_hamiltonian_nv_frame([0.0, 0.0, 0.0]);

    let d = config.zero_field_splitting_hz;
    let e = config.transverse_strain_hz;

    assert!((eigen.eigenenergies_hz[0] - 0.0).abs() < 1e-3);
    assert!((eigen.eigenenergies_hz[1] - (d - e)).abs() < 1e-3);
    assert!((eigen.eigenenergies_hz[2] - (d + e)).abs() < 1e-3);
    assert!((eigen.zeeman_splitting_hz - 2.0 * e).abs() < 1e-3);
}

#[test]
fn test_linear_zeeman_splitting() {
    let config = NvCenterConfig::default();
    let solver = NvHamiltonianSolver::new(config.clone());

    // Apply parallel field along NV axis (B_z = 2.0 mT = 2.0e-3 T)
    let b_z = 2.0e-3;
    let eigen = solver.solve_hamiltonian_nv_frame([0.0, 0.0, b_z]);

    let gamma = config.electron_gyromagnetic_ratio_hz_per_t;
    let e = config.transverse_strain_hz;
    let expected_split = 2.0 * ((gamma * b_z).powi(2) + e.powi(2)).sqrt();

    assert!(
        (eigen.zeeman_splitting_hz - expected_split).abs() / expected_split < 1e-5,
        "Zeeman splitting {} deviated from expected {}",
        eigen.zeeman_splitting_hz,
        expected_split
    );
    // Splitting for 2 mT should be ~ 112 MHz (28 MHz/mT * 2 * 2)
    assert!(eigen.zeeman_splitting_hz > 1.10e8);
}

#[test]
fn test_3d_vector_magnetic_field_reconstruction() {
    let config = NvCenterConfig::default();
    let solver = NvHamiltonianSolver::new(config.clone());

    // Arbitrary true test field in laboratory frame: [1.2 mT, -2.4 mT, 3.1 mT]
    let b_true = [1.2e-3, -2.4e-3, 3.1e-3];

    let mut resonance_pairs = [(0.0, 0.0); 4];
    for (i, pair) in resonance_pairs.iter_mut().enumerate() {
        let eigen = solver.solve_orientation(b_true, i);
        *pair = (eigen.lower_transition_hz, eigen.upper_transition_hz);
    }

    let b_rec = solver.reconstruct_vector_field_from_odmr(&resonance_pairs, Some(b_true));

    let err_x = (b_rec[0] - b_true[0]).abs();
    let err_y = (b_rec[1] - b_true[1]).abs();
    let err_z = (b_rec[2] - b_true[2]).abs();
    let total_err = (err_x.powi(2) + err_y.powi(2) + err_z.powi(2)).sqrt();

    assert!(
        total_err < 1.0e-7,
        "Total vector reconstruction error was {} T, expected < 1e-7 T (0.1 uT)",
        total_err
    );
}

#[test]
fn test_4_nv_crystallographic_orientations() {
    // Check that the 4 NV unit vectors are normalized and mutually symmetric
    for axis in &NV_AXES {
        let norm_sq = axis[0].powi(2) + axis[1].powi(2) + axis[2].powi(2);
        assert!((norm_sq - 1.0).abs() < 1e-12);
    }

    // Dot product between any two distinct NV axes is cos(109.47 deg) = -1/3
    for (i, axis_i) in NV_AXES.iter().enumerate() {
        for (j, axis_j) in NV_AXES.iter().enumerate().skip(i + 1) {
            let dot = axis_i[0] * axis_j[0] + axis_i[1] * axis_j[1] + axis_i[2] * axis_j[2];
            assert!(
                (dot + 1.0 / 3.0).abs() < 1e-12,
                "Dot product between axes {} and {} was {}, expected -1/3",
                i,
                j,
                dot
            );
        }
    }
}
