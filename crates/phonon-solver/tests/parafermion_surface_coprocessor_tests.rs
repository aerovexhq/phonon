#![deny(unsafe_code)]

//! Integration and verification test suite for Phase 461:
//! Quantum Metamaterial Non-Abelian Parafermion Lattice Co-Processor & Universal Quantum Acoustic Surface Engine.

use phonon_solver::parafermion_surface_coprocessor::{
    CryogenicCoprocessorParams, CryogenicCoprocessorSolver, ParafermionSurfaceCodeParams,
    ParafermionSurfaceCodeSolver, ParafermionSurfaceProcessor, SurfaceParafermionParams,
    SurfaceParafermionSolver, UniversalQuditGate,
};
use std::f64::consts::PI;

#[test]
fn test_parafermion_lattice_algebra_and_braiding() {
    // 1. Z_4 parafermion test
    let params_z4 = SurfaceParafermionParams {
        parafermion_order_m: 4,
        mode_count: 6,
        pairing_coupling_mhz: 3.5,
        chern_gap_mhz: 18.0,
        braid_duration_ns: 120.0,
        domain_wall_separation_um: 15.0,
    };
    let solver_z4 = SurfaceParafermionSolver::new(params_z4);
    let metrics_z4 = solver_z4.evaluate_metrics();

    // Check commutation phase theta = 2*pi / 4 = pi / 2
    let expected_theta_z4 = PI / 2.0;
    assert!(
        (metrics_z4.commutation_phase_rad - expected_theta_z4).abs() <= 1e-4,
        "Z_4 commutation phase {} rad deviates from pi/2",
        metrics_z4.commutation_phase_rad
    );
    assert!(
        metrics_z4.topological_gap_mhz >= 2.0,
        "Topological protection gap {} MHz < 2.0 MHz",
        metrics_z4.topological_gap_mhz
    );
    assert!(
        metrics_z4.artin_braid_error <= 1e-4,
        "Artin braid relation error {} > 1e-4",
        metrics_z4.artin_braid_error
    );
    assert!(
        metrics_z4.braid_fidelity_percent >= 99.5,
        "Adiabatic braid fidelity {} % < 99.5 %",
        metrics_z4.braid_fidelity_percent
    );
    assert!(
        metrics_z4.diabatic_leakage_prob <= 1e-4,
        "Diabatic transition leakage {} > 1e-4",
        metrics_z4.diabatic_leakage_prob
    );
    assert!(
        metrics_z4.localization_length_um > 0.0 && metrics_z4.localization_length_um < 10.0,
        "Unexpected localization length: {} um",
        metrics_z4.localization_length_um
    );

    // Verify spatial profiles
    let profiles = solver_z4.compute_spatial_profiles(40);
    assert!(!profiles.is_empty());
    for p in &profiles {
        assert!(p.mode_amplitude >= 0.0 && p.mode_amplitude <= 1.0);
    }

    // Verify braid trajectory
    let trajectory = solver_z4.compute_braid_trajectory(25);
    assert_eq!(trajectory.len(), 25);
    assert_eq!(trajectory[0].time_ns, 0.0);
    assert!((trajectory[24].exchange_angle_rad - PI).abs() <= 0.05);
    for pt in &trajectory {
        assert!(pt.instantaneous_gap_mhz >= 1.5);
        assert!(pt.ground_state_overlap >= 0.98);
    }

    // 2. Z_3 parafermion test
    let params_z3 = SurfaceParafermionParams {
        parafermion_order_m: 3,
        mode_count: 4,
        pairing_coupling_mhz: 3.5,
        chern_gap_mhz: 18.0,
        braid_duration_ns: 120.0,
        domain_wall_separation_um: 15.0,
    };
    let solver_z3 = SurfaceParafermionSolver::new(params_z3);
    let metrics_z3 = solver_z3.evaluate_metrics();
    let expected_theta_z3 = 2.0 * PI / 3.0;
    assert!(
        (metrics_z3.commutation_phase_rad - expected_theta_z3).abs() <= 1e-4,
        "Z_3 commutation phase {} rad deviates from 2*pi/3",
        metrics_z3.commutation_phase_rad
    );
    assert!(metrics_z3.topological_gap_mhz >= 2.0);
}

