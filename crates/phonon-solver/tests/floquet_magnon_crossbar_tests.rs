#![deny(unsafe_code)]

//! Integration test suite for Topological Acoustic Floquet Chiral Magnon-Phonon
//! Crossbar Transceiver & Entanglement Router Super-Array (Phase 464).

use phonon_solver::floquet_magnon_crossbar::{
    ClusterEntanglementRouterParams, ClusterEntanglementRouterSolver,
    FloquetChiralTransceiverParams, FloquetChiralTransceiverSolver, FloquetMagnonCrossbarProcessor,
    SyntheticCirculatorArrayParams, SyntheticCirculatorArraySolver,
};

#[test]
fn test_floquet_chiral_transceiver_physics() {
    let params = FloquetChiralTransceiverParams {
        bare_acoustic_freq_ghz: 4.8,
        bare_magnon_freq_ghz: 4.8,
        bias_field_oe: 1720.0,
        floquet_drive_freq_mhz: 500.0,
        floquet_drive_amplitude_oe: 12.0,
        magnetoelastic_coupling_mhz: 42.5,
        gilbert_damping: 1.2e-4,
        acoustic_loss_rate_khz: 15.0,
        propagation_length_um: 80.0,
    };
    let solver = FloquetChiralTransceiverSolver::new(params);
    let metrics = solver.solve();

    // 1. Dynamic time-reversal symmetry breaking
    assert!(
        metrics.wavevector_asymmetry_rad_per_um >= 0.05,
        "Wavevector asymmetry {:.3} below 0.05 rad/um",
        metrics.wavevector_asymmetry_rad_per_um
    );

    // 2. Forward insertion loss <= 0.30 dB
    assert!(
        metrics.insertion_loss_db <= 0.30,
        "Insertion loss {:.2} dB exceeded 0.30 dB",
        metrics.insertion_loss_db
    );

    // 3. Reverse non-reciprocal isolation >= 40.0 dB
    assert!(
        metrics.reverse_isolation_db >= 40.0,
        "Reverse isolation {:.1} dB below 40.0 dB",
        metrics.reverse_isolation_db
    );

    // 4. Directivity >= 38.0 dB
    assert!(
        metrics.directivity_db >= 38.0,
        "Directivity {:.1} dB below 38.0 dB",
        metrics.directivity_db
    );

    // 5. Bandwidth >= 120.0 MHz
    assert!(
        metrics.transduction_bandwidth_mhz >= 120.0,
        "Bandwidth {:.1} MHz below 120.0 MHz",
        metrics.transduction_bandwidth_mhz
    );

    // 6. Transduction efficiency >= 88.0%
    assert!(
        metrics.transduction_efficiency_percent >= 88.0,
        "Transduction efficiency {:.1}% below 88.0%",
        metrics.transduction_efficiency_percent
    );

    // Dispersion spectrum calculation
    let spectrum = solver.compute_dispersion_spectrum();
    assert_eq!(spectrum.len(), 60);
    assert!(spectrum.iter().all(|pt| pt.forward_transmission_db > pt.reverse_transmission_db));
}

#[test]
fn test_synthetic_circulator_array_scattering_matrix() {
    let params = SyntheticCirculatorArrayParams {
        port_count: 4,
        center_freq_ghz: 4.8,
        synthetic_phase_rad: std::f64::consts::FRAC_PI_2,
        coupling_quality_q: 35_000.0,
        junction_loss_db: 0.08,
    };
    let solver = SyntheticCirculatorArraySolver::new(params);
    let metrics = solver.solve();

    // Insertion loss <= 0.35 dB
    assert!(
        metrics.insertion_loss_db <= 0.35,
        "Insertion loss {:.2} dB exceeded 0.35 dB",
        metrics.insertion_loss_db
    );

    // Isolation >= 40.0 dB
    assert!(
        metrics.isolation_db >= 40.0,
        "Isolation {:.1} dB below 40.0 dB",
        metrics.isolation_db
    );

    // Directivity >= 38.0 dB
    assert!(
        metrics.directivity_db >= 38.0,
        "Directivity {:.1} dB below 38.0 dB",
        metrics.directivity_db
    );

    // Return loss >= 22.0 dB
    assert!(
        metrics.return_loss_db >= 22.0,
        "Return loss {:.1} dB below 22.0 dB",
        metrics.return_loss_db
    );

    // Cross-port isolation >= 42.0 dB
    assert!(
        metrics.cross_port_isolation_db >= 42.0,
        "Cross-port isolation {:.1} dB below 42.0 dB",
        metrics.cross_port_isolation_db
    );

    // Permutation symmetry error <= 0.05 dB
    assert!(
        metrics.permutation_symmetry_error_db <= 0.05,
        "Symmetry error {:.3} dB exceeded 0.05 dB",
        metrics.permutation_symmetry_error_db
    );

    // Full 4x4 scattering matrix
    let s_matrix = solver.compute_s_matrix();
    assert_eq!(s_matrix.len(), 16);
}

