#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic twisted bilayer
//! moire polariton superlattices and flat-band phonon superconductors.

use phonon_models::twisted_bilayer_moire_polariton::TwistedBilayerMoirePolaritonParams;
use phonon_solver::twisted_bilayer_moire_polariton::TwistedBilayerMoirePolaritonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TwistedBilayerMoirePolaritonParams::new(
        0.50,  // below 0.80 deg
        20.0,  // below 50.0 meV
        1.0,   // below 2.0 eV
        0.2,   // below 0.5 GHz
        0.5,   // below 1.0 mK
        0.10,  // below 0.20
        10.0,  // below 20.0 nm
        0.5,   // below 1.0 um
    );
    assert_eq!(underflow.twist_angle_degrees, 0.80);
    assert_eq!(underflow.interlayer_tunneling_energy_mev, 50.0);
    assert_eq!(underflow.acoustic_deformation_potential_ev, 2.0);
    assert_eq!(underflow.moire_acoustic_frequency_ghz, 0.5);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.electron_phonon_coupling_lambda, 0.20);
    assert_eq!(underflow.inter_valley_coherence_length_nm, 20.0);
    assert_eq!(underflow.superconducting_channel_length_um, 1.0);

    // Test values strictly above physical maximum bounds
    let overflow = TwistedBilayerMoirePolaritonParams::new(
        2.50,   // above 1.40 deg
        200.0,  // above 150.0 meV
        25.0,   // above 15.0 eV
        15.0,   // above 10.0 GHz
        80.0,   // above 50.0 mK
        4.00,   // above 2.50
        500.0,  // above 300.0 nm
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.twist_angle_degrees, 1.40);
    assert_eq!(overflow.interlayer_tunneling_energy_mev, 150.0);
    assert_eq!(overflow.acoustic_deformation_potential_ev, 15.0);
    assert_eq!(overflow.moire_acoustic_frequency_ghz, 10.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.electron_phonon_coupling_lambda, 2.50);
    assert_eq!(overflow.inter_valley_coherence_length_nm, 300.0);
    assert_eq!(overflow.superconducting_channel_length_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TwistedBilayerMoirePolaritonParams::default();
    let solver = TwistedBilayerMoirePolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.polariton_superconducting_fidelity >= 0.9970,
        "Polariton superconducting fidelity must be >= 0.9970, got {:.6}",
        metrics.polariton_superconducting_fidelity
    );
    assert!(
        metrics.flat_band_group_velocity_mps <= 150.0,
        "Flat-band group velocity must be <= 150.0 m/s, got {:.2} m/s",
        metrics.flat_band_group_velocity_mps
    );
    assert!(
        metrics.tc_enhancement_factor >= 4.50,
        "Tc enhancement factor must be >= 4.50, got {:.2}",
        metrics.tc_enhancement_factor
    );
    assert!(
        metrics.inter_valley_crosstalk_isolation_db >= 50.0,
        "Inter-valley crosstalk isolation must be >= 50.0 dB, got {:.2} dB",
        metrics.inter_valley_crosstalk_isolation_db
    );
    assert!(
        metrics.magic_angle_alignment_tolerance_fraction >= 0.9980,
        "Magic-angle alignment tolerance fraction must be >= 0.9980, got {:.6}",
        metrics.magic_angle_alignment_tolerance_fraction
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_twist_angle_scaling() {
    let magic_angle = TwistedBilayerMoirePolaritonParams {
        twist_angle_degrees: 1.08,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let detuned_angle = TwistedBilayerMoirePolaritonParams {
        twist_angle_degrees: 1.38,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_magic = TwistedBilayerMoirePolaritonSolver::new(magic_angle);
    let solver_detuned = TwistedBilayerMoirePolaritonSolver::new(detuned_angle);

    let m_magic = solver_magic.evaluate_metrics();
    let m_detuned = solver_detuned.evaluate_metrics();

    assert!(
        m_magic.polariton_superconducting_fidelity > m_detuned.polariton_superconducting_fidelity,
        "Magic angle alignment must maximize polariton superconducting fidelity"
    );
    assert!(
        m_magic.flat_band_group_velocity_mps < m_detuned.flat_band_group_velocity_mps,
        "Magic angle alignment must quench flat-band group velocity"
    );
    assert!(
        m_magic.tc_enhancement_factor > m_detuned.tc_enhancement_factor,
        "Magic angle alignment must maximize Tc enhancement factor"
    );
    assert!(
        m_magic.inter_valley_crosstalk_isolation_db > m_detuned.inter_valley_crosstalk_isolation_db,
        "Magic angle alignment must maximize inter-valley isolation"
    );
    assert!(
        m_magic.magic_angle_alignment_tolerance_fraction > m_detuned.magic_angle_alignment_tolerance_fraction,
        "Magic angle alignment must maximize tolerance fraction"
    );
}

#[test]
fn test_interlayer_tunneling_scaling() {
    let low_w = TwistedBilayerMoirePolaritonParams {
        interlayer_tunneling_energy_mev: 60.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let high_w = TwistedBilayerMoirePolaritonParams {
        interlayer_tunneling_energy_mev: 140.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_low = TwistedBilayerMoirePolaritonSolver::new(low_w);
    let solver_high = TwistedBilayerMoirePolaritonSolver::new(high_w);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.polariton_superconducting_fidelity > m_low.polariton_superconducting_fidelity,
        "Higher interlayer tunneling must enhance superconducting fidelity"
    );
    assert!(
        m_high.flat_band_group_velocity_mps < m_low.flat_band_group_velocity_mps,
        "Higher interlayer tunneling must suppress flat-band group velocity"
    );
    assert!(
        m_high.tc_enhancement_factor > m_low.tc_enhancement_factor,
        "Higher interlayer tunneling must increase Tc enhancement"
    );
    assert!(
        m_high.inter_valley_crosstalk_isolation_db > m_low.inter_valley_crosstalk_isolation_db,
        "Higher interlayer tunneling must enhance inter-valley isolation"
    );
}

#[test]
fn test_acoustic_deformation_potential_scaling() {
    let low_d = TwistedBilayerMoirePolaritonParams {
        acoustic_deformation_potential_ev: 3.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let high_d = TwistedBilayerMoirePolaritonParams {
        acoustic_deformation_potential_ev: 13.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_low = TwistedBilayerMoirePolaritonSolver::new(low_d);
    let solver_high = TwistedBilayerMoirePolaritonSolver::new(high_d);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.polariton_superconducting_fidelity > m_low.polariton_superconducting_fidelity,
        "Higher acoustic deformation potential must enhance superconducting fidelity"
    );
    assert!(
        m_high.flat_band_group_velocity_mps < m_low.flat_band_group_velocity_mps,
        "Higher acoustic deformation potential must quench group velocity"
    );
    assert!(
        m_high.tc_enhancement_factor > m_low.tc_enhancement_factor,
        "Higher acoustic deformation potential must enhance Tc"
    );
    assert!(
        m_high.inter_valley_crosstalk_isolation_db > m_low.inter_valley_crosstalk_isolation_db,
        "Higher deformation potential must enhance inter-valley isolation"
    );
}

#[test]
fn test_moire_acoustic_frequency_scaling() {
    let low_f = TwistedBilayerMoirePolaritonParams {
        moire_acoustic_frequency_ghz: 1.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let high_f = TwistedBilayerMoirePolaritonParams {
        moire_acoustic_frequency_ghz: 8.5,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_low = TwistedBilayerMoirePolaritonSolver::new(low_f);
    let solver_high = TwistedBilayerMoirePolaritonSolver::new(high_f);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.polariton_superconducting_fidelity > m_low.polariton_superconducting_fidelity,
        "Higher acoustic driving frequency must enhance polariton fidelity"
    );
    assert!(
        m_high.flat_band_group_velocity_mps < m_low.flat_band_group_velocity_mps,
        "Higher acoustic frequency must suppress group velocity"
    );
    assert!(
        m_high.tc_enhancement_factor > m_low.tc_enhancement_factor,
        "Higher acoustic frequency must increase Tc enhancement"
    );
    assert!(
        m_high.inter_valley_crosstalk_isolation_db > m_low.inter_valley_crosstalk_isolation_db,
        "Higher acoustic frequency must improve inter-valley isolation"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let low_t = TwistedBilayerMoirePolaritonParams {
        cryogenic_temperature_mk: 2.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let high_t = TwistedBilayerMoirePolaritonParams {
        cryogenic_temperature_mk: 45.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_low = TwistedBilayerMoirePolaritonSolver::new(low_t);
    let solver_high = TwistedBilayerMoirePolaritonSolver::new(high_t);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_low.polariton_superconducting_fidelity > m_high.polariton_superconducting_fidelity,
        "Lower cryogenic temperature must enhance polariton fidelity"
    );
    assert!(
        m_low.flat_band_group_velocity_mps < m_high.flat_band_group_velocity_mps,
        "Lower cryogenic temperature must reduce thermal broadening and group velocity"
    );
    assert!(
        m_low.tc_enhancement_factor > m_high.tc_enhancement_factor,
        "Lower cryogenic temperature must maximize Tc enhancement factor"
    );
    assert!(
        m_low.inter_valley_crosstalk_isolation_db > m_high.inter_valley_crosstalk_isolation_db,
        "Lower cryogenic temperature must improve inter-valley isolation"
    );
    assert!(
        m_low.magic_angle_alignment_tolerance_fraction > m_high.magic_angle_alignment_tolerance_fraction,
        "Lower cryogenic temperature must improve alignment tolerance"
    );
}

#[test]
fn test_electron_phonon_coupling_scaling() {
    let low_lambda = TwistedBilayerMoirePolaritonParams {
        electron_phonon_coupling_lambda: 0.30,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let high_lambda = TwistedBilayerMoirePolaritonParams {
        electron_phonon_coupling_lambda: 2.20,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_low = TwistedBilayerMoirePolaritonSolver::new(low_lambda);
    let solver_high = TwistedBilayerMoirePolaritonSolver::new(high_lambda);

    let m_low = solver_low.evaluate_metrics();
    let m_high = solver_high.evaluate_metrics();

    assert!(
        m_high.polariton_superconducting_fidelity > m_low.polariton_superconducting_fidelity,
        "Higher electron-phonon coupling lambda must enhance polariton fidelity"
    );
    assert!(
        m_high.flat_band_group_velocity_mps < m_low.flat_band_group_velocity_mps,
        "Higher electron-phonon coupling lambda must quench flat-band group velocity"
    );
    assert!(
        m_high.tc_enhancement_factor > m_low.tc_enhancement_factor,
        "Higher electron-phonon coupling lambda must significantly boost Tc enhancement"
    );
    assert!(
        m_high.inter_valley_crosstalk_isolation_db > m_low.inter_valley_crosstalk_isolation_db,
        "Higher electron-phonon coupling lambda must improve inter-valley isolation"
    );
    assert!(
        m_high.magic_angle_alignment_tolerance_fraction > m_low.magic_angle_alignment_tolerance_fraction,
        "Higher electron-phonon coupling lambda must improve alignment tolerance"
    );
}

#[test]
fn test_inter_valley_coherence_length_scaling() {
    let short_xi = TwistedBilayerMoirePolaritonParams {
        inter_valley_coherence_length_nm: 30.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let long_xi = TwistedBilayerMoirePolaritonParams {
        inter_valley_coherence_length_nm: 280.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_short = TwistedBilayerMoirePolaritonSolver::new(short_xi);
    let solver_long = TwistedBilayerMoirePolaritonSolver::new(long_xi);

    let m_short = solver_short.evaluate_metrics();
    let m_long = solver_long.evaluate_metrics();

    assert!(
        m_long.inter_valley_crosstalk_isolation_db > m_short.inter_valley_crosstalk_isolation_db,
        "Longer inter-valley coherence length must strongly improve channel isolation"
    );
    assert!(
        m_long.magic_angle_alignment_tolerance_fraction > m_short.magic_angle_alignment_tolerance_fraction,
        "Longer inter-valley coherence length must improve alignment tolerance"
    );
    assert!(
        m_long.polariton_superconducting_fidelity > m_short.polariton_superconducting_fidelity,
        "Longer inter-valley coherence length must enhance superconducting fidelity"
    );
    assert!(
        m_long.tc_enhancement_factor > m_short.tc_enhancement_factor,
        "Longer inter-valley coherence length must enhance Tc factor"
    );
}

#[test]
fn test_superconducting_channel_length_scaling() {
    let short_l = TwistedBilayerMoirePolaritonParams {
        superconducting_channel_length_um: 1.5,
        ..TwistedBilayerMoirePolaritonParams::default()
    };
    let long_l = TwistedBilayerMoirePolaritonParams {
        superconducting_channel_length_um: 18.0,
        ..TwistedBilayerMoirePolaritonParams::default()
    };

    let solver_short = TwistedBilayerMoirePolaritonSolver::new(short_l);
    let solver_long = TwistedBilayerMoirePolaritonSolver::new(long_l);

    let m_short = solver_short.evaluate_metrics();
    let m_long = solver_long.evaluate_metrics();

    assert!(
        m_long.inter_valley_crosstalk_isolation_db > m_short.inter_valley_crosstalk_isolation_db,
        "Longer superconducting channel length must enhance channel isolation"
    );
    assert!(
        m_long.polariton_superconducting_fidelity > m_short.polariton_superconducting_fidelity,
        "Longer superconducting channel length must enhance fidelity"
    );
}
