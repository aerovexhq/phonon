#![deny(unsafe_code)]

//! Integration and verification test suite for Phase 458:
//! Fractional Hall Skyrmion Synaptic Memory & Anyonic Neural Crossbar.

use phonon_solver::fractional_skyrmion_synapse::{
    ChiralNeuromorphicParams, ChiralNeuromorphicSolver, CryogenicNeuralCrossbarParams,
    CryogenicNeuralCrossbarSolver, FractionalSkyrmionParams, FractionalSkyrmionSolver,
    FractionalSkyrmionSynapseProcessor, NonAbelianSynapseParams, NonAbelianSynapseSolver,
};

#[test]
fn test_fractional_skyrmion_topological_charge_and_hall_deflection() {
    // Test m = 3 (nu = 1/3)
    let params_3 = FractionalSkyrmionParams {
        filling_fraction_denominator: 3,
        skyrmion_radius_nm: 45.0,
        lattice_pitch_nm: 120.0,
        ..Default::default()
    };
    let solver_3 = FractionalSkyrmionSolver::new(params_3);
    let metrics_3 = solver_3.compute_metrics();

    assert!(
        (metrics_3.topological_charge_q - 1.0 / 3.0).abs() <= 0.02,
        "Expected Q approx 1/3, got {}",
        metrics_3.topological_charge_q
    );
    assert!(metrics_3.quantization_error <= 0.02);
    assert!(
        metrics_3.hall_deflection_angle_deg >= 15.0,
        "Expected Hall angle >= 15 deg, got {}",
        metrics_3.hall_deflection_angle_deg
    );
    assert!(metrics_3.skyrmion_density_um2 > 50.0);

    // Test profile generation
    let profile = solver_3.generate_radial_profile(31);
    assert_eq!(profile.len(), 31);
    let expected_theta_0 = (1.0f64 / 3.0).acos();
    assert!((profile[0].theta_rad - expected_theta_0).abs() < 1e-3);
    assert!(profile.last().unwrap().theta_rad < 1e-4);

    // Test 2D field slice
    let (xs, ys, field) = solver_3.generate_2d_field_slice();
    assert_eq!(xs.len(), field.len());
    assert_eq!(ys.len(), field[0].len());

    // Test m = 4 (nu = 1/4)
    let params_4 = FractionalSkyrmionParams {
        filling_fraction_denominator: 4,
        ..Default::default()
    };
    let solver_4 = FractionalSkyrmionSolver::new(params_4);
    let metrics_4 = solver_4.compute_metrics();
    assert!(
        (metrics_4.topological_charge_q - 0.25).abs() <= 0.02,
        "Expected Q approx 1/4, got {}",
        metrics_4.topological_charge_q
    );
}

#[test]
fn test_non_abelian_synaptic_weight_programming() {
    let params = NonAbelianSynapseParams {
        num_levels: 128,
        g_min_us: 1.0,
        g_max_us: 100.0,
        pulse_width_ns: 2.5,
        pulse_voltage_mv: 45.0,
        operating_temp_k: 0.020,
        pinning_barrier_ev: 0.85,
    };
    let solver = NonAbelianSynapseSolver::new(params);
    let metrics = solver.compute_metrics();

    assert!(metrics.num_quantized_levels >= 64);
    assert!(
        metrics.non_linearity_alpha_ltp <= 0.15,
        "LTP non-linearity {} exceeds 0.15",
        metrics.non_linearity_alpha_ltp
    );
    assert!(
        metrics.non_linearity_alpha_ltd <= 0.15,
        "LTD non-linearity {} exceeds 0.15",
        metrics.non_linearity_alpha_ltd
    );
    assert!(
        metrics.write_energy_fj <= 1.5,
        "Write energy {} fJ exceeds 1.5 fJ",
        metrics.write_energy_fj
    );
    assert!(
        metrics.retention_lifetime_us >= 100.0,
        "Retention lifetime {} us < 100 us",
        metrics.retention_lifetime_us
    );
    assert!(metrics.resolution_bits >= 6.0);

    // Verify LTP/LTD curves
    let (ltp, ltd) = solver.generate_ltp_ltd_curves(40);
    assert_eq!(ltp.len(), 41);
    assert_eq!(ltd.len(), 41);
    assert!(ltp.last().unwrap().conductance_norm >= 0.95);
    assert!(ltd.last().unwrap().conductance_norm <= 0.05);

    // Verify retention curve
    let ret = solver.generate_retention_curve(20, 200.0);
    assert_eq!(ret.len(), 20);
    assert!(ret[0].1 >= 0.99);
}

