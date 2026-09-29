//! Automated unit and physical validation tests for quantum acoustic metasurface
//! holography and dynamic phonon routing.

use phonon_models::acoustic_metasurface_holography::MetasurfaceHolographyParams;
use phonon_solver::acoustic_metasurface_holography::AcousticMetasurfaceHolographySolver;

#[test]
fn test_beam_steering_efficiency() {
    let params = MetasurfaceHolographyParams::default();
    let solver = AcousticMetasurfaceHolographySolver::new(params);
    let efficiency = solver.compute_beam_steering_efficiency();

    // Target holographic beam steering efficiency >= 88.0% (0.880)
    assert!(
        efficiency >= 0.880,
        "Beam steering efficiency must be >= 0.880, got {:.4}",
        efficiency
    );
    assert!(
        efficiency <= 1.0,
        "Beam steering efficiency cannot exceed unity, got {:.4}",
        efficiency
    );

    // Higher phase resolution bits should improve efficiency
    let mut params_high_bits = params;
    params_high_bits.phase_resolution_bits = 10;
    let solver_high_bits = AcousticMetasurfaceHolographySolver::new(params_high_bits);
    assert!(
        solver_high_bits.compute_beam_steering_efficiency() >= efficiency,
        "Higher phase resolution should yield higher or equal efficiency"
    );
}

#[test]
fn test_inter_channel_crosstalk() {
    let params = MetasurfaceHolographyParams::default();
    let solver = AcousticMetasurfaceHolographySolver::new(params);
    let crosstalk = solver.compute_inter_channel_crosstalk_db();

    // Target inter-channel crosstalk <= -35.0 dB
    assert!(
        crosstalk <= -35.0,
        "Inter-channel acoustic crosstalk must be <= -35.0 dB, got {:.2} dB",
        crosstalk
    );
    assert!(
        crosstalk >= -55.0,
        "Inter-channel crosstalk must be within physical clamp bounds >= -55.0 dB, got {:.2} dB",
        crosstalk
    );

    // More bits should provide stronger crosstalk suppression (more negative dB)
    let mut params_more_bits = params;
    params_more_bits.phase_resolution_bits = 8;
    let solver_more_bits = AcousticMetasurfaceHolographySolver::new(params_more_bits);
    assert!(
        solver_more_bits.compute_inter_channel_crosstalk_db() <= crosstalk,
        "Higher phase resolution must yield stronger crosstalk suppression"
    );
}

#[test]
fn test_reconfiguration_latency() {
    let params = MetasurfaceHolographyParams::default();
    let solver = AcousticMetasurfaceHolographySolver::new(params);
    let latency = solver.compute_reconfiguration_latency_ns();

    // Target reconfiguration latency <= 10.0 ns
    assert!(
        latency <= 10.0,
        "Wavefront reconfiguration latency must be <= 10.0 ns, got {:.4} ns",
        latency
    );
    assert!(
        latency >= 0.05,
        "Latency must be >= 0.05 ns, got {:.4} ns",
        latency
    );

    // Higher RC parasitics should increase latency
    let mut params_higher_rc = params;
    params_higher_rc.electrode_resistance_ohms = 200.0;
    params_higher_rc.electrode_capacitance_pf = 2.0;
    let solver_higher_rc = AcousticMetasurfaceHolographySolver::new(params_higher_rc);
    assert!(
        solver_higher_rc.compute_reconfiguration_latency_ns() > latency,
        "Higher RC parasitics must result in higher switching latency"
    );
}

#[test]
fn test_insertion_loss() {
    let params = MetasurfaceHolographyParams::default();
    let solver = AcousticMetasurfaceHolographySolver::new(params);
    let il = solver.compute_insertion_loss_db();

    // Target acoustic transmission insertion loss <= 1.20 dB
    assert!(
        il <= 1.20,
        "Metasurface insertion loss must be <= 1.20 dB, got {:.4} dB",
        il
    );
    assert!(
        il >= 0.20,
        "Insertion loss must be >= 0.20 dB, got {:.4} dB",
        il
    );

    // Larger array size or higher loss per um should increase insertion loss
    let mut params_larger = params;
    params_larger.array_elements_count = 128;
    let solver_larger = AcousticMetasurfaceHolographySolver::new(params_larger);
    assert!(
        solver_larger.compute_insertion_loss_db() > il,
        "Larger metasurface array must have higher propagation loss"
    );
}

#[test]
fn test_routing_channel_fidelity() {
    let params = MetasurfaceHolographyParams::default();
    let solver = AcousticMetasurfaceHolographySolver::new(params);
    let fidelity = solver.compute_routing_channel_fidelity();

    // Target multi-channel routing fidelity >= 0.960
    assert!(
        fidelity >= 0.960,
        "Routing channel fidelity must be >= 0.960, got {:.4}",
        fidelity
    );
    assert!(
        fidelity <= 1.0,
        "Routing fidelity cannot exceed unity, got {:.4}",
        fidelity
    );
}

#[test]
fn test_physical_compliance_evaluation() {
    let params = MetasurfaceHolographyParams::default();
    let solver = AcousticMetasurfaceHolographySolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default metasurface holography parameters must be physically compliant"
    );
    assert!(metrics.beam_steering_efficiency >= 0.880);
    assert!(metrics.inter_channel_crosstalk_db <= -35.0);
    assert!(metrics.reconfiguration_latency_ns <= 10.0);
    assert!(metrics.insertion_loss_db <= 1.20);
    assert!(metrics.routing_channel_fidelity >= 0.960);

    // Test parameter sweep extreme corner
    let corner_params = MetasurfaceHolographyParams::new(
        14.5,  // high freq
        2.5,   // large pitch
        128,   // large element count
        4,     // moderate bits
        5.0,   // voltage
        120.0, // resistance
        0.5,   // capacitance
        0.005, // loss
    );
    let corner_solver = AcousticMetasurfaceHolographySolver::new(corner_params);
    let corner_metrics = corner_solver.evaluate_metrics();
    assert!(
        corner_metrics.is_physically_compliant,
        "Corner parameter configuration must remain physically compliant"
    );
}
