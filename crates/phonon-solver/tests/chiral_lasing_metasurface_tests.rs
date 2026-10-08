#![deny(unsafe_code)]

//! Unit and physics integration tests for Phase 441:
//! Topological Non-Hermitian Floquet Acoustic Chiral Lasing Metasurface & Vortex Waveguide.

use phonon_solver::chiral_lasing_metasurface::{
    AcousticVortexParams, AcousticVortexWaveguideSolver, ChiralLasingMetasurfaceProcessor,
    FloquetChiralLatticeParams, FloquetChiralLatticeSolver, ModeCompetitionParams,
    ModeCompetitionRateSolver,
};

#[test]
fn test_floquet_chiral_lattice_gain_and_isolation() {
    let mut params = FloquetChiralLatticeParams::default();
    params.floquet_modulation_mhz = 22.0;
    params.gain_rate_gamma_mhz = 14.5;
    params.loss_rate_gamma_mhz = 18.0;

    let solver = FloquetChiralLatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Floquet topological bandgap >= 5.0 MHz
    assert!(
        metrics.floquet_bandgap_mhz >= 5.0,
        "Floquet bandgap {:.2} MHz is below 5.0 MHz threshold",
        metrics.floquet_bandgap_mhz
    );

    // 2. Net positive chiral modal gain for forward edge mode (Im(epsilon) > 0)
    assert!(
        metrics.chiral_mode_gain_mhz > 0.0,
        "Chiral mode gain {:.2} MHz must be strictly positive",
        metrics.chiral_mode_gain_mhz
    );

    // 3. Counter-propagating mode and bulk mode attenuation (Im(epsilon) < 0)
    assert!(
        metrics.counter_mode_loss_mhz < 0.0,
        "Counter mode loss {:.2} MHz must be strictly negative",
        metrics.counter_mode_loss_mhz
    );
    assert!(
        metrics.bulk_mode_loss_mhz < 0.0,
        "Bulk mode loss {:.2} MHz must be strictly negative",
        metrics.bulk_mode_loss_mhz
    );

    // 4. Non-reciprocal chiral lasing isolation >= 25.0 dB
    assert!(
        metrics.chiral_isolation_db >= 25.0,
        "Chiral isolation {:.2} dB is below 25.0 dB threshold",
        metrics.chiral_isolation_db
    );

    // 5. Lasing threshold pump power <= 15.0 mW
    assert!(
        metrics.threshold_power_mw <= 15.0,
        "Threshold power {:.2} mW exceeds 15.0 mW threshold",
        metrics.threshold_power_mw
    );

    // 6. Quasi-energy spectrum points
    let spectrum = solver.compute_quasi_energy_spectrum(32);
    assert_eq!(spectrum.len(), 32);

    // 7. Real-space metasurface nodes
    let nodes = solver.generate_metasurface_field();
    assert!(!nodes.is_empty());
}

#[test]
fn test_acoustic_vortex_waveguide_and_oam_purity() {
    for charge in [1, -1, 2] {
        let mut params = AcousticVortexParams::default();
        params.topological_charge_ell = charge;
        params.beam_waist_um = 35.0;

        let solver = AcousticVortexWaveguideSolver::new(params);
        let metrics = solver.evaluate_metrics();

        // 1. Quantized topological charge verified
        assert_eq!(metrics.measured_topological_charge, charge);

        // 2. OAM vortex beam modal purity >= 92.0%
        assert!(
            metrics.oam_modal_purity >= 0.920,
            "Modal purity {:.3} for ell = {} is below 92.0%",
            metrics.oam_modal_purity,
            charge
        );

        // 3. Beam divergence half-angle <= 4.5 degrees
        assert!(
            metrics.beam_divergence_deg <= 4.5,
            "Beam divergence {:.2} deg exceeds 4.5 deg threshold",
            metrics.beam_divergence_deg
        );

        // 4. Acoustic radiation efficiency >= 75.0%
        assert!(
            metrics.radiation_efficiency >= 0.75,
            "Radiation efficiency {:.3} is below 75.0%",
            metrics.radiation_efficiency
        );

        // 5. Donut peak ring radius > 0
        assert!(metrics.peak_ring_radius_um > 0.0);
    }

    let solver = AcousticVortexWaveguideSolver::default();
    let radial = solver.compute_radial_profile(40);
    assert_eq!(radial.len(), 40);
    assert_eq!(radial[0].normalized_intensity, 0.0); // Core null

    let far_field = solver.compute_far_field_pattern(50);
    assert_eq!(far_field.len(), 50);
}

#[test]
fn test_mode_competition_rate_equations_and_smsr() {
    let mut params = ModeCompetitionParams::default();
    params.pump_current_ma = 24.0;

    let solver = ModeCompetitionRateSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Side-mode suppression ratio SMSR >= 30.0 dB
    assert!(
        metrics.smsr_db >= 30.0,
        "SMSR {:.2} dB is below 30.0 dB threshold",
        metrics.smsr_db
    );

    // 2. Schawlow-Townes narrowed emission linewidth <= 5.0 kHz
    assert!(
        metrics.emission_linewidth_khz <= 5.0,
        "Emission linewidth {:.2} kHz exceeds 5.0 kHz threshold",
        metrics.emission_linewidth_khz
    );

    // 3. Turn-on transient delay <= 12.0 ns
    assert!(
        metrics.turn_on_delay_ns <= 12.0,
        "Turn-on delay {:.2} ns exceeds 12.0 ns threshold",
        metrics.turn_on_delay_ns
    );

    // 4. Steady-state dominant mode power > 0
    assert!(metrics.steady_state_power_mw > 0.0);

    // 5. Dynamic transient trajectory computation
    let transient = solver.compute_transient_dynamics(50);
    assert_eq!(transient.len(), 50);

    // 6. Lasing spectrum computation
    let spectrum = solver.compute_lasing_spectrum(7);
    assert_eq!(spectrum.len(), 7);
}

#[test]
fn test_chiral_lasing_metasurface_10_point_audit_all_passed() {
    let processor = ChiralLasingMetasurfaceProcessor::default();
    let audit = processor.audit_lasing_metasurface();

    assert_eq!(
        audit.total_score, 10,
        "Audit score {} / 10 does not meet full pass requirement",
        audit.total_score
    );
    assert!(audit.all_passed);
    assert!(audit.trs_breaking_pass);
    assert!(audit.chiral_modal_gain_pass);
    assert!(audit.bulk_counter_suppression_pass);
    assert!(audit.chiral_isolation_pass);
    assert!(audit.quantized_oam_charge_pass);
    assert!(audit.oam_modal_purity_pass);
    assert!(audit.threshold_power_pass);
    assert!(audit.smsr_stability_pass);
    assert!(audit.linewidth_narrowing_pass);
    assert!(audit.turn_on_latency_pass);
}
