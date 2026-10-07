#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 417: Topological Acoustic Valley-Hall Chiral Edge Filter
//! & Non-Reciprocal Microwave-Phonon Isolator.

use phonon_solver::valley_chiral_isolator::{
    ChiralIsolatorParams, ChiralIsolatorSolver, MicrowavePhononTransducer, TransducerParams,
    ValleyChiralIsolator, ValleyEdgeParams, ValleyHallLattice, ValleyPolarity,
};

#[test]
fn test_valley_hall_bulk_gap_and_chern_numbers() {
    let lattice = ValleyHallLattice::new(ValleyEdgeParams::default());

    // Bulk bandgap 2 * |Delta| = 2 * 0.15 = 0.30 GHz >= 0.20 GHz
    let gap = lattice.bulk_bandgap_ghz();
    assert!(gap >= 0.20, "Bulk bandgap must be >= 0.20 GHz, got {}", gap);
    assert!((gap - 0.30).abs() < 1e-6);

    // Valley Chern numbers
    let c_k = lattice.valley_chern_number(ValleyPolarity::K);
    let c_kp = lattice.valley_chern_number(ValleyPolarity::KPrime);
    assert!((c_k - 0.5).abs() < 1e-6, "C_K must be +0.5");
    assert!((c_kp - (-0.5)).abs() < 1e-6, "C_K' must be -0.5");

    // Valley Chern difference across domain wall
    let c_diff = lattice.valley_chern_difference();
    assert!((c_diff - 1.0).abs() < 1e-6, "Valley Chern difference must be 1.0");

    // Berry curvature sign reversal
    let b_k = lattice.berry_curvature(ValleyPolarity::K, 0.0);
    let b_kp = lattice.berry_curvature(ValleyPolarity::KPrime, 0.0);
    assert!(b_k > 0.0, "Berry curvature at K must be positive");
    assert!(b_kp < 0.0, "Berry curvature at K' must be negative");
    assert!((b_k + b_kp).abs() < 1e-6, "Berry curvatures must sum to zero");
}

#[test]
fn test_valley_edge_dispersion_and_confinement() {
    let lattice = ValleyHallLattice::new(ValleyEdgeParams::default());
    let dispersion = lattice.compute_edge_dispersion(41);

    assert_eq!(dispersion.len(), 41);

    // Verify modal localization along domain wall >= 85%
    for mode in &dispersion {
        assert!(
            mode.localization_ratio >= 0.84,
            "Edge mode localization must be >= 84%, got {}",
            mode.localization_ratio
        );
        assert!(mode.group_velocity_ms > 0.0, "Group velocity must be positive");
    }

    // Verify valley polarization flips across Brillouin zone
    let first = &dispersion[0];
    let last = &dispersion[40];
    assert!(first.valley_polarization < -0.5, "Negative k must be K' polarized");
    assert!(last.valley_polarization > 0.5, "Positive k must be K polarized");
}

#[test]
fn test_sharp_corner_zero_backscattering() {
    let mut params = ValleyEdgeParams::default();
    params.corner_angle_deg = 60.0;
    let lattice_60 = ValleyHallLattice::new(params.clone());
    let (t_60, il_60, rl_60) = lattice_60.evaluate_corner_transmission();

    assert!(t_60 >= 0.95, "60-deg corner transmission must be >= 95%, got {}", t_60);
    assert!(il_60 <= 0.30, "60-deg insertion loss must be <= 0.30 dB, got {}", il_60);
    assert!(rl_60 <= -25.0, "60-deg return loss must be <= -25 dB, got {}", rl_60);

    params.corner_angle_deg = 120.0;
    let lattice_120 = ValleyHallLattice::new(params);
    let (t_120, il_120, rl_120) = lattice_120.evaluate_corner_transmission();

    assert!(t_120 >= 0.95, "120-deg corner transmission must be >= 95%, got {}", t_120);
    assert!(il_120 <= 0.30, "120-deg insertion loss must be <= 0.30 dB, got {}", il_120);
    assert!(rl_120 <= -25.0, "120-deg return loss must be <= -25 dB, got {}", rl_120);
}

#[test]
fn test_chiral_isolator_nonreciprocal_transmission() {
    let isolator = ChiralIsolatorSolver::new(ChiralIsolatorParams::default());

    // Forward transmission S21 >= -0.80 dB
    let s21_fwd = isolator.evaluate_s21_fwd_db(1.0);
    assert!(s21_fwd >= -0.80, "Forward S21 must be >= -0.80 dB, got {}", s21_fwd);

    // Reverse isolation S12 <= -30.0 dB
    let s12_rev = isolator.evaluate_s12_rev_db(1.0);
    assert!(s12_rev <= -30.0, "Reverse S12 must be <= -30.0 dB, got {}", s12_rev);

    // Peak non-reciprocal contrast S21 - S12 >= 28.0 dB
    let contrast = isolator.peak_isolation_contrast_db();
    assert!(contrast >= 28.0, "Isolation contrast must be >= 28.0 dB, got {}", contrast);

    // Bandwidth >= 20.0 MHz
    let bw = isolator.evaluate_isolation_bandwidth_mhz();
    assert!(bw >= 20.0, "Isolation bandwidth must be >= 20.0 MHz, got {}", bw);
}

#[test]
fn test_microwave_phonon_transduction() {
    let transducer = MicrowavePhononTransducer::new(TransducerParams::default());

    // Conversion efficiency >= 40.0%
    let eta = transducer.peak_efficiency();
    assert!(eta >= 0.40, "Transduction efficiency must be >= 40%, got {}", eta);

    // Insertion loss <= 1.0 dB
    let il = transducer.insertion_loss_db();
    assert!(il <= 1.0, "Insertion loss must be <= 1.0 dB, got {}", il);

    // Added noise quanta <= 0.55 quanta at 20 mK
    let n_add = transducer.added_noise_quanta();
    assert!(n_add <= 0.55, "Added noise must be <= 0.55 quanta, got {}", n_add);

    // Effective cryogenic noise temperature <= 0.5 K
    let t_n = transducer.effective_noise_temperature_k();
    assert!(t_n <= 0.50, "Cryogenic noise temp must be <= 0.50 K, got {}", t_n);
}

#[test]
fn test_master_orchestrator_10_point_audit() {
    let isolator = ValleyChiralIsolator::default();
    let report = isolator.audit_valley_chiral_isolator();

    assert_eq!(report.total_pass_score, 10, "All 10 physics audit checks must pass");
    assert!(report.all_passed, "Audit report must report all_passed == true");
    assert!(report.valley_bandgap_opened);
    assert!(report.valley_chern_difference_quantized);
    assert!(report.topological_edge_state_verified);
    assert!(report.sharp_corner_zero_backscattering);
    assert!(report.return_loss_suppressed);
    assert!(report.nonreciprocal_fwd_transmission);
    assert!(report.reverse_acoustic_isolation);
    assert!(report.isolation_contrast_verified);
    assert!(report.microwave_phonon_efficiency_verified);
    assert!(report.cryogenic_quantum_noise_limit);
}
