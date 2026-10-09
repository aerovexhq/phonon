#![deny(unsafe_code)]

use phonon_solver::moire_valley_qubit::{
    FlatBandMemoryParams, MoireValleyQubitParams, MoireValleyQubitProcessor,
    MultiNodeValleyBus, PhononMemoryCell, ValleyBlochVector, ValleyBusParams,
    ValleyQubitEngine,
};

#[test]
fn test_valley_qubit_coherence_and_gate_fidelity() {
    let params = MoireValleyQubitParams::default();
    let engine = ValleyQubitEngine::new(params);

    let metrics = engine.compute_metrics();
    assert!(
        metrics.coherence_time_t2_star_us >= 100.0,
        "Coherence time T2* must be >= 100.0 us, got {} us",
        metrics.coherence_time_t2_star_us
    );
    assert!(
        metrics.gate_fidelity >= 0.999,
        "Single-qubit gate fidelity must be >= 0.999, got {}",
        metrics.gate_fidelity
    );
    assert!(
        metrics.leakage_probability <= 1.0e-4,
        "Valley leakage probability must be <= 1.0e-4, got {}",
        metrics.leakage_probability
    );
    assert_eq!(metrics.delta_valley_chern, 2);

    let trajectory = engine.generate_rabi_trajectory(100.0, 50);
    assert_eq!(trajectory.len(), 50);
    for pt in &trajectory {
        assert!(
            (pt.prob_k + pt.prob_k_prime - 1.0).abs() < 1.0e-5,
            "Total probability must equal 1.0"
        );
        assert!(pt.valley_contrast >= -1.0 && pt.valley_contrast <= 1.0);
    }
}

#[test]
fn test_flatband_memory_cell_quenching_and_storage() {
    let params = FlatBandMemoryParams::default();
    let cell = PhononMemoryCell::new(params);

    let metrics = cell.compute_metrics();
    assert!(
        metrics.group_velocity_ratio <= 0.015,
        "Group velocity ratio must be <= 0.015, got {}",
        metrics.group_velocity_ratio
    );
    assert!(
        metrics.flatband_bandwidth_mhz <= 0.50,
        "Flatband bandwidth must be <= 0.50 MHz, got {} MHz",
        metrics.flatband_bandwidth_mhz
    );
    assert!(
        metrics.aa_confinement_ratio >= 0.90,
        "AA site confinement ratio must be >= 0.90, got {}",
        metrics.aa_confinement_ratio
    );
    assert!(
        metrics.storage_lifetime_ms >= 1.5,
        "Storage lifetime must be >= 1.5 ms, got {} ms",
        metrics.storage_lifetime_ms
    );
    assert!(
        metrics.retrieval_efficiency >= 0.92,
        "Retrieval efficiency must be >= 0.92, got {}",
        metrics.retrieval_efficiency
    );
    assert!(
        metrics.thermal_phonon_occupancy <= 1.0e-4,
        "Thermal phonon occupancy at 15 mK must be <= 1.0e-4, got {}",
        metrics.thermal_phonon_occupancy
    );
    assert!(
        metrics.memory_insertion_loss_db <= 0.30,
        "Memory insertion loss must be <= 0.30 dB, got {} dB",
        metrics.memory_insertion_loss_db
    );
}

#[test]
fn test_cryogenic_valley_bus_routing_and_entanglement() {
    let params = ValleyBusParams::default();
    let bus = MultiNodeValleyBus::new(params);

    let metrics = bus.compute_metrics();
    assert!(
        metrics.end_to_end_insertion_loss_db <= 0.35,
        "Bus end-to-end insertion loss must be <= 0.35 dB, got {} dB",
        metrics.end_to_end_insertion_loss_db
    );
    assert!(
        metrics.reverse_isolation_db >= 40.0,
        "Reverse isolation must be >= 40.0 dB, got {} dB",
        metrics.reverse_isolation_db
    );
    assert!(
        metrics.directivity_db >= 40.0,
        "Directivity must be >= 40.0 dB, got {} dB",
        metrics.directivity_db
    );
    assert!(
        metrics.crosstalk_suppression_db >= 42.0,
        "Crosstalk suppression must be >= 42.0 dB, got {} dB",
        metrics.crosstalk_suppression_db
    );
    assert!(
        metrics.total_cryo_power_mw <= 1.5,
        "Total Cryo-CMOS power must be <= 1.5 mW, got {} mW",
        metrics.total_cryo_power_mw
    );

    let ent = &metrics.entanglement;
    assert!(
        ent.concurrence >= 0.90,
        "Concurrence must be >= 0.90, got {}",
        ent.concurrence
    );
    assert!(
        ent.bell_fidelity >= 0.990,
        "Bell state fidelity must be >= 0.990, got {}",
        ent.bell_fidelity
    );
    assert!(
        ent.chsh_parameter > 2.0,
        "CHSH parameter must violate classical limit 2.0, got {}",
        ent.chsh_parameter
    );
}

#[test]
fn test_moire_valley_qubit_10_point_physics_audit() {
    let processor = MoireValleyQubitProcessor::default();
    let audit = processor.audit();

    assert!(audit.magic_angle_quenching_pass, "Check 1 failed");
    assert!(audit.aa_site_confinement_pass, "Check 2 failed");
    assert!(audit.valley_coherence_pass, "Check 3 failed");
    assert!(audit.valley_leakage_pass, "Check 4 failed");
    assert!(audit.storage_lifetime_pass, "Check 5 failed");
    assert!(audit.retrieval_efficiency_pass, "Check 6 failed");
    assert!(audit.thermal_occupancy_pass, "Check 7 failed");
    assert!(audit.valley_chiral_isolation_pass, "Check 8 failed");
    assert!(audit.bus_insertion_loss_pass, "Check 9 failed");
    assert!(audit.multi_node_concurrence_pass, "Check 10 failed");

    assert_eq!(audit.passed_count, 10);
    assert_eq!(audit.total_count, 10);
    assert!(audit.is_all_pass(), "10-point audit must pass completely");
}

#[test]
fn test_bloch_vector_and_dispersion_profiles() {
    let bv = ValleyBlochVector::from_angles(0.5 * std::f64::consts::PI, 0.0);
    assert!((bv.norm - 1.0).abs() < 1.0e-5);
    assert!((bv.tau_x - 1.0).abs() < 1.0e-5);
    assert!(bv.tau_z.abs() < 1.0e-5);

    let decayed_bv = bv.with_purity(0.95);
    assert!((decayed_bv.norm - 0.95).abs() < 1.0e-5);

    let cell = PhononMemoryCell::new(FlatBandMemoryParams::default());
    let dispersion = cell.generate_dispersion_profile(20);
    assert_eq!(dispersion.len(), 20);

    let decay = cell.generate_storage_decay(5.0, 30);
    assert_eq!(decay.len(), 30);
    assert!(decay.first().unwrap().stored_occupancy > decay.last().unwrap().stored_occupancy);

    let bus = MultiNodeValleyBus::new(ValleyBusParams::default());
    let spectrum = bus.generate_transmission_spectrum(25);
    assert_eq!(spectrum.len(), 25);
}
