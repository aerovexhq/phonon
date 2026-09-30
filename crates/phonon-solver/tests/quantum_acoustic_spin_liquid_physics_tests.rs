#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian quantum acoustic
//! fractional spin liquids and topological resonating valence bond networks.

use phonon_models::quantum_acoustic_spin_liquid::QuantumAcousticSpinLiquidParams;
use phonon_solver::quantum_acoustic_spin_liquid::QuantumAcousticSpinLiquidSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuantumAcousticSpinLiquidParams::new(
        5.0,  // below 10.0 MHz
        0.01, // below 0.05
        0.5,  // below 1.0 MHz
        0.1,  // below 0.5 MHz
        4,    // below 8
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        0,    // 0
    );
    assert_eq!(underflow.heisenberg_exchange_coupling_mhz, 10.0);
    assert_eq!(underflow.frustration_ratio_j2_j1, 0.05);
    assert_eq!(underflow.spinon_phonon_coupling_mhz, 1.0);
    assert_eq!(underflow.chiral_three_spin_scalar_chirality, 0.5);
    assert_eq!(underflow.kagome_plaquette_count, 8);
    assert_eq!(underflow.acoustic_driving_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.lattice_geometry_type, 0);

    // Test values strictly above physical maximum bounds
    let overflow = QuantumAcousticSpinLiquidParams::new(
        200.0, // above 150.0 MHz
        0.85,  // above 0.60
        50.0,  // above 30.0 MHz
        35.0,  // above 20.0 MHz
        100,   // above 64
        20.0,  // above 12.0 GHz
        80.0,  // above 50.0 mK
        5,     // above 1
    );
    assert_eq!(overflow.heisenberg_exchange_coupling_mhz, 150.0);
    assert_eq!(overflow.frustration_ratio_j2_j1, 0.60);
    assert_eq!(overflow.spinon_phonon_coupling_mhz, 30.0);
    assert_eq!(overflow.chiral_three_spin_scalar_chirality, 20.0);
    assert_eq!(overflow.kagome_plaquette_count, 64);
    assert_eq!(overflow.acoustic_driving_frequency_ghz, 12.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.lattice_geometry_type, 1);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumAcousticSpinLiquidParams::default();
    let solver = QuantumAcousticSpinLiquidSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.spinon_excitation_fidelity >= 0.9960,
        "Default spinon excitation fidelity must be >= 0.9960, got {:.6}",
        metrics.spinon_excitation_fidelity
    );
    assert!(
        metrics.topological_entanglement_entropy >= 0.6793,
        "Default topological entanglement entropy must be >= 0.6793, got {:.6}",
        metrics.topological_entanglement_entropy
    );
    assert!(
        metrics.topological_entropy_error <= 0.0020,
        "Default topological entropy error must be <= 0.0020, got {:.6}",
        metrics.topological_entropy_error
    );
    assert!(
        metrics.spin_mechanical_crosstalk_isolation_db >= 44.0,
        "Default spin-mechanical crosstalk isolation must be >= 44.0 dB, got {:.2} dB",
        metrics.spin_mechanical_crosstalk_isolation_db
    );
    assert!(
        metrics.ground_state_degeneracy_protection_db >= 40.0,
        "Default ground-state degeneracy protection must be >= 40.0 dB, got {:.2} dB",
        metrics.ground_state_degeneracy_protection_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_heisenberg_exchange_scaling() {
    let base = QuantumAcousticSpinLiquidParams::default();
    let solver_base = QuantumAcousticSpinLiquidSolver::new(base);

    let mut enhanced = base;
    enhanced.heisenberg_exchange_coupling_mhz = 120.0;
    let solver_enhanced = QuantumAcousticSpinLiquidSolver::new(enhanced);

    let fidelity_base = solver_base.compute_spinon_excitation_fidelity();
    let fidelity_enhanced = solver_enhanced.compute_spinon_excitation_fidelity();
    assert!(
        fidelity_enhanced >= fidelity_base,
        "Enhanced Heisenberg exchange coupling must improve spinon fidelity: {:.6} vs {:.6}",
        fidelity_enhanced,
        fidelity_base
    );

    let entropy_base = solver_base.compute_topological_entanglement_entropy();
    let entropy_enhanced = solver_enhanced.compute_topological_entanglement_entropy();
    assert!(
        entropy_enhanced >= entropy_base,
        "Enhanced Heisenberg exchange coupling must preserve topological entropy: {:.6} vs {:.6}",
        entropy_enhanced,
        entropy_base
    );
}

#[test]
fn test_frustration_ratio_scaling() {
    let optimal = QuantumAcousticSpinLiquidParams::default(); // frustration_ratio_j2_j1 = 0.28
    let solver_optimal = QuantumAcousticSpinLiquidSolver::new(optimal);

    let mut detuned = optimal;
    detuned.frustration_ratio_j2_j1 = 0.58; // severe detuning towards unfrustrated regime
    let solver_detuned = QuantumAcousticSpinLiquidSolver::new(detuned);

    let protection_optimal = solver_optimal.compute_ground_state_degeneracy_protection_db();
    let protection_detuned = solver_detuned.compute_ground_state_degeneracy_protection_db();
    assert!(
        protection_optimal > protection_detuned,
        "Optimal frustration ratio (0.28) must provide higher degeneracy protection: {:.2} dB vs {:.2} dB",
        protection_optimal,
        protection_detuned
    );

    let fidelity_optimal = solver_optimal.compute_spinon_excitation_fidelity();
    let fidelity_detuned = solver_detuned.compute_spinon_excitation_fidelity();
    assert!(
        fidelity_optimal > fidelity_detuned,
        "Optimal frustration ratio must maximize spinon fidelity: {:.6} vs {:.6}",
        fidelity_optimal,
        fidelity_detuned
    );
}

#[test]
fn test_spinon_phonon_coupling_scaling() {
    let base = QuantumAcousticSpinLiquidParams::default();
    let solver_base = QuantumAcousticSpinLiquidSolver::new(base);

    let mut boosted = base;
    boosted.spinon_phonon_coupling_mhz = 25.0;
    let solver_boosted = QuantumAcousticSpinLiquidSolver::new(boosted);

    let isolation_base = solver_base.compute_spin_mechanical_crosstalk_isolation_db();
    let isolation_boosted = solver_boosted.compute_spin_mechanical_crosstalk_isolation_db();
    assert!(
        isolation_boosted > isolation_base,
        "Boosted spinon-phonon coupling must increase crosstalk isolation: {:.2} dB vs {:.2} dB",
        isolation_boosted,
        isolation_base
    );
}

#[test]
fn test_scalar_chirality_scaling() {
    let base = QuantumAcousticSpinLiquidParams::default();
    let solver_base = QuantumAcousticSpinLiquidSolver::new(base);

    let mut high_chirality = base;
    high_chirality.chiral_three_spin_scalar_chirality = 18.0;
    let solver_high = QuantumAcousticSpinLiquidSolver::new(high_chirality);

    let protection_base = solver_base.compute_ground_state_degeneracy_protection_db();
    let protection_high = solver_high.compute_ground_state_degeneracy_protection_db();
    assert!(
        protection_high > protection_base,
        "High scalar chirality must enhance ground-state degeneracy protection: {:.2} dB vs {:.2} dB",
        protection_high,
        protection_base
    );
}

#[test]
fn test_kagome_plaquette_count_scaling() {
    let base = QuantumAcousticSpinLiquidParams::default(); // 24 plaquettes
    let solver_base = QuantumAcousticSpinLiquidSolver::new(base);

    let mut large_cluster = base;
    large_cluster.kagome_plaquette_count = 60;
    let solver_large = QuantumAcousticSpinLiquidSolver::new(large_cluster);

    let error_base = solver_base.compute_topological_entropy_error();
    let error_large = solver_large.compute_topological_entropy_error();
    assert!(
        error_large < error_base,
        "Larger Kagome cluster must suppress finite-size topological entropy error: {:.6} vs {:.6}",
        error_large,
        error_base
    );

    let protection_base = solver_base.compute_ground_state_degeneracy_protection_db();
    let protection_large = solver_large.compute_ground_state_degeneracy_protection_db();
    assert!(
        protection_large > protection_base,
        "Larger cluster must increase degeneracy protection: {:.2} dB vs {:.2} dB",
        protection_large,
        protection_base
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let cold = QuantumAcousticSpinLiquidParams::default(); // 10.0 mK
    let solver_cold = QuantumAcousticSpinLiquidSolver::new(cold);

    let mut warm = cold;
    warm.cryogenic_temperature_mk = 45.0; // elevated cryogenic temperature
    let solver_warm = QuantumAcousticSpinLiquidSolver::new(warm);

    let entropy_cold = solver_cold.compute_topological_entanglement_entropy();
    let entropy_warm = solver_warm.compute_topological_entanglement_entropy();
    assert!(
        entropy_cold > entropy_warm,
        "Colder operating temperature must better preserve topological entanglement entropy: {:.6} vs {:.6}",
        entropy_cold,
        entropy_warm
    );

    let error_cold = solver_cold.compute_topological_entropy_error();
    let error_warm = solver_warm.compute_topological_entropy_error();
    assert!(
        error_cold < error_warm,
        "Colder operating temperature must yield lower topological entropy error: {:.6} vs {:.6}",
        error_cold,
        error_warm
    );
}

#[test]
fn test_acoustic_driving_frequency_scaling() {
    let resonant = QuantumAcousticSpinLiquidParams::default(); // 4.6 GHz
    let solver_resonant = QuantumAcousticSpinLiquidSolver::new(resonant);

    let mut detuned = resonant;
    detuned.acoustic_driving_frequency_ghz = 10.5; // strongly detuned
    let solver_detuned = QuantumAcousticSpinLiquidSolver::new(detuned);

    let fidelity_res = solver_resonant.compute_spinon_excitation_fidelity();
    let fidelity_det = solver_detuned.compute_spinon_excitation_fidelity();
    assert!(
        fidelity_res > fidelity_det,
        "Resonant acoustic driving must yield higher spinon fidelity: {:.6} vs {:.6}",
        fidelity_res,
        fidelity_det
    );
}

#[test]
fn test_lattice_geometry_scaling() {
    let kagome = QuantumAcousticSpinLiquidParams::default(); // geometry = 0 (Kagome)
    let solver_kagome = QuantumAcousticSpinLiquidSolver::new(kagome);

    let mut triangular = kagome;
    triangular.lattice_geometry_type = 1; // geometry = 1 (Triangular)
    let solver_triangular = QuantumAcousticSpinLiquidSolver::new(triangular);

    let fidelity_kagome = solver_kagome.compute_spinon_excitation_fidelity();
    let fidelity_triangular = solver_triangular.compute_spinon_excitation_fidelity();
    assert!(
        fidelity_kagome >= fidelity_triangular,
        "Kagome geometry must offer superior or equal spinon fidelity: {:.6} vs {:.6}",
        fidelity_kagome,
        fidelity_triangular
    );
}
