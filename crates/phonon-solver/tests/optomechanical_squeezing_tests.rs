#![deny(unsafe_code)]

//! Test suite for Phase 359: Cavity Optomechanical Squeezing, Wigner Function & Phonon Counting Engine.

use phonon_solver::optomechanical_squeezing::{
    FockStateDistribution, NonClassicalityMetrics, OptomechanicalSqueezingParams,
    PhononCountingResolvedSpectrum, PhononStateKind, QuadratureSqueezingSolver,
    WignerQuasiProbability, SQL_VARIANCE,
};

#[test]
fn test_mechanical_quadrature_squeezing_below_sql() {
    let params = OptomechanicalSqueezingParams::preset_squeezed_vacuum();
    let solver = QuadratureSqueezingSolver::new(params.clone());

    let variance = solver.solve_variance();

    // Verify minimum quadrature variance is strictly below SQL (0.5)
    assert!(
        variance.var_min < SQL_VARIANCE,
        "Minimum variance {} must be below SQL ({})",
        variance.var_min,
        SQL_VARIANCE
    );
    assert!(variance.is_squeezed_below_sql());

    // Verify squeezing level exceeds 3.0 dB below SQL
    assert!(
        variance.squeezing_db >= 3.0,
        "Squeezing {} dB must be >= 3.0 dB below SQL",
        variance.squeezing_db
    );

    // Verify higher squeezing parameter r >= 1.10 produces > 8.0 dB squeezing
    let mut high_squeeze_params = params;
    high_squeeze_params.squeezing_parameter_r = 1.15;
    let high_solver = QuadratureSqueezingSolver::new(high_squeeze_params);
    let high_variance = high_solver.solve_variance();
    assert!(
        high_variance.squeezing_db > 8.0,
        "High squeezing parameter must produce > 8.0 dB squeezing (got {} dB)",
        high_variance.squeezing_db
    );

    // Verify polar scan computes smooth profile
    let scan = solver.compute_quadrature_scan(36);
    assert_eq!(scan.len(), 37);
    for &(_theta, v) in &scan {
        assert!(v > 0.0);
    }
}

#[test]
fn test_wigner_function_negativity_for_single_phonon_fock_state() {
    let grid_size = 61;
    let range = 4.0;
    let wigner = WignerQuasiProbability::compute_single_phonon_fock1(grid_size, range);

    // Single phonon Fock state |1> must have strictly negative core at (0, 0)
    assert!(
        wigner.w_min < 0.0,
        "W_min {} must be strictly negative for Fock |1>",
        wigner.w_min
    );

    // Analytical minimum at origin (0, 0) is -1 / pi approx -0.31831
    let center_idx = grid_size / 2;
    let w_origin = wigner.at(center_idx, center_idx);
    let expected_origin = -1.0 / std::f64::consts::PI;
    assert!(
        (w_origin - expected_origin).abs() < 1e-4,
        "W(0, 0) {} should be near expected {}",
        w_origin,
        expected_origin
    );

    assert!(wigner.is_nonclassical);
    assert!(wigner.negative_volume > 0.05);

    // Ground state |0> must be strictly non-negative (Gaussian)
    let wigner_0 = WignerQuasiProbability::compute_ground_state(grid_size, range);
    assert!(
        wigner_0.w_min >= 0.0,
        "Ground state Wigner must be non-negative everywhere (got {})",
        wigner_0.w_min
    );
    assert!(!wigner_0.is_nonclassical);
}

#[test]
fn test_wigner_function_normalization() {
    let grid_size = 61;
    let range = 4.5;

    // Normalization test for Fock |1>
    let wigner_fock1 = WignerQuasiProbability::compute_single_phonon_fock1(grid_size, range);
    assert!(
        (wigner_fock1.total_integral - 1.0).abs() < 0.01,
        "Fock |1> Wigner integral {} must be approximately 1.0",
        wigner_fock1.total_integral
    );

    // Normalization test for Ground state |0>
    let wigner_ground = WignerQuasiProbability::compute_ground_state(grid_size, range);
    assert!(
        (wigner_ground.total_integral - 1.0).abs() < 0.005,
        "Ground state Wigner integral {} must be approximately 1.0",
        wigner_ground.total_integral
    );

    // Normalization test for Squeezed Vacuum
    let wigner_sqz = WignerQuasiProbability::compute_squeezed_vacuum(0.80, 0.0, grid_size, range);
    assert!(
        (wigner_sqz.total_integral - 1.0).abs() < 0.02,
        "Squeezed vacuum Wigner integral {} must be approximately 1.0",
        wigner_sqz.total_integral
    );
}

