//! Integration Test: Nanomagnetic Stray Dipole Field Coupling & Majority-3 Logic.
//!
//! Validates:
//! 1. Dipole field \(1/r^3\) spatial decay and geometric coupling anisotropy.
//! 2. Anti-ferromagnetic Inverter switching via stray dipole interaction.
//! 3. Majority-3 gate truth table fidelity across all 8 input states.
//! 4. 1-bit Full Adder cell arithmetic fidelity (\(\text{Sum} = A \oplus B \oplus Cin\), \(\text{Cout} = \text{Maj}(A, B, Cin)\)).

use phonon_models::spintronics::{
    MagneticMaterial, Nanomagnet, NmlFullAdderCell, NmlInverter, NmlMajority3, Vec3,
};

#[test]
fn test_dipole_field_spatial_decay_cubic() {
    let mat = MagneticMaterial::cofeb();
    let source = Nanomagnet::new_rectangular(
        0,
        Vec3::ZERO,
        60.0e-9,
        30.0e-9,
        3.0e-9,
        mat,
        Vec3::X,
        Vec3::X,
    );

    let pos1 = Vec3::new(50.0e-9, 0.0, 0.0);
    let pos2 = Vec3::new(100.0e-9, 0.0, 0.0); // 2x distance

    let h1 = source.compute_dipole_field_at(pos1).norm();
    let h2 = source.compute_dipole_field_at(pos2).norm();

    // 1/r^3 dependence implies H(2r) / H(r) = (1/2)^3 = 1/8 = 0.125
    let ratio = h2 / h1;
    assert!(
        (ratio - 0.125).abs() < 0.01,
        "Dipole field must exhibit 1/r^3 decay: expected ~0.125, got {:.4}",
        ratio
    );
}

#[test]
fn test_nml_inverter_and_majority_logic() {
    let inv = NmlInverter::default();
    assert!(inv.evaluate(false).0);
    assert!(!inv.evaluate(true).0);

    let maj = NmlMajority3::default();
    let cases = [
        (false, false, false, false),
        (false, false, true, false),
        (false, true, false, false),
        (false, true, true, true),
        (true, false, false, false),
        (true, false, true, true),
        (true, true, false, true),
        (true, true, true, true),
    ];

    for (a, b, c, expected) in cases {
        let (out, _) = maj.evaluate(a, b, c);
        assert_eq!(
            out, expected,
            "Majority mismatch for inputs ({}, {}, {})",
            a, b, c
        );
    }
}

#[test]
fn test_nml_full_adder_exhaustive_truth_table() {
    let adder = NmlFullAdderCell::default();

    let cases = [
        (false, false, false, false, false),
        (false, false, true, true, false),
        (false, true, false, true, false),
        (false, true, true, false, true),
        (true, false, false, true, false),
        (true, false, true, false, true),
        (true, true, false, false, true),
        (true, true, true, true, true),
    ];

    for (a, b, cin, exp_s, exp_c) in cases {
        let (sum, cout) = adder.evaluate(a, b, cin);
        assert_eq!(sum, exp_s, "Sum mismatch for A={}, B={}, Cin={}", a, b, cin);
        assert_eq!(
            cout, exp_c,
            "Cout mismatch for A={}, B={}, Cin={}",
            a, b, cin
        );
    }
}