#[test]
fn test_quantum_acoustic_surface_code_stabilizers_and_readout() {
    let params_d3 = ParafermionSurfaceCodeParams {
        code_distance: 3,
        qudit_dimension_m: 4,
        physical_error_rate: 0.005,
        dispersive_shift_chi_mhz: 4.2,
        cavity_linewidth_mhz: 0.8,
        integration_time_ns: 150.0,
    };
    let solver_d3 = ParafermionSurfaceCodeSolver::new(params_d3);
    let metrics_d3 = solver_d3.evaluate_metrics();

    // d = 3 has 3^2 + 2^2 = 13? Wait: d^2 + (d-1)^2 = 9 + 4 = 13.
    assert_eq!(metrics_d3.total_data_qudits, 13);
    assert_eq!(metrics_d3.total_syndrome_ancillas, 12); // 2 * 3 * 2 = 12
    assert_eq!(metrics_d3.stabilizer_commutator_residual, 0.0);
    assert!(
        metrics_d3.logical_error_rate <= 1.0e-4,
        "Logical error rate {} > 1.0e-4",
        metrics_d3.logical_error_rate
    );
    assert!(
        metrics_d3.parity_readout_splitting_mhz >= 3.0,
        "Parity readout splitting {} MHz < 3.0 MHz",
        metrics_d3.parity_readout_splitting_mhz
    );
    assert!(
        metrics_d3.readout_snr_db >= 16.0,
        "Readout SNR {} dB < 16.0 dB",
        metrics_d3.readout_snr_db
    );
    assert!(
        metrics_d3.readout_fidelity_percent >= 99.5,
        "Readout fidelity {} % < 99.5 %",
        metrics_d3.readout_fidelity_percent
    );

    // Verify distance d = 5 patch
    let params_d5 = ParafermionSurfaceCodeParams {
        code_distance: 5,
        qudit_dimension_m: 4,
        physical_error_rate: 0.005,
        dispersive_shift_chi_mhz: 4.2,
        cavity_linewidth_mhz: 0.8,
        integration_time_ns: 150.0,
    };
    let solver_d5 = ParafermionSurfaceCodeSolver::new(params_d5);
    let metrics_d5 = solver_d5.evaluate_metrics();
    assert_eq!(metrics_d5.total_data_qudits, 25 + 16); // 41
    assert_eq!(metrics_d5.total_syndrome_ancillas, 2 * 5 * 4); // 40

    // Higher distance achieves lower logical error rate
    assert!(
        metrics_d5.logical_error_rate <= metrics_d3.logical_error_rate,
        "d=5 error rate {} not suppressed relative to d=3 {}",
        metrics_d5.logical_error_rate,
        metrics_d3.logical_error_rate
    );

    // Dispersive cavity spectra computation
    let spectra = solver_d3.compute_readout_spectra(35);
    assert_eq!(spectra.len(), 35);
    // Peak transmission should be at 0 dB near resonant frequencies
    let max_even = spectra.iter().map(|s| s.transmission_even_db).fold(f64::NEG_INFINITY, f64::max);
    let max_odd = spectra.iter().map(|s| s.transmission_odd_db).fold(f64::NEG_INFINITY, f64::max);
    assert!(max_even >= -3.0, "Even transmission peak {} dB < -3.0 dB", max_even);
    assert!(max_odd >= -3.0, "Odd transmission peak {} dB < -3.0 dB", max_odd);

    // Threshold scaling curves verify crossing at fault tolerance threshold p_th = 1.5%
    let scaling = solver_d3.compute_threshold_scaling(25);
    assert_eq!(scaling.len(), 25);
    for pt in &scaling {
        if pt.physical_error < 0.014 {
            assert!(
                pt.logical_error_d5 < pt.logical_error_d3,
                "Sub-threshold d=5 error {} not lower than d=3 {} at p={}",
                pt.logical_error_d5,
                pt.logical_error_d3,
                pt.physical_error
            );
        } else if pt.physical_error > 0.016 {
            assert!(
                pt.logical_error_d5 > pt.logical_error_d3,
                "Supra-threshold d=5 error {} not higher than d=3 {} at p={}",
                pt.logical_error_d5,
                pt.logical_error_d3,
                pt.physical_error
            );
        }
    }
}