#[test]
fn test_parity_and_fock_distribution_for_squeezed_vacuum() {
    let r = 0.80;
    let dist = FockStateDistribution::for_squeezed_vacuum(r, 12);

    assert_eq!(dist.state_kind, PhononStateKind::SqueezedVacuum);

    // Squeezed vacuum must have non-zero probability ONLY for even phonon numbers
    for n in 0..=12 {
        let p = dist.probability(n);
        if n % 2 == 1 {
            assert_eq!(
                p, 0.0,
                "Odd Fock state population P({}) must be strictly 0.0 for squeezed vacuum",
                n
            );
        } else {
            assert!(
                p > 0.0,
                "Even Fock state population P({}) must be positive",
                n
            );
        }
    }

    // Parity expectation value <(-1)^n> must be strictly +1.0
    let parity = dist.parity();
    assert!(
        (parity - 1.0).abs() < 1e-6,
        "Squeezed vacuum parity {} must be +1.0",
        parity
    );

    // Analytical P(0) = 1 / cosh(r)
    let expected_p0 = 1.0 / r.cosh();
    assert!(
        (dist.probability(0) - expected_p0).abs() < 1e-6,
        "P(0) {} must match 1 / cosh(r) = {}",
        dist.probability(0),
        expected_p0
    );

    // Analytical P(2) = 0.5 * tanh^2(r) / cosh(r)
    let expected_p2 = 0.5 * r.tanh().powi(2) / r.cosh();
    assert!(
        (dist.probability(2) - expected_p2).abs() < 1e-6,
        "P(2) {} must match expected {}",
        dist.probability(2),
        expected_p2
    );
}

#[test]
fn test_nonclassical_g2_for_single_phonon_fock_state() {
    let dist = FockStateDistribution::for_single_fock1(12);

    assert_eq!(dist.probability(1), 1.0);
    assert_eq!(dist.mean_phonon_number, 1.0);
    for n in 2..=12 {
        assert_eq!(dist.probability(n), 0.0);
    }

    let metrics = NonClassicalityMetrics::evaluate(&dist, 150.0, 20.0, 100_000.0, 2.0, 15.0);

    // Second-order correlation g^(2)(0) for single phonon state must be strictly 0.0
    assert_eq!(
        metrics.second_order_correlation_g2, 0.0,
        "g^(2)(0) for single phonon state must be 0.0"
    );
    assert!(metrics.is_sub_poissonian);
    assert!(metrics.is_quantum_pure);
}

#[test]
fn test_sideband_cooling_ground_state_occupancy() {
    let gamma_m_hz = 150.0;
    let n_th = 20.0;
    let gamma_opt_hz = 100_000.0; // 100 kHz optomechanical optical damping rate
    let kappa_mhz = 2.0;
    let omega_m_mhz = 15.0;

    let n_final = NonClassicalityMetrics::compute_sideband_cooling(
        gamma_m_hz,
        n_th,
        gamma_opt_hz,
        kappa_mhz,
        omega_m_mhz,
    );

    // Must achieve quantum ground state cooling: n_final < 0.10
    assert!(
        n_final < 0.10,
        "Sideband cooled final occupancy {} must be < 0.10",
        n_final
    );
    assert!(
        n_final > 0.0,
        "Occupancy must be physically bounded above zero"
    );
}

#[test]
fn test_dispersive_cavity_resolved_spectrum() {
    let dist = FockStateDistribution::for_squeezed_vacuum(0.80, 8);
    let spectrum = PhononCountingResolvedSpectrum::compute(10.0, 5.0, 1.0, &dist, 200);

    assert!(spectrum.is_resolved);
    assert_eq!(spectrum.peaks.len(), 9);

    // Verify peak frequency spacing: delta_n = -2 * n * chi
    let chi = 5.0;
    for (n, peak) in spectrum.peaks.iter().enumerate() {
        let expected_detuning = -2.0 * (n as f64) * chi;
        assert!(
            (peak.detuning_mhz - expected_detuning).abs() < 1e-6,
            "Peak {} detuning {} must match expected {}",
            n,
            peak.detuning_mhz,
            expected_detuning
        );
        // Even peaks have non-zero amplitude, odd peaks are zero for squeezed vacuum
        if n % 2 == 1 {
            assert_eq!(peak.probability, 0.0);
        } else {
            assert!(peak.probability > 0.0);
        }
    }
}
