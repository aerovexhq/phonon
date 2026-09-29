//! Integration tests for Acoustic Spin Pumping & Non-Reciprocal Transduction.

use phonon_models::chiral_polariton::{AcousticSpinPumpingInterface, MagnetoElasticMedium};
use phonon_solver::chiral_polariton::AcousticTransductionSolver;

#[test]
fn test_acoustic_spin_pumping_and_ishe_voltage() {
    let medium = MagnetoElasticMedium::default();
    let interface = AcousticSpinPumpingInterface::default();
    let solver = AcousticTransductionSolver::new();

    let strain = 1.0e-5;
    let w = medium.kittel_magnon_frequency_rad_per_s();

    let res = solver.solve_transduction(&medium, &interface, strain, w);

    // Dynamic precession cone angle
    assert!(res.precession_angle_rad > 0.001 && res.precession_angle_rad < 0.20);

    // Pumped spin current density
    assert!(
        res.spin_current_density_j_per_m2 > 5.0e-9,
        "Pumped spin current should be > 5 nJ/m^2, got {:e}",
        res.spin_current_density_j_per_m2
    );

    // Transverse ISHE DC voltage in microvolts
    let ishe_uv = res.ishe_dc_voltage_volts * 1e6;
    assert!(
        ishe_uv > 0.10,
        "ISHE voltage should be > 0.1 uV, got {:.3} uV",
        ishe_uv
    );
}

#[test]
fn test_non_reciprocal_acoustic_isolation() {
    let medium = MagnetoElasticMedium::default();
    let interface = AcousticSpinPumpingInterface::default();
    let solver = AcousticTransductionSolver::new();

    let strain = 1.0e-5;
    let w = medium.kittel_magnon_frequency_rad_per_s();

    let res = solver.solve_transduction(&medium, &interface, strain, w);

    // Non-reciprocal acoustic isolation must be >= 20.0 dB
    assert!(
        res.non_reciprocal_isolation_db >= 20.0,
        "Isolation must be >= 20 dB, got {:.2} dB",
        res.non_reciprocal_isolation_db
    );

    // S-parameters: reverse transmission near 1, forward transmission strongly attenuated
    assert!(res.reverse_transmission_s12 > 0.90);
    assert!(res.forward_transmission_s21 < 0.10);
    let ratio_db = 20.0 * (res.reverse_transmission_s12 / res.forward_transmission_s21).log10();
    assert!((ratio_db - res.non_reciprocal_isolation_db).abs() < 1e-3);
}
