#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 456: Cryogenic Quantum Metamaterial Multi-Terminal Anyon Interferometer & Topologically Protected Qudit Crossbar.

use phonon_solver::anyon_interferometer_qudit::*;
use std::f64::consts::PI;

#[test]
fn test_multi_terminal_interferometer_fringe_visibility() {
    let params = MultiTerminalInterferometerParams::default();
    let solver = MultiTerminalInterferometerSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Visibility >= 85.0%
    assert!(
        metrics.interference_visibility_pct >= 85.0,
        "Expected visibility >= 85.0%, got {}%",
        metrics.interference_visibility_pct
    );

    // Invariant 2: Forward transmission vs destructive port
    assert!(
        metrics.transmission_port2 > metrics.transmission_port3,
        "Expected port2 > port3 at default flux, got T2={}, T3={}",
        metrics.transmission_port2,
        metrics.transmission_port3
    );

    // Invariant 3: Peak-to-valley ratio > 2.0
    assert!(
        metrics.peak_to_valley_ratio > 2.0,
        "Expected peak-to-valley > 2.0, got {}",
        metrics.peak_to_valley_ratio
    );

    // Test flux sweep oscillation
    let sweep = solver.sweep_flux(60);
    assert_eq!(sweep.len(), 60);

    // Verify conductance conservation G2 + G3 + G4 <= 1.05
    for pt in &sweep {
        let total_g = pt.conductance_terminal2 + pt.conductance_terminal3 + pt.conductance_terminal4;
        assert!(
            total_g <= 1.05 && total_g >= 0.85,
            "Expected total conductance near 1.0, got {}",
            total_g
        );
    }

    // Verify oscillation range across full flux sweep
    let min_g2 = sweep.iter().map(|p| p.conductance_terminal2).fold(f64::INFINITY, f64::min);
    let max_g2 = sweep.iter().map(|p| p.conductance_terminal2).fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max_g2 - min_g2 >= 0.70,
        "Expected strong oscillation amplitude >= 0.70, got {}",
        max_g2 - min_g2
    );
}

#[test]
fn test_topological_phase_shift_quantization_and_protection() {
    let params = TopologicalPhaseShiftParams {
        qudit_dimension: 3,
        braid_turn_count: 1,
        path_perturbation_ratio: 0.05,
        temperature_mk: 20.0,
        topological_gap_mhz: 28.0,
        cavity_finesse: 22.0,
    };
    let solver = TopologicalPhaseShiftSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Quantization error <= 1.0e-4 rad
    let ideal_phase = 2.0 * PI / 3.0;
    assert!(
        (metrics.quantized_phase_rad - ideal_phase).abs() <= 1.0e-4,
        "Expected phase near 2*pi/3 ({}), got {}, error={}",
        ideal_phase,
        metrics.quantized_phase_rad,
        metrics.phase_quantization_error_rad
    );
    assert!(
        metrics.phase_quantization_error_rad <= 1.0e-4,
        "Expected quantization error <= 1.0e-4 rad, got {}",
        metrics.phase_quantization_error_rad
    );

    // Invariant 2: Perturbation phase error <= 0.01 rad
    assert!(
        metrics.perturbation_phase_error_rad <= 0.01,
        "Expected perturbation phase error <= 0.01 rad, got {}",
        metrics.perturbation_phase_error_rad
    );

    // Invariant 3: Monodromy unitarity error <= 1.0e-6
    assert!(
        metrics.monodromy_unitarity_error <= 1.0e-6,
        "Expected unitarity error <= 1.0e-6, got {}",
        metrics.monodromy_unitarity_error
    );

    // Invariant 4: Fringe contrast >= 90.0%
    assert!(
        metrics.fringe_contrast_pct >= 90.0,
        "Expected fringe contrast >= 90.0%, got {}%",
        metrics.fringe_contrast_pct
    );

    // Test sweep of path deformations (-15% to +15%)
    let sweep = solver.sweep_perturbation(30);
    for pt in &sweep {
        assert!(
            pt.phase_deviation_rad <= 0.01,
            "Perturbation {}% produced unacceptable phase deviation {} rad",
            pt.perturbation_pct,
            pt.phase_deviation_rad
        );
        assert!(pt.visibility_pct >= 85.0);
    }
}

