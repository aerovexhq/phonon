#![deny(unsafe_code)]

//! Automated test suite for Phase 406: Phonon Studio Multi-Octave Acoustic Metasurface
//! Wavefront Hologram & Ultrasonic Tractor Beam Engine.

use phonon_solver::acoustic_metasurface_hologram::{
    bessel_j, AcousticMedium, AcousticMetasurfaceProcessor, AiryBeamParams, BesselAirySolver,
    BesselBeamParams, GerchbergSaxtonParams, HologramSynthesizer, HologramTargetType,
    MetasurfaceArray, MetasurfaceCellParams, MetasurfaceUnitCell, TractorBeamEngine,
    TrappedParticle,
};
use std::f64::consts::PI;

#[test]
fn test_metasurface_unit_cell_phase_span_and_impedance() {
    let cell_params = MetasurfaceCellParams::default();
    let cell = MetasurfaceUnitCell::new(cell_params);

    // 1. Verify 2*pi continuous phase modulation span (>= 1.95*pi)
    let phase_span = cell.compute_phase_span(40_000.0);
    assert!(
        phase_span >= 1.95 * PI,
        "Phase span was {:.3} pi, expected >= 1.95 pi",
        phase_span / PI
    );

    // 2. Verify acoustic impedance matching transmissivity |T| >= 0.85 across control range
    for step in 0..=20 {
        let w = (step as f64) / 20.0;
        let resp = cell.evaluate_response(w, 40_000.0);
        assert!(
            resp.transmission_amplitude >= 0.85,
            "Transmission |T| = {:.3} at w = {:.2}, expected >= 0.85",
            resp.transmission_amplitude,
            w
        );
        assert!(
            resp.normalized_impedance >= 0.80 && resp.normalized_impedance <= 1.25,
            "Normalized impedance Z/Z0 = {:.3}, expected within [0.80, 1.25]",
            resp.normalized_impedance
        );
    }

    // 3. Multi-octave broadband frequency span verification (>= 2 octaves)
    let (avg_trans, octave_ratio) = cell.evaluate_multi_octave_performance();
    assert!(
        octave_ratio >= 4.0,
        "Octave ratio was {:.2}x, expected >= 4.0x",
        octave_ratio
    );
    assert!(
        avg_trans >= 0.85,
        "Average multi-octave transmission was {:.3}, expected >= 0.85",
        avg_trans
    );
}

#[test]
fn test_gerchberg_saxton_holographic_convergence() {
    let array = MetasurfaceArray::new(16, 16, 0.00386, 40_000.0);
    let mut gs_params = GerchbergSaxtonParams::default();
    gs_params.focal_grid_res = 20;
    gs_params.max_iterations = 20;
    gs_params.target_type = HologramTargetType::QuadTrapArray { spacing_m: 0.018 };

    let c_0 = 343.0;
    let result = HologramSynthesizer::synthesize(&array, &gs_params, c_0);

    // 1. Verify PSNR >= 25.0 dB
    assert!(
        result.final_psnr_db >= 25.0,
        "Final PSNR was {:.1} dB, expected >= 25.0 dB",
        result.final_psnr_db
    );

    // 2. Verify Pearson correlation SSIM >= 0.90
    assert!(
        result.correlation_ssim >= 0.90,
        "Correlation was {:.3}, expected >= 0.90",
        result.correlation_ssim
    );

    // 3. Verify focal spot contrast >= 20.0 dB
    assert!(
        result.focal_contrast_db >= 20.0,
        "Focal contrast was {:.1} dB, expected >= 20.0 dB",
        result.focal_contrast_db
    );

    // 4. Verify iteration history recorded
    assert_eq!(result.history.len(), 20);
    let first = &result.history[0];
    let last = &result.history[19];
    assert!(
        last.psnr_db >= first.psnr_db,
        "PSNR should improve or remain high: first = {:.1} dB, last = {:.1} dB",
        first.psnr_db,
        last.psnr_db
    );
}

#[test]
fn test_3d_acoustic_field_reconstruction_and_focus() {
    let mut array = MetasurfaceArray::new(16, 16, 0.00386, 40_000.0);
    let c_0 = 343.0;
    let z_focus = 0.045; // 45 mm
    array.set_focusing_lens_phase(0.0, 0.0, z_focus, c_0);

    let tractor = TractorBeamEngine::default();

    // Check pressure at focal center vs off-axis point
    let p_center = tractor.evaluate_pressure_at(&array, [0.0, 0.0, z_focus]).norm();
    let p_off_axis = tractor.evaluate_pressure_at(&array, [0.015, 0.015, z_focus]).norm();

    assert!(
        p_center > 2.0 * p_off_axis,
        "Focal center pressure ({:.1} Pa) should be significantly higher than off-axis ({:.1} Pa)",
        p_center,
        p_off_axis
    );
}

