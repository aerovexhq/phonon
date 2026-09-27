//! Integration tests for Balanced Ternary logic primitives and arithmetic cells.
//!
//! Validates:
//! 1. Exhaustive truth table compliance for STI, PTI, and NTI.
//! 2. Canonical ternary gates: TNAND, TNOR, TMIN, TMAX, and Consensus.
//! 3. Balanced Ternary Full Adder (TFA) cell across all 27 combinations.
//! 4. Mathematical inversion symmetry: \(TFA(-A, -B, -C_{in}) = (-C_{out}, -S)\).
//! 5. Quaternary (radix-4) logic levels and analog quantization.

use phonon_models::mvl::{
    Quat, TernaryFullAdderCell, TernaryGates, TernaryInverters, TfaOutput, Trit,
};

#[test]
fn test_exhaustive_ternary_inverters() {
    // 1. Simple Ternary Inverter (STI)
    assert_eq!(TernaryInverters::sti(Trit::Neg), Trit::Pos);
    assert_eq!(TernaryInverters::sti(Trit::Zero), Trit::Zero);
    assert_eq!(TernaryInverters::sti(Trit::Pos), Trit::Neg);

    // 2. Positive Ternary Inverter (PTI)
    assert_eq!(TernaryInverters::pti(Trit::Neg), Trit::Pos);
    assert_eq!(TernaryInverters::pti(Trit::Zero), Trit::Pos);
    assert_eq!(TernaryInverters::pti(Trit::Pos), Trit::Neg);

    // 3. Negative Ternary Inverter (NTI)
    assert_eq!(TernaryInverters::nti(Trit::Neg), Trit::Pos);
    assert_eq!(TernaryInverters::nti(Trit::Zero), Trit::Neg);
    assert_eq!(TernaryInverters::nti(Trit::Pos), Trit::Neg);

    // Involutory property of STI: STI(STI(x)) == x
    for &t in &[Trit::Neg, Trit::Zero, Trit::Pos] {
        assert_eq!(TernaryInverters::sti(TernaryInverters::sti(t)), t);
    }
}

#[test]
fn test_exhaustive_ternary_logic_gates() {
    let trits = [Trit::Neg, Trit::Zero, Trit::Pos];

    for &a in &trits {
        for &b in &trits {
            // TMIN: algebraic minimum
            let min_expected = if a.as_i8() <= b.as_i8() { a } else { b };
            assert_eq!(TernaryGates::min(a, b), min_expected);

            // TMAX: algebraic maximum
            let max_expected = if a.as_i8() >= b.as_i8() { a } else { b };
            assert_eq!(TernaryGates::max(a, b), max_expected);

            // TNAND = STI(min(a, b))
            assert_eq!(TernaryGates::nand(a, b), -min_expected);

            // TNOR = STI(max(a, b))
            assert_eq!(TernaryGates::nor(a, b), -max_expected);
        }
    }

    // Consensus function: majority voting across 3 trits
    assert_eq!(
        TernaryGates::consensus(Trit::Pos, Trit::Pos, Trit::Neg),
        Trit::Pos
    );
    assert_eq!(
        TernaryGates::consensus(Trit::Neg, Trit::Neg, Trit::Zero),
        Trit::Neg
    );
    assert_eq!(
        TernaryGates::consensus(Trit::Pos, Trit::Neg, Trit::Zero),
        Trit::Zero
    );
}

#[test]
fn test_exhaustive_tfa_27_combinations_and_symmetry() {
    let tfa = TernaryFullAdderCell::default();
    let trits = [Trit::Neg, Trit::Zero, Trit::Pos];

    let mut evaluated_count = 0;

    for &a in &trits {
        for &b in &trits {
            for &cin in &trits {
                let TfaOutput { sum, carry_out } = tfa.evaluate(a, b, cin);

                // Exact arithmetic equation: A + B + Cin = 3 * Cout + S
                let lhs = a.as_i8() + b.as_i8() + cin.as_i8();
                let rhs = 3 * carry_out.as_i8() + sum.as_i8();
                assert_eq!(
                    lhs, rhs,
                    "Arithmetic balance violated at A={a:?}, B={b:?}, Cin={cin:?}"
                );

                // Arithmetic inversion symmetry
                assert!(
                    tfa.verify_inversion_symmetry(a, b, cin),
                    "Inversion symmetry violated at A={a:?}, B={b:?}, Cin={cin:?}"
                );

                evaluated_count += 1;
            }
        }
    }

    assert_eq!(
        evaluated_count, 27,
        "Must exhaustively test all 27 ternary input combinations"
    );
}

#[test]
fn test_quaternary_quantization_and_levels() {
    let v_dd = 1.2;

    // Test nominal voltage levels for quaternary states
    assert!((Quat::Q0.nominal_voltage(v_dd) - 0.0).abs() < 1e-6);
    assert!((Quat::Q1.nominal_voltage(v_dd) - 0.4).abs() < 1e-6);
    assert!((Quat::Q2.nominal_voltage(v_dd) - 0.8).abs() < 1e-6);
    assert!((Quat::Q3.nominal_voltage(v_dd) - 1.2).abs() < 1e-6);

    // Test analog voltage quantization into discrete Quat levels
    assert_eq!(Quat::from_voltage(0.05, v_dd), Quat::Q0);
    assert_eq!(Quat::from_voltage(0.38, v_dd), Quat::Q1);
    assert_eq!(Quat::from_voltage(0.82, v_dd), Quat::Q2);
    assert_eq!(Quat::from_voltage(1.15, v_dd), Quat::Q3);
}
