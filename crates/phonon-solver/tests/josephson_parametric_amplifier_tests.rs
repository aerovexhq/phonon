#![deny(unsafe_code)]

//! Test suite for Phase 402: Superconducting Josephson Parametric Acoustic Waveguide Amplification
//! & Quantum Squeezed Vacuum Engine in phonon-solver.
//!
//! Verifies:
//! 1. Flux-tunable Josephson inductance L_J(Phi) and resonant frequency tuning curve.
//! 2. Parametric signal gain G_max >= 20.0 dB and 3-dB bandwidth characteristics.
//! 3. Sub-SQL quadrature squeezing >= 6.0 dB below vacuum standard quantum limit.
//! 4. Heisenberg uncertainty relation Delta X_max^2 * Delta X_min^2 >= 0.0625.
//! 5. 2D Wigner quasi-probability distribution normalization and phase-space ellipticity.
//! 6. Duan-Simon EPR inseparability criterion Delta(X_1 - X_2)^2 + Delta(P_1 + P_2)^2 < 1.0.
//! 7. Cryogenic added noise n_add <= 0.55 quanta and thermal occupancy n_th << 1e-4 at 10 mK.
//! 8. 10-point comprehensive physics audit 10/10 PASS certification.

use phonon_solver::josephson_parametric_amplifier::{
    CvClusterStateParams, CvClusterStateSolver, JpaWaveguideParams,
    JosephsonInductanceModel, JosephsonParametricProcessor, SqueezedVacuumSolver,
    SqueezingParams, WignerQuasiProbability, VACUUM_SQL_VARIANCE,
};

#[test]
fn test_josephson_inductance_and_flux_tuning() {
    let params = JpaWaveguideParams::default();

    // At Phi = 0: L_J(0) = Phi_0 / (2*pi*I_c)
    let l_j_zero = JosephsonInductanceModel::josephson_inductance_ph(params.critical_current_ua, 0.0);
    assert!(
        (l_j_zero - 131.64).abs() < 1.0,
        "L_J(0) should be ~131.64 pH for I_c=2.5 uA, got {}",
        l_j_zero
    );

    // At Phi = 0.25 Phi_0: |cos(pi/4)| = 1/sqrt(2) => L_J(0.25) = sqrt(2) * L_J(0)
    let l_j_quarter = JosephsonInductanceModel::josephson_inductance_ph(params.critical_current_ua, 0.25);
    let expected_quarter = l_j_zero * std::f64::consts::SQRT_2;
    assert!(
        (l_j_quarter - expected_quarter).abs() < 1.0,
        "L_J(0.25) should scale with 1/cos(pi/4): expected {}, got {}",
        expected_quarter,
        l_j_quarter
    );

    // Resonant frequency at Phi = 0.0 vs Phi = 0.45
    let f_0_max = JosephsonInductanceModel::frequency_at_flux(&params, 0.0);
    let f_0_min = JosephsonInductanceModel::frequency_at_flux(&params, 0.45);
    assert!(
        f_0_max > f_0_min,
        "Resonant frequency must decrease as flux bias increases: f_0(0)={}, f_0(0.45)={}",
        f_0_max,
        f_0_min
    );

    let tuning_range = params.tuning_range_ghz();
    assert!(
        tuning_range >= 1.0,
        "Frequency tuning range Delta_f must be >= 1.0 GHz, got {}",
        tuning_range
    );

    // Verify tuning curve evaluation
    let curve = JosephsonInductanceModel::compute_tuning_curve(&params, 50);
    assert_eq!(curve.len(), 50);
    for window in curve.windows(2) {
        let [flux_a, _] = window[0];
        let [flux_b, _] = window[1];
        assert!(flux_b > flux_a, "Flux bias must be monotonically increasing");
    }
}

