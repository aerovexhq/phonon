#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic metasurface
//! holography and chiral phonon beamforming arrays.

use phonon_models::chiral_holographic_beamforming::ChiralHolographicBeamformingParams;
use phonon_solver::chiral_holographic_beamforming::ChiralHolographicBeamformingSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = ChiralHolographicBeamformingParams::new(
        8,      // below 16
        0.05,   // below 0.20 um
        0.5,    // below 1.0 GHz
        0.1,    // below 0.5 rad/um
        0.30,   // below 0.50
        0.80,   // below 1.10
        2.0,    // below 5.0 mK
        10.0,   // below 20.0 dB
    );
    assert_eq!(underflow.metasurface_elements_count, 16);
    assert!((underflow.element_spacing_um - 0.20).abs() < 1e-9);
    assert!((underflow.operating_frequency_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.synthetic_gauge_phase_gradient_rad_per_um - 0.5).abs() < 1e-9);
    assert!((underflow.piezoelectric_coupling_efficiency - 0.50).abs() < 1e-9);
    assert!((underflow.sub_diffraction_focusing_ratio - 1.10).abs() < 1e-9);
    assert!((underflow.cryogenic_temperature_mk - 5.0).abs() < 1e-9);
    assert!((underflow.chiral_isolation_db - 20.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = ChiralHolographicBeamformingParams::new(
        256,    // above 128
        8.0,    // above 5.0 um
        15.0,   // above 10.0 GHz
        12.0,   // above 8.0 rad/um
        1.50,   // above 0.99
        5.00,   // above 3.50
        80.0,   // above 50.0 mK
        90.0,   // above 60.0 dB
    );
    assert_eq!(overflow.metasurface_elements_count, 128);
    assert!((overflow.element_spacing_um - 5.0).abs() < 1e-9);
    assert!((overflow.operating_frequency_ghz - 10.0).abs() < 1e-9);
    assert!((overflow.synthetic_gauge_phase_gradient_rad_per_um - 8.0).abs() < 1e-9);
    assert!((overflow.piezoelectric_coupling_efficiency - 0.99).abs() < 1e-9);
    assert!((overflow.sub_diffraction_focusing_ratio - 3.50).abs() < 1e-9);
    assert!((overflow.cryogenic_temperature_mk - 50.0).abs() < 1e-9);
    assert!((overflow.chiral_isolation_db - 60.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralHolographicBeamformingParams::default();
    let solver = ChiralHolographicBeamformingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.holographic_reconstruction_fidelity >= 0.9960,
        "Default holographic reconstruction fidelity must be >= 0.9960, got {:.6}",
        metrics.holographic_reconstruction_fidelity
    );
    assert!(
        metrics.acoustic_beam_directivity_db >= 32.0,
        "Default acoustic beam directivity must be >= 32.0 dB, got {:.2} dB",
        metrics.acoustic_beam_directivity_db
    );
    assert!(
        metrics.beam_steering_angular_resolution_deg <= 0.050,
        "Default beam steering angular resolution must be <= 0.050 deg, got {:.4} deg",
        metrics.beam_steering_angular_resolution_deg
    );
    assert!(
        metrics.side_lobe_suppression_ratio_db >= 28.0,
        "Default side lobe suppression ratio must be >= 28.0 dB, got {:.2} dB",
        metrics.side_lobe_suppression_ratio_db
    );
    assert!(
        metrics.acoustic_mode_insertion_loss_db <= 1.20,
        "Default acoustic mode insertion loss must be <= 1.20 dB, got {:.2} dB",
        metrics.acoustic_mode_insertion_loss_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_metasurface_elements_count_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let high_elements = ChiralHolographicBeamformingParams::new(
        64, // increased from 48
        base.element_spacing_um,
        base.operating_frequency_ghz,
        base.synthetic_gauge_phase_gradient_rad_per_um,
        base.piezoelectric_coupling_efficiency,
        base.sub_diffraction_focusing_ratio,
        base.cryogenic_temperature_mk,
        base.chiral_isolation_db,
    );
    let solver_high = ChiralHolographicBeamformingSolver::new(high_elements);

    let base_dir = solver_base.compute_acoustic_beam_directivity_db();
    let high_dir = solver_high.compute_acoustic_beam_directivity_db();
    assert!(
        high_dir > base_dir,
        "Higher element count must increase directivity ({} -> {})",
        base_dir,
        high_dir
    );

    let base_res = solver_base.compute_beam_steering_angular_resolution_deg();
    let high_res = solver_high.compute_beam_steering_angular_resolution_deg();
    assert!(
        high_res < base_res,
        "Higher element count must sharpen angular resolution ({} -> {})",
        base_res,
        high_res
    );

    let base_fidelity = solver_base.compute_holographic_reconstruction_fidelity();
    let high_fidelity = solver_high.compute_holographic_reconstruction_fidelity();
    assert!(
        high_fidelity > base_fidelity,
        "Higher element count must increase reconstruction fidelity ({} -> {})",
        base_fidelity,
        high_fidelity
    );

    let base_slsr = solver_base.compute_side_lobe_suppression_ratio_db();
    let high_slsr = solver_high.compute_side_lobe_suppression_ratio_db();
    assert!(
        high_slsr > base_slsr,
        "Higher element count must enhance side-lobe suppression ({} -> {})",
        base_slsr,
        high_slsr
    );
}

#[test]
fn test_element_spacing_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let wider_spacing = ChiralHolographicBeamformingParams::new(
        base.metasurface_elements_count,
        1.20, // increased from 0.85 um
        base.operating_frequency_ghz,
        base.synthetic_gauge_phase_gradient_rad_per_um,
        base.piezoelectric_coupling_efficiency,
        base.sub_diffraction_focusing_ratio,
        base.cryogenic_temperature_mk,
        base.chiral_isolation_db,
    );
    let solver_wider = ChiralHolographicBeamformingSolver::new(wider_spacing);

    let base_dir = solver_base.compute_acoustic_beam_directivity_db();
    let wider_dir = solver_wider.compute_acoustic_beam_directivity_db();
    assert!(
        wider_dir > base_dir,
        "Wider aperture must increase directivity ({} -> {})",
        base_dir,
        wider_dir
    );

    let base_res = solver_base.compute_beam_steering_angular_resolution_deg();
    let wider_res = solver_wider.compute_beam_steering_angular_resolution_deg();
    assert!(
        wider_res < base_res,
        "Wider aperture must sharpen angular resolution ({} -> {})",
        base_res,
        wider_res
    );
}

#[test]
fn test_operating_frequency_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let higher_freq = ChiralHolographicBeamformingParams::new(
        base.metasurface_elements_count,
        base.element_spacing_um,
        5.5, // increased from 3.8 GHz
        base.synthetic_gauge_phase_gradient_rad_per_um,
        base.piezoelectric_coupling_efficiency,
        base.sub_diffraction_focusing_ratio,
        base.cryogenic_temperature_mk,
        base.chiral_isolation_db,
    );
    let solver_higher = ChiralHolographicBeamformingSolver::new(higher_freq);

    let base_dir = solver_base.compute_acoustic_beam_directivity_db();
    let higher_dir = solver_higher.compute_acoustic_beam_directivity_db();
    assert!(
        higher_dir > base_dir,
        "Higher frequency must increase directivity ({} -> {})",
        base_dir,
        higher_dir
    );

    let base_loss = solver_base.compute_acoustic_mode_insertion_loss_db();
    let higher_loss = solver_higher.compute_acoustic_mode_insertion_loss_db();
    assert!(
        higher_loss > base_loss,
        "Higher frequency must increase propagation loss slightly ({} -> {})",
        base_loss,
        higher_loss
    );
}

#[test]
fn test_piezoelectric_coupling_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let higher_piezo = ChiralHolographicBeamformingParams::new(
        base.metasurface_elements_count,
        base.element_spacing_um,
        base.operating_frequency_ghz,
        base.synthetic_gauge_phase_gradient_rad_per_um,
        0.96, // increased from 0.91
        base.sub_diffraction_focusing_ratio,
        base.cryogenic_temperature_mk,
        base.chiral_isolation_db,
    );
    let solver_higher = ChiralHolographicBeamformingSolver::new(higher_piezo);

    let base_loss = solver_base.compute_acoustic_mode_insertion_loss_db();
    let higher_loss = solver_higher.compute_acoustic_mode_insertion_loss_db();
    assert!(
        higher_loss < base_loss,
        "Higher coupling efficiency must reduce insertion loss ({} -> {})",
        base_loss,
        higher_loss
    );

    let base_fidelity = solver_base.compute_holographic_reconstruction_fidelity();
    let higher_fidelity = solver_higher.compute_holographic_reconstruction_fidelity();
    assert!(
        higher_fidelity > base_fidelity,
        "Higher coupling efficiency must improve fidelity ({} -> {})",
        base_fidelity,
        higher_fidelity
    );
}

