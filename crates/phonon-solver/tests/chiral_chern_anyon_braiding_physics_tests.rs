#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian anyon
//! braiding in chiral acoustic Chern metamaterials and fault-tolerant phononic
//! topological qubits.

use phonon_models::chiral_chern_anyon_braiding::ChiralChernAnyonBraidingParams;
use phonon_solver::chiral_chern_anyon_braiding::ChiralChernAnyonBraidingSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = ChiralChernAnyonBraidingParams::new(
        0.5,    // below 1.0 GHz
        10.0,   // below 20.0 MHz
        2.0,    // below 5.0 MHz
        5.0,    // below 10.0 um
        500.0,  // below 1000.0 m/s
        0.05,   // below 0.1 kHz
        0.5,    // below 1.0 mK
        1,      // below 2 (order)
    );
    assert!((underflow.acoustic_center_freq_ghz - 1.0).abs() < 1e-9);
    assert!((underflow.chern_bandgap_mhz - 20.0).abs() < 1e-9);
    assert!((underflow.strain_modulation_amplitude_mhz - 5.0).abs() < 1e-9);
    assert!((underflow.braiding_arm_length_um - 10.0).abs() < 1e-9);
    assert!((underflow.anyon_wavepacket_speed_m_per_s - 1000.0).abs() < 1e-9);
    assert!((underflow.acoustic_loss_rate_khz - 0.1).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert_eq!(underflow.anyon_type_parafermion_order, 2);

    // Test values above physical maximum bounds
    let overflow = ChiralChernAnyonBraidingParams::new(
        20.0,   // above 15.0 GHz
        250.0,  // above 200.0 MHz
        60.0,   // above 50.0 MHz
        200.0,  // above 150.0 um
        7000.0, // above 6000.0 m/s
        25.0,   // above 20.0 kHz
        60.0,   // above 50.0 mK
        8,      // above 6 (order)
    );
    assert!((overflow.acoustic_center_freq_ghz - 15.0).abs() < 1e-9);
    assert!((overflow.chern_bandgap_mhz - 200.0).abs() < 1e-9);
    assert!((overflow.strain_modulation_amplitude_mhz - 50.0).abs() < 1e-9);
    assert!((overflow.braiding_arm_length_um - 150.0).abs() < 1e-9);
    assert!((overflow.anyon_wavepacket_speed_m_per_s - 6000.0).abs() < 1e-9);
    assert!((overflow.acoustic_loss_rate_khz - 20.0).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert_eq!(overflow.anyon_type_parafermion_order, 6);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralChernAnyonBraidingParams::default();
    let solver = ChiralChernAnyonBraidingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_gate_fidelity >= 0.9980,
        "Default braiding gate fidelity must be >= 0.9980, got {:.6}",
        metrics.braiding_gate_fidelity
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 18.0,
        "Default topological protection gap must be >= 18.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.anyon_collision_visibility >= 0.950,
        "Default anyon collision visibility must be >= 0.950, got {:.4}",
        metrics.anyon_collision_visibility
    );
    assert!(
        metrics.non_adiabatic_leakage_rate <= 1.0e-5,
        "Default non-adiabatic leakage rate must be <= 1.0e-5, got {:.6e}",
        metrics.non_adiabatic_leakage_rate
    );
    assert!(
        metrics.topological_qubit_coherence_ms >= 12.0,
        "Default topological qubit coherence must be >= 12.0 ms, got {:.4} ms",
        metrics.topological_qubit_coherence_ms
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_chern_gap_scaling() {
    let base = ChiralChernAnyonBraidingParams::default();
    let solver_base = ChiralChernAnyonBraidingSolver::new(base);
    let gap_base = solver_base.compute_topological_protection_gap_mhz();
    let fid_base = solver_base.compute_braiding_gate_fidelity();
    let leak_base = solver_base.compute_non_adiabatic_leakage_rate();

    // Increase Chern bandgap
    let larger_gap = ChiralChernAnyonBraidingParams::new(
        base.acoustic_center_freq_ghz,
        120.0, // increased from 80.0
        base.strain_modulation_amplitude_mhz,
        base.braiding_arm_length_um,
        base.anyon_wavepacket_speed_m_per_s,
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.anyon_type_parafermion_order,
    );
    let solver_larger = ChiralChernAnyonBraidingSolver::new(larger_gap);
    let gap_larger = solver_larger.compute_topological_protection_gap_mhz();
    let fid_larger = solver_larger.compute_braiding_gate_fidelity();
    let leak_larger = solver_larger.compute_non_adiabatic_leakage_rate();

    assert!(
        gap_larger > gap_base,
        "Larger Chern bandgap must increase topological protection gap: {:.4} > {:.4}",
        gap_larger,
        gap_base
    );
    assert!(
        fid_larger >= fid_base,
        "Larger Chern bandgap must maintain or improve braiding fidelity: {:.6} >= {:.6}",
        fid_larger,
        fid_base
    );
    assert!(
        leak_larger < leak_base,
        "Larger Chern bandgap must reduce non-adiabatic leakage: {:.6e} < {:.6e}",
        leak_larger,
        leak_base
    );
}

#[test]
fn test_temperature_degradation() {
    let base = ChiralChernAnyonBraidingParams::default();
    let solver_base = ChiralChernAnyonBraidingSolver::new(base);
    let fid_base = solver_base.compute_braiding_gate_fidelity();
    let gap_base = solver_base.compute_topological_protection_gap_mhz();
    let vis_base = solver_base.compute_anyon_collision_visibility();
    let leak_base = solver_base.compute_non_adiabatic_leakage_rate();
    let coh_base = solver_base.compute_topological_qubit_coherence_ms();

    // Elevated temperature
    let hot = ChiralChernAnyonBraidingParams::new(
        base.acoustic_center_freq_ghz,
        base.chern_bandgap_mhz,
        base.strain_modulation_amplitude_mhz,
        base.braiding_arm_length_um,
        base.anyon_wavepacket_speed_m_per_s,
        base.acoustic_loss_rate_khz,
        25.0, // increased from 15.0 mK
        base.anyon_type_parafermion_order,
    );
    let solver_hot = ChiralChernAnyonBraidingSolver::new(hot);
    let fid_hot = solver_hot.compute_braiding_gate_fidelity();
    let gap_hot = solver_hot.compute_topological_protection_gap_mhz();
    let vis_hot = solver_hot.compute_anyon_collision_visibility();
    let leak_hot = solver_hot.compute_non_adiabatic_leakage_rate();
    let coh_hot = solver_hot.compute_topological_qubit_coherence_ms();

    assert!(
        fid_hot < fid_base,
        "Elevated temperature must degrade gate fidelity: {:.6} < {:.6}",
        fid_hot,
        fid_base
    );
    assert!(
        gap_hot < gap_base,
        "Elevated temperature must reduce protection gap: {:.4} < {:.4}",
        gap_hot,
        gap_base
    );
    assert!(
        vis_hot < vis_base,
        "Elevated temperature must reduce collision visibility: {:.4} < {:.4}",
        vis_hot,
        vis_base
    );
    assert!(
        leak_hot > leak_base,
        "Elevated temperature must increase leakage rate: {:.6e} > {:.6e}",
        leak_hot,
        leak_base
    );
    assert!(
        coh_hot < coh_base,
        "Elevated temperature must degrade qubit coherence: {:.4} < {:.4}",
        coh_hot,
        coh_base
    );
}

#[test]
fn test_leakage_scaling() {
    let base = ChiralChernAnyonBraidingParams::default();
    let solver_base = ChiralChernAnyonBraidingSolver::new(base);
    let leak_base = solver_base.compute_non_adiabatic_leakage_rate();

    // Higher anyon steering speed increases Landau-Zener non-adiabatic transition rate
    let fast_anyon = ChiralChernAnyonBraidingParams::new(
        base.acoustic_center_freq_ghz,
        base.chern_bandgap_mhz,
        base.strain_modulation_amplitude_mhz,
        base.braiding_arm_length_um,
        4500.0, // increased from 3400.0 m/s
        base.acoustic_loss_rate_khz,
        base.operating_temp_m_k,
        base.anyon_type_parafermion_order,
    );
    let solver_fast = ChiralChernAnyonBraidingSolver::new(fast_anyon);
    let leak_fast = solver_fast.compute_non_adiabatic_leakage_rate();

    assert!(
        leak_fast > leak_base,
        "Higher steering velocity must increase non-adiabatic leakage: {:.6e} > {:.6e}",
        leak_fast,
        leak_base
    );
}

#[test]
fn test_coherence_scaling() {
    let base = ChiralChernAnyonBraidingParams::default();
    let solver_base = ChiralChernAnyonBraidingSolver::new(base);
    let coh_base = solver_base.compute_topological_qubit_coherence_ms();

    // Lower acoustic loss rate improves coherence lifetime
    let low_loss = ChiralChernAnyonBraidingParams::new(
        base.acoustic_center_freq_ghz,
        base.chern_bandgap_mhz,
        base.strain_modulation_amplitude_mhz,
        base.braiding_arm_length_um,
        base.anyon_wavepacket_speed_m_per_s,
        1.2, // reduced from 2.5 kHz
        base.operating_temp_m_k,
        base.anyon_type_parafermion_order,
    );
    let solver_low_loss = ChiralChernAnyonBraidingSolver::new(low_loss);
    let coh_low_loss = solver_low_loss.compute_topological_qubit_coherence_ms();

    assert!(
        coh_low_loss > coh_base,
        "Lower acoustic loss rate must increase coherence lifetime: {:.4} > {:.4}",
        coh_low_loss,
        coh_base
    );
}

#[test]
fn test_physical_compliance_flag() {
    let default_params = ChiralChernAnyonBraidingParams::default();
    let solver = ChiralChernAnyonBraidingSolver::new(default_params);
    let metrics = solver.evaluate_metrics();
    assert!(metrics.is_physically_compliant);
}
