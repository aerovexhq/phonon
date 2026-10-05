#![deny(unsafe_code)]

use phonon_solver::{
    MultiplexerJunctionParams, ValleyIndex, ValleyLatticeParams, ValleyLatticeSolver,
    ValleyMultiplexerSolver,
};

#[test]
fn test_valley_berry_curvature_and_chern_numbers() {
    let params = ValleyLatticeParams {
        lattice_constant_a: 0.02,
        speed_of_sound: 343.0,
        cylinder_radius_a: 0.0055,
        cylinder_radius_b: 0.0035,
        asymmetry_delta: 0.25,
        dirac_frequency_hz: 4800.0,
    };

    let solver = ValleyLatticeSolver::new(params);
    let metrics = solver.compute_berry_metrics();

    assert!(
        (metrics.c_k - 0.5).abs() < 1e-10,
        "K valley Chern number must be +0.5, got {}",
        metrics.c_k
    );
    assert!(
        (metrics.c_k_prime - (-0.5)).abs() < 1e-10,
        "K' valley Chern number must be -0.5, got {}",
        metrics.c_k_prime
    );
    assert!(
        metrics.total_chern_number.abs() < 1e-10,
        "Total Chern number must be strictly 0.0, got {}",
        metrics.total_chern_number
    );
    assert!(
        (metrics.valley_chern_number - 0.5).abs() < 1e-10,
        "Valley Chern number Cv must be 0.5, got {}",
        metrics.valley_chern_number
    );
    assert!(
        (metrics.delta_cv_interface - 1.0).abs() < 1e-10,
        "Interface valley Chern jump must be 1.0, got {}",
        metrics.delta_cv_interface
    );

    // Berry curvature at Dirac point
    let omega_k = solver.berry_curvature_at([0.0, 0.0], ValleyIndex::K);
    let omega_kp = solver.berry_curvature_at([0.0, 0.0], ValleyIndex::KPrime);
    assert!(
        omega_k > 0.0,
        "Berry curvature at K valley must be positive, got {}",
        omega_k
    );
    assert!(
        omega_kp < 0.0,
        "Berry curvature at K' valley must be negative, got {}",
        omega_kp
    );
    assert!(
        (omega_k + omega_kp).abs() < 1e-10,
        "Berry curvatures must be anti-symmetric between K and K'"
    );
}

#[test]
fn test_topological_valley_bandgap_scaling() {
    let mut params = ValleyLatticeParams::default();
    params.asymmetry_delta = 0.20;
    let solver = ValleyLatticeSolver::new(params.clone());
    let gap = solver.params.valley_bandgap_hz();
    assert!(gap > 200.0, "Valley gap must be open, got {} Hz", gap);

    // Inversion-symmetric case (Delta = 0)
    params.asymmetry_delta = 0.0;
    let gap_zero = params.valley_bandgap_hz();
    assert!(
        gap_zero < 1e-10,
        "Valley gap must close at Dirac degeneracy (Delta = 0), got {}",
        gap_zero
    );
}

#[test]
fn test_gapless_kink_edge_dispersion() {
    let params = ValleyLatticeParams::default();
    let solver = ValleyLatticeSolver::new(params);

    let dk = 10.0; // 10 rad/m
    let f_k_plus = solver.kink_edge_dispersion(dk, ValleyIndex::K);
    let f_k_minus = solver.kink_edge_dispersion(-dk, ValleyIndex::K);
    assert!(
        f_k_plus > f_k_minus,
        "K valley kink mode must have positive group velocity"
    );

    let f_kp_plus = solver.kink_edge_dispersion(dk, ValleyIndex::KPrime);
    let f_kp_minus = solver.kink_edge_dispersion(-dk, ValleyIndex::KPrime);
    assert!(
        f_kp_plus < f_kp_minus,
        "K' valley kink mode must have negative group velocity"
    );
}

#[test]
fn test_valley_selective_multiplexer_routing() {
    let mut params = MultiplexerJunctionParams::default();
    params.valley_polarization = 1.0; // Pure K valley
    params.splitting_bias = 0.0;

    let mut mux = ValleyMultiplexerSolver::new(params);

    // 1. K-valley -> routes to Port 2
    assert!(
        mux.s_parameters.s21_db >= -0.8,
        "Port 2 transmission S21 must be >= -0.8 dB for K-valley, got {}",
        mux.s_parameters.s21_db
    );
    assert!(
        mux.s_parameters.s31_db <= -20.0,
        "Port 3 rejection S31 must be <= -20.0 dB for K-valley, got {}",
        mux.s_parameters.s31_db
    );
    assert!(
        mux.s_parameters.valley_isolation_db >= 20.0,
        "Valley crosstalk isolation must be >= 20.0 dB, got {}",
        mux.s_parameters.valley_isolation_db
    );
    assert!(
        mux.s_parameters.valley_contrast_ratio >= 0.90,
        "Valley contrast ratio must be >= 90%, got {}",
        mux.s_parameters.valley_contrast_ratio
    );

    // 2. K'-valley -> routes to Port 3
    mux.set_valley(ValleyIndex::KPrime);
    assert!(
        mux.s_parameters.s31_db >= -0.8,
        "Port 3 transmission S31 must be >= -0.8 dB for K'-valley, got {}",
        mux.s_parameters.s31_db
    );
    assert!(
        mux.s_parameters.s21_db <= -20.0,
        "Port 2 rejection S21 must be <= -20.0 dB for K'-valley, got {}",
        mux.s_parameters.s21_db
    );
    assert!(
        mux.s_parameters.valley_isolation_db >= 20.0,
        "Valley crosstalk isolation must be >= 20.0 dB, got {}",
        mux.s_parameters.valley_isolation_db
    );
}

#[test]
fn test_tunable_beam_splitter_bias() {
    let mut params = MultiplexerJunctionParams::default();
    params.valley_polarization = 0.0; // Equal valley injection

    params.splitting_bias = -0.5;
    let mux_low = ValleyMultiplexerSolver::new(params.clone());

    params.splitting_bias = 0.5;
    let mux_high = ValleyMultiplexerSolver::new(params);

    assert!(
        mux_high.s_parameters.splitting_ratio > mux_low.s_parameters.splitting_ratio,
        "Splitting ratio must scale monotonically with bias parameter"
    );
}

#[test]
fn test_sharp_corner_defect_immunity() {
    let mut params = MultiplexerJunctionParams::default();
    params.has_corner_bend = true;
    params.corner_bend_angle_deg = 60.0;

    let mux = ValleyMultiplexerSolver::new(params);
    assert!(
        mux.s_parameters.corner_immunity_ratio >= 0.95,
        "Sharp corner defect immunity ratio must be >= 95%, got {}",
        mux.s_parameters.corner_immunity_ratio
    );
}
