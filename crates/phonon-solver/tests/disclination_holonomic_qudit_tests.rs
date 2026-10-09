#![deny(unsafe_code)]

//! Integration test suite for Topological Acoustic Second-Order Disclination Cavity
//! & Non-Abelian Holonomic Quantum Qudit Processor (Phase 463).

use phonon_solver::disclination_holonomic_qudit::{
    DisclinationCavityParams, DisclinationCavitySolver, DisclinationHolonomicProcessor,
    FrankAngleKind, HolonomicQuditEngine, HolonomicQuditParams, QuantumQuditProcessorEngine,
    QuantumQuditProcessorParams, QuditDimension, QuditHolonomicGateKind,
};

#[test]
fn test_disclination_cavity_physics_and_fractional_charge() {
    let params = DisclinationCavityParams {
        bare_frequency_mhz: 150.0,
        frank_angle: FrankAngleKind::C4Minus90Deg,
        intracell_hopping_gamma_mhz: 3.5,
        intercell_hopping_lambda_mhz: 12.5,
        lattice_size_n: 8,
        cavity_loss_rate_khz: 3.2,
        core_radius_um: 25.0,
    };
    let solver = DisclinationCavitySolver::new(params);
    let metrics = solver.solve();

    // 1. Bulk bandgap Delta_bulk = 2 * |12.5 - 3.5| = 18.0 MHz >= 4.0 MHz
    assert!(
        metrics.bulk_bandgap_mhz >= 4.0,
        "Bulk bandgap {:.2} MHz is less than 4.0 MHz",
        metrics.bulk_bandgap_mhz
    );
    assert!((metrics.bulk_bandgap_mhz - 18.0).abs() < 1e-4);

    // 2. Fractional topological bound charge at core: Q_nom = 0.250 for C4
    assert!(
        metrics.fractional_charge_error <= 0.05,
        "Fractional charge error {:.4} exceeded 0.05 limit",
        metrics.fractional_charge_error
    );
    assert!((metrics.fractional_topological_charge - 0.250).abs() < 0.01);

    // 3. Core spatial energy confinement >= 82.0%
    assert!(
        metrics.core_energy_confinement_percent >= 82.0,
        "Core confinement {:.2}% is below 82.0%",
        metrics.core_energy_confinement_percent
    );

    // 4. Acoustic cavity quality factor Q >= 25,000
    assert!(
        metrics.cavity_quality_factor >= 25_000.0,
        "Cavity Q factor {:.1} is below 25,000",
        metrics.cavity_quality_factor
    );

    // 5. Real-space profile and spectrum
    let spatial = solver.compute_spatial_profile();
    assert!(!spatial.is_empty());
    assert!(spatial.iter().any(|pt| pt.is_core_region));

    let spectrum = solver.compute_spectrum();
    assert!(!spectrum.is_empty());
    let core_modes: Vec<_> = spectrum.iter().filter(|m| m.is_midgap_core).collect();
    assert_eq!(core_modes.len(), 3);
}

#[test]
fn test_holonomic_qudit_gate_synthesis_and_fidelity() {
    let qutrit_params = HolonomicQuditParams {
        dimension: QuditDimension::QutritD3,
        selected_gate: QuditHolonomicGateKind::FourierF,
        loop_duration_ns: 150.0,
        loop_radius_parameter: 1.0,
        dephasing_rate_khz: 1.5,
        rotation_angle_rad: std::f64::consts::FRAC_PI_3,
    };
    let engine = HolonomicQuditEngine::new(qutrit_params);
    let metrics = engine.solve(18.0);

    // Gate fidelity >= 99.5%
    assert!(
        metrics.gate_fidelity_percent >= 99.5,
        "Gate fidelity {:.2}% below 99.5%",
        metrics.gate_fidelity_percent
    );

    // Diabatic leakage <= 1.0e-4
    assert!(
        metrics.diabatic_leakage_rate <= 1.0e-4,
        "Diabatic leakage rate {:.2e} exceeded 1e-4",
        metrics.diabatic_leakage_rate
    );

    // Adiabatic ratio tau * Delta / hbar >> 1
    assert!(
        metrics.adiabatic_ratio > 10.0,
        "Adiabatic ratio {:.1} too low",
        metrics.adiabatic_ratio
    );

    // Verify unitary matrix structure
    let matrix = engine.compute_unitary_matrix();
    assert_eq!(matrix.len(), 9); // 3x3
    for r in 0..3 {
        let row_norm_sq: f64 = matrix
            .iter()
            .filter(|e| e.row == r)
            .map(|e| e.magnitude.powi(2))
            .sum();
        assert!(
            (row_norm_sq - 1.0).abs() < 1e-4,
            "Row {} unitarity violation: {}",
            r,
            row_norm_sq
        );
    }
}

