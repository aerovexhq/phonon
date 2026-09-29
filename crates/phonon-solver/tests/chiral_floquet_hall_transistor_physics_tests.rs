#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for chiral phononic Floquet-SBT gauge
//! fields and dissipationless acoustic topological Hall transistors.

use phonon_models::chiral_floquet_hall_transistor::ChiralFloquetHallTransistorParams;
use phonon_solver::chiral_floquet_hall_transistor::ChiralFloquetHallTransistorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralFloquetHallTransistorParams::new(
        2.0,    // below 5.0 MHz
        0.5,    // below 1.0 GHz
        5.0,    // below 10.0 ppm/um
        0.5,    // below 1.0 MHz
        0.05,   // below 0.1 V
        0.2,    // below 0.5 um
        0.5,    // below 1.0 mK
        0.005,  // below 0.01
    );
    assert_eq!(underflow.floquet_modulation_amplitude_mhz, 5.0);
    assert_eq!(underflow.floquet_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.strain_torsion_gradient_ppm_per_um, 10.0);
    assert_eq!(underflow.chiral_valley_coupling_mhz, 1.0);
    assert_eq!(underflow.transistor_gate_voltage_v, 0.1);
    assert_eq!(underflow.channel_length_um, 0.5);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.piezoelectric_electromechanical_coupling, 0.01);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralFloquetHallTransistorParams::new(
        150.0,  // above 100.0 MHz
        25.0,   // above 15.0 GHz
        500.0,  // above 300.0 ppm/um
        80.0,   // above 50.0 MHz
        20.0,   // above 10.0 V
        45.0,   // above 20.0 um
        100.0,  // above 50.0 mK
        0.50,   // above 0.25
    );
    assert_eq!(overflow.floquet_modulation_amplitude_mhz, 100.0);
    assert_eq!(overflow.floquet_drive_frequency_ghz, 15.0);
    assert_eq!(overflow.strain_torsion_gradient_ppm_per_um, 300.0);
    assert_eq!(overflow.chiral_valley_coupling_mhz, 50.0);
    assert_eq!(overflow.transistor_gate_voltage_v, 10.0);
    assert_eq!(overflow.channel_length_um, 20.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.piezoelectric_electromechanical_coupling, 0.25);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralFloquetHallTransistorParams::default();
    let solver = ChiralFloquetHallTransistorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.valley_hall_contrast_ratio_db >= 35.0,
        "Default valley Hall contrast ratio must be >= 35.0 dB, got {:.2} dB",
        metrics.valley_hall_contrast_ratio_db
    );
    assert!(
        metrics.topological_switching_time_ns <= 15.0,
        "Default topological switching time must be <= 15.0 ns, got {:.2} ns",
        metrics.topological_switching_time_ns
    );
    assert!(
        metrics.cross_talk_isolation_db >= 40.0,
        "Default cross-talk isolation must be >= 40.0 dB, got {:.2} dB",
        metrics.cross_talk_isolation_db
    );
    assert!(
        metrics.non_adiabatic_insertion_loss_db <= 0.60,
        "Default non-adiabatic insertion loss must be <= 0.60 dB, got {:.4} dB",
        metrics.non_adiabatic_insertion_loss_db
    );
    assert!(
        metrics.hall_transistor_state_fidelity >= 0.9960,
        "Default hall transistor state fidelity must be >= 0.9960, got {:.6}",
        metrics.hall_transistor_state_fidelity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_floquet_modulation_amplitude_scaling() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let high_amp = ChiralFloquetHallTransistorParams::new(
        60.0, // increased from 35.0 MHz
        base.floquet_drive_frequency_ghz,
        base.strain_torsion_gradient_ppm_per_um,
        base.chiral_valley_coupling_mhz,
        base.transistor_gate_voltage_v,
        base.channel_length_um,
        base.cryogenic_temperature_mk,
        base.piezoelectric_electromechanical_coupling,
    );
    let solver_high = ChiralFloquetHallTransistorSolver::new(high_amp);

    let contrast_base = solver_base.compute_valley_hall_contrast_ratio_db();
    let contrast_high = solver_high.compute_valley_hall_contrast_ratio_db();
    assert!(
        contrast_high > contrast_base,
        "Higher Floquet modulation amplitude must increase valley Hall contrast: {:.2} vs {:.2}",
        contrast_high,
        contrast_base
    );

    let isolation_base = solver_base.compute_cross_talk_isolation_db();
    let isolation_high = solver_high.compute_cross_talk_isolation_db();
    assert!(
        isolation_high > isolation_base,
        "Higher Floquet modulation amplitude must improve cross-talk isolation: {:.2} vs {:.2}",
        isolation_high,
        isolation_base
    );

    let loss_base = solver_base.compute_non_adiabatic_insertion_loss_db();
    let loss_high = solver_high.compute_non_adiabatic_insertion_loss_db();
    assert!(
        loss_high < loss_base,
        "Higher Floquet modulation amplitude must reduce non-adiabatic insertion loss: {:.4} vs {:.4}",
        loss_high,
        loss_base
    );
}

