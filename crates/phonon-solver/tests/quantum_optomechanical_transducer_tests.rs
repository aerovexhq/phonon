#![deny(unsafe_code)]

//! Integration tests for cryogenic quantum optomechanical transducer physics engine,
//! bidirectional microwave-to-optical conversion, ground-state sideband cooling,
//! and superconducting transmon qubit coherent state transfer.

use phonon_solver::quantum_optomechanical_transducer::{
    QuantumOptomechanicalTransducer, SidebandCoolingEngine, SidebandCoolingParams,
    TransductionEngine, TransductionParams, TransmonInterfaceEngine, TransmonInterfaceParams,
};

#[test]
fn test_phononic_cavity_resonance_and_high_q() {
    let params = TransductionParams::default();
    let engine = TransductionEngine::new(params);

    assert_eq!(engine.params.omega_m_ghz, 4.0);
    assert_eq!(engine.params.gamma_m_khz, 1.8);

    let q_m = (engine.params.omega_m_ghz * 1.0e6) / engine.params.gamma_m_khz;
    assert!(
        q_m >= 1.0e6,
        "Mechanical quality factor must exceed 1.0e6, got {:.2e}",
        q_m
    );
}

#[test]
fn test_electromechanical_and_optomechanical_cooperativity() {
    let params = TransductionParams::default();
    let engine = TransductionEngine::new(params);

    let c_e = engine.compute_electromechanical_cooperativity();
    let c_o = engine.compute_optomechanical_cooperativity();
    let match_ratio = engine.compute_cooperativity_matching_ratio();

    assert!(
        c_e >= 150.0,
        "Electromechanical cooperativity below threshold: {:.2}",
        c_e
    );
    assert!(
        c_o >= 150.0,
        "Optomechanical cooperativity below threshold: {:.2}",
        c_o
    );
    assert!(
        match_ratio <= 0.15,
        "Cooperativity matching ratio exceeds 15%: {:.2}%",
        match_ratio * 100.0
    );
}

#[test]
fn test_bidirectional_transduction_efficiency() {
    let params = TransductionParams::default();
    let engine = TransductionEngine::new(params);

    let eta_trans = engine.compute_peak_transduction_efficiency();
    assert!(
        eta_trans >= 0.70,
        "Peak transduction efficiency below 70% target: {:.2}%",
        eta_trans * 100.0
    );
    assert!(
        eta_trans >= 0.45,
        "Peak transduction efficiency below 45% minimum roadmap requirement: {:.2}%",
        eta_trans * 100.0
    );
}

#[test]
fn test_exact_bidirectional_scattering_symmetry() {
    let params = TransductionParams::default();
    let engine = TransductionEngine::new(params);

    // Test resonance and multiple detuning points
    for detuning in [-0.5, -0.2, 0.0, 0.2, 0.5] {
        let pt = engine.compute_scattering_point(detuning);
        assert!(
            pt.asymmetry <= 1.0e-6,
            "Bidirectional asymmetry violation at detuning {:.2} MHz: {:.2e}",
            detuning,
            pt.asymmetry
        );
        assert!(
            (pt.transmission_oe - pt.transmission_eo).abs() <= 1.0e-6,
            "S_oe and S_eo mismatch at detuning {:.2} MHz",
            detuning
        );
    }
}

#[test]
fn test_transduction_3db_bandwidth() {
    let params = TransductionParams::default();
    let engine = TransductionEngine::new(params);

    let bw_mhz = engine.compute_transduction_bandwidth_mhz();
    assert!(
        bw_mhz >= 0.50,
        "Transduction bandwidth below 0.50 MHz target: {:.3} MHz",
        bw_mhz
    );
}

#[test]
fn test_ground_state_sideband_cooling() {
    let params = SidebandCoolingParams::default();
    let engine = SidebandCoolingEngine::new(params);

    let n_th = engine.compute_thermal_occupancy();
    let n_eff = engine.compute_effective_occupancy();
    let p_ground = engine.compute_ground_state_purity();

    assert!(
        n_th < 0.01,
        "Equilibrium thermal occupancy unexpectedly high at 20 mK: {:.4e}",
        n_th
    );
    assert!(
        n_eff < 0.05,
        "Sideband cooled effective phonon occupancy exceeds 0.05 limit: {:.4}",
        n_eff
    );
    assert!(
        p_ground >= 0.95,
        "Ground state probability below 95%: {:.2}%",
        p_ground * 100.0
    );
}

#[test]
fn test_input_referred_added_quantum_noise() {
    let params = SidebandCoolingParams::default();
    let engine = SidebandCoolingEngine::new(params);

    let c_e = 200.0;
    let n_add = engine.compute_added_quantum_noise(c_e);
    assert!(
        n_add < 0.50,
        "Input-referred added quantum noise exceeds 0.50 quanta: {:.3}",
        n_add
    );
}

#[test]
fn test_transmon_iswap_gate_time_and_fidelity() {
    let params = TransmonInterfaceParams::default();
    let engine = TransmonInterfaceEngine::new(params);

    let tau_swap = engine.compute_iswap_time_ns();
    assert!(
        tau_swap < 25.0,
        "iSWAP gate time exceeds 25 ns target: {:.2} ns",
        tau_swap
    );
    assert!(
        tau_swap > 2.0,
        "iSWAP gate time unreasonably small: {:.2} ns",
        tau_swap
    );

    let eta_trans = 0.80;
    let f_state = engine.compute_state_transfer_fidelity(eta_trans);
    assert!(
        f_state >= 0.950,
        "State transfer fidelity below 0.950: {:.4}",
        f_state
    );
}

#[test]
fn test_transmon_bell_pair_concurrence() {
    let params = TransmonInterfaceParams::default();
    let engine = TransmonInterfaceEngine::new(params);

    let eta_trans = 0.80;
    let concurrence = engine.compute_bell_pair_concurrence(eta_trans);
    assert!(
        concurrence >= 0.90,
        "Bell-pair entanglement concurrence below 0.90 target: {:.4}",
        concurrence
    );
}

#[test]
fn test_comprehensive_10_point_physics_audit() {
    let transducer = QuantumOptomechanicalTransducer::default();
    let audit = transducer.audit_transducer();

    assert_eq!(
        audit.total_count, 10,
        "Expected exactly 10 audit items, got {}",
        audit.total_count
    );
    assert_eq!(
        audit.passed_count, 10,
        "All 10 audit items must pass. Passed: {}/10. Failed items: {:?}",
        audit.passed_count,
        audit
            .items
            .iter()
            .filter(|i| !i.passed)
            .map(|i| &i.name)
            .collect::<Vec<_>>()
    );
    assert!(
        audit.is_fully_compliant,
        "Transducer system must be 100% physically compliant with roadmap"
    );
}