#[test]
fn test_parametric_signal_gain() {
    let waveguide = JpaWaveguideParams::default();
    let squeezing = SqueezingParams::default();

    let response = SqueezedVacuumSolver::solve(&waveguide, &squeezing);

    // Maximum signal gain must meet or exceed 20.0 dB
    assert!(
        response.gain_max_db >= 20.0,
        "Parametric maximum signal gain G_max must be >= 20.0 dB, got {}",
        response.gain_max_db
    );

    // Minimum de-amplified quadrature gain must be <= -6.0 dB
    assert!(
        response.gain_min_db <= -6.0,
        "Minimum de-amplified quadrature gain G_min must be <= -6.0 dB, got {}",
        response.gain_min_db
    );

    // Center frequency gain matches gain_max_db
    let center_gain = response.gain_db_at_freq(response.center_freq_ghz);
    assert!(
        (center_gain - response.gain_max_db).abs() < 1e-4,
        "Gain at center frequency must equal G_max"
    );

    // At 3-dB bandwidth edge (f_0 +/- BW/2), gain drops by 3.01 dB
    let delta_f_half_bw = (response.bandwidth_3db_mhz * 0.5) * 1e-3;
    let edge_gain = response.gain_db_at_freq(response.center_freq_ghz + delta_f_half_bw);
    let drop = response.gain_max_db - edge_gain;
    assert!(
        (drop - 3.0103).abs() < 0.05,
        "Gain drop at 3-dB bandwidth edge must be ~3 dB, got {}",
        drop
    );
}

#[test]
fn test_sub_sql_quadrature_squeezing() {
    let waveguide = JpaWaveguideParams::default();
    let squeezing = SqueezingParams::default();

    let response = SqueezedVacuumSolver::solve(&waveguide, &squeezing);

    // Squeezing depth S_dB >= 6.0 dB below standard quantum limit (0.5)
    assert!(
        response.squeezing_depth_db >= 6.0,
        "Sub-SQL squeezing depth must be >= 6.0 dB, got {}",
        response.squeezing_depth_db
    );

    // Squeezed variance Delta X_min^2 < vacuum level (0.5)
    assert!(
        response.delta_x_min_sq < VACUUM_SQL_VARIANCE,
        "Squeezed variance {} must be less than vacuum variance {}",
        response.delta_x_min_sq,
        VACUUM_SQL_VARIANCE
    );

    // Anti-squeezed variance Delta X_max^2 > vacuum level (0.5)
    assert!(
        response.delta_x_max_sq > VACUUM_SQL_VARIANCE,
        "Anti-squeezed variance {} must be greater than vacuum variance {}",
        response.delta_x_max_sq,
        VACUUM_SQL_VARIANCE
    );

    // Squeezing parameter r in [0.8, 1.5]
    assert!(
        response.squeezing_parameter_r >= 0.8 && response.squeezing_parameter_r <= 1.5,
        "Squeezing parameter r must be in [0.8, 1.5], got {}",
        response.squeezing_parameter_r
    );
}

#[test]
fn test_heisenberg_uncertainty_relation() {
    let waveguide = JpaWaveguideParams::default();
    let squeezing = SqueezingParams::default();

    let response = SqueezedVacuumSolver::solve(&waveguide, &squeezing);

    // Delta X_max^2 * Delta X_min^2 >= 0.0625
    assert!(
        response.heisenberg_preserved,
        "Heisenberg uncertainty relation must be preserved"
    );
    assert!(
        response.heisenberg_product >= 0.0625 - 1e-9,
        "Uncertainty product {} must be >= 0.0625 (1/16)",
        response.heisenberg_product
    );

    // In units of standard deviation: Delta X * Delta P >= 0.25
    let delta_x = response.delta_x_min_sq.sqrt();
    let delta_p = response.delta_x_max_sq.sqrt();
    assert!(
        delta_x * delta_p >= 0.25 - 1e-9,
        "Delta X * Delta P must be >= 0.25, got {}",
        delta_x * delta_p
    );
}

