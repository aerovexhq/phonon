#![deny(unsafe_code)]

//! Unit and Physics Test Suite for Phase 421:
//! Phonon Studio Topological Acoustic Floquet Higher-Order Corner-State Laser
//! & Non-Hermitian Vortex Amplifier.

use phonon_solver::floquet_corner_laser::{
    CornerLaserParams, CornerLaserSolver, FloquetCornerLaserParams, FloquetCornerLaserProcessor,
    VortexAmplifierParams, VortexAmplifierSolver,
};

#[test]
fn test_higher_order_topology_and_bandgap() {
    let params = CornerLaserParams::default();
    let solver = CornerLaserSolver::new(params);

    assert!(solver.is_topological());
    assert_eq!(solver.bulk_bandgap_mhz(), 12.0); // 2 * (8.0 - 2.0) = 12.0 MHz

    // Trivial regime test
    let trivial_params = CornerLaserParams {
        intracell_gamma_mhz: 9.0,
        intercell_lambda_mhz: 3.0,
        ..Default::default()
    };
    let trivial_solver = CornerLaserSolver::new(trivial_params);
    assert!(!trivial_solver.is_topological());
}

#[test]
fn test_corner_state_localization_and_modes() {
    let solver = CornerLaserSolver::new(CornerLaserParams::default());
    let modes = solver.solve_corner_modes();

    assert_eq!(modes.len(), 4);
    for mode in &modes {
        assert!(
            mode.confinement_ratio >= 0.85,
            "Corner mode confinement {:?} must be >= 85%",
            mode.confinement_ratio
        );
        assert!(mode.frequency_ghz >= 1.199 && mode.frequency_ghz <= 1.201);
    }

    let intensity = solver.generate_realspace_intensity();
    assert_eq!(intensity.len(), 12);
    assert_eq!(intensity[0].len(), 12);
    // Corner sites (0,0) and (11, 11) must have peak energy
    assert!(intensity[0][0] > 0.5);
    assert!(intensity[11][11] > 0.5);
    // Center site (6,6) must have lower energy
    assert!(intensity[6][6] < intensity[0][0]);
}

#[test]
fn test_lasing_threshold_and_coherent_power() {
    let solver = CornerLaserSolver::new(CornerLaserParams::default());
    let p_th = solver.threshold_pump_power_mw();

    assert!(p_th > 0.0 && p_th <= 15.0);

    // Below threshold: low spontaneous emission
    let p_sub = solver.calculate_output_power_mw(p_th * 0.5);
    assert!(p_sub < 0.05);

    // Above threshold: linear growth into coherent acoustic output
    let p_super = solver.calculate_output_power_mw(25.0);
    assert!(p_super >= 2.0);
}

#[test]
fn test_side_mode_suppression_and_linewidth() {
    let solver = CornerLaserSolver::new(CornerLaserParams::default());
    let smsr = solver.calculate_smsr_db();

    assert!(
        smsr >= 35.0,
        "Side-mode suppression ratio {:?} dB must be >= 35.0 dB",
        smsr
    );

    let lw = solver.calculate_laser_linewidth_khz();
    assert!(
        lw <= 50.0,
        "Linewidth {:?} kHz must undergo Schawlow-Townes narrowing <= 50.0 kHz",
        lw
    );

    let spectrum = solver.generate_emission_spectrum(51);
    assert_eq!(spectrum.len(), 51);
    // Peak must be near center (detuning 0.0)
    let center_pt = &spectrum[25];
    assert!(center_pt.detuning_mhz.abs() < 1.0);
    let edge_pt = &spectrum[0];
    assert!(center_pt.intensity_db > edge_pt.intensity_db + 30.0);
}

#[test]
fn test_vortex_amplifier_gain_and_non_reciprocal_isolation() {
    let solver = VortexAmplifierSolver::new(VortexAmplifierParams::default());
    let f_gain = solver.forward_power_gain_db();
    let r_gain = solver.reverse_power_gain_db();
    let isolation = solver.isolation_contrast_db();

    assert!(
        f_gain >= 22.0,
        "Forward vortex power gain {:?} dB must be >= 22.0 dB",
        f_gain
    );
    assert!(r_gain <= 0.0, "Reverse transmission must be attenuated");
    assert!(
        isolation >= 25.0,
        "Directional isolation contrast {:?} dB must be >= 25.0 dB",
        isolation
    );

    let purity = solver.calculate_oam_purity_percent();
    assert!(
        purity >= 90.0,
        "Vortex OAM purity {:?} % must be >= 90.0%",
        purity
    );

    let (intensity, phase) = solver.generate_spatial_vortex_slice(32);
    assert_eq!(intensity.len(), 32);
    assert_eq!(phase.len(), 32);
    // Donut core at center must be zero for l = +1
    let center = 15;
    assert!(intensity[center][center] < 0.15);
}

#[test]
fn test_master_processor_10_point_physics_audit() {
    let processor = FloquetCornerLaserProcessor::new(FloquetCornerLaserParams::default());
    let report = processor.audit_laser_amplifier();

    assert_eq!(
        report.total_pass_score, 10,
        "10-Point physics audit must achieve 10/10 PASS score"
    );
    assert!(report.all_passed);
    assert!(report.higher_order_topology_pass);
    assert!(report.corner_confinement_pass);
    assert!(report.lasing_threshold_pass);
    assert!(report.coherent_output_power_pass);
    assert!(report.side_mode_suppression_pass);
    assert!(report.linewidth_narrowing_pass);
    assert!(report.vortex_forward_gain_pass);
    assert!(report.non_reciprocal_isolation_pass);
    assert!(report.oam_modal_purity_pass);
    assert!(report.floquet_stability_pass);
}
