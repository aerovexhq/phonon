use phonon_models::non_hermitian_skin::{NhseLattice, NhseLatticeParams};
use phonon_solver::non_hermitian_skin::NonBlochTransferMatrixSolver;

#[test]
fn test_nhse_asymmetric_coupling_and_gbz_radius() {
    let params = NhseLatticeParams::standard_chain();

    let tr = params.forward_hopping();
    let tl = params.backward_hopping();

    // Forward hopping must strictly exceed backward hopping for gamma > 0
    assert!(
        tr > tl,
        "Forward hopping {} must exceed backward hopping {}",
        tr,
        tl
    );

    let gbz_r = params.gbz_radius();
    assert!(
        gbz_r < 1.0,
        "GBZ radius must be strictly deformed inside the unit circle (r < 1): got {}",
        gbz_r
    );

    // Skin localization factor must exceed 30 dB
    let skin_db = params.skin_localization_factor_db();
    assert!(
        skin_db >= 30.0,
        "Skin localization factor must exceed 30 dB: got {} dB",
        skin_db
    );
}

#[test]
fn test_point_gap_topological_winding_and_spatial_localization() {
    let params = NhseLatticeParams::standard_chain();
    let solver = NonBlochTransferMatrixSolver::new(params);
    let res = solver.solve();

    // Winding number around loop center must be non-trivial (|W| = 1)
    assert_ne!(
        res.point_gap_winding, 0,
        "Point-gap winding number must be non-zero inside PBC spectrum loop"
    );

    assert!(
        res.skin_localization_db >= 30.0,
        "Skin localization must be >= 30 dB: got {} dB",
        res.skin_localization_db
    );

    // Spatial wavepacket profile must be exponentially localized at boundary site 0
    let lattice = NhseLattice::new(params);
    let profile = lattice.skin_wavepacket_profile();
    assert_eq!(profile.len(), params.num_sites);

    let ratio_boundary = profile[0] / profile[params.num_sites - 1];
    assert!(
        ratio_boundary > 50.0,
        "Wavepacket ratio between boundaries must be >> 1: got {}",
        ratio_boundary
    );
}
