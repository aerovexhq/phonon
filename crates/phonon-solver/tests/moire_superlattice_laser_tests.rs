#![deny(unsafe_code)]

//! Integration and verification test suite for Phase 462:
//! Topological Moire Superlattice Flat-Band Acoustic Polariton Laser & Chiral Valley Sensor Network.

use phonon_solver::moire_superlattice_laser::{
    MoireLaserParams, MoireLaserSolver, MoireSuperlatticeLaserProcessor, MoireSuperlatticeParams,
    MoireSuperlatticeSolver, ValleySensorNetworkParams, ValleySensorNetworkSolver,
};

#[test]
fn test_moire_superlattice_magic_angle_and_flat_band() {
    let params = MoireSuperlatticeParams {
        twist_angle_deg: 1.08,
        lattice_constant_um: 100.0,
        bare_velocity_ms: 3430.0,
        tunneling_w0_mhz: 8.5,
        tunneling_w1_mhz: 11.2,
        center_freq_mhz: 15.0,
    };
    let solver = MoireSuperlatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.velocity_quenching_ratio <= 0.05,
        "Velocity quenching ratio {} > 0.05 at magic angle",
        metrics.velocity_quenching_ratio
    );
    assert!(
        metrics.flat_band_bandwidth_mhz <= 1.5,
        "Flat band bandwidth {} MHz > 1.5 MHz",
        metrics.flat_band_bandwidth_mhz
    );
    assert!(
        metrics.dos_enhancement_factor >= 20.0,
        "Acoustic DOS enhancement {} < 20.0",
        metrics.dos_enhancement_factor
    );
    assert!(
        metrics.aa_spatial_confinement_percent >= 80.0,
        "AA spatial confinement {} % < 80.0 %",
        metrics.aa_spatial_confinement_percent
    );
    assert!(metrics.moire_period_um > 2000.0);
    assert!(metrics.dirac_velocity_ms <= 3430.0 * 0.05);

    // Verify dispersion path
    let dispersion = solver.compute_dispersion(31);
    assert_eq!(dispersion.len(), 31);
    for pt in &dispersion {
        // Flat bands must be isolated from dispersive remote bands
        assert!(pt.energy_dispersive_upper_mhz > pt.energy_flat_upper_mhz);
        assert!(pt.energy_dispersive_lower_mhz < pt.energy_flat_lower_mhz);
        let flat_width = pt.energy_flat_upper_mhz - pt.energy_flat_lower_mhz;
        assert!(flat_width <= metrics.flat_band_bandwidth_mhz + 0.01);
    }

    // Verify 2D spatial acoustic profile
    let profile = solver.compute_spatial_profile(19);
    assert_eq!(profile.len(), 19 * 19);
    let aa_points: Vec<_> = profile.iter().filter(|p| p.is_aa_stacking).collect();
    assert!(!aa_points.is_empty());
    for p in &aa_points {
        assert!(p.acoustic_intensity > 0.50);
    }
}

#[test]
fn test_polariton_laser_threshold_and_coherence_transition() {
    let params_pumped = MoireLaserParams {
        pump_power_mw: 5.0, // Above threshold
        cavity_decay_rate_mhz: 1.2,
        polariton_lifetime_ps: 150.0,
        non_linear_interaction_uev: 4.5,
        spontaneous_coupling_beta: 0.08,
    };
    let solver = MoireLaserSolver::new(params_pumped.clone());
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.threshold_pump_power_mw <= 2.5,
        "Threshold pump power {} mW > 2.5 mW",
        metrics.threshold_pump_power_mw
    );
    assert!(
        (metrics.second_order_coherence_g2 - 1.0).abs() <= 0.05,
        "Coherent state g^(2)(0) {} deviates from 1.0",
        metrics.second_order_coherence_g2
    );
    assert!(
        metrics.lasing_linewidth_khz <= 15.0,
        "Linewidth {} kHz > 15.0 kHz",
        metrics.lasing_linewidth_khz
    );
    assert!(
        metrics.condensation_fraction_percent >= 78.0,
        "Condensation fraction {} % < 78.0 %",
        metrics.condensation_fraction_percent
    );
    assert!(metrics.linewidth_narrowing_factor >= 80.0);

    // Test sub-threshold thermal regime
    let params_sub = MoireLaserParams {
        pump_power_mw: 0.4, // Well below threshold
        ..params_pumped
    };
    let solver_sub = MoireLaserSolver::new(params_sub);
    let metrics_sub = solver_sub.evaluate_metrics();
    assert!(
        metrics_sub.second_order_coherence_g2 >= 1.85,
        "Sub-threshold thermal g^(2)(0) {} < 1.85",
        metrics_sub.second_order_coherence_g2
    );
    assert!(metrics_sub.condensation_fraction_percent < 50.0);

    // L-L curve generation
    let ll_curve = solver.compute_light_in_light_out_curve(25);
    assert_eq!(ll_curve.len(), 25);
    assert!(ll_curve.last().unwrap().emission_power_uw > ll_curve.first().unwrap().emission_power_uw);

    // Spectrum generation
    let spectrum = solver.compute_spectrum(35);
    assert_eq!(spectrum.len(), 35);
    let peak = spectrum.iter().map(|s| s.intensity_db).fold(f64::NEG_INFINITY, f64::max);
    assert!(peak >= -0.5);
}

