#![deny(unsafe_code)]

use phonon_solver::non_hermitian_braiding::{
    ExceptionalSurfaceParams, ExceptionalSurfaceSensorSolver, HolonomicCompilerParams,
    HolonomicStateCompiler, NonHermitianBraidingProcessor, NonHermitianGateKind,
    SkinBraidingParams, SkinBraidingSolver,
};

#[test]
fn test_skin_braiding_confinement_and_gbz() {
    let params = SkinBraidingParams::default();
    let solver = SkinBraidingSolver::new(params);

    let metrics = solver.compute_metrics();
    assert!(
        metrics.gbz_radius < 0.95 && metrics.gbz_radius > 0.10,
        "GBZ radius must demonstrate point-gap non-triviality, got {}",
        metrics.gbz_radius
    );
    assert!(
        metrics.bulk_point_gap_mhz >= 15.0,
        "Bulk point-gap must be >= 15.0 MHz, got {} MHz",
        metrics.bulk_point_gap_mhz
    );
    assert!(
        metrics.skin_confinement_ratio >= 0.88,
        "Skin mode spatial confinement must be >= 88.0%, got {}",
        metrics.skin_confinement_ratio
    );
    assert!(
        metrics.braid_process_fidelity >= 0.996,
        "Braid process fidelity must be >= 99.6%, got {}",
        metrics.braid_process_fidelity
    );
    assert!(
        metrics.diabatic_leakage_prob <= 1.0e-4,
        "Diabatic excitation leakage must be <= 1.0e-4, got {}",
        metrics.diabatic_leakage_prob
    );
    assert!(
        metrics.non_reciprocal_contrast_db >= 20.0,
        "Non-reciprocal contrast must be >= 20.0 dB, got {} dB",
        metrics.non_reciprocal_contrast_db
    );
}

#[test]
fn test_exceptional_surface_sensor_splitting_and_enhancement() {
    let params = ExceptionalSurfaceParams::default();
    let solver = ExceptionalSurfaceSensorSolver::new(params);

    let metrics = solver.compute_metrics();
    assert_eq!(
        metrics.coalescence_residual_mhz, 0.0,
        "Exact EP coalescence requires zero splitting at eps = 0"
    );
    assert!(
        metrics.eigenvalue_splitting_mhz > 0.0,
        "Eigenvalue splitting must be non-zero at test perturbation"
    );
    assert!(
        metrics.responsivity_enhancement >= 120.0,
        "EP responsivity enhancement must be >= 120x, got {}",
        metrics.responsivity_enhancement
    );
    assert!(
        metrics.min_detectable_perturbation <= 1.0e-10,
        "Minimum detectable perturbation must be <= 1.0e-10, got {}",
        metrics.min_detectable_perturbation
    );
    assert!(
        metrics.dynamic_range_db >= 65.0,
        "Dynamic range must be >= 65.0 dB, got {} dB",
        metrics.dynamic_range_db
    );

    let spectrum = solver.generate_splitting_spectrum(25);
    assert_eq!(spectrum.len(), 25);
    for pt in &spectrum {
        assert!(pt.eigenvalue_splitting_mhz > pt.hermitian_reference_mhz);
        assert!(pt.responsivity_enhancement >= 1.0);
    }
}

#[test]
fn test_holonomic_state_compiler_gates_and_fidelity() {
    let gates = [
        NonHermitianGateKind::Hadamard,
        NonHermitianGateKind::PhaseS,
        NonHermitianGateKind::PauliX,
        NonHermitianGateKind::PauliZ,
        NonHermitianGateKind::NonHermitianFilter,
    ];

    for gate in gates {
        let params = HolonomicCompilerParams {
            target_gate: gate,
            ..HolonomicCompilerParams::default()
        };
        let compiler = HolonomicStateCompiler::new(params);
        let metrics = compiler.compute_metrics();

        assert!(
            metrics.gate_fidelity >= 0.998,
            "Gate fidelity for {} must be >= 0.998, got {}",
            gate.name(),
            metrics.gate_fidelity
        );
        assert!(
            metrics.qnd_preservation_fidelity >= 0.997,
            "QND preservation fidelity for {} must be >= 0.997, got {}",
            gate.name(),
            metrics.qnd_preservation_fidelity
        );

        if gate == NonHermitianGateKind::NonHermitianFilter {
            assert!(
                metrics.filter_extinction_db >= 25.0,
                "Non-Hermitian filter extinction must be >= 25.0 dB, got {} dB",
                metrics.filter_extinction_db
            );
        }
    }
}

#[test]
fn test_spatial_distribution_and_braid_trajectory() {
    let params = SkinBraidingParams::default();
    let solver = SkinBraidingSolver::new(params);

    let spatial = solver.generate_spatial_distribution();
    assert_eq!(spatial.len(), 6 * 6 * 4);

    let skin_corners: Vec<_> = spatial.iter().filter(|p| p.is_skin_corner).collect();
    assert_eq!(skin_corners.len(), 1);
    assert!(skin_corners[0].intensity >= 0.20);

    let trajectory = solver.generate_braid_trajectory(30);
    assert_eq!(trajectory.len(), 30);
    assert_eq!(trajectory[0].normalized_time, 0.0);
    assert_eq!(trajectory.last().unwrap().normalized_time, 1.0);
    assert!(trajectory.last().unwrap().geometric_phase_rad > 1.50);
}

#[test]
fn test_non_hermitian_braiding_10_point_physics_audit() {
    let processor = NonHermitianBraidingProcessor::default();
    let audit = processor.audit();

    assert!(audit.skin_confinement_pass, "Check 1 failed");
    assert!(audit.gbz_radius_pass, "Check 2 failed");
    assert!(audit.chiral_braid_fidelity_pass, "Check 3 failed");
    assert!(audit.diabatic_leakage_pass, "Check 4 failed");
    assert!(audit.ep_coalescence_pass, "Check 5 failed");
    assert!(audit.square_root_splitting_pass, "Check 6 failed");
    assert!(audit.responsivity_enhancement_pass, "Check 7 failed");
    assert!(audit.min_detectable_strain_pass, "Check 8 failed");
    assert!(audit.holonomic_gate_fidelity_pass, "Check 9 failed");
    assert!(audit.metric_normalization_pass, "Check 10 failed");

    assert_eq!(audit.passed_count, 10);
    assert_eq!(audit.total_count, 10);
    assert!(audit.is_all_pass());
}
