#![deny(unsafe_code)]

//! Automated Verification Test Suite for Phase 397:
//! Topological Acoustic Quadrupole Parametric Waveguide & Second-Harmonic Generation Engine.

use std::f64::consts::PI;
use phonon_solver::quadrupole_parametric::{
    ParametricDriveParams, ParametricEdgeAmplifier, QuadrupoleWaveguide,
    QuadrupoleWaveguideParams, ShgParams, ShgSolver,
};

#[test]
fn test_bbh_bulk_bandgap_and_band_structure() {
    let params = QuadrupoleWaveguideParams {
        gamma_mhz: 2.0,
        lambda_mhz: 8.0,
        omega_0_ghz: 1.0,
        length_mm: 50.0,
        nx: 6,
        ny: 6,
    };
    let wg = QuadrupoleWaveguide::new(params.clone());

    // 1. Bulk bandgap Delta_bulk = 2 * |lambda - gamma|
    let expected_gap = 2.0 * (8.0 - 2.0);
    assert!(
        (wg.bulk_gap_mhz - expected_gap).abs() < 1e-12,
        "Bulk bandgap must match 2 * |lambda - gamma| = {} MHz, got {}",
        expected_gap,
        wg.bulk_gap_mhz
    );

    // 2. Chiral symmetry: eigenvalues must be symmetric (+/- E)
    let ev_gamma = wg.bulk_eigenvalues(0.0, 0.0);
    assert!((ev_gamma[0] + ev_gamma[3]).abs() < 1e-12);
    assert!((ev_gamma[1] + ev_gamma[2]).abs() < 1e-12);

    let ev_m = wg.bulk_eigenvalues(PI, PI);
    assert!((ev_m[0] + ev_m[3]).abs() < 1e-12);
    assert!((ev_m[1] + ev_m[2]).abs() < 1e-12);

    // 3. Hamiltonian matrix Hermiticity: H_ij^* = H_ji
    let h_mat = wg.hamiltonian_matrix(0.4 * PI, 0.7 * PI);
    for r in 0..4 {
        for c in 0..4 {
            let (re_rc, im_rc) = h_mat[r][c];
            let (re_cr, im_cr) = h_mat[c][r];
            assert!(
                (re_rc - re_cr).abs() < 1e-12,
                "Real parts must be symmetric at ({}, {})",
                r,
                c
            );
            assert!(
                (im_rc + im_cr).abs() < 1e-12,
                "Imaginary parts must be skew-symmetric at ({}, {})",
                r,
                c
            );
        }
    }

    // 4. Band structure high-symmetry path generation
    let path = wg.high_symmetry_band_structure(16);
    assert!(path.len() >= 48, "High-symmetry path must contain at least 48 points");
    assert!(path[0].label.as_deref() == Some("Gamma"));
}

#[test]
fn test_quantized_quadrupole_moment_topological_vs_trivial() {
    // Topological SOTI phase (gamma < lambda)
    let topo_params = QuadrupoleWaveguideParams {
        gamma_mhz: 2.0,
        lambda_mhz: 8.0,
        omega_0_ghz: 1.0,
        length_mm: 50.0,
        nx: 6,
        ny: 6,
    };
    let topo_wg = QuadrupoleWaveguide::new(topo_params);

    assert!(topo_wg.params.is_topological(), "Must be topological when gamma < lambda");
    assert_eq!(
        topo_wg.quadrupole_moment, 0.5,
        "Quantized quadrupole moment q_xy must be exactly 0.5 in topological phase"
    );
    assert_eq!(
        topo_wg.edge_dipoles,
        (0.5, 0.5),
        "Edge dipole moments must be (0.5, 0.5) in topological phase"
    );
    assert!(
        topo_wg.boundary_confinement >= 0.80,
        "Boundary modal confinement must be >= 80% (got {:.2}%)",
        topo_wg.boundary_confinement * 100.0
    );

    // Trivial phase (gamma > lambda)
    let trivial_params = QuadrupoleWaveguideParams {
        gamma_mhz: 9.0,
        lambda_mhz: 3.0,
        omega_0_ghz: 1.0,
        length_mm: 50.0,
        nx: 6,
        ny: 6,
    };
    let trivial_wg = QuadrupoleWaveguide::new(trivial_params);

    assert!(!trivial_wg.params.is_topological(), "Must be trivial when gamma > lambda");
    assert_eq!(
        trivial_wg.quadrupole_moment, 0.0,
        "Quantized quadrupole moment q_xy must be 0.0 in trivial phase"
    );
    assert_eq!(
        trivial_wg.edge_dipoles,
        (0.0, 0.0),
        "Edge dipole moments must be (0.0, 0.0) in trivial phase"
    );
    assert!(
        trivial_wg.boundary_confinement < 0.40,
        "Boundary modal confinement in trivial phase must be < 40% (got {:.2}%)",
        trivial_wg.boundary_confinement * 100.0
    );
}