#[test]
fn test_valley_sensor_network_polarization_and_sensitivity() {
    let params = ValleySensorNetworkParams {
        sensor_node_count: 6,
        valley_polarization_ratio: 0.94,
        inter_node_distance_um: 250.0,
        piezo_strain_responsivity_hz: 1.8e14,
        acoustic_damping_rate_khz: 45.0,
        applied_strain_perturbation: 1.0e-9,
    };
    let solver = ValleySensorNetworkSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.minimum_detectable_strain <= 1.0e-8,
        "Minimum detectable strain {} > 1.0e-8",
        metrics.minimum_detectable_strain
    );
    assert!(
        metrics.valley_crosstalk_isolation_db >= 35.0,
        "Valley isolation {} dB < 35.0 dB",
        metrics.valley_crosstalk_isolation_db
    );
    assert!(
        metrics.inter_node_insertion_loss_db <= 0.40,
        "Edge routing loss {} dB > 0.40 dB",
        metrics.inter_node_insertion_loss_db
    );
    assert!(metrics.differential_valley_splitting_khz > 0.0);
    assert!(metrics.sensor_network_snr_db >= 22.0);
    assert!(metrics.network_topological_robustness_percent >= 95.0);

    // Node telemetry readouts
    let nodes = solver.compute_node_readouts();
    assert_eq!(nodes.len(), 6);
    for n in &nodes {
        assert!(n.frequency_shift_khz > 0.0);
        assert!(n.measured_strain > 0.0);
    }

    // Valley transmission spectra
    let spectra = solver.compute_valley_transmission_spectra(31);
    assert_eq!(spectra.len(), 31);
    for pt in &spectra {
        assert!(pt.transmission_k_valley_db >= pt.transmission_k_prime_valley_db + 30.0);
    }
}

#[test]
fn test_moire_superlattice_laser_10_point_audit() {
    let processor = MoireSuperlatticeLaserProcessor::new(
        MoireSuperlatticeParams::default(),
        MoireLaserParams::default(),
        ValleySensorNetworkParams::default(),
    );

    let audit = processor.audit_system();
    let (passed, total) = audit.score();

    assert_eq!(total, 10, "Total audit criteria must be 10");
    assert_eq!(
        passed, 10,
        "Failed audit items. Score: ({}/{})\nAudit: {:#?}",
        passed, total, audit
    );
    assert!(audit.is_pass());

    // Explicit individual criterion assertions
    assert!(audit.magic_angle_flat_band, "Criterion 1 failed");
    assert!(audit.flat_band_bandwidth_quenching, "Criterion 2 failed");
    assert!(audit.aa_site_spatial_confinement, "Criterion 3 failed");
    assert!(audit.van_hove_dos_enhancement, "Criterion 4 failed");
    assert!(audit.polariton_laser_threshold, "Criterion 5 failed");
    assert!(audit.coherence_transition, "Criterion 6 failed");
    assert!(audit.lasing_linewidth_narrowing, "Criterion 7 failed");
    assert!(audit.valley_crosstalk_isolation, "Criterion 8 failed");
    assert!(audit.distributed_sensor_sensitivity, "Criterion 9 failed");
    assert!(audit.inter_node_insertion_loss, "Criterion 10 failed");
}

#[test]
fn test_twist_angle_detuning_and_dispersion_robustness() {
    let solver_magic = MoireSuperlatticeSolver::new(MoireSuperlatticeParams {
        twist_angle_deg: 1.08,
        ..Default::default()
    });
    let solver_detuned = MoireSuperlatticeSolver::new(MoireSuperlatticeParams {
        twist_angle_deg: 2.20,
        ..Default::default()
    });

    let m_magic = solver_magic.evaluate_metrics();
    let m_detuned = solver_detuned.evaluate_metrics();

    // Magic angle achieves far lower Dirac velocity and narrower flat band
    assert!(m_magic.velocity_quenching_ratio < m_detuned.velocity_quenching_ratio);
    assert!(m_magic.flat_band_bandwidth_mhz < m_detuned.flat_band_bandwidth_mhz);
    assert!(m_magic.dos_enhancement_factor > m_detuned.dos_enhancement_factor);
    assert!(m_magic.aa_spatial_confinement_percent > m_detuned.aa_spatial_confinement_percent);
}