#[test]
fn test_cluster_entanglement_router_squeezing_and_duan_simon() {
    let params = ClusterEntanglementRouterParams {
        cluster_node_count: 6,
        parametric_squeezing_r: 0.95,
        operating_freq_ghz: 4.8,
        dilution_temp_mk: 15.0,
        bus_attenuation_db_per_cm: 0.15,
        cryo_cmos_bias_current_ua: 120.0,
    };
    let solver = ClusterEntanglementRouterSolver::new(params);
    let metrics = solver.solve();

    // Squeezing depth >= 6.0 dB below shot noise
    assert!(
        metrics.squeezing_depth_db >= 6.0,
        "Squeezing depth {:.2} dB below 6.0 dB",
        metrics.squeezing_depth_db
    );

    // Duan-Simon inseparability nullifier <= 0.50 (well below 1.0)
    assert!(
        metrics.duan_simon_nullifier <= 0.50,
        "Duan-Simon nullifier {:.3} exceeded 0.50",
        metrics.duan_simon_nullifier
    );

    // Entanglement routing fidelity >= 99.2%
    assert!(
        metrics.entanglement_routing_fidelity_percent >= 99.2,
        "Routing fidelity {:.2}% below 99.2%",
        metrics.entanglement_routing_fidelity_percent
    );

    // Node generation
    let nodes = solver.compute_cluster_nodes();
    assert_eq!(nodes.len(), 6);

    // Polar quadrature profile
    let profile = solver.compute_quadrature_profile();
    assert_eq!(profile.len(), 60);
    assert!(profile.iter().any(|pt| pt.is_squeezed_below_sql));
}

#[test]
fn test_cryo_cmos_quantum_interface_metrics() {
    let params = ClusterEntanglementRouterParams {
        dilution_temp_mk: 15.0,
        operating_freq_ghz: 4.8,
        cryo_cmos_bias_current_ua: 120.0,
        ..Default::default()
    };
    let solver = ClusterEntanglementRouterSolver::new(params);
    let metrics = solver.solve();

    // Dilution temperature thermal occupancy n_th <= 1.0e-3
    assert!(
        metrics.thermal_phonon_occupancy <= 1.0e-3,
        "Thermal occupancy {:.2e} exceeded 1e-3",
        metrics.thermal_phonon_occupancy
    );

    // Cryo-CMOS added noise n_add <= 0.08 quanta
    assert!(
        metrics.added_noise_quanta <= 0.08,
        "Added noise quanta {:.3} exceeded 0.08",
        metrics.added_noise_quanta
    );

    // Power dissipation <= 2.5 mW
    assert!(
        metrics.cryo_cmos_power_dissipation_mw <= 2.5,
        "Power dissipation {:.2} mW exceeded 2.5 mW",
        metrics.cryo_cmos_power_dissipation_mw
    );
}

#[test]
fn test_floquet_magnon_crossbar_10_point_physics_audit() {
    let processor = FloquetMagnonCrossbarProcessor::default();
    let audit = processor.audit();

    assert!(audit.floquet_time_reversal_symmetry_breaking);
    assert!(audit.magnetoelastic_transduction_coupling);
    assert!(audit.non_reciprocal_chiral_isolation);
    assert!(audit.forward_insertion_loss);
    assert!(audit.transceiver_operational_bandwidth);
    assert!(audit.multi_terminal_circulator_directivity);
    assert!(audit.port_return_loss_matching);
    assert!(audit.cv_squeezing_below_shot_noise);
    assert!(audit.duan_simon_epr_inseparability);
    assert!(audit.cryo_cmos_quantum_limited_noise);

    let (score, total) = audit.score();
    assert_eq!(total, 10);
    assert_eq!(score, 10);
    assert!(audit.is_pass());
}