#[test]
fn test_shg_coupled_mode_spatial_integration_and_sinc2_curve() {
    let params = ShgParams {
        kappa: 0.05,
        pump_power_w: 0.1,
        f1_ghz: 1.0,
        f2_ghz: 2.0,
        nz_steps: 100,
        alpha1_db_cm: 0.2,
        alpha2_db_cm: 0.2,
        delta_k_rad_mm: 0.0,
        length_mm: 50.0,
    };
    let solver = ShgSolver::new(params);

    // 1. Spatial modal overlap integral I_overlap >= 0.85
    assert!(
        solver.modal_overlap_integral >= 0.85,
        "Modal overlap integral between fundamental and second-harmonic modes must be >= 0.85, got {}",
        solver.modal_overlap_integral
    );
    assert!(solver.modal_overlap_integral <= 1.0);

    // 2. Sweep phase mismatch Delta_k
    let sweep = solver.sweep_phase_mismatch(0.25, 41);
    assert_eq!(sweep.len(), 41);

    // Peak conversion must be at Delta_k = 0
    let center_idx = 20; // 0.0 rad/mm
    assert!(
        sweep[center_idx].delta_k_rad_mm.abs() < 1e-9,
        "Center index must correspond to Delta_k = 0"
    );
    let peak_eff = sweep[center_idx].efficiency;

    // Both edges must have significantly lower efficiency due to sinc^2 dephasing
    let left_eff = sweep[0].efficiency;
    let right_eff = sweep[40].efficiency;
    assert!(
        peak_eff > left_eff * 5.0,
        "Peak efficiency at Delta_k=0 ({}) must exceed dephased efficiency ({})",
        peak_eff,
        left_eff
    );
    assert!(
        peak_eff > right_eff * 5.0,
        "Peak efficiency at Delta_k=0 ({}) must exceed dephased efficiency ({})",
        peak_eff,
        right_eff
    );
}

#[test]
fn test_shg_power_conversion_efficiency_delta_k_zero() {
    let params = ShgParams::default();
    let solver = ShgSolver::new(params.clone());

    // 1. Exact phase matching monotonic growth verification
    assert!(
        solver.is_monotonic_conversion(),
        "Second-harmonic power conversion must be strictly monotonic under Delta_k = 0"
    );

    // 2. Conversion efficiency and power
    assert!(
        solver.conversion_efficiency > 0.10,
        "Terminal conversion efficiency must be substantial (> 10%), got {:.2}%",
        solver.conversion_efficiency * 100.0
    );
    assert!(
        solver.terminal_shg_power_w > 0.01,
        "Generated SHG power must exceed 10 mW, got {:.4} W",
        solver.terminal_shg_power_w
    );

    // 3. Power bounds check: P1(z) and P2(z) do not individually exceed input pump power
    for pt in &solver.trajectory {
        assert!(
            pt.p1_w <= params.pump_power_w + 1e-9,
            "Fundamental power {} must not exceed input pump power {}",
            pt.p1_w,
            params.pump_power_w
        );
        assert!(
            pt.p2_w <= params.pump_power_w + 1e-9,
            "Second-harmonic power {} must not exceed input pump power {}",
            pt.p2_w,
            params.pump_power_w
        );
    }
}

#[test]
fn test_non_reciprocal_parametric_edge_gain_and_isolation() {
    let params = ParametricDriveParams::default();
    let amp = ParametricEdgeAmplifier::new(params);

    // 1. Forward signal power gain >= 20.0 dB
    assert!(
        amp.metrics.gain_forward_db >= 20.0,
        "Forward signal power gain must achieve >= 20.0 dB, got {:.2} dB",
        amp.metrics.gain_forward_db
    );

    // 2. Backward signal power gain <= 0.5 dB
    assert!(
        amp.metrics.gain_backward_db <= 0.5,
        "Backward signal power gain must be <= 0.5 dB (severely phase-mismatched), got {:.2} dB",
        amp.metrics.gain_backward_db
    );

    // 3. Directional isolation >= 25.0 dB
    assert!(
        amp.metrics.isolation_db >= 25.0,
        "Directional isolation must achieve >= 25.0 dB, got {:.2} dB",
        amp.metrics.isolation_db
    );

    // 4. Spectrum peak
    assert!(!amp.gain_spectrum.is_empty());
    let max_gain = amp
        .gain_spectrum
        .iter()
        .map(|s| s.gain_forward_db)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (max_gain - amp.metrics.gain_forward_db).abs() < 0.1,
        "Gain spectrum peak must coincide with resonance forward gain"
    );
}

#[test]
fn test_quantum_limited_added_noise() {
    let params = ParametricDriveParams::default();
    let amp = ParametricEdgeAmplifier::new(params);

    // Caves quantum limit: n_add = 0.5 * (1 - 1 / G_forward) <= 0.55 quanta
    assert!(
        amp.metrics.added_noise_quanta <= 0.55,
        "Added noise must be <= 0.55 quanta, got {:.5}",
        amp.metrics.added_noise_quanta
    );
    assert!(
        amp.metrics.added_noise_quanta >= 0.45,
        "Added noise must approach standard quantum limit ~0.5 quanta, got {:.5}",
        amp.metrics.added_noise_quanta
    );

    // Check theoretical limit as G -> infinity
    let huge_gain_noise = ParametricEdgeAmplifier::compute_added_noise(1.0e6);
    assert!(
        (huge_gain_noise - 0.5).abs() < 1e-5,
        "At asymptotic high gain, added noise must strictly approach 0.5 quanta"
    );
}
