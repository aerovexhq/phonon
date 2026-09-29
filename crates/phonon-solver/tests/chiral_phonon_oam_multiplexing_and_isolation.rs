//! Integration tests for OAM mode multiplexing, isolation, and crosstalk.

use phonon_models::chiral_phonon_spin_mechanics::ChiralPhononSpinParams;
use phonon_solver::chiral_phonon_spin_mechanics::ChiralPhononSpinSolver;

#[test]
fn test_oam_topological_charges_isolation_and_crosstalk() {
    let charges = [-3, -2, -1, 1, 2, 3];
    for &charge in &charges {
        let params = ChiralPhononSpinParams {
            topological_oam_charge: charge,
            waveguide_radius_um: 18.0,
            transducer_finger_pairs: 55,
            ..Default::default()
        };
        let solver = ChiralPhononSpinSolver::new(params);
        let metrics = solver.solve();

        assert!(
            metrics.oam_mode_isolation_db >= 25.0,
            "Mode isolation for l={} was {} dB < 25.0 dB",
            charge,
            metrics.oam_mode_isolation_db
        );
        assert!(
            metrics.channel_crosstalk_db <= -20.0,
            "Channel crosstalk for l={} was {} dB > -20.0 dB",
            charge,
            metrics.channel_crosstalk_db
        );
        assert!(
            metrics.router_extinction_ratio_db >= 25.0,
            "Router extinction for l={} was {} dB < 25.0 dB",
            charge,
            metrics.router_extinction_ratio_db
        );
        assert!(
            metrics.multiplexed_capacity_gbps >= 10.0,
            "Capacity for l={} was {} Gbps < 10.0 Gbps",
            charge,
            metrics.multiplexed_capacity_gbps
        );
    }
}

#[test]
fn test_router_extinction_and_bandwidth_capacity() {
    let params = ChiralPhononSpinParams {
        acoustic_frequency_ghz: 5.0,
        topological_oam_charge: 2,
        waveguide_radius_um: 20.0,
        ..Default::default()
    };
    let solver = ChiralPhononSpinSolver::new(params);
    let ext = solver.compute_router_extinction_ratio_db();
    let cap = solver.compute_multiplexed_capacity_gbps();

    assert!(
        ext >= 25.0,
        "Router extinction {} dB should be >= 25.0 dB",
        ext
    );
    assert!(cap >= 15.0, "Capacity {} Gbps should be >= 15.0 Gbps", cap);
}
