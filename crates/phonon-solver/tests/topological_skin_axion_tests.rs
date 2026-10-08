#![deny(unsafe_code)]

//! Integration test suite for Phase 446:
//! Topological Phononic Non-Hermitian Skin-Effect Microwave Amplification & Directional Axion Transducer.

use phonon_solver::topological_skin_axion::{
    AxionPhononTransducerSolver, AxionTransducerParams, CryogenicCrossbarParams,
    CryogenicReadoutCrossbarSolver, SkinAmplifierParams, SkinMicrowaveAmplifierSolver,
    TopologicalSkinAxionProcessor,
};

#[test]
fn test_skin_microwave_amplifier_gbz_and_gain() {
    let params = SkinAmplifierParams::default();
    let solver = SkinMicrowaveAmplifierSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        (metrics.point_gap_winding_number - 1.0).abs() < 1e-4,
        "Point-gap winding number {:.2} is not quantized to 1.0",
        metrics.point_gap_winding_number
    );
    assert!(
        metrics.gbz_radius < 1.0,
        "GBZ radius {:.3} must be less than 1.0 for skin localization",
        metrics.gbz_radius
    );
    assert!(
        metrics.skin_localization_ratio >= 0.85,
        "Skin localization ratio {:.3} is below 85%",
        metrics.skin_localization_ratio
    );
    assert!(
        metrics.forward_power_gain_db >= 24.0,
        "Forward gain {:.1} dB is below 24.0 dB",
        metrics.forward_power_gain_db
    );
    assert!(
        metrics.backward_isolation_db >= 25.0,
        "Backward isolation {:.1} dB is below 25.0 dB",
        metrics.backward_isolation_db
    );
    assert!(
        metrics.added_noise_quanta <= 0.55,
        "Added noise {:.3} quanta exceeds Caves limit bound 0.55",
        metrics.added_noise_quanta
    );
    assert!(
        metrics.skin_depth_sites <= 2.0,
        "Skin depth {:.2} exceeds 2.0 sites",
        metrics.skin_depth_sites
    );

    let gbz = solver.compute_gbz_spectrum(32);
    assert_eq!(gbz.len(), 32);

    let spatial = solver.compute_spatial_skin_modes();
    assert_eq!(spatial.len(), 32);
    let last = spatial.last().unwrap();
    let first = spatial.first().unwrap();
    assert!(last.probability_density > first.probability_density);

    let bw = solver.compute_gain_bandwidth_spectrum(32);
    assert_eq!(bw.len(), 32);
}

#[test]
fn test_axion_phonon_transducer_resonance_and_sensitivity() {
    let params = AxionTransducerParams::default();
    let solver = AxionPhononTransducerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        (metrics.resonance_frequency_ghz - 3.0).abs() < 0.2,
        "Resonance frequency {:.3} GHz is not near 3.0 GHz",
        metrics.resonance_frequency_ghz
    );
    assert!(
        metrics.conversion_efficiency >= 1.0e-4,
        "Conversion efficiency {:.2e} is below 1e-4",
        metrics.conversion_efficiency
    );
    assert!(
        metrics.acoustic_quality_factor >= 1.0e5,
        "Acoustic Q_m {:.1e} is below 1e5",
        metrics.acoustic_quality_factor
    );
    assert!(
        metrics.yoctowatt_sensitivity_w_sqrt_hz <= 1.0e-21,
        "Yoctowatt sensitivity {:.2e} W/rtHz exceeds 1e-21",
        metrics.yoctowatt_sensitivity_w_sqrt_hz
    );
    assert!(
        metrics.coupling_sensitivity_gev_inv <= 1.0e-13,
        "Coupling sensitivity {:.2e} GeV^-1 exceeds 1e-13",
        metrics.coupling_sensitivity_gev_inv
    );

    let scan = solver.compute_coupling_scan(32);
    assert_eq!(scan.len(), 32);
    let peak = scan.iter().max_by(|a, b| {
        a.converted_power_yoctowatt
            .partial_cmp(&b.converted_power_yoctowatt)
            .unwrap()
    }).unwrap();
    assert!(peak.converted_power_yoctowatt >= 10.0);

    let res = solver.compute_conversion_resonance_curve(32);
    assert_eq!(res.len(), 32);
}

#[test]
fn test_cryogenic_readout_crossbar_s_parameters() {
    let params = CryogenicCrossbarParams::default();
    let solver = CryogenicReadoutCrossbarSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.directivity_db >= 30.0,
        "Directivity {:.1} dB is below 30.0 dB",
        metrics.directivity_db
    );
    assert!(
        metrics.dynamic_range_db >= 60.0,
        "Dynamic range {:.1} dB is below 60.0 dB",
        metrics.dynamic_range_db
    );
    assert!(
        metrics.p1db_compression_dbm >= -60.0,
        "P1dB {:.1} dBm is below -60.0 dBm",
        metrics.p1db_compression_dbm
    );
    assert!(
        metrics.dispersive_readout_snr_db >= 18.0,
        "Readout SNR {:.1} dB is below 18.0 dB",
        metrics.dispersive_readout_snr_db
    );
    assert!(
        metrics.readout_quantum_efficiency >= 0.88,
        "Readout quantum efficiency {:.3} is below 0.88",
        metrics.readout_quantum_efficiency
    );
    assert!(metrics.cross_port_isolation_db >= 35.0);

    let s_params = solver.compute_s_parameters(32);
    assert_eq!(s_params.len(), 32);

    let lin = solver.compute_dynamic_range_linearity(32);
    assert_eq!(lin.len(), 32);
}

#[test]
fn test_topological_skin_axion_system_10_point_audit() {
    let processor = TopologicalSkinAxionProcessor::default();
    let audit = processor.audit_system();

    assert!(audit.point_gap_winding_pass, "Point gap winding failed");
    assert!(audit.skin_localization_pass, "Skin localization failed");
    assert!(audit.forward_gain_pass, "Forward gain failed");
    assert!(audit.reverse_isolation_pass, "Reverse isolation failed");
    assert!(audit.quantum_added_noise_pass, "Quantum added noise failed");
    assert!(audit.acoustic_quality_factor_pass, "Acoustic quality factor failed");
    assert!(audit.axion_conversion_efficiency_pass, "Axion conversion efficiency failed");
    assert!(audit.yoctowatt_sensitivity_pass, "Yoctowatt sensitivity failed");
    assert!(audit.cryogenic_directivity_pass, "Cryogenic directivity failed");
    assert!(audit.dispersive_readout_snr_pass, "Dispersive readout SNR failed");

    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
