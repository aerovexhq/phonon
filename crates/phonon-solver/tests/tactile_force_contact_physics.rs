//! Integration Tests: Tactile Force, Piezoresistive & Capacitive Contact Physics
//!
//! Validates:
//! 1. Piezoresistive power-law resistance curve and Wheatstone bridge differential voltage.
//! 2. Capacitive elastomeric deflection and dynamic permittivity capacitance.
//! 3. 2D multi-taxel spatial matrix bilinear force distribution and Center of Pressure (CoP).

#![deny(unsafe_code)]

use phonon_models::em::Vector3D;
use phonon_models::sensors::{
    CapacitiveSensorConfig, CollisionContactInput, PiezoresistiveSensorConfig, TactileMatrixArray,
};

#[test]
fn test_piezoresistive_resistance_and_wheatstone_voltage() {
    let config = PiezoresistiveSensorConfig {
        rest_resistance_ohms: 10_000.0,
        characteristic_force_n: 2.0,
        piezoresistive_exponent: 0.8,
        bridge_excitation_voltage_v: 3.3,
    };

    // 1. Zero force: R = R_0, Delta_V = 0
    let r0 = config.resistance_ohms(0.0);
    assert!((r0 - 10_000.0).abs() < 1e-6);

    let v0 = config.wheatstone_differential_voltage_v(0.0);
    assert!(v0.abs() < 1e-6);

    // 2. Normal force = 5 N:
    // R = 10000 * (1 + 5/2)^(-0.8) = 10000 * (3.5)^(-0.8) ~= 3670.67 ohms
    let r5 = config.resistance_ohms(5.0);
    assert!((r5 - 3670.67).abs() < 1.0);

    // Delta_V = 3.3 * (3662.84 / (10000 + 3662.84) - 0.5) ~= 3.3 * (0.26809 - 0.5) ~= -0.7653 V
    let v5 = config.wheatstone_differential_voltage_v(5.0);
    assert!(v5 < 0.0);
    assert!((v5 - (-0.7653)).abs() < 0.02);

    // 3. Monotonicity check: R(10) < R(5) < R(1) < R(0)
    let r1 = config.resistance_ohms(1.0);
    let r10 = config.resistance_ohms(10.0);
    assert!(r10 < r5);
    assert!(r5 < r1);
    assert!(r1 < r0);
}

#[test]
fn test_capacitive_elastomeric_deflection_and_capacitance() {
    let config = CapacitiveSensorConfig {
        dielectric_thickness_m: 5.0e-4,        // 0.5 mm
        plate_area_m2: 1.0e-4,                 // 1 cm^2
        relative_permittivity: 3.2,            // Silicone dielectric
        elastomer_stiffness_n_per_m: 20_000.0, // 20 kN/m
    };

    // 1. Rest capacitance at 0 N:
    // C_0 = 3.2 * 8.8541878e-12 * 1e-4 / 5e-4 ~= 5.6667 pF
    let c0 = config.rest_capacitance_f();
    assert!((c0 - 5.6667e-12).abs() < 1e-15);

    // 2. Deflection under 4.0 N:
    // delta_d = 4.0 / 20000 = 0.2 mm = 2.0e-4 m
    let d4 = config.deflection_m(4.0);
    assert!((d4 - 2.0e-4).abs() < 1e-9);

    // C(4 N) = 3.2 * 8.8541878e-12 * 1e-4 / (5e-4 - 2e-4) = 3.2 * 8.854e-16 / 3e-4 ~= 9.444 pF
    let c4 = config.capacitance_f(4.0);
    assert!(c4 > c0);
    assert!((c4 - 9.4445e-12).abs() < 1e-15);

    // 3. Extreme load compression limit (capped before zero thickness):
    let c_heavy = config.capacitance_f(50.0);
    assert!(c_heavy > c4);
    assert!(c_heavy.is_finite());
}

#[test]
fn test_tactile_matrix_bilinear_force_distribution_and_cop() {
    let rows = 4;
    let cols = 4;
    let pitch_m = 0.005; // 5 mm pitch
    let mut matrix =
        TactileMatrixArray::new(rows, cols, pitch_m, PiezoresistiveSensorConfig::default());

    let sensor_center = Vector3D::new(0.0, 0.0, 0.0);
    let sensor_normal = Vector3D::new(0.0, 0.0, 1.0);
    let sensor_up = Vector3D::new(0.0, 1.0, 0.0);

    // 1. Apply contact force directly at sensor center (0, 0):
    let normal_force = 12.0;
    let contact_center = CollisionContactInput::new(
        Vector3D::new(0.0, 0.0, 0.0),
        sensor_normal,
        normal_force,
        Vector3D::new(0.0, 0.0, 0.0),
        0.001,
    );

    matrix.update_from_contact(&contact_center, sensor_center, sensor_normal, sensor_up);

    // Force conservation:
    let total_f = matrix.total_normal_force_n();
    assert!((total_f - normal_force).abs() < 1e-5);

    // Center of pressure should be at (0, 0):
    let (cop_x, cop_y) = matrix.center_of_pressure_m();
    assert!(cop_x.abs() < 1e-4);
    assert!(cop_y.abs() < 1e-4);

    // Voltages should be non-zero on active taxels:
    let voltages = matrix.taxel_voltages_v();
    assert_eq!(voltages.len(), 16);
    let active_taxels = voltages.iter().filter(|&&v| v < -0.01).count();
    assert!(active_taxels >= 4); // Bilinear spread to 4 center taxels

    // 2. Apply contact force off-center at (+2.5 mm, +2.5 mm):
    let offset_contact = CollisionContactInput::new(
        Vector3D::new(0.0, 0.0025, 0.0), // along up vector (y_local = +2.5 mm)
        sensor_normal,
        8.0,
        Vector3D::new(0.0, 0.0, 0.0),
        0.001,
    );

    matrix.update_from_contact(&offset_contact, sensor_center, sensor_normal, sensor_up);
    let (cop_x_off, cop_y_off) = matrix.center_of_pressure_m();
    assert!(cop_x_off.abs() < 1e-4);
    assert!((cop_y_off - 0.0025).abs() < 5e-4);

    // 3. Out-of-bounds contact:
    let out_contact = CollisionContactInput::new(
        Vector3D::new(0.05, 0.0, 0.0), // 50 mm away
        sensor_normal,
        15.0,
        Vector3D::new(0.0, 0.0, 0.0),
        0.001,
    );
    matrix.update_from_contact(&out_contact, sensor_center, sensor_normal, sensor_up);
    assert_eq!(matrix.total_normal_force_n(), 0.0);
}
