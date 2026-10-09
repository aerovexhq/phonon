#![deny(unsafe_code)]

//! Integration and verification test suite for Phase 459:
//! Chiral Topological Metamaterial Photonic-Phononic Qubit Transducer & Quantum Network Repeater Node.

use phonon_solver::chiral_transducer_repeater::{
    ChiralRouterParams, ChiralRouterSolver, ChiralTransducerRepeaterProcessor,
    EntanglementRepeaterParams, EntanglementRepeaterSolver, PiezoOptomechanicalParams,
    PiezoOptomechanicalSolver,
};

#[test]
fn test_piezo_optomechanical_transducer_efficiency_and_noise() {
    let params = PiezoOptomechanicalParams {
        mech_freq_ghz: 4.5,
        mech_linewidth_khz: 120.0,
        optical_freq_thz: 193.4,
        optical_linewidth_mhz: 35.0,
        microwave_linewidth_mhz: 25.0,
        optomech_coupling_g0_khz: 850.0,
        electromech_coupling_gem_mhz: 2.8,
        optical_pump_power_mw: 1.5,
        optical_coupling_efficiency: 0.65,
        microwave_coupling_efficiency: 0.70,
        operating_temp_k: 0.020,
    };
    let solver = PiezoOptomechanicalSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.bidirectional_efficiency_percent >= 15.0,
        "Transduction efficiency {} % < 15.0 %",
        metrics.bidirectional_efficiency_percent
    );
    assert!(
        metrics.transduction_bandwidth_mhz >= 2.0,
        "Bandwidth {} MHz < 2.0 MHz",
        metrics.transduction_bandwidth_mhz
    );
    assert!(
        metrics.added_noise_quanta <= 0.10,
        "Added noise {} quanta > 0.10 quanta",
        metrics.added_noise_quanta
    );
    assert!(metrics.optomechanical_cooperativity > 0.0);
    assert!(metrics.electromechanical_cooperativity > 0.0);
    assert!(metrics.intracavity_photon_count > 0.0);

    // Verify optical pump power sweep
    let sweep = solver.sweep_pump_power(25);
    assert_eq!(sweep.len(), 25);
    assert!(sweep[0].pump_power_mw < sweep.last().unwrap().pump_power_mw);
    assert!(sweep.iter().any(|pt| pt.efficiency_percent >= 15.0));
}

#[test]
fn test_non_reciprocal_chiral_router_s_parameters_and_isolation() {
    let params = ChiralRouterParams {
        center_freq_ghz: 4.5,
        bandwidth_mhz: 180.0,
        chiral_phase_rad: std::f64::consts::FRAC_PI_2,
        corner_defect_ratio: 0.15,
    };
    let solver = ChiralRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.forward_insertion_loss_db <= 0.40,
        "Forward insertion loss {} dB > 0.40 dB",
        metrics.forward_insertion_loss_db
    );
    assert!(
        metrics.backward_isolation_db >= 38.0,
        "Backward isolation {} dB < 38.0 dB",
        metrics.backward_isolation_db
    );
    assert!(
        metrics.port_return_loss_db >= 22.0,
        "Port return loss {} dB < 22.0 dB",
        metrics.port_return_loss_db
    );
    assert!(
        metrics.corner_defect_transmission_percent >= 95.0,
        "Corner defect transmission {} % < 95.0 %",
        metrics.corner_defect_transmission_percent
    );
    assert!(metrics.cyclic_symmetry_error < 1e-3);

    // Verify 4x4 S-matrix
    let s = solver.compute_s_matrix();
    // Port 1 -> Port 2 is forward transmission
    assert!(s[1][0] > 0.90, "Expected high forward transmission, got {}", s[1][0]);
    // Port 2 -> Port 1 is backward transmission (isolated)
    assert!(s[0][1] < 0.05, "Expected strong isolation, got {}", s[0][1]);

    // Frequency spectrum sweep
    let spectrum = solver.sweep_frequency(31);
    assert_eq!(spectrum.len(), 31);
    let center_pt = &spectrum[15];
    assert!(center_pt.s21_forward_db > -1.0);
    assert!(center_pt.s12_backward_db < -35.0);
}

#[test]
fn test_entanglement_swapping_repeater_fidelity_and_gain() {
    let params = EntanglementRepeaterParams {
        total_distance_km: 80.0,
        repeater_segments: 2,
        fiber_attenuation_db_km: 0.20,
        bsm_detector_efficiency: 0.85,
        memory_coherence_time_us: 500.0,
        transduction_fidelity: 0.985,
        source_rate_khz: 50.0,
    };
    let solver = EntanglementRepeaterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.swapped_state_fidelity_percent >= 92.0,
        "Swapped Bell fidelity {} % < 92.0 %",
        metrics.swapped_state_fidelity_percent
    );
    assert!(
        metrics.swapped_concurrence >= 0.88,
        "Swapped concurrence {} < 0.88",
        metrics.swapped_concurrence
    );
    assert!(
        metrics.entanglement_rate_pairs_sec >= 1.0e3,
        "Repeater rate {} pairs/s < 1000 pairs/s",
        metrics.entanglement_rate_pairs_sec
    );
    assert!(
        metrics.repeater_rate_gain >= 2.0,
        "Repeater gain {} < 2.0x",
        metrics.repeater_rate_gain
    );
    assert!(metrics.segment_transmission_prob > metrics.direct_transmission_prob);

    // Verify density matrix
    let rho = solver.compute_density_matrix();
    let trace = rho[0][0] + rho[1][1] + rho[2][2] + rho[3][3];
    assert!((trace - 1.0).abs() < 1e-4, "Density matrix trace {} != 1.0", trace);
    assert!(rho[0][3] > 0.40, "Coherence element |Phi+> rho_03 too low");
}

#[test]
fn test_repeater_distance_sweep_scaling() {
    let solver = EntanglementRepeaterSolver::new(EntanglementRepeaterParams::default());
    let sweep = solver.sweep_distance(25);
    assert_eq!(sweep.len(), 25);

    // At long distances, the repeater rate must vastly exceed the direct fiber rate
    let last = sweep.last().unwrap();
    assert!(
        last.repeater_rate_hz > last.direct_rate_hz,
        "Repeater advantage failed at {} km: rep={}, dir={}",
        last.distance_km,
        last.repeater_rate_hz,
        last.direct_rate_hz
    );
    assert!(last.fidelity_percent >= 92.0);
}

#[test]
fn test_10_point_physics_audit() {
    let processor = ChiralTransducerRepeaterProcessor::new(
        PiezoOptomechanicalParams::default(),
        ChiralRouterParams::default(),
        EntanglementRepeaterParams::default(),
    );

    let audit = processor.audit_system();
    let (passed, total) = audit.score();

    assert_eq!(total, 10);
    assert_eq!(
        passed, 10,
        "Audit failed with score {}/{}: {:?}",
        passed, total, audit
    );
    assert!(audit.is_pass());
}
