#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic topological
//! chiral parafermionic Josephson junctions and non-Abelian readout interferometers.

use phonon_models::parafermionic_josephson_interferometer::ParafermionicJosephsonInterferometerParams;
use phonon_solver::parafermionic_josephson_interferometer::ParafermionicJosephsonInterferometerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ParafermionicJosephsonInterferometerParams::new(
        1.0,     // below 2.0
        1.0,     // below 2.0 meV
        0.5,     // below 1.0 meV
        0.5,     // below 1.0 GHz
        0.5,     // below 1.0 mK
        0.1,     // below 0.5 uW
        20.0,    // below 50.0 nm
        0.20,    // below 0.40
    );
    assert_eq!(underflow.parafermion_order_m, 2.0);
    assert_eq!(underflow.josephson_coupling_energy_mev, 2.0);
    assert_eq!(underflow.superconducting_pairing_gap_mev, 1.0);
    assert_eq!(underflow.acoustic_resonator_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_readout_power_uw, 0.5);
    assert_eq!(underflow.junction_length_nm, 50.0);
    assert_eq!(underflow.barrier_transparency, 0.40);

    // Test values strictly above physical maximum bounds
    let overflow = ParafermionicJosephsonInterferometerParams::new(
        8.0,     // above 6.0
        60.0,    // above 45.0 meV
        35.0,    // above 25.0 meV
        18.0,    // above 12.0 GHz
        80.0,    // above 50.0 mK
        50.0,    // above 30.0 uW
        1200.0,  // above 800.0 nm
        1.05,    // above 0.98
    );
    assert_eq!(overflow.parafermion_order_m, 6.0);
    assert_eq!(overflow.josephson_coupling_energy_mev, 45.0);
    assert_eq!(overflow.superconducting_pairing_gap_mev, 25.0);
    assert_eq!(overflow.acoustic_resonator_frequency_ghz, 12.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_readout_power_uw, 30.0);
    assert_eq!(overflow.junction_length_nm, 800.0);
    assert_eq!(overflow.barrier_transparency, 0.98);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ParafermionicJosephsonInterferometerParams::default();
    assert_eq!(params.parafermion_order_m, 3.0);
    assert_eq!(params.josephson_coupling_energy_mev, 20.0);
    assert_eq!(params.superconducting_pairing_gap_mev, 12.5);
    assert_eq!(params.acoustic_resonator_frequency_ghz, 6.2);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_readout_power_uw, 5.0);
    assert_eq!(params.junction_length_nm, 220.0);
    assert_eq!(params.barrier_transparency, 0.85);

    let solver = ParafermionicJosephsonInterferometerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.state_readout_fidelity >= 0.9980,
        "State readout fidelity must be >= 0.9980, got {:.6}",
        metrics.state_readout_fidelity
    );
    assert!(
        metrics.parafermionic_retention_fraction >= 0.9970,
        "Parafermionic retention fraction must be >= 0.9970, got {:.6}",
        metrics.parafermionic_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_junction_crosstalk_isolation_db >= 54.0,
        "Inter-junction crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_junction_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 12.0,
        "Topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_parafermion_order_m_scaling() {
    let mut low_m = ParafermionicJosephsonInterferometerParams::default();
    low_m.parafermion_order_m = 2.0;
    let mut high_m = ParafermionicJosephsonInterferometerParams::default();
    high_m.parafermion_order_m = 6.0;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_m);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_m);

    assert!(
        high_solver.compute_state_readout_fidelity()
            > low_solver.compute_state_readout_fidelity(),
        "Higher parafermion order m must enhance state readout fidelity"
    );
    assert!(
        high_solver.compute_parafermionic_retention_fraction()
            > low_solver.compute_parafermionic_retention_fraction(),
        "Higher parafermion order m must boost quantum retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher parafermion order m must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher parafermion order m must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_josephson_coupling_energy_scaling() {
    let mut low_ej = ParafermionicJosephsonInterferometerParams::default();
    low_ej.josephson_coupling_energy_mev = 3.0;
    let mut high_ej = ParafermionicJosephsonInterferometerParams::default();
    high_ej.josephson_coupling_energy_mev = 42.0;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_ej);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_ej);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher Josephson coupling energy must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_state_readout_fidelity()
            > low_solver.compute_state_readout_fidelity(),
        "Higher Josephson coupling energy must improve state readout fidelity"
    );
    assert!(
        high_solver.compute_parafermionic_retention_fraction()
            > low_solver.compute_parafermionic_retention_fraction(),
        "Higher Josephson coupling energy must enhance retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher Josephson coupling energy must suppress dephasing rate"
    );
}

#[test]
fn test_superconducting_pairing_gap_scaling() {
    let mut low_sc = ParafermionicJosephsonInterferometerParams::default();
    low_sc.superconducting_pairing_gap_mev = 2.0;
    let mut high_sc = ParafermionicJosephsonInterferometerParams::default();
    high_sc.superconducting_pairing_gap_mev = 24.0;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_sc);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_sc);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher superconducting pairing gap must widen the protection gap"
    );
    assert!(
        high_solver.compute_state_readout_fidelity()
            > low_solver.compute_state_readout_fidelity(),
        "Higher superconducting pairing gap must boost state readout fidelity"
    );
    assert!(
        high_solver.compute_inter_junction_crosstalk_isolation_db()
            > low_solver.compute_inter_junction_crosstalk_isolation_db(),
        "Higher superconducting pairing gap must increase inter-junction crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher superconducting pairing gap must suppress thermal dephasing rate"
    );
}