#[test]
fn test_cryogenic_coprocessor_compilation_and_entanglement() {
    let params = CryogenicCoprocessorParams {
        logical_qudit_count: 4,
        operating_temp_k: 0.015,
        acoustic_freq_ghz: 4.8,
        clock_rate_khz: 650.0,
        bus_coupling_mhz: 12.0,
        crossbar_isolation_db: 45.0,
    };
    let solver = CryogenicCoprocessorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.thermal_phonon_occupancy <= 1.0e-3,
        "Thermal phonon occupancy {} > 1.0e-3 at 15 mK",
        metrics.thermal_phonon_occupancy
    );
    assert!(
        metrics.effective_clock_khz >= 500.0,
        "Clock rate {} kHz < 500.0 kHz",
        metrics.effective_clock_khz
    );
    assert!(
        metrics.entanglement_concurrence >= 0.90,
        "Entanglement concurrence {} < 0.90",
        metrics.entanglement_concurrence
    );
    assert!(
        metrics.bus_insertion_loss_db <= 0.35,
        "Bus insertion loss {} dB > 0.35 dB",
        metrics.bus_insertion_loss_db
    );
    assert!(
        metrics.inter_qudit_isolation_db >= 42.0,
        "Inter-qudit isolation {} dB < 42.0 dB",
        metrics.inter_qudit_isolation_db
    );
    assert!(
        metrics.single_qudit_gate_fidelity_percent >= 99.8,
        "Single-qudit gate fidelity {} % < 99.8 %",
        metrics.single_qudit_gate_fidelity_percent
    );
    assert!(
        metrics.two_qudit_gate_fidelity_percent >= 99.2,
        "Two-qudit gate fidelity {} % < 99.2 %",
        metrics.two_qudit_gate_fidelity_percent
    );

    // Gate sequence compilation
    let test_gates = [
        UniversalQuditGate::Identity,
        UniversalQuditGate::GeneralizedHadamardF4,
        UniversalQuditGate::PhaseShiftS4,
        UniversalQuditGate::ClockZ4,
        UniversalQuditGate::ShiftX4,
        UniversalQuditGate::ControlledZ4,
    ];
    let compiled = solver.compile_gate_sequence(&test_gates);
    assert_eq!(compiled.len(), 6);
    for report in &compiled {
        assert!(report.duration_ns > 0.0);
        assert!(report.fidelity_percent >= 99.0);
        assert!(report.target_qudit < 4);
    }
}

#[test]
fn test_parafermion_surface_processor_10_point_audit() {
    let processor = ParafermionSurfaceProcessor::new(
        SurfaceParafermionParams::default(),
        ParafermionSurfaceCodeParams::default(),
        CryogenicCoprocessorParams::default(),
    );

    let audit = processor.audit_system();
    let (passed, total) = audit.score();

    assert_eq!(total, 10, "Total audit criteria must be 10");
    assert_eq!(
        passed, 10,
        "Failed audit items. Score: ({}/{})\nAudit: {:#?}",
        passed, total, audit
    );
    assert!(audit.is_pass(), "Audit report is_pass() must return true");

    // Explicitly verify each item
    assert!(audit.parafermion_commutation_algebra, "Criterion 1 failed");
    assert!(audit.topological_protection_gap, "Criterion 2 failed");
    assert!(audit.artin_braid_relations, "Criterion 3 failed");
    assert!(audit.adiabatic_braid_fidelity, "Criterion 4 failed");
    assert!(audit.stabilizer_algebra_commutation, "Criterion 5 failed");
    assert!(audit.logical_qudit_error_suppression, "Criterion 6 failed");
    assert!(audit.dispersive_parity_readout, "Criterion 7 failed");
    assert!(audit.cryogenic_thermal_occupancy, "Criterion 8 failed");
    assert!(audit.universal_qudit_concurrence, "Criterion 9 failed");
    assert!(audit.inter_qudit_crosstalk_isolation, "Criterion 10 failed");
}

#[test]
fn test_fault_tolerant_threshold_and_thermal_robustness() {
    // Check below threshold
    let params_sub = ParafermionSurfaceCodeParams {
        code_distance: 5,
        qudit_dimension_m: 4,
        physical_error_rate: 0.002, // Well below 1.5% threshold
        ..Default::default()
    };
    let solver_sub = ParafermionSurfaceCodeSolver::new(params_sub);
    let metrics_sub = solver_sub.evaluate_metrics();
    assert!(
        metrics_sub.logical_error_rate <= 1.0e-4,
        "Sub-threshold logical error rate {} > 1.0e-4",
        metrics_sub.logical_error_rate
    );

    // Verify thermal scaling at 15 mK vs 4.2 K
    let coproc_15mk = CryogenicCoprocessorSolver::new(CryogenicCoprocessorParams {
        operating_temp_k: 0.015,
        ..Default::default()
    });
    let coproc_4k = CryogenicCoprocessorSolver::new(CryogenicCoprocessorParams {
        operating_temp_k: 4.2,
        ..Default::default()
    });

    let m_15mk = coproc_15mk.evaluate_metrics();
    let m_4k = coproc_4k.evaluate_metrics();

    assert!(m_15mk.thermal_phonon_occupancy < 1.0e-3);
    assert!(m_4k.thermal_phonon_occupancy > m_15mk.thermal_phonon_occupancy);
    assert!(m_15mk.entanglement_concurrence >= m_4k.entanglement_concurrence);
}
