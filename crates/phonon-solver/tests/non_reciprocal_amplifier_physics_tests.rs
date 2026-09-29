//! Analytical physics validation tests for non-reciprocal topological phonon
//! amplification and directional quantum routing (Phase 117).

use phonon_models::non_reciprocal_phonon_amplifier::NonReciprocalAmplifierParams;
use phonon_solver::non_reciprocal_phonon_amplifier::NonReciprocalPhononAmplifierSolver;

#[test]
fn test_forward_gain_exceeds_threshold() {
    let params = NonReciprocalAmplifierParams::default();
    let solver = NonReciprocalPhononAmplifierSolver::new(params);
    let gain_db = solver.compute_forward_gain_db();

    // Roadmap target: Forward non-reciprocal gain >= 20.0 dB
    assert!(
        gain_db >= 20.0,
        "Forward gain must be >= 20.0 dB, got {:.4} dB",
        gain_db
    );
    assert!(
        gain_db <= 35.0,
        "Forward gain must respect clamp ceiling 35.0 dB, got {:.4} dB",
        gain_db
    );

    // Higher parametric coupling rate should yield higher or equal gain
    let mut high_coupling_params = params;
    high_coupling_params.parametric_coupling_rate_mhz = 30.0;
    let high_coupling_solver = NonReciprocalPhononAmplifierSolver::new(high_coupling_params);
    assert!(
        high_coupling_solver.compute_forward_gain_db() >= gain_db,
        "Higher parametric coupling rate must increase or preserve gain"
    );

    // Higher acoustic loss should yield lower or equal gain
    let mut high_loss_params = params;
    high_loss_params.acoustic_loss_rate_mhz = 2.5;
    let high_loss_solver = NonReciprocalPhononAmplifierSolver::new(high_loss_params);
    assert!(
        high_loss_solver.compute_forward_gain_db() <= gain_db,
        "Higher acoustic dissipation must reduce or preserve gain"
    );
}

#[test]
fn test_backward_isolation_exceeds_threshold() {
    let params = NonReciprocalAmplifierParams::default();
    let solver = NonReciprocalPhononAmplifierSolver::new(params);
    let isolation_db = solver.compute_backward_isolation_db();

    // Roadmap target: Backward isolation >= 30.0 dB
    assert!(
        isolation_db >= 30.0,
        "Backward isolation must be >= 30.0 dB, got {:.4} dB",
        isolation_db
    );
    assert!(
        isolation_db <= 50.0,
        "Backward isolation must respect clamp ceiling 50.0 dB, got {:.4} dB",
        isolation_db
    );

    // Synthetic gauge flux at pi/2 (~1.5708 rad) should maximize isolation
    let mut low_gauge_params = params;
    low_gauge_params.synthetic_phase_gradient_rad = 0.20;
    let low_gauge_solver = NonReciprocalPhononAmplifierSolver::new(low_gauge_params);
    assert!(
        isolation_db >= low_gauge_solver.compute_backward_isolation_db(),
        "Phase gradient at pi/2 must provide greater or equal isolation than small phase gradient"
    );
}

#[test]
fn test_quantum_added_noise_within_quantum_limit() {
    let params = NonReciprocalAmplifierParams::default();
    let solver = NonReciprocalPhononAmplifierSolver::new(params);
    let added_noise = solver.compute_added_noise_photons();

    // Caves quantum limit: n_add <= 0.50 quanta
    assert!(
        added_noise <= 0.50,
        "Added noise quanta must be <= 0.50, got {:.6}",
        added_noise
    );
    assert!(
        added_noise >= 0.01,
        "Added noise quanta must respect physical floor 0.01, got {:.6}",
        added_noise
    );

    // Lower temperature should yield lower or equal added noise
    let mut cold_params = params;
    cold_params.operating_temp_m_k = 5.0;
    let cold_solver = NonReciprocalPhononAmplifierSolver::new(cold_params);
    assert!(
        cold_solver.compute_added_noise_photons() <= added_noise,
        "Colder operating temperature must decrease or preserve added thermal noise"
    );
}