#[test]
fn test_chiral_domain_wall_neuromorphic_routing_and_lif() {
    let params = ChiralNeuromorphicParams {
        waveguide_width_um: 1.5,
        carrier_freq_ghz: 3.5,
        v_threshold_mv: 50.0,
        membrane_time_constant_ns: 15.0,
        defect_size_ratio: 0.25,
        acoustic_velocity_ms: 3400.0,
    };
    let solver = ChiralNeuromorphicSolver::new(params);
    let metrics = solver.compute_metrics();

    assert!(
        metrics.forward_insertion_loss_db <= 0.40,
        "Insertion loss {} dB > 0.40 dB",
        metrics.forward_insertion_loss_db
    );
    assert!(
        metrics.backward_isolation_db >= 38.0,
        "Isolation {} dB < 38.0 dB",
        metrics.backward_isolation_db
    );
    assert!(
        metrics.defect_transmission_ratio >= 0.950,
        "Defect transmission {} < 0.950",
        metrics.defect_transmission_ratio
    );
    assert!(metrics.spike_firing_rate_mhz > 20.0);

    // Verify S-parameter sweep
    let spectrum = solver.generate_s_parameter_sweep(31);
    assert_eq!(spectrum.len(), 31);
    let center = &spectrum[15];
    assert!(center.s21_db >= -0.40);
    assert!(center.s12_db <= -38.0);

    // Verify LIF spike trajectory
    let lif_traj = solver.generate_lif_simulation(100.0, 35.0);
    let spike_count = lif_traj.iter().filter(|p| p.spike_fired).count();
    assert!(spike_count >= 1, "Expected at least 1 LIF spike fired");
}

#[test]
fn test_cryogenic_neural_crossbar_mvm_and_classification() {
    let params = CryogenicNeuralCrossbarParams {
        rows: 8,
        cols: 8,
        operating_temp_k: 0.020,
        crosstalk_capacitance_ff: 0.15,
        read_noise_std: 0.001,
        clock_rate_mhz: 250.0,
    };
    let solver = CryogenicNeuralCrossbarSolver::new(params);
    let metrics = solver.compute_metrics();

    assert!(
        metrics.mvm_accuracy_error_percent <= 0.50,
        "MVM error {} % > 0.50 %",
        metrics.mvm_accuracy_error_percent
    );
    assert!(
        metrics.crosstalk_isolation_db >= 42.0,
        "Crosstalk isolation {} dB < 42.0 dB",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.thermal_noise_occupancy <= 0.01,
        "Thermal noise {} > 0.01",
        metrics.thermal_noise_occupancy
    );
    assert!(
        metrics.inference_accuracy_percent >= 96.0,
        "Inference accuracy {} % < 96.0 %",
        metrics.inference_accuracy_percent
    );

    // Check cells
    let cells = solver.get_weight_matrix_cells();
    assert_eq!(cells.len(), 64);

    // Test benchmark classification
    let class_results = solver.classify_test_patterns();
    assert_eq!(class_results.len(), 3);
    for (name, conf, pass) in class_results {
        assert!(pass, "Pattern {} failed classification", name);
        assert!(conf > 60.0, "Confidence too low for {}", name);
    }
}

#[test]
fn test_10_point_physics_audit() {
    let processor = FractionalSkyrmionSynapseProcessor::new(
        FractionalSkyrmionParams::default(),
        NonAbelianSynapseParams::default(),
        ChiralNeuromorphicParams::default(),
        CryogenicNeuralCrossbarParams::default(),
    );

    let audit = processor.audit_synapse_system();
    let (passed, total) = audit.score();

    assert_eq!(total, 10);
    assert_eq!(
        passed, 10,
        "Audit failed with score {}/{}: {:?}",
        passed, total, audit
    );
    assert!(audit.is_pass());
}