#[test]
fn test_floquet_drive_frequency_scaling() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let high_freq = ChiralFloquetHallTransistorParams::new(
        base.floquet_modulation_amplitude_mhz,
        8.0, // increased from 4.6 GHz
        base.strain_torsion_gradient_ppm_per_um,
        base.chiral_valley_coupling_mhz,
        base.transistor_gate_voltage_v,
        base.channel_length_um,
        base.cryogenic_temperature_mk,
        base.piezoelectric_electromechanical_coupling,
    );
    let solver_high = ChiralFloquetHallTransistorSolver::new(high_freq);

    let switch_base = solver_base.compute_topological_switching_time_ns();
    let switch_high = solver_high.compute_topological_switching_time_ns();
    assert!(
        switch_high < switch_base,
        "Higher Floquet driving frequency must reduce switching time: {:.2} vs {:.2}",
        switch_high,
        switch_base
    );
}

#[test]
fn test_strain_torsion_gradient_scaling() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let high_strain = ChiralFloquetHallTransistorParams::new(
        base.floquet_modulation_amplitude_mhz,
        base.floquet_drive_frequency_ghz,
        150.0, // increased from 85.0 ppm/um
        base.chiral_valley_coupling_mhz,
        base.transistor_gate_voltage_v,
        base.channel_length_um,
        base.cryogenic_temperature_mk,
        base.piezoelectric_electromechanical_coupling,
    );
    let solver_high = ChiralFloquetHallTransistorSolver::new(high_strain);

    let contrast_base = solver_base.compute_valley_hall_contrast_ratio_db();
    let contrast_high = solver_high.compute_valley_hall_contrast_ratio_db();
    assert!(
        contrast_high > contrast_base,
        "Higher strain torsion gradient must boost valley Hall contrast: {:.2} vs {:.2}",
        contrast_high,
        contrast_base
    );

    let isolation_base = solver_base.compute_cross_talk_isolation_db();
    let isolation_high = solver_high.compute_cross_talk_isolation_db();
    assert!(
        isolation_high > isolation_base,
        "Higher strain torsion gradient must increase cross-talk isolation: {:.2} vs {:.2}",
        isolation_high,
        isolation_base
    );
}

#[test]
fn test_transistor_gate_voltage_scaling() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let high_gate = ChiralFloquetHallTransistorParams::new(
        base.floquet_modulation_amplitude_mhz,
        base.floquet_drive_frequency_ghz,
        base.strain_torsion_gradient_ppm_per_um,
        base.chiral_valley_coupling_mhz,
        5.0, // increased from 2.5 V
        base.channel_length_um,
        base.cryogenic_temperature_mk,
        base.piezoelectric_electromechanical_coupling,
    );
    let solver_high = ChiralFloquetHallTransistorSolver::new(high_gate);

    let switch_base = solver_base.compute_topological_switching_time_ns();
    let switch_high = solver_high.compute_topological_switching_time_ns();
    assert!(
        switch_high < switch_base,
        "Higher gate voltage must accelerate topological switching: {:.2} vs {:.2}",
        switch_high,
        switch_base
    );
}

