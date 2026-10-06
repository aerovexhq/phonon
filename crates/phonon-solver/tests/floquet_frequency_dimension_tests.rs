#![deny(unsafe_code)]

//! Automated Test Suite for Phase 374:
//! Topological Acoustic Floquet-Bloch Synthetic Frequency Dimension & Frequency-Lattice Soliton Engine.

use phonon_solver::floquet_frequency_dimension::{
    BoundaryModulationParams, FloquetFrequencyEngine, FrequencySolitonParams, SolitonRegime,
    SyntheticFrequencyLattice, SyntheticLatticeKind,
};
use std::f64::consts::PI;
use std::time::Instant;

#[test]
fn test_synthetic_frequency_lattice_hermiticity_and_dimension() {
    let params = BoundaryModulationParams {
        num_spatial_sites: 4,
        num_frequency_modes: 7,
        synthetic_gauge_flux_rad: PI / 3.0,
        ..Default::default()
    };
    let lattice = SyntheticFrequencyLattice::new(params, SyntheticLatticeKind::Synthetic2DRealFrequency);

    let dim = lattice.total_dimension();
    assert_eq!(dim, 4 * 7);

    let h = lattice.build_hamiltonian();
    assert_eq!(h.len(), dim * dim);

    // Verify Hermiticity: H(i, j) == H(j, i)*
    for i in 0..dim {
        for j in 0..dim {
            let h_ij = h[i * dim + j];
            let h_ji = h[j * dim + i];
            assert!(
                (h_ij.re - h_ji.re).abs() < 1e-10,
                "Real part mismatch at ({}, {}): {} vs {}",
                i, j, h_ij.re, h_ji.re
            );
            assert!(
                (h_ij.im + h_ji.im).abs() < 1e-10,
                "Imaginary part mismatch at ({}, {}): {} vs {}",
                i, j, h_ij.im, h_ji.im
            );
        }
    }
}

#[test]
fn test_synthetic_gauge_flux_and_topological_chern_number() {
    // Topological regime: 0 < Phi < pi -> Chern number = +1
    let topo_params = BoundaryModulationParams {
        synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_2,
        modulation_depth: 0.40,
        ..Default::default()
    };
    let topo_lattice = SyntheticFrequencyLattice::new(topo_params, SyntheticLatticeKind::AnomalousFloquetLattice);
    assert_eq!(topo_lattice.compute_synthetic_chern_number(), 1);
    assert!(topo_lattice.topological_bandgap_khz() > 0.1);

    // Trivial regime: Phi = 0 -> Chern number = 0
    let trivial_params = BoundaryModulationParams {
        synthetic_gauge_flux_rad: 0.0,
        ..Default::default()
    };
    let trivial_lattice = SyntheticFrequencyLattice::new(trivial_params, SyntheticLatticeKind::AnomalousFloquetLattice);
    assert_eq!(trivial_lattice.compute_synthetic_chern_number(), 0);
    assert_eq!(trivial_lattice.topological_bandgap_khz(), 0.0);
}

#[test]
fn test_jacobi_eigensolver_diagonalization_accuracy() {
    let params = BoundaryModulationParams {
        num_spatial_sites: 3,
        num_frequency_modes: 5,
        synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_4,
        ..Default::default()
    };
    let lattice = SyntheticFrequencyLattice::new(params, SyntheticLatticeKind::Synthetic2DRealFrequency);
    let (evals, evecs) = lattice.diagonalize();

    let dim = lattice.total_dimension();
    assert_eq!(evals.len(), dim);
    assert_eq!(evecs.len(), dim);

    // Verify eigenvalues are sorted in non-decreasing order
    for i in 1..evals.len() {
        assert!(evals[i] >= evals[i - 1] - 1e-10);
    }

    // Verify normalization of complex eigenvectors
    for (idx, vec) in evecs.iter().enumerate() {
        let norm_sq: f64 = vec.iter().map(|c| c.norm_sq()).sum();
        assert!(
            (norm_sq - 1.0).abs() < 1e-8,
            "Eigenvector {} not normalized: norm_sq = {}",
            idx, norm_sq
        );
    }
}

#[test]
fn test_unidirectional_chiral_frequency_conversion() {
    let boundary_params = BoundaryModulationParams {
        num_spatial_sites: 6,
        num_frequency_modes: 11,
        synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_2,
        modulation_depth: 0.35,
        ..Default::default()
    };
    let soliton_params = FrequencySolitonParams {
        initial_mode_center: -3,
        propagation_time_steps: 40,
        time_step_dt_ms: 0.05,
        ..Default::default()
    };

    let engine = FloquetFrequencyEngine::new(
        boundary_params,
        soliton_params,
        SolitonRegime::ChiralEdgeCurrent,
    );

    let metrics = &engine.metrics;
    assert!(
        metrics.forward_conversion_efficiency_percent >= 90.0,
        "Forward conversion efficiency must be >= 90%, got {}",
        metrics.forward_conversion_efficiency_percent
    );
    assert!(
        metrics.reverse_isolation_db <= -25.0,
        "Reverse isolation must be <= -25.0 dB, got {}",
        metrics.reverse_isolation_db
    );
    assert!(
        metrics.synthetic_edge_velocity_modes_per_ms > 0.0,
        "Edge velocity must be positive"
    );
    assert_eq!(metrics.synthetic_chern_number, 1);
}

#[test]
fn test_kerr_frequency_soliton_localization_and_fidelity() {
    let boundary_params = BoundaryModulationParams {
        num_spatial_sites: 5,
        num_frequency_modes: 11,
        synthetic_gauge_flux_rad: std::f64::consts::FRAC_PI_2,
        ..Default::default()
    };
    let soliton_params = FrequencySolitonParams {
        kerr_nonlinearity: 0.08,
        initial_mode_center: -2,
        propagation_time_steps: 50,
        time_step_dt_ms: 0.04,
        ..Default::default()
    };

    let engine = FloquetFrequencyEngine::new(
        boundary_params,
        soliton_params,
        SolitonRegime::LocalizedFrequencySoliton,
    );

    let metrics = &engine.metrics;
    assert!(
        metrics.soliton_stability_fidelity_percent >= 95.0,
        "Soliton fidelity must be >= 95.0%, got {}",
        metrics.soliton_stability_fidelity_percent
    );

    let spectrum = engine.compute_conversion_spectrum();
    assert_eq!(spectrum.len(), 11);

    let flux_sweep = engine.compute_flux_sweep(12);
    assert_eq!(flux_sweep.len(), 12);
}

#[test]
fn test_fast_initialization_cold_boot_latency() {
    let start = Instant::now();
    let params = BoundaryModulationParams::default();
    let engine = FloquetFrequencyEngine::new_fast(params);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "FloquetFrequencyEngine::new_fast must execute in < 5ms, took {:?}",
        elapsed
    );
    assert_eq!(engine.metrics.synthetic_chern_number, 1);
    assert!(engine.metrics.forward_conversion_efficiency_percent > 90.0);
}