#[test]
fn test_instantaneous_amplification_bandwidth() {
    let params = NonReciprocalAmplifierParams::default();
    let solver = NonReciprocalPhononAmplifierSolver::new(params);
    let bandwidth_mhz = solver.compute_instantaneous_bandwidth_mhz();

    // Roadmap target: Instantaneous bandwidth Delta f >= 15.0 MHz
    assert!(
        bandwidth_mhz >= 15.0,
        "Instantaneous bandwidth must be >= 15.0 MHz, got {:.4} MHz",
        bandwidth_mhz
    );
    assert!(
        bandwidth_mhz <= 60.0,
        "Instantaneous bandwidth must respect clamp ceiling 60.0 MHz, got {:.4} MHz",
        bandwidth_mhz
    );
}

#[test]
fn test_directional_routing_fidelity() {
    let params = NonReciprocalAmplifierParams::default();
    let solver = NonReciprocalPhononAmplifierSolver::new(params);
    let fidelity = solver.compute_directional_routing_fidelity();

    // Roadmap target: Directional routing fidelity >= 0.960 (96.0%)
    assert!(
        fidelity >= 0.960,
        "Directional routing fidelity must be >= 0.960, got {:.6}",
        fidelity
    );
    assert!(
        fidelity <= 0.999,
        "Directional routing fidelity must respect clamp ceiling 0.999, got {:.6}",
        fidelity
    );
}

#[test]
fn test_parameter_bounds_clamping() {
    let out_of_bounds = NonReciprocalAmplifierParams::new(
        0.5,    // Below min 1.0 GHz -> clamps to 1.0
        5.0,    // Below min 10.0 MHz -> clamps to 10.0
        0.05,   // Below min 0.1 rad -> clamps to 0.1
        0.5,    // Below min 1.0 MHz -> clamps to 1.0
        0.01,   // Below min 0.05 MHz -> clamps to 0.05
        2.0,    // Below min 5.0 MHz -> clamps to 5.0
        0.05,   // Below min 0.1 mW -> clamps to 0.1
        0.5,    // Below min 1.0 mK -> clamps to 1.0
    );

    assert_eq!(out_of_bounds.operating_frequency_ghz, 1.0);
    assert_eq!(out_of_bounds.pump_modulation_frequency_mhz, 10.0);
    assert_eq!(out_of_bounds.synthetic_phase_gradient_rad, 0.1);
    assert_eq!(out_of_bounds.parametric_coupling_rate_mhz, 1.0);
    assert_eq!(out_of_bounds.acoustic_loss_rate_mhz, 0.05);
    assert_eq!(out_of_bounds.inter_site_hopping_mhz, 5.0);
    assert_eq!(out_of_bounds.pump_power_mw, 0.1);
    assert_eq!(out_of_bounds.operating_temp_m_k, 1.0);

    let upper_bounds = NonReciprocalAmplifierParams::new(
        15.0,   // Above max 12.0 GHz -> clamps to 12.0
        250.0,  // Above max 200.0 MHz -> clamps to 200.0
        4.0,    // Above max 3.14159 rad -> clamps to 3.14159
        70.0,   // Above max 50.0 MHz -> clamps to 50.0
        10.0,   // Above max 5.0 MHz -> clamps to 5.0
        150.0,  // Above max 100.0 MHz -> clamps to 100.0
        80.0,   // Above max 50.0 mW -> clamps to 50.0
        150.0,  // Above max 100.0 mK -> clamps to 100.0
    );

    assert_eq!(upper_bounds.operating_frequency_ghz, 12.0);
    assert_eq!(upper_bounds.pump_modulation_frequency_mhz, 200.0);
    assert_eq!(upper_bounds.synthetic_phase_gradient_rad, 3.14159);
    assert_eq!(upper_bounds.parametric_coupling_rate_mhz, 50.0);
    assert_eq!(upper_bounds.acoustic_loss_rate_mhz, 5.0);
    assert_eq!(upper_bounds.inter_site_hopping_mhz, 100.0);
    assert_eq!(upper_bounds.pump_power_mw, 50.0);
    assert_eq!(upper_bounds.operating_temp_m_k, 100.0);
}

#[test]
fn test_full_roadmap_physical_compliance() {
    let params = NonReciprocalAmplifierParams::default();
    let solver = NonReciprocalPhononAmplifierSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default amplifier parameter configuration must be physically compliant"
    );
    assert!(metrics.forward_gain_db >= 20.0);
    assert!(metrics.backward_isolation_db >= 30.0);
    assert!(metrics.added_noise_photons <= 0.50);
    assert!(metrics.instantaneous_bandwidth_mhz >= 15.0);
    assert!(metrics.directional_routing_fidelity >= 0.960);
}
