//! Integration tests for chiral phonon drive, induced orbital magnetic moment,
//! dynamic crystal inversion breaking, and transient non-adiabatic Eliashberg pairing enhancement.

use phonon_core::constants::H_BAR;
use phonon_models::chiral_phonon_sc::{ChiralPhononDriveParams, TransientPairingParams};
use phonon_solver::chiral_phonon_sc::ChiralEliashbergSolver;

#[test]
fn test_chiral_phonon_drive_and_magnetic_moment() {
    let drive = ChiralPhononDriveParams {
        phonon_frequency_thz: 20.0,
        normalized_amplitude_q0: 2.5,
        nonlinear_coupling_g12_mev: 20.0,
        effective_born_charge: 3.2,
        pulse_duration_ps: 0.5,
    };
    let pairing = TransientPairingParams::default();
    let solver = ChiralEliashbergSolver::new(drive, pairing);

    let drive_metrics = solver.solve_chiral_drive();

    // Verify chiral phonon angular momentum L_ph = hbar
    assert!((drive_metrics.angular_momentum_j_s - H_BAR).abs() < 1e-40);

    // Verify induced orbital magnetic moment in 1.0 - 10.0 mu_B range
    assert!(
        drive_metrics.induced_magnetic_moment_bohr >= 1.0,
        "Induced moment {} should be >= 1.0 mu_B",
        drive_metrics.induced_magnetic_moment_bohr
    );
    assert!(
        drive_metrics.induced_magnetic_moment_bohr <= 15.0,
        "Induced moment {} should be <= 15.0 mu_B",
        drive_metrics.induced_magnetic_moment_bohr
    );

    // Verify dynamic inversion breaking parameter
    assert!(
        drive_metrics.inversion_breaking_parameter > 0.5,
        "Inversion breaking parameter should be > 0.5"
    );

    // Verify rectified coordinate displacement
    assert!(
        drive_metrics.rectified_displacement_pm > 1.0,
        "Rectified displacement should be > 1.0 pm"
    );

    // Verify effective pseudo-magnetic field in perovskite lattice (V_cell ~ 0.06 nm^3)
    let b_eff = solver.solve_effective_magnetic_field_tesla(0.06); // 0.06 nm^3 unit cell
    assert!(
        b_eff > 1.0,
        "Effective magnetic field {} T should be > 1.0 T",
        b_eff
    );
}

#[test]
fn test_transient_pairing_enhancement_exceeds_fifty_percent() {
    let drive = ChiralPhononDriveParams {
        phonon_frequency_thz: 22.0,
        normalized_amplitude_q0: 2.2, // (2.2)^2 = 4.84
        nonlinear_coupling_g12_mev: 18.0,
        effective_born_charge: 3.2,
        pulse_duration_ps: 0.45,
    };
    let pairing = TransientPairingParams {
        equilibrium_gap_mev: 15.0,
        equilibrium_lambda: 0.45,
        pairing_sensitivity: 0.18,
        fermi_velocity_m_s: 2.0e5,
    };
    let solver = ChiralEliashbergSolver::new(drive, pairing);

    let pairing_metrics = solver.solve_transient_pairing();

    // Pairing enhancement fraction must strictly exceed 50%
    let enhancement_percent = pairing_metrics.pairing_enhancement_fraction * 100.0;
    assert!(
        enhancement_percent > 50.0,
        "Transient pairing enhancement {:.2}% must exceed 50.0%",
        enhancement_percent
    );

    // Transient gap must exceed equilibrium gap by > 50%
    assert!(
        pairing_metrics.transient_gap_mev > pairing.equilibrium_gap_mev * 1.50,
        "Transient gap {:.2} meV must exceed 1.5x base gap {:.2} meV",
        pairing_metrics.transient_gap_mev,
        pairing.equilibrium_gap_mev
    );

    // Dynamic electron-phonon coupling lambda_eff must be renormalized upwards
    assert!(
        pairing_metrics.effective_coupling_lambda > pairing.equilibrium_lambda * 1.50,
        "Effective lambda {:.3} must exceed 1.5x equilibrium lambda {:.3}",
        pairing_metrics.effective_coupling_lambda,
        pairing.equilibrium_lambda
    );

    // Check transient critical temperature scaling
    let tc_transient = solver.solve_transient_critical_temperature_k();
    assert!(
        tc_transient > 100.0,
        "Transient Tc {:.1} K should exceed 100 K",
        tc_transient
    );

    // Check finite-temperature gap below and above Tc
    let gap_50k = solver.solve_transient_gap_at_temperature(50.0);
    let gap_above_tc = solver.solve_transient_gap_at_temperature(tc_transient + 10.0);
    assert!(gap_50k > 0.0, "Gap at 50 K should be positive");
    assert_eq!(gap_above_tc, 0.0, "Gap above Tc must vanish");
}

#[test]
fn test_pair_density_wave_spatial_modulation() {
    let drive = ChiralPhononDriveParams {
        phonon_frequency_thz: 20.0,
        normalized_amplitude_q0: 2.0,
        nonlinear_coupling_g12_mev: 15.0,
        effective_born_charge: 3.0,
        pulse_duration_ps: 0.5,
    };
    let pairing = TransientPairingParams {
        equilibrium_gap_mev: 12.0,
        equilibrium_lambda: 0.40,
        pairing_sensitivity: 0.18,
        fermi_velocity_m_s: 2.0e5,
    };
    let solver = ChiralEliashbergSolver::new(drive, pairing);

    let metrics = solver.solve_transient_pairing();
    // PDW wavelength: lambda = 2*pi / q_pdw = v_F / f_ph = 2e5 / 20e12 = 10 nm
    let lambda_pdw_nm = (2.0 * std::f64::consts::PI / metrics.pdw_wavevector_inv_m) * 1.0e9;
    assert!(
        (lambda_pdw_nm - 10.0).abs() < 0.1,
        "PDW wavelength {:.2} nm should be ~10 nm",
        lambda_pdw_nm
    );

    let delta_origin = solver.solve_pdw_spatial_profile(0.0);
    let delta_half_period = solver.solve_pdw_spatial_profile(lambda_pdw_nm * 0.5);
    let delta_full_period = solver.solve_pdw_spatial_profile(lambda_pdw_nm);

    assert!(
        (delta_origin - metrics.transient_gap_mev).abs() < 1e-6,
        "Gap at origin must equal peak transient gap"
    );
    assert!(
        (delta_half_period + metrics.transient_gap_mev).abs() < 1e-4,
        "Gap at half period must equal negative peak transient gap"
    );
    assert!(
        (delta_full_period - metrics.transient_gap_mev).abs() < 1e-4,
        "Gap at full period must equal peak transient gap"
    );
}