#[test]
fn test_non_abelian_wilczek_zee_commutator_norm() {
    let params = HolonomicQuditParams {
        dimension: QuditDimension::QutritD3,
        selected_gate: QuditHolonomicGateKind::ShiftX,
        loop_duration_ns: 160.0,
        ..Default::default()
    };
    let engine = HolonomicQuditEngine::new(params);
    let metrics = engine.solve(18.0);

    // Commutator norm ||[U_1, U_2]|| >= 0.50 certifying true non-Abelian holonomy
    assert!(
        metrics.non_abelian_commutator_norm >= 0.50,
        "Commutator norm {:.3} below 0.50",
        metrics.non_abelian_commutator_norm
    );

    // Parameter loop path
    let trajectory = engine.compute_parameter_loop();
    assert_eq!(trajectory.len(), 50);
    assert!(trajectory.iter().all(|pt| pt.instantaneous_fidelity > 0.99));
}

#[test]
fn test_multi_qudit_processor_entanglement_and_cryogenic_readout() {
    let params = QuantumQuditProcessorParams {
        cavity_count: 4,
        qudit_dimension: QuditDimension::QutritD3,
        dilution_temp_mk: 15.0,
        bus_coupling_mhz: 6.5,
        dispersive_shift_chi_mhz: 2.8,
        readout_resonator_linewidth_mhz: 0.65,
        measurement_duration_ns: 180.0,
        inter_cavity_distance_um: 120.0,
    };
    let engine = QuantumQuditProcessorEngine::new(params);
    let metrics = engine.solve(150.0);

    // Entangling concurrence C >= 0.90
    assert!(
        metrics.entangling_concurrence >= 0.90,
        "Concurrence {:.3} below 0.90",
        metrics.entangling_concurrence
    );

    // Cryogenic thermal phonon occupancy <= 1.0e-3 at 15 mK
    assert!(
        metrics.thermal_phonon_occupancy <= 1.0e-3,
        "Thermal occupancy {:.2e} exceeded 1e-3",
        metrics.thermal_phonon_occupancy
    );

    // Routing insertion loss <= 0.35 dB and crosstalk isolation >= 40.0 dB
    assert!(
        metrics.inter_qudit_insertion_loss_db <= 0.35,
        "Insertion loss {:.2} dB exceeded 0.35 dB",
        metrics.inter_qudit_insertion_loss_db
    );
    assert!(
        metrics.crosstalk_isolation_db >= 40.0,
        "Isolation {:.1} dB below 40.0 dB",
        metrics.crosstalk_isolation_db
    );

    // Dispersive readout SNR >= 16.0 dB and fidelity >= 99.5%
    assert!(
        metrics.readout_snr_db >= 16.0,
        "Readout SNR {:.2} dB below 16.0 dB",
        metrics.readout_snr_db
    );
    assert!(
        metrics.readout_fidelity_percent >= 99.5,
        "Readout fidelity {:.2}% below 99.5%",
        metrics.readout_fidelity_percent
    );

    // Tomography and routing nodes
    let tomo = engine.compute_tomography();
    assert_eq!(tomo.len(), 9); // 3x3 for d=3
    let pop_sum: f64 = tomo.iter().map(|s| s.population).sum();
    assert!((pop_sum - 1.0).abs() < 1e-2);

    let nodes = engine.compute_routing_nodes();
    assert_eq!(nodes.len(), 4);
}

#[test]
fn test_disclination_holonomic_10_point_physics_audit() {
    let processor = DisclinationHolonomicProcessor::default();
    let audit = processor.audit();

    assert!(audit.fractional_charge_quantization);
    assert!(audit.bulk_topological_bandgap);
    assert!(audit.core_energy_confinement);
    assert!(audit.cavity_quality_factor);
    assert!(audit.wilczek_zee_non_abelian_holonomy);
    assert!(audit.holonomic_gate_fidelity);
    assert!(audit.diabatic_leakage_suppression);
    assert!(audit.entangling_concurrence);
    assert!(audit.cryogenic_thermal_occupancy);
    assert!(audit.dispersive_readout_fidelity_and_snr);

    let (score, total) = audit.score();
    assert_eq!(total, 10);
    assert_eq!(score, 10);
    assert!(audit.is_pass());
}