#[test]
fn test_acoustic_resonator_frequency_scaling() {
    let mut low_f = ParafermionicJosephsonInterferometerParams::default();
    low_f.acoustic_resonator_frequency_ghz = 1.5;
    let mut high_f = ParafermionicJosephsonInterferometerParams::default();
    high_f.acoustic_resonator_frequency_ghz = 11.5;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_f);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_f);

    assert!(
        high_solver.compute_state_readout_fidelity()
            > low_solver.compute_state_readout_fidelity(),
        "Higher acoustic resonator frequency must improve state readout fidelity"
    );
    assert!(
        high_solver.compute_parafermionic_retention_fraction()
            > low_solver.compute_parafermionic_retention_fraction(),
        "Higher acoustic resonator frequency must increase retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic resonator frequency must reduce transit dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = ParafermionicJosephsonInterferometerParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = ParafermionicJosephsonInterferometerParams::default();
    high_temp.cryogenic_temperature_mk = 45.0;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_temp);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_temp);

    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must suppress thermal dephasing"
    );
    assert!(
        low_solver.compute_state_readout_fidelity()
            > high_solver.compute_state_readout_fidelity(),
        "Lower cryogenic temperature must improve state readout fidelity"
    );
    assert!(
        low_solver.compute_parafermionic_retention_fraction()
            > high_solver.compute_parafermionic_retention_fraction(),
        "Lower cryogenic temperature must enhance parafermionic retention fraction"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower cryogenic temperature must sustain a wider protection gap"
    );
}

#[test]
fn test_microwave_readout_power_scaling() {
    let mut low_power = ParafermionicJosephsonInterferometerParams::default();
    low_power.microwave_readout_power_uw = 1.0;
    let mut high_power = ParafermionicJosephsonInterferometerParams::default();
    high_power.microwave_readout_power_uw = 28.0;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_power);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_power);

    assert!(
        high_solver.compute_state_readout_fidelity()
            > low_solver.compute_state_readout_fidelity(),
        "Higher microwave readout power must elevate state readout fidelity"
    );
    assert!(
        high_solver.compute_parafermionic_retention_fraction()
            > low_solver.compute_parafermionic_retention_fraction(),
        "Higher microwave readout power must improve retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave readout power must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave readout power must suppress dephasing rate"
    );
}

#[test]
fn test_junction_length_scaling() {
    let mut short_len = ParafermionicJosephsonInterferometerParams::default();
    short_len.junction_length_nm = 70.0;
    let mut long_len = ParafermionicJosephsonInterferometerParams::default();
    long_len.junction_length_nm = 720.0;

    let short_solver = ParafermionicJosephsonInterferometerSolver::new(short_len);
    let long_solver = ParafermionicJosephsonInterferometerSolver::new(long_len);

    assert!(
        short_solver.compute_state_readout_fidelity()
            > long_solver.compute_state_readout_fidelity(),
        "Shorter junction length must boost state readout fidelity via ballistic Andreev overlap"
    );
    assert!(
        short_solver.compute_parafermionic_retention_fraction()
            > long_solver.compute_parafermionic_retention_fraction(),
        "Shorter junction length must enhance quantum state retention fraction"
    );
    assert!(
        short_solver.compute_topological_protection_gap_mhz()
            > long_solver.compute_topological_protection_gap_mhz(),
        "Shorter junction length must widen the topological protection gap"
    );
    assert!(
        short_solver.compute_topological_mode_dephasing_rate_hz()
            < long_solver.compute_topological_mode_dephasing_rate_hz(),
        "Shorter junction length must suppress topological dephasing rate"
    );
}

#[test]
fn test_barrier_transparency_scaling() {
    let mut low_trans = ParafermionicJosephsonInterferometerParams::default();
    low_trans.barrier_transparency = 0.45;
    let mut high_trans = ParafermionicJosephsonInterferometerParams::default();
    high_trans.barrier_transparency = 0.95;

    let low_solver = ParafermionicJosephsonInterferometerSolver::new(low_trans);
    let high_solver = ParafermionicJosephsonInterferometerSolver::new(high_trans);

    assert!(
        high_trans.barrier_transparency > low_trans.barrier_transparency
    );
    assert!(
        high_solver.compute_inter_junction_crosstalk_isolation_db()
            > low_solver.compute_inter_junction_crosstalk_isolation_db(),
        "Higher barrier transparency must increase inter-junction crosstalk isolation"
    );
    assert!(
        high_solver.compute_state_readout_fidelity()
            > low_solver.compute_state_readout_fidelity(),
        "Higher barrier transparency must boost state readout fidelity"
    );
    assert!(
        high_solver.compute_parafermionic_retention_fraction()
            > low_solver.compute_parafermionic_retention_fraction(),
        "Higher barrier transparency must enhance parafermionic retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher barrier transparency must widen the protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher barrier transparency must suppress dephasing rate"
    );
}