#[test]
fn test_wigner_quasi_probability_distribution() {
    let waveguide = JpaWaveguideParams::default();
    let squeezing = SqueezingParams::default();

    let response = SqueezedVacuumSolver::solve(&waveguide, &squeezing);
    let wigner = WignerQuasiProbability::compute_squeezed_vacuum(
        response.delta_x_min_sq,
        response.delta_x_max_sq,
        squeezing.squeezing_angle_rad,
        35,
        3.0,
    );

    assert_eq!(wigner.grid_size, 35);
    assert_eq!(wigner.values.len(), 35);
    assert_eq!(wigner.values[0].len(), 35);

    // Peak Wigner quasi-probability is strictly positive at center (0, 0)
    let center_idx = 17;
    let w_center = wigner.at(center_idx, center_idx);
    assert!(
        w_center > 0.0,
        "Center Wigner value must be positive, got {}",
        w_center
    );

    // Total integral over phase-space should be normalized (~1.0 within numerical grid truncation)
    assert!(
        (wigner.total_integral - 1.0).abs() < 0.25,
        "Phase-space integral must be normalized near 1.0, got {}",
        wigner.total_integral
    );

    // Ellipticity aspect ratio Delta X_max / Delta X_min >= 2.0
    assert!(
        wigner.aspect_ratio >= 2.0,
        "Aspect ratio must be >= 2.0 reflecting squeezing, got {}",
        wigner.aspect_ratio
    );
}

#[test]
fn test_duan_simon_epr_inseparability() {
    let waveguide = JpaWaveguideParams::default();
    let squeezing = SqueezingParams::default();
    let cluster = CvClusterStateParams::default();

    let response = SqueezedVacuumSolver::solve(&waveguide, &squeezing);
    let metrics = CvClusterStateSolver::evaluate(&waveguide, &squeezing, &cluster, &response);

    // Duan-Simon inseparability criterion: Delta(X_1 - X_2)^2 + Delta(P_1 + P_2)^2 < 1.0
    assert!(
        metrics.inseparability_certified,
        "CV EPR entanglement must be certified"
    );
    assert!(
        metrics.epr_nullifier_variance < 1.0,
        "EPR nullifier variance must be < 1.0, got {}",
        metrics.epr_nullifier_variance
    );
    assert!(
        metrics.entanglement_depth_db > 0.0,
        "Entanglement depth in dB must be positive, got {}",
        metrics.entanglement_depth_db
    );
    assert!(
        metrics.cluster_fidelity > 0.5,
        "Cluster state fidelity must exceed 0.5, got {}",
        metrics.cluster_fidelity
    );
}

#[test]
fn test_cryogenic_added_noise_and_thermal_occupancy() {
    let waveguide = JpaWaveguideParams::default();
    let squeezing = SqueezingParams::default(); // 10 mK
    let cluster = CvClusterStateParams::default();

    let response = SqueezedVacuumSolver::solve(&waveguide, &squeezing);
    let metrics = CvClusterStateSolver::evaluate(&waveguide, &squeezing, &cluster, &response);

    // Thermal occupancy at 10 mK: n_th << 1e-4
    assert!(
        metrics.thermal_occupancy_n_th < 1e-4,
        "Thermal occupancy at 10 mK must be << 1e-4, got {}",
        metrics.thermal_occupancy_n_th
    );

    // Added noise quanta <= 0.55 quanta at 10 mK
    assert!(
        metrics.added_noise_quanta <= 0.55,
        "Total added noise quanta must be <= 0.55 quanta, got {}",
        metrics.added_noise_quanta
    );

    // Phase-sensitive added noise < 0.05 quanta
    assert!(
        metrics.phase_sensitive_added_noise < 0.05,
        "Phase-sensitive added noise must be < 0.05 quanta, got {}",
        metrics.phase_sensitive_added_noise
    );

    // 1-dB compression point >= -110.0 dBm
    assert!(
        metrics.power_1db_compression_dbm >= -110.0,
        "1-dB compression point must be >= -110.0 dBm, got {}",
        metrics.power_1db_compression_dbm
    );
}

#[test]
fn test_ten_point_physics_audit() {
    let processor = JosephsonParametricProcessor::default();
    let audit = processor.audit_jpa();

    assert_eq!(
        audit.total_count, 10,
        "Total evaluated criteria must be 10"
    );
    assert_eq!(
        audit.passed_count, 10,
        "All 10 physics criteria must pass, failed: {:?}",
        audit
            .criteria
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
    assert!(
        audit.overall_pass,
        "Overall audit report must pass 10/10"
    );
    assert!(
        audit.cold_boot_latency_us < 2000.0,
        "Cold boot latency {} us must be < 2000 us (2.0 ms)",
        audit.cold_boot_latency_us
    );
}