#[test]
fn test_protected_qudit_crossbar_and_entanglement() {
    let params = ProtectedQuditParams {
        dimension_d: 3,
        qudit_count: 2,
        gate_duration_ns: 85.0,
        crossbar_coupling_mhz: 18.0,
        dephasing_time_us: 65.0,
        selected_gate: QuditGateType::ControlledSum,
    };
    let solver = ProtectedQuditCrossbarSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Hilbert space dimension d >= 3
    assert!(metrics.dimension_d >= 3, "Expected d >= 3, got {}", metrics.dimension_d);

    // Invariant 2: Single-qudit transversal gate fidelity >= 99.0%
    assert!(
        metrics.single_qudit_gate_fidelity_pct >= 99.0,
        "Expected single-qudit fidelity >= 99.0%, got {}%",
        metrics.single_qudit_gate_fidelity_pct
    );

    // Invariant 3: Two-qudit entangling fidelity >= 98.5%
    assert!(
        metrics.two_qudit_entangling_fidelity_pct >= 98.5,
        "Expected two-qudit entangling fidelity >= 98.5%, got {}%",
        metrics.two_qudit_entangling_fidelity_pct
    );

    // Invariant 4: Entangled qudit generalized concurrence C >= 0.92
    assert!(
        metrics.entangled_concurrence >= 0.92,
        "Expected concurrence >= 0.92, got {}",
        metrics.entangled_concurrence
    );

    // Invariant 5: Leakage suppression >= 30.0 dB
    assert!(
        metrics.leakage_suppression_db >= 30.0,
        "Expected leakage suppression >= 30.0 dB, got {} dB",
        metrics.leakage_suppression_db
    );

    // Test density matrix structure
    let rho = solver.compute_density_matrix();
    assert!(!rho.is_empty());
    let trace: f64 = rho.iter().filter(|e| e.row == e.col).map(|e| e.magnitude).sum();
    assert!(
        (trace - 1.0).abs() <= 0.05,
        "Expected density matrix trace near 1.0, got {}",
        trace
    );
}

#[test]
fn test_cryogenic_qudit_bus_acoustics() {
    let params = CryogenicQuditBusParams {
        operating_temperature_k: 0.020,
        bus_frequency_ghz: 5.0,
        channel_count: 4,
        bus_length_um: 150.0,
        acoustic_velocity_ms: 3200.0,
        internal_quality_factor: 2.0e6,
        piezoelectric_efficiency: 0.975,
    };
    let solver = CryogenicQuditBusSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Invariant 1: Thermal noise occupancy n_th <= 0.05 quanta
    assert!(
        metrics.thermal_noise_occupancy <= 0.05,
        "Expected thermal occupancy <= 0.05 quanta, got {}",
        metrics.thermal_noise_occupancy
    );

    // Invariant 2: Channel cross-isolation >= 38.0 dB
    assert!(
        metrics.channel_cross_isolation_db >= 38.0,
        "Expected isolation >= 38.0 dB, got {} dB",
        metrics.channel_cross_isolation_db
    );

    // Invariant 3: Forward insertion loss <= 0.45 dB
    assert!(
        metrics.bus_insertion_loss_db <= 0.45,
        "Expected insertion loss <= 0.45 dB, got {} dB",
        metrics.bus_insertion_loss_db
    );

    // Invariant 4: Dephasing lifetime T_2^* >= 50.0 us
    assert!(
        metrics.dephasing_lifetime_us >= 50.0,
        "Expected dephasing lifetime >= 50.0 us, got {} us",
        metrics.dephasing_lifetime_us
    );

    // Test frequency sweep (odd count 51 ensures f0 center point is sampled exactly)
    let sweep = solver.sweep_frequency(51);
    assert_eq!(sweep.len(), 51);
    let max_s21 = sweep.iter().map(|p| p.transmission_s21_db).fold(f64::NEG_INFINITY, f64::max);
    assert!(max_s21 >= -0.50, "Expected peak S21 >= -0.50 dB, got {} dB", max_s21);
    let min_iso = sweep.iter().map(|p| p.isolation_s31_db).fold(f64::INFINITY, f64::min);
    assert!(min_iso <= -38.0, "Expected isolation <= -38.0 dB, got {} dB", min_iso);
}

#[test]
fn test_10_point_physics_audit() {
    let processor = AnyonInterferometerQuditProcessor::new(
        MultiTerminalInterferometerParams::default(),
        TopologicalPhaseShiftParams::default(),
        ProtectedQuditParams::default(),
        CryogenicQuditBusParams::default(),
    );

    let audit = processor.evaluate_audit();
    let (passed, total) = audit.score();

    println!("Phase 456 Anyon Interferometer & Qudit Crossbar Audit Score: {}/{}", passed, total);
    assert_eq!(total, 10, "Expected 10 audit criteria");
    assert_eq!(passed, 10, "Expected all 10 audit criteria to pass, got: {:?}", audit);
    assert!(audit.is_pass(), "Audit must pass 10/10");
}
