//! Integration tests for topological SSH lattices and non-Hermitian edge modes.

use phonon_models::non_hermitian::{SshLatticeParams, TopologicalLatticePhase};
use phonon_solver::non_hermitian::NonHermitianEigensolver;

#[test]
fn test_ssh_lattice_topological_phases() {
    let topo = SshLatticeParams::standard_topological_laser_lattice();
    assert_eq!(topo.phase(), TopologicalLatticePhase::Topological);
    assert_eq!(topo.topological_bandgap_hz(), 40.0e9); // 2 * |30 - 10| GHz

    let loc_length = topo.edge_state_localization_length();
    assert!(
        loc_length < 1.0,
        "Edge state should be strongly localized within < 1 unit cell, got {}",
        loc_length
    );

    let triv = SshLatticeParams::new(10, 30.0e9, 10.0e9, 2.0e9, 193.4e12);
    assert_eq!(triv.phase(), TopologicalLatticePhase::Trivial);
    assert_eq!(triv.edge_state_localization_length(), f64::INFINITY);
}

#[test]
fn test_non_hermitian_eigensolver_and_edge_gain() {
    let topo = SshLatticeParams::standard_topological_laser_lattice();
    let res = NonHermitianEigensolver::solve(&topo);

    assert_eq!(res.bulk_bandgap_hz, 40.0e9);
    assert!(
        res.edge_to_bulk_ratio > 50.0,
        "Edge mode must be strongly localized, got ratio {}",
        res.edge_to_bulk_ratio
    );

    let edge_freq = res.eigenfrequencies[res.edge_mode_index];
    assert!(
        edge_freq.1 > 0.0,
        "Edge mode must have positive net modal gain Im(w) > 0, got {}",
        edge_freq.1
    );

    assert!(
        (edge_freq.0 - topo.resonance_frequency_hz).abs() < 1e9,
        "Edge mode should sit inside the topological midgap near w0"
    );
}
