#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological acoustic
//! skyrmion lattices and chiral phononic neuromorphic processing engines.

use phonon_models::topological_acoustic_skyrmion::TopologicalAcousticSkyrmionParams;
use phonon_solver::topological_acoustic_skyrmion::TopologicalAcousticSkyrmionSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = TopologicalAcousticSkyrmionParams::new(
        0.1,    // below 0.5 mJ/m^2
        2.0,    // below 5.0 pJ/m
        0.05,   // below 0.1 MJ/m^3
        0.0005, // below 0.001
        0.05,   // below 0.1 mA/um^2
        10.0,   // below 20.0 nm
        10.0,   // below 15.0 nm
        0.005,  // below 0.01 K
    );
    assert!((underflow.dmi_strength_mj_m2 - 0.5).abs() < 1e-9);
    assert!((underflow.exchange_stiffness_pj_m - 5.0).abs() < 1e-9);
    assert!((underflow.anisotropy_mj_m3 - 0.1).abs() < 1e-9);
    assert!((underflow.gilbert_damping_alpha - 0.001).abs() < 1e-9);
    assert!((underflow.acoustic_drive_current_ma_um2 - 0.1).abs() < 1e-9);
    assert!((underflow.lattice_constant_nm - 20.0).abs() < 1e-9);
    assert!((underflow.skyrmion_diameter_nm - 15.0).abs() < 1e-9);
    assert!((underflow.cryogenic_temp_k - 0.01).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = TopologicalAcousticSkyrmionParams::new(
        8.0,   // above 5.0 mJ/m^2
        45.0,  // above 30.0 pJ/m
        4.0,   // above 2.5 MJ/m^3
        0.15,  // above 0.08
        15.0,  // above 10.0 mA/um^2
        350.0, // above 200.0 nm
        180.0, // above 120.0 nm
        25.0,  // above 10.0 K
    );
    assert!((overflow.dmi_strength_mj_m2 - 5.0).abs() < 1e-9);
    assert!((overflow.exchange_stiffness_pj_m - 30.0).abs() < 1e-9);
    assert!((overflow.anisotropy_mj_m3 - 2.5).abs() < 1e-9);
    assert!((overflow.gilbert_damping_alpha - 0.08).abs() < 1e-9);
    assert!((overflow.acoustic_drive_current_ma_um2 - 10.0).abs() < 1e-9);
    assert!((overflow.lattice_constant_nm - 200.0).abs() < 1e-9);
    assert!((overflow.skyrmion_diameter_nm - 120.0).abs() < 1e-9);
    assert!((overflow.cryogenic_temp_k - 10.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TopologicalAcousticSkyrmionParams::default();
    let solver = TopologicalAcousticSkyrmionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.synaptic_state_fidelity >= 0.9960,
        "Default synaptic state fidelity must be >= 0.9960, got {:.6}",
        metrics.synaptic_state_fidelity
    );
    assert!(
        metrics.skyrmion_propagation_velocity_mps >= 850.0,
        "Default propagation velocity must be >= 850.0 m/s, got {:.4} m/s",
        metrics.skyrmion_propagation_velocity_mps
    );
    assert!(
        metrics.topological_charge_quantization_error <= 0.0030,
        "Default topological charge error must be <= 0.0030, got {:.6}",
        metrics.topological_charge_quantization_error
    );
    assert!(
        metrics.neuromorphic_energy_dissipation_aj <= 15.0,
        "Default energy dissipation must be <= 15.0 aJ, got {:.4} aJ",
        metrics.neuromorphic_energy_dissipation_aj
    );
    assert!(
        metrics.state_retention_isolation_db >= 42.0,
        "Default state retention isolation must be >= 42.0 dB, got {:.4} dB",
        metrics.state_retention_isolation_db
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_acoustic_drive_current_scaling() {
    let base = TopologicalAcousticSkyrmionParams::default();
    let solver_base = TopologicalAcousticSkyrmionSolver::new(base);

    let high_current = TopologicalAcousticSkyrmionParams::new(
        base.dmi_strength_mj_m2,
        base.exchange_stiffness_pj_m,
        base.anisotropy_mj_m3,
        base.gilbert_damping_alpha,
        5.5, // increased from 3.5 mA/um^2
        base.lattice_constant_nm,
        base.skyrmion_diameter_nm,
        base.cryogenic_temp_k,
    );
    let solver_high_current = TopologicalAcousticSkyrmionSolver::new(high_current);

    let m_base = solver_base.evaluate_metrics();
    let m_current = solver_high_current.evaluate_metrics();

    assert!(
        m_current.skyrmion_propagation_velocity_mps > m_base.skyrmion_propagation_velocity_mps,
        "Higher acoustic drive current must increase skyrmion propagation velocity"
    );
    assert!(
        m_current.neuromorphic_energy_dissipation_aj > m_base.neuromorphic_energy_dissipation_aj,
        "Higher acoustic drive current must increase synaptic energy dissipation"
    );
}

#[test]
fn test_dmi_and_anisotropy_scaling() {
    let base = TopologicalAcousticSkyrmionParams::default();
    let solver_base = TopologicalAcousticSkyrmionSolver::new(base);

    let high_dmi = TopologicalAcousticSkyrmionParams::new(
        3.2, // increased from 2.2 mJ/m^2
        base.exchange_stiffness_pj_m,
        base.anisotropy_mj_m3,
        base.gilbert_damping_alpha,
        base.acoustic_drive_current_ma_um2,
        base.lattice_constant_nm,
        base.skyrmion_diameter_nm,
        base.cryogenic_temp_k,
    );
    let solver_high_dmi = TopologicalAcousticSkyrmionSolver::new(high_dmi);

    let high_anisotropy = TopologicalAcousticSkyrmionParams::new(
        base.dmi_strength_mj_m2,
        base.exchange_stiffness_pj_m,
        1.4, // increased from 0.8 MJ/m^3
        base.gilbert_damping_alpha,
        base.acoustic_drive_current_ma_um2,
        base.lattice_constant_nm,
        base.skyrmion_diameter_nm,
        base.cryogenic_temp_k,
    );
    let solver_high_anisotropy = TopologicalAcousticSkyrmionSolver::new(high_anisotropy);

    let m_base = solver_base.evaluate_metrics();
    let m_dmi = solver_high_dmi.evaluate_metrics();
    let m_aniso = solver_high_anisotropy.evaluate_metrics();

    assert!(
        m_dmi.synaptic_state_fidelity > m_base.synaptic_state_fidelity,
        "Higher DMI strength must increase synaptic state fidelity"
    );
    assert!(
        m_dmi.state_retention_isolation_db > m_base.state_retention_isolation_db,
        "Higher DMI strength must increase state retention isolation"
    );
    assert!(
        m_dmi.topological_charge_quantization_error < m_base.topological_charge_quantization_error,
        "Higher DMI strength must reduce topological charge quantization error"
    );
    assert!(
        m_aniso.state_retention_isolation_db > m_base.state_retention_isolation_db,
        "Higher perpendicular anisotropy must increase state retention isolation"
    );
}

#[test]
fn test_gilbert_damping_scaling() {
    let base = TopologicalAcousticSkyrmionParams::default();
    let solver_base = TopologicalAcousticSkyrmionSolver::new(base);

    let high_damping = TopologicalAcousticSkyrmionParams::new(
        base.dmi_strength_mj_m2,
        base.exchange_stiffness_pj_m,
        base.anisotropy_mj_m3,
        0.035, // increased from 0.015
        base.acoustic_drive_current_ma_um2,
        base.lattice_constant_nm,
        base.skyrmion_diameter_nm,
        base.cryogenic_temp_k,
    );
    let solver_high_damping = TopologicalAcousticSkyrmionSolver::new(high_damping);

    let m_base = solver_base.evaluate_metrics();
    let m_damping = solver_high_damping.evaluate_metrics();

    assert!(
        m_damping.skyrmion_propagation_velocity_mps < m_base.skyrmion_propagation_velocity_mps,
        "Higher Gilbert damping must reduce skyrmion propagation velocity"
    );
    assert!(
        m_damping.neuromorphic_energy_dissipation_aj > m_base.neuromorphic_energy_dissipation_aj,
        "Higher Gilbert damping must increase synaptic energy dissipation"
    );
    assert!(
        m_damping.synaptic_state_fidelity < m_base.synaptic_state_fidelity,
        "Higher Gilbert damping must degrade synaptic state fidelity"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = TopologicalAcousticSkyrmionParams::default();
    let solver_base = TopologicalAcousticSkyrmionSolver::new(base);

    let warm = TopologicalAcousticSkyrmionParams::new(
        base.dmi_strength_mj_m2,
        base.exchange_stiffness_pj_m,
        base.anisotropy_mj_m3,
        base.gilbert_damping_alpha,
        base.acoustic_drive_current_ma_um2,
        base.lattice_constant_nm,
        base.skyrmion_diameter_nm,
        4.0, // increased from 1.5 K
    );
    let solver_warm = TopologicalAcousticSkyrmionSolver::new(warm);

    let m_base = solver_base.evaluate_metrics();
    let m_warm = solver_warm.evaluate_metrics();

    assert!(
        m_warm.synaptic_state_fidelity < m_base.synaptic_state_fidelity,
        "Elevated temperature must degrade synaptic state fidelity"
    );
    assert!(
        m_warm.topological_charge_quantization_error > m_base.topological_charge_quantization_error,
        "Elevated temperature must increase topological charge quantization error"
    );
    assert!(
        m_warm.state_retention_isolation_db < m_base.state_retention_isolation_db,
        "Elevated temperature must reduce state retention isolation"
    );
}

#[test]
fn test_lattice_and_diameter_scaling() {
    let base = TopologicalAcousticSkyrmionParams::default();
    let solver_base = TopologicalAcousticSkyrmionSolver::new(base);

    let large_lattice = TopologicalAcousticSkyrmionParams::new(
        base.dmi_strength_mj_m2,
        base.exchange_stiffness_pj_m,
        base.anisotropy_mj_m3,
        base.gilbert_damping_alpha,
        base.acoustic_drive_current_ma_um2,
        110.0, // increased from 65.0 nm
        base.skyrmion_diameter_nm,
        base.cryogenic_temp_k,
    );
    let solver_large_lattice = TopologicalAcousticSkyrmionSolver::new(large_lattice);

    let m_base = solver_base.evaluate_metrics();
    let m_lattice = solver_large_lattice.evaluate_metrics();

    assert!(
        m_lattice.topological_charge_quantization_error > m_base.topological_charge_quantization_error,
        "Coarser lattice relative to diameter must increase topological charge quantization error"
    );
}

#[test]
fn test_physical_compliance_thresholds() {
    let params = TopologicalAcousticSkyrmionParams::default();
    let solver = TopologicalAcousticSkyrmionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.synaptic_state_fidelity >= 0.9960);
    assert!(metrics.skyrmion_propagation_velocity_mps >= 850.0);
    assert!(metrics.topological_charge_quantization_error <= 0.0030);
    assert!(metrics.neuromorphic_energy_dissipation_aj <= 15.0);
    assert!(metrics.state_retention_isolation_db >= 42.0);
}