#[test]
fn test_sub_diffraction_focusing_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let higher_focus = ChiralHolographicBeamformingParams::new(
        base.metasurface_elements_count,
        base.element_spacing_um,
        base.operating_frequency_ghz,
        base.synthetic_gauge_phase_gradient_rad_per_um,
        base.piezoelectric_coupling_efficiency,
        2.80, // increased from 2.10
        base.cryogenic_temperature_mk,
        base.chiral_isolation_db,
    );
    let solver_higher = ChiralHolographicBeamformingSolver::new(higher_focus);

    let base_dir = solver_base.compute_acoustic_beam_directivity_db();
    let higher_dir = solver_higher.compute_acoustic_beam_directivity_db();
    assert!(
        higher_dir > base_dir,
        "Higher sub-diffraction ratio must increase directivity ({} -> {})",
        base_dir,
        higher_dir
    );

    let base_res = solver_base.compute_beam_steering_angular_resolution_deg();
    let higher_res = solver_higher.compute_beam_steering_angular_resolution_deg();
    assert!(
        higher_res < base_res,
        "Higher sub-diffraction ratio must sharpen angular resolution ({} -> {})",
        base_res,
        higher_res
    );
}

#[test]
fn test_chiral_isolation_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let higher_chiral = ChiralHolographicBeamformingParams::new(
        base.metasurface_elements_count,
        base.element_spacing_um,
        base.operating_frequency_ghz,
        base.synthetic_gauge_phase_gradient_rad_per_um,
        base.piezoelectric_coupling_efficiency,
        base.sub_diffraction_focusing_ratio,
        base.cryogenic_temperature_mk,
        48.0, // increased from 36.0 dB
    );
    let solver_higher = ChiralHolographicBeamformingSolver::new(higher_chiral);

    let base_slsr = solver_base.compute_side_lobe_suppression_ratio_db();
    let higher_slsr = solver_higher.compute_side_lobe_suppression_ratio_db();
    assert!(
        higher_slsr > base_slsr,
        "Higher chiral isolation must increase side-lobe suppression ratio ({} -> {})",
        base_slsr,
        higher_slsr
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = ChiralHolographicBeamformingParams::default();
    let solver_base = ChiralHolographicBeamformingSolver::new(base);

    let warmer = ChiralHolographicBeamformingParams::new(
        base.metasurface_elements_count,
        base.element_spacing_um,
        base.operating_frequency_ghz,
        base.synthetic_gauge_phase_gradient_rad_per_um,
        base.piezoelectric_coupling_efficiency,
        base.sub_diffraction_focusing_ratio,
        35.0, // increased from 20.0 mK
        base.chiral_isolation_db,
    );
    let solver_warmer = ChiralHolographicBeamformingSolver::new(warmer);

    let base_fidelity = solver_base.compute_holographic_reconstruction_fidelity();
    let warmer_fidelity = solver_warmer.compute_holographic_reconstruction_fidelity();
    assert!(
        warmer_fidelity < base_fidelity,
        "Higher cryogenic temperature must degrade reconstruction fidelity ({} -> {})",
        base_fidelity,
        warmer_fidelity
    );

    let base_loss = solver_base.compute_acoustic_mode_insertion_loss_db();
    let warmer_loss = solver_warmer.compute_acoustic_mode_insertion_loss_db();
    assert!(
        warmer_loss > base_loss,
        "Higher cryogenic temperature must increase acoustic insertion loss ({} -> {})",
        base_loss,
        warmer_loss
    );
}