#[test]
fn test_channel_length_scaling() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let long_channel = ChiralFloquetHallTransistorParams::new(
        base.floquet_modulation_amplitude_mhz,
        base.floquet_drive_frequency_ghz,
        base.strain_torsion_gradient_ppm_per_um,
        base.chiral_valley_coupling_mhz,
        base.transistor_gate_voltage_v,
        8.0, // increased from 4.2 um
        base.cryogenic_temperature_mk,
        base.piezoelectric_electromechanical_coupling,
    );
    let solver_long = ChiralFloquetHallTransistorSolver::new(long_channel);

    let isolation_base = solver_base.compute_cross_talk_isolation_db();
    let isolation_long = solver_long.compute_cross_talk_isolation_db();
    assert!(
        isolation_long > isolation_base,
        "Longer channel length must increase cross-talk isolation: {:.2} vs {:.2}",
        isolation_long,
        isolation_base
    );

    let switch_base = solver_base.compute_topological_switching_time_ns();
    let switch_long = solver_long.compute_topological_switching_time_ns();
    assert!(
        switch_long > switch_base,
        "Longer channel length must increase acoustic transit switching time: {:.2} vs {:.2}",
        switch_long,
        switch_base
    );
}

#[test]
fn test_cryogenic_temperature_degradation() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let elevated_temp = ChiralFloquetHallTransistorParams::new(
        base.floquet_modulation_amplitude_mhz,
        base.floquet_drive_frequency_ghz,
        base.strain_torsion_gradient_ppm_per_um,
        base.chiral_valley_coupling_mhz,
        base.transistor_gate_voltage_v,
        base.channel_length_um,
        35.0, // increased from 15.0 mK
        base.piezoelectric_electromechanical_coupling,
    );
    let solver_temp = ChiralFloquetHallTransistorSolver::new(elevated_temp);

    let contrast_base = solver_base.compute_valley_hall_contrast_ratio_db();
    let contrast_temp = solver_temp.compute_valley_hall_contrast_ratio_db();
    assert!(
        contrast_temp < contrast_base,
        "Elevated temperature must degrade valley Hall contrast: {:.2} vs {:.2}",
        contrast_temp,
        contrast_base
    );

    let loss_base = solver_base.compute_non_adiabatic_insertion_loss_db();
    let loss_temp = solver_temp.compute_non_adiabatic_insertion_loss_db();
    assert!(
        loss_temp > loss_base,
        "Elevated temperature must increase acoustic insertion loss: {:.4} vs {:.4}",
        loss_temp,
        loss_base
    );

    let fidelity_base = solver_base.compute_hall_transistor_state_fidelity();
    let fidelity_temp = solver_temp.compute_hall_transistor_state_fidelity();
    assert!(
        fidelity_temp < fidelity_base,
        "Elevated temperature must degrade transistor state fidelity: {:.6} vs {:.6}",
        fidelity_temp,
        fidelity_base
    );
}

#[test]
fn test_piezoelectric_coupling_scaling() {
    let base = ChiralFloquetHallTransistorParams::default();
    let solver_base = ChiralFloquetHallTransistorSolver::new(base);

    let strong_coupling = ChiralFloquetHallTransistorParams::new(
        base.floquet_modulation_amplitude_mhz,
        base.floquet_drive_frequency_ghz,
        base.strain_torsion_gradient_ppm_per_um,
        base.chiral_valley_coupling_mhz,
        base.transistor_gate_voltage_v,
        base.channel_length_um,
        base.cryogenic_temperature_mk,
        0.15, // increased from 0.08
    );
    let solver_strong = ChiralFloquetHallTransistorSolver::new(strong_coupling);

    let switch_base = solver_base.compute_topological_switching_time_ns();
    let switch_strong = solver_strong.compute_topological_switching_time_ns();
    assert!(
        switch_strong < switch_base,
        "Stronger electromechanical coupling must reduce switching time: {:.2} vs {:.2}",
        switch_strong,
        switch_base
    );

    let loss_base = solver_base.compute_non_adiabatic_insertion_loss_db();
    let loss_strong = solver_strong.compute_non_adiabatic_insertion_loss_db();
    assert!(
        loss_strong < loss_base,
        "Stronger electromechanical coupling must reduce insertion loss: {:.4} vs {:.4}",
        loss_strong,
        loss_base
    );
}
