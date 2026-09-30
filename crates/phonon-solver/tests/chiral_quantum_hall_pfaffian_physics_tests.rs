#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for chiral acoustic
//! quantum Hall metamaterials and non-Abelian Moore-Read Pfaffian edge waveguide synthesizers.

use phonon_models::chiral_quantum_hall_pfaffian::ChiralQuantumHallPfaffianParams;
use phonon_solver::chiral_quantum_hall_pfaffian::ChiralQuantumHallPfaffianSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralQuantumHallPfaffianParams::new(
        1.0,   // below 2.0 T
        0.20,  // below 0.40
        2.0,   // below 5.0 MHz
        0.30,  // below 0.50
        0.5,   // below 1.0 um
        0.5,   // below 1.0 mK
        20.0,  // below 50.0 nm
        0.5,   // below 1.0 GHz
    );
    assert_eq!(underflow.magnetic_field_tesla, 2.0);
    assert_eq!(underflow.fractional_filling_factor, 0.40);
    assert_eq!(underflow.pfaffian_pairing_gap_mhz, 5.0);
    assert_eq!(underflow.piezoelectric_acoustic_coupling_efficiency, 0.50);
    assert_eq!(underflow.waveguide_channel_length_um, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.inter_edge_spacing_nm, 50.0);
    assert_eq!(underflow.acoustic_driving_frequency_ghz, 1.0);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralQuantumHallPfaffianParams::new(
        25.0,   // above 18.0 T
        3.50,   // above 2.80
        80.0,   // above 60.0 MHz
        1.20,   // above 0.99
        40.0,   // above 25.0 um
        100.0,  // above 50.0 mK
        800.0,  // above 500.0 nm
        18.0,   // above 12.0 GHz
    );
    assert_eq!(overflow.magnetic_field_tesla, 18.0);
    assert_eq!(overflow.fractional_filling_factor, 2.80);
    assert_eq!(overflow.pfaffian_pairing_gap_mhz, 60.0);
    assert_eq!(overflow.piezoelectric_acoustic_coupling_efficiency, 0.99);
    assert_eq!(overflow.waveguide_channel_length_um, 25.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.inter_edge_spacing_nm, 500.0);
    assert_eq!(overflow.acoustic_driving_frequency_ghz, 12.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralQuantumHallPfaffianParams::default();
    let solver = ChiralQuantumHallPfaffianSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.pfaffian_topological_state_fidelity >= 0.9970,
        "Pfaffian state fidelity must be >= 0.9970, got {:.6}",
        metrics.pfaffian_topological_state_fidelity
    );
    assert!(
        metrics.edge_channel_isolation_db >= 46.0,
        "Edge channel isolation must be >= 46.0 dB, got {:.2} dB",
        metrics.edge_channel_isolation_db
    );
    assert!(
        metrics.neutral_mode_transmission_speed_mps >= 1400.0,
        "Neutral mode transmission speed must be >= 1400.0 m/s, got {:.2} m/s",
        metrics.neutral_mode_transmission_speed_mps
    );
    assert!(
        metrics.thermal_hall_quantization_error <= 0.0020,
        "Thermal Hall quantization error must be <= 0.0020, got {:.6}",
        metrics.thermal_hall_quantization_error
    );
    assert!(
        metrics.quasiparticle_braiding_visibility >= 0.985,
        "Quasiparticle braiding visibility must be >= 0.985, got {:.4}",
        metrics.quasiparticle_braiding_visibility
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_pfaffian_pairing_gap_scaling() {
    let low_gap = ChiralQuantumHallPfaffianParams {
        pfaffian_pairing_gap_mhz: 10.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let high_gap = ChiralQuantumHallPfaffianParams {
        pfaffian_pairing_gap_mhz: 50.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_low = ChiralQuantumHallPfaffianSolver::new(low_gap);
    let solver_high = ChiralQuantumHallPfaffianSolver::new(high_gap);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.pfaffian_topological_state_fidelity > m_low.pfaffian_topological_state_fidelity,
        "Higher pairing gap must increase topological state fidelity"
    );
    assert!(
        m_high.edge_channel_isolation_db > m_low.edge_channel_isolation_db,
        "Higher pairing gap must improve edge channel isolation"
    );
    assert!(
        m_high.neutral_mode_transmission_speed_mps > m_low.neutral_mode_transmission_speed_mps,
        "Higher pairing gap must increase neutral mode transmission velocity"
    );
    assert!(
        m_high.thermal_hall_quantization_error < m_low.thermal_hall_quantization_error,
        "Higher pairing gap must reduce thermal Hall quantization error"
    );
    assert!(
        m_high.quasiparticle_braiding_visibility > m_low.quasiparticle_braiding_visibility,
        "Higher pairing gap must increase quasiparticle braiding visibility"
    );
}

#[test]
fn test_magnetic_field_scaling() {
    let low_b = ChiralQuantumHallPfaffianParams {
        magnetic_field_tesla: 3.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let high_b = ChiralQuantumHallPfaffianParams {
        magnetic_field_tesla: 15.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_low = ChiralQuantumHallPfaffianSolver::new(low_b);
    let solver_high = ChiralQuantumHallPfaffianSolver::new(high_b);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.edge_channel_isolation_db > m_low.edge_channel_isolation_db,
        "Higher magnetic field must increase edge channel isolation"
    );
    assert!(
        m_high.neutral_mode_transmission_speed_mps > m_low.neutral_mode_transmission_speed_mps,
        "Higher magnetic field must increase neutral mode transmission speed"
    );
    assert!(
        m_high.thermal_hall_quantization_error < m_low.thermal_hall_quantization_error,
        "Higher magnetic field must decrease thermal Hall quantization error"
    );
}

#[test]
fn test_piezoelectric_coupling_scaling() {
    let low_piezo = ChiralQuantumHallPfaffianParams {
        piezoelectric_acoustic_coupling_efficiency: 0.60,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let high_piezo = ChiralQuantumHallPfaffianParams {
        piezoelectric_acoustic_coupling_efficiency: 0.98,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_low = ChiralQuantumHallPfaffianSolver::new(low_piezo);
    let solver_high = ChiralQuantumHallPfaffianSolver::new(high_piezo);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.pfaffian_topological_state_fidelity > m_low.pfaffian_topological_state_fidelity,
        "Higher piezoelectric efficiency must increase state fidelity"
    );
    assert!(
        m_high.neutral_mode_transmission_speed_mps > m_low.neutral_mode_transmission_speed_mps,
        "Higher piezoelectric coupling must accelerate acoustic neutral mode speed"
    );
    assert!(
        m_high.thermal_hall_quantization_error < m_low.thermal_hall_quantization_error,
        "Higher piezoelectric efficiency must decrease thermal Hall quantization error"
    );
    assert!(
        m_high.quasiparticle_braiding_visibility > m_low.quasiparticle_braiding_visibility,
        "Higher piezoelectric efficiency must improve braiding visibility"
    );
}

#[test]
fn test_inter_edge_spacing_scaling() {
    let narrow_spacing = ChiralQuantumHallPfaffianParams {
        inter_edge_spacing_nm: 80.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let wide_spacing = ChiralQuantumHallPfaffianParams {
        inter_edge_spacing_nm: 400.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_narrow = ChiralQuantumHallPfaffianSolver::new(narrow_spacing);
    let solver_wide = ChiralQuantumHallPfaffianSolver::new(wide_spacing);

    let m_narrow = solver_narrow.evaluate_metrics();
    let m_wide = solver_wide.evaluate_metrics();

    assert!(
        m_wide.edge_channel_isolation_db > m_narrow.edge_channel_isolation_db,
        "Wider inter-edge spacing must dramatically improve edge channel isolation"
    );
    assert!(
        m_wide.thermal_hall_quantization_error < m_narrow.thermal_hall_quantization_error,
        "Wider spacing must reduce thermal tunneling quantization error"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let low_temp = ChiralQuantumHallPfaffianParams {
        cryogenic_temperature_mk: 5.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let high_temp = ChiralQuantumHallPfaffianParams {
        cryogenic_temperature_mk: 40.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_low = ChiralQuantumHallPfaffianSolver::new(low_temp);
    let solver_high = ChiralQuantumHallPfaffianSolver::new(high_temp);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_low.pfaffian_topological_state_fidelity > m_high.pfaffian_topological_state_fidelity,
        "Lower temperature must improve topological ground state fidelity"
    );
    assert!(
        m_low.thermal_hall_quantization_error < m_high.thermal_hall_quantization_error,
        "Lower temperature must reduce thermal Hall quantization error"
    );
    assert!(
        m_low.quasiparticle_braiding_visibility > m_high.quasiparticle_braiding_visibility,
        "Lower temperature must enhance braiding visibility"
    );
}

#[test]
fn test_waveguide_channel_length_scaling() {
    let short_guide = ChiralQuantumHallPfaffianParams {
        waveguide_channel_length_um: 2.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let long_guide = ChiralQuantumHallPfaffianParams {
        waveguide_channel_length_um: 20.0,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_short = ChiralQuantumHallPfaffianSolver::new(short_guide);
    let solver_long = ChiralQuantumHallPfaffianSolver::new(long_guide);

    let m_short = solver_short.evaluate_metrics();
    let m_long = solver_long.evaluate_metrics();

    assert!(
        m_short.edge_channel_isolation_db > m_long.edge_channel_isolation_db,
        "Shorter waveguide channel length must exhibit higher isolation"
    );
    assert!(
        m_short.thermal_hall_quantization_error < m_long.thermal_hall_quantization_error,
        "Shorter waveguide length must yield lower thermal Hall error"
    );
}

#[test]
fn test_fractional_filling_factor_detuning() {
    let resonant_filling = ChiralQuantumHallPfaffianParams {
        fractional_filling_factor: 2.50,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let detuned_filling = ChiralQuantumHallPfaffianParams {
        fractional_filling_factor: 2.10,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_res = ChiralQuantumHallPfaffianSolver::new(resonant_filling);
    let solver_det = ChiralQuantumHallPfaffianSolver::new(detuned_filling);

    let m_res = solver_res.evaluate_metrics();
    let m_det = solver_det.evaluate_metrics();

    assert!(
        m_res.pfaffian_topological_state_fidelity > m_det.pfaffian_topological_state_fidelity,
        "Exact nu = 5/2 filling must achieve higher fidelity than detuned filling"
    );
    assert!(
        m_res.thermal_hall_quantization_error < m_det.thermal_hall_quantization_error,
        "Exact nu = 5/2 filling must minimize thermal Hall quantization error"
    );
}

#[test]
fn test_acoustic_driving_frequency_response() {
    let on_resonance = ChiralQuantumHallPfaffianParams {
        acoustic_driving_frequency_ghz: 4.2,
        ..ChiralQuantumHallPfaffianParams::default()
    };
    let off_resonance = ChiralQuantumHallPfaffianParams {
        acoustic_driving_frequency_ghz: 11.5,
        ..ChiralQuantumHallPfaffianParams::default()
    };

    let solver_on = ChiralQuantumHallPfaffianSolver::new(on_resonance);
    let solver_off = ChiralQuantumHallPfaffianSolver::new(off_resonance);

    let m_on = solver_on.evaluate_metrics();
    let m_off = solver_off.evaluate_metrics();

    assert!(
        m_on.pfaffian_topological_state_fidelity >= m_off.pfaffian_topological_state_fidelity,
        "On-resonance acoustic driving must preserve higher state fidelity"
    );
    assert!(
        m_on.thermal_hall_quantization_error <= m_off.thermal_hall_quantization_error,
        "On-resonance driving must yield lower thermal Hall error"
    );
}
