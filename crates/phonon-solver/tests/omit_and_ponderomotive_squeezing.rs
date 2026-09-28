//! Integration tests for Optomechanically Induced Transparency (OMIT) and Ponderomotive Light Squeezing.

use phonon_models::optomechanics::{OmitParams, PonderomotiveSqueezingParams};
use phonon_solver::optomechanics::OptomechanicalMasterEquationSolver;

#[test]
fn test_omit_transparency_window_and_group_delay() {
    let omit = OmitParams::standard_nanobeam_omit();

    // On-resonance transparency peak (at two-photon resonance delta = 0)
    let trans_omit = omit.probe_transmission(0.0);
    // Off-resonance transmission dip (at delta = 500 kHz)
    let trans_dip = omit.probe_transmission(500.0e3);

    assert!(
        trans_omit > trans_dip,
        "OMIT transmission at two-photon resonance ({}) must exceed background dip ({})",
        trans_omit,
        trans_dip
    );
    assert!(
        trans_omit > 0.8,
        "OMIT transparency window peak should exceed 80% transmission, got {}",
        trans_omit
    );

    let width = omit.transparency_linewidth_hz();
    assert!(
        width > omit.system.mechanical_damping_hz,
        "OMIT window is broadened by cooperativity: width {} vs gamma_m {}",
        width,
        omit.system.mechanical_damping_hz
    );

    let delay = omit.group_delay_s();
    assert!(delay > 0.0, "Group delay must be positive (slow light)");
}

#[test]
fn test_ponderomotive_squeezing_and_entanglement() {
    let squeeze_params = PonderomotiveSqueezingParams::standard_squeezed_source();

    let v_min_analytical = squeeze_params.minimum_quadrature_variance();
    assert!(
        v_min_analytical < 1.0,
        "Analytical variance must be below shot noise (1.0), got {}",
        v_min_analytical
    );

    let sq_db_analytical = squeeze_params.squeezing_depth_db();
    assert!(
        sq_db_analytical > 3.0,
        "Squeezing depth must exceed 3 dB, got {} dB",
        sq_db_analytical
    );

    let theta_opt = squeeze_params.optimal_quadrature_angle_rad();
    assert!(theta_opt > 0.0 && theta_opt < std::f64::consts::PI);

    let en_analytical = squeeze_params.logarithmic_negativity_entanglement();
    assert!(
        en_analytical > 0.0,
        "Logarithmic negativity entanglement must be non-zero in quantum regime"
    );

    // Continuous Lyapunov master equation solver
    let cov_res = OptomechanicalMasterEquationSolver::solve(&squeeze_params);
    assert!(
        cov_res.optical_min_variance < 1.0,
        "Numerical Lyapunov variance must show squeezing below shot noise, got {}",
        cov_res.optical_min_variance
    );
    assert!(
        cov_res.optical_squeezing_db > 3.0,
        "Numerical optical squeezing must exceed 3 dB, got {} dB",
        cov_res.optical_squeezing_db
    );
}
