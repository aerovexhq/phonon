#![deny(unsafe_code)]

//! Unit and integration test suite for Phase 448:
//! Moiré Exciton-Polariton Valley Hall Chiral Lasing Metasurface & Opto-Acoustic Synthesizer.

use phonon_solver::moire_polariton_laser::*;

#[test]
fn test_moire_polariton_lattice_and_berry_curvature() {
    let params = MoirePolaritonParams::default();
    let solver = MoirePolaritonLatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Rabi splitting must exceed 15.0 meV
    assert!(
        metrics.rabi_splitting_mev >= 15.0,
        "Rabi splitting {:.2} meV must be >= 15.0 meV",
        metrics.rabi_splitting_mev
    );

    // 2. Moiré period should be physically reasonable for 1.8 deg twist (~10 nm)
    assert!(
        metrics.moire_period_nm >= 8.0 && metrics.moire_period_nm <= 25.0,
        "Moiré period {:.2} nm outside expected range [8, 25] nm",
        metrics.moire_period_nm
    );

    // 3. Valley Chern number quantization
    assert_eq!(metrics.valley_chern_number, 1);

    // 4. Bulk valley-polariton gap
    assert!(
        metrics.bulk_valley_polariton_gap_mev >= 8.0,
        "Bulk gap {:.2} meV must be >= 8.0 meV",
        metrics.bulk_valley_polariton_gap_mev
    );

    // 5. Hopfield partition sum |X|^2 + |C|^2 approx 1.0
    let hopfield_sum = metrics.hopfield_exciton_fraction + metrics.hopfield_photon_fraction;
    assert!(
        (hopfield_sum - 1.0).abs() < 1.0e-5,
        "Hopfield sum {:.6} must equal 1.0",
        hopfield_sum
    );

    // 6. Dispersion curve computation
    let dispersion = solver.compute_dispersion(32);
    assert_eq!(dispersion.len(), 32);
    assert!(dispersion[0].upper_polariton_mev > dispersion[0].lower_polariton_mev);

    // 7. Berry curvature profile
    let berry = solver.compute_berry_curvature_profile(32);
    assert_eq!(berry.len(), 32);
}

#[test]
fn test_valley_hall_edge_waveguide_and_chiral_isolation() {
    let params = ValleyHallEdgeParams::default();
    let solver = ValleyHallEdgeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Chiral group velocity
    assert!(
        metrics.chiral_group_velocity_ms >= 1.0e5,
        "Edge mode velocity {:.2e} m/s must be >= 1.0e5 m/s",
        metrics.chiral_group_velocity_ms
    );

    // 2. Sharp-bend transmission efficiency
    assert!(
        metrics.sharp_bend_transmission_ratio >= 0.94,
        "Bend transmission {:.4} must be >= 0.94 (loss < 0.3 dB)",
        metrics.sharp_bend_transmission_ratio
    );

    // 3. Chiral valley isolation
    assert!(
        metrics.chiral_valley_isolation_db >= 25.0,
        "Chiral valley isolation {:.1} dB must be >= 25.0 dB",
        metrics.chiral_valley_isolation_db
    );

    // 4. Intervalley backscattering probability
    assert!(
        metrics.intervalley_backscattering_probability < 0.05,
        "Backscattering prob {:.4} must be < 0.05",
        metrics.intervalley_backscattering_probability
    );

    // 5. Edge dispersion computation
    let edge_disp = solver.compute_edge_dispersion(32);
    assert_eq!(edge_disp.len(), 32);

    // 6. Transmission spectrum computation
    let spectrum = solver.compute_transmission_spectrum(32);
    assert_eq!(spectrum.len(), 32);
}

#[test]
fn test_chiral_lasing_and_linewidth_narrowing() {
    let params = ChiralLasingParams::default();
    let solver = ChiralLasingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Condensation threshold
    assert!(
        metrics.is_above_threshold,
        "Default pump power must exceed condensation threshold"
    );

    // 2. Linewidth narrowing across threshold
    assert!(
        metrics.emission_linewidth_khz <= 50.0,
        "Laser linewidth {:.2} kHz must be <= 50.0 kHz",
        metrics.emission_linewidth_khz
    );

    // 3. Extended temporal coherence time
    assert!(
        metrics.temporal_coherence_time_ps >= 120.0,
        "Temporal coherence time {:.1} ps must be >= 120.0 ps",
        metrics.temporal_coherence_time_ps
    );

    // 4. Directional front-to-back emission ratio
    assert!(
        metrics.chiral_front_to_back_ratio_db >= 25.0,
        "Front-to-back ratio {:.1} dB must be >= 25.0 dB",
        metrics.chiral_front_to_back_ratio_db
    );

    // 5. Non-linear blueshift
    assert!(metrics.polariton_blueshift_mev > 0.0);

    // 6. Input-output S-curve
    let io_curve = solver.compute_input_output_curve(32);
    assert_eq!(io_curve.len(), 32);
    assert!(io_curve.last().unwrap().emission_intensity_arb > io_curve[0].emission_intensity_arb);

    // 7. Temporal coherence decay
    let g1_curve = solver.compute_temporal_coherence(32);
    assert_eq!(g1_curve.len(), 32);
    assert!(g1_curve[0].first_order_coherence_g1 > g1_curve.last().unwrap().first_order_coherence_g1);
}

#[test]
fn test_opto_acoustic_synthesizer_and_audit() {
    let synth_params = OptoAcousticSynthesizerParams::default();
    let synth_solver = OptoAcousticSynthesizerSolver::new(synth_params.clone());
    let synth_metrics = synth_solver.evaluate_metrics();

    // 1. Modulation depth beta
    assert!(
        synth_metrics.modulation_index_beta >= 0.80,
        "Modulation index beta {:.2} rad must be >= 0.80 rad",
        synth_metrics.modulation_index_beta
    );

    // 2. Microwave-to-optical conversion efficiency
    assert!(
        synth_metrics.microwave_to_optical_efficiency >= 0.15,
        "Transduction efficiency {:.4} must be >= 0.15 (15.0%)",
        synth_metrics.microwave_to_optical_efficiency
    );

    // 3. Phase noise at 10 kHz offset
    assert!(
        synth_metrics.phase_noise_at_10khz_dbc_hz <= -100.0,
        "Phase noise {:.1} dBc/Hz must be <= -100.0 dBc/Hz",
        synth_metrics.phase_noise_at_10khz_dbc_hz
    );

    // 4. Frequency comb lines
    let comb = synth_solver.compute_frequency_comb_spectrum();
    assert_eq!(comb.len(), 7); // -3 to +3
    assert_eq!(comb[3].order, 0); // carrier

    // 5. Full 10-point physics audit checklist
    let processor = MoirePolaritonLaserProcessor::default();
    let audit = processor.audit_processor();

    assert!(audit.moire_rabi_splitting_pass);
    assert!(audit.valley_berry_quantization_pass);
    assert!(audit.bulk_valley_gap_pass);
    assert!(audit.valley_edge_velocity_pass);
    assert!(audit.sharp_bend_immunity_pass);
    assert!(audit.chiral_valley_isolation_pass);
    assert!(audit.polariton_lasing_threshold_pass);
    assert!(audit.chiral_lasing_directionality_pass);
    assert!(audit.opto_acoustic_modulation_pass);
    assert!(audit.microwave_to_optical_transduction_pass);

    assert_eq!(
        audit.total_score, 10,
        "Audit score must be 10/10 PASS, got {}",
        audit.total_score
    );
    assert!(audit.all_passed);
}