#[test]
fn test_gorkov_acoustic_potential_and_radiation_forces() {
    let particle = TrappedParticle::default();
    let medium = AcousticMedium::Air;

    // Contrast factor checks
    let f1 = particle.monopole_contrast_f1(&medium);
    let f2 = particle.dipole_contrast_f2(&medium);

    assert!(f1 > 0.90, "f1 monopole contrast was {:.3}, expected > 0.90", f1);
    assert!(f2 > 0.95, "f2 dipole contrast was {:.3}, expected > 0.95", f2);

    let tractor = TractorBeamEngine::new(medium, particle, 2500.0);
    let array = MetasurfaceArray::new(16, 16, 0.00386, 40_000.0);

    let pt = tractor.evaluate_gorkov_point(&array, [0.0, 0.0, 0.050]);
    assert!(pt.pressure_pa > 0.0);
    assert!(pt.velocity_m_s > 0.0);
    assert_ne!(pt.gorkov_potential_j, 0.0);
}

#[test]
fn test_ultrasonic_tractor_beam_negative_axial_pulling_force() {
    let mut array = MetasurfaceArray::new(16, 16, 0.00386, 40_000.0);
    let c_0 = 343.0;
    let z_focus = 0.050; // 50 mm
    array.set_twin_trap_phase(0.0, 0.0, z_focus, c_0);

    let tractor = TractorBeamEngine::default();
    let metrics = tractor.evaluate_trap_stability(&array, [0.0, 0.0, z_focus]);

    // 1. Verify negative axial radiation force (tractor beam pulling: F_z < 0)
    assert!(
        metrics.axial_pulling_force_n < 0.0,
        "Axial force F_z was {:.3e} N, expected < 0.0 (tractor beam pulling)",
        metrics.axial_pulling_force_n
    );
    assert!(metrics.is_tractor_beam_pulling);

    // 2. Verify strictly positive 3D trap stiffnesses (k_x > 0, k_y > 0, k_z > 0)
    assert!(
        metrics.stiffness_kx_n_m > 0.0,
        "Transverse stiffness k_x was {:.3e} N/m, expected > 0",
        metrics.stiffness_kx_n_m
    );
    assert!(
        metrics.stiffness_ky_n_m > 0.0,
        "Transverse stiffness k_y was {:.3e} N/m, expected > 0",
        metrics.stiffness_ky_n_m
    );
    assert!(
        metrics.stiffness_kz_n_m > 0.0,
        "Axial stiffness k_z was {:.3e} N/m, expected > 0",
        metrics.stiffness_kz_n_m
    );
    assert!(metrics.is_3d_stable);

    // 3. Levitation safety factor > 1.0 (overcomes gravity)
    assert!(
        metrics.levitation_safety_factor >= 1.0,
        "Levitation safety factor was {:.2}x, expected >= 1.0x",
        metrics.levitation_safety_factor
    );
}

#[test]
fn test_non_diffracting_bessel_vortex_beam() {
    let mut params = BesselBeamParams::default();
    params.topological_charge = 1;

    let res = BesselAirySolver::solve_bessel_beam(&params);

    // 1. Verify propagation distance ratio >= 0.85
    assert!(
        res.propagation_distance_ratio >= 0.85,
        "Propagation ratio was {:.2}, expected >= 0.85",
        res.propagation_distance_ratio
    );

    // 2. Intensity variation along propagation path < 15%
    assert!(
        res.intensity_variation_ratio < 0.15,
        "Intensity variation was {:.1}%, expected < 15%",
        res.intensity_variation_ratio * 100.0
    );

    // 3. Quantized phase circulation around the axis = topological charge
    let err = (res.phase_circulation_charge - 1.0).abs();
    assert!(
        err < 0.10,
        "Phase circulation charge was {:.2}, expected 1.0",
        res.phase_circulation_charge
    );

    // 4. Test higher order Bessel functions J_0, J_1, J_2
    assert!((bessel_j(0, 0.0) - 1.0).abs() < 1e-6);
    assert!(bessel_j(1, 0.0).abs() < 1e-6);
    assert!(bessel_j(2, 0.0).abs() < 1e-6);
    assert!(bessel_j(0, 2.4048).abs() < 1e-3); // First root of J0
}

#[test]
fn test_acoustic_beam_self_healing_behind_obstacle() {
    let params = BesselBeamParams::default();
    let res = BesselAirySolver::solve_bessel_beam(&params);

    // Central peak recovery ratio behind on-axis obstacle >= 80%
    assert!(
        res.self_healing_recovery_ratio >= 0.80,
        "Self healing recovery was {:.1}%, expected >= 80%",
        res.self_healing_recovery_ratio * 100.0
    );

    // Airy beam trajectory
    let airy_params = AiryBeamParams::default();
    let traj = BesselAirySolver::compute_airy_trajectory(&airy_params, 343.0, 15);
    assert_eq!(traj.len(), 15);
    assert!(traj.last().unwrap().1 > traj.first().unwrap().1); // Parabolic bending
}

#[test]
fn test_ten_point_physics_audit_full_pass() {
    let processor = AcousticMetasurfaceProcessor::default();
    let report = processor.audit_metasurface();

    assert_eq!(report.total_tests, 10, "Expected 10 audit items");
    assert_eq!(
        report.pass_count, 10,
        "Expected 10/10 PASS, failed items: {:?}",
        report.items.iter().filter(|it| !it.passed).collect::<Vec<_>>()
    );
    assert!(report.all_passed, "10-point physics audit should fully pass");
}
