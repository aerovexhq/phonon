#![deny(unsafe_code)]

use phonon_solver::quadrupole_shg::{
    QuadrupoleShgParams, QuadrupoleShgSolver, ShgEmissionEngine, ShgEmissionParams,
};

#[test]
fn test_bbh_topological_phase_and_quadrupole_moment() {
    // Topological SOTI phase: gamma < lambda
    let topo_params = QuadrupoleShgParams {
        intracell_gamma_khz: 2.0,
        intercell_lambda_khz: 10.0,
        ..Default::default()
    };
    assert!(topo_params.is_topological_soti());
    assert!((topo_params.bulk_quadrupole_moment() - 0.5).abs() < 1e-6);
    assert!((topo_params.bulk_bandgap_khz() - 16.0).abs() < 1e-6);
    assert!(topo_params.corner_decay_length() < 1.0);

    // Trivial phase: gamma > lambda
    let trivial_params = QuadrupoleShgParams {
        intracell_gamma_khz: 12.0,
        intercell_lambda_khz: 10.0,
        ..Default::default()
    };
    assert!(!trivial_params.is_topological_soti());
    assert!((trivial_params.bulk_quadrupole_moment() - 0.0).abs() < 1e-6);
    assert!((trivial_params.bulk_bandgap_khz() - 4.0).abs() < 1e-6);
}

#[test]
fn test_corner_modal_confinement() {
    let topo_params = QuadrupoleShgParams {
        intracell_gamma_khz: 2.0,
        intercell_lambda_khz: 10.0,
        nx: 4,
        ny: 4,
        disorder_w: 0.0,
        ..Default::default()
    };
    let topo_solver = QuadrupoleShgSolver::new(topo_params);
    assert!(
        topo_solver.metrics.fundamental_corner_confinement >= 0.80,
        "Fundamental corner confinement should be >= 80%, got {:.3}",
        topo_solver.metrics.fundamental_corner_confinement
    );
    assert!(
        topo_solver.metrics.shg_corner_confinement >= 0.80,
        "SHG corner confinement should be >= 80%, got {:.3}",
        topo_solver.metrics.shg_corner_confinement
    );

    // Trivial phase
    let trivial_params = QuadrupoleShgParams {
        intracell_gamma_khz: 12.0,
        intercell_lambda_khz: 10.0,
        nx: 4,
        ny: 4,
        disorder_w: 0.0,
        ..Default::default()
    };
    let trivial_solver = QuadrupoleShgSolver::new(trivial_params);
    assert!(
        trivial_solver.metrics.fundamental_corner_confinement < 0.35,
        "Trivial phase should have low corner confinement, got {:.3}",
        trivial_solver.metrics.fundamental_corner_confinement
    );
}

#[test]
fn test_nonlinear_overlap_and_conversion_efficiency() {
    let topo_params = ShgEmissionParams {
        lattice: QuadrupoleShgParams {
            intracell_gamma_khz: 2.0,
            intercell_lambda_khz: 10.0,
            fundamental_freq_hz: 2500.0,
            non_linear_chi2: 0.08,
            quality_factor_q1: 1200.0,
            quality_factor_q2: 1800.0,
            ..Default::default()
        },
        pump_power_w: 0.5,
    };
    let topo_engine = ShgEmissionEngine::new(topo_params);
    assert!(topo_engine.lattice_solver.metrics.non_linear_overlap_integral > 0.0);
    assert!(
        topo_engine.metrics.conversion_efficiency_percent >= 15.0,
        "Conversion efficiency should be >= 15%, got {:.2}%",
        topo_engine.metrics.conversion_efficiency_percent
    );
    assert!(
        (topo_engine.lattice_solver.metrics.second_harmonic_freq_hz - 5000.0).abs() < 1e-3,
        "Second harmonic frequency must be exactly 2 * f1"
    );

    // In trivial phase, conversion efficiency is < 0.1%
    let trivial_params = ShgEmissionParams {
        lattice: QuadrupoleShgParams {
            intracell_gamma_khz: 12.0,
            intercell_lambda_khz: 10.0,
            ..Default::default()
        },
        pump_power_w: 0.5,
    };
    let trivial_engine = ShgEmissionEngine::new(trivial_params);
    assert!(
        trivial_engine.metrics.conversion_efficiency_percent < 0.1,
        "Trivial phase efficiency should be < 0.1%, got {:.4}%",
        trivial_engine.metrics.conversion_efficiency_percent
    );
}

#[test]
fn test_pump_power_saturation_curve() {
    let params = ShgEmissionParams {
        pump_power_w: 0.8,
        ..Default::default()
    };
    let engine = ShgEmissionEngine::new(params);
    let curve = engine.efficiency_curve(20, 0.05, 2.0);
    assert_eq!(curve.len(), 20);

    // Initial power should have non-zero efficiency
    assert!(curve[0].1 > 0.0);

    // Check spectrum lines
    assert_eq!(engine.metrics.spectrum_lines.len(), 2);
    assert_eq!(engine.metrics.spectrum_lines[0].harmonic_order, 1);
    assert_eq!(engine.metrics.spectrum_lines[1].harmonic_order, 2);
    assert!(engine.metrics.spectrum_lines[0].power_w > 0.0);
    assert!(engine.metrics.spectrum_lines[1].power_w > 0.0);
    assert!(engine.metrics.second_harmonic_power_w > 0.0);
}

#[test]
fn test_topological_disorder_immunity() {
    let clean_params = ShgEmissionParams {
        lattice: QuadrupoleShgParams {
            disorder_w: 0.0,
            ..Default::default()
        },
        pump_power_w: 0.5,
    };
    let clean_engine = ShgEmissionEngine::new(clean_params);

    let disordered_params = ShgEmissionParams {
        lattice: QuadrupoleShgParams {
            disorder_w: 0.2,
            ..Default::default()
        },
        pump_power_w: 0.5,
    };
    let disordered_engine = ShgEmissionEngine::new(disordered_params);

    assert!(
        disordered_engine.metrics.disorder_immunity_ratio >= 0.90,
        "Disorder immunity ratio should be >= 90%, got {:.3}",
        disordered_engine.metrics.disorder_immunity_ratio
    );
    assert!(
        disordered_engine.metrics.conversion_efficiency_percent >= 12.0,
        "Efficiency with disorder should remain high, got {:.2}%",
        disordered_engine.metrics.conversion_efficiency_percent
    );
    assert!(
        disordered_engine.metrics.conversion_efficiency_percent
            >= 0.85 * clean_engine.metrics.conversion_efficiency_percent
    );
}
