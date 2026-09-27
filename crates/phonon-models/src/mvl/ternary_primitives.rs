//! Balanced ternary and quaternary logic primitives and arithmetic cells.
//!
//! Implements:
//! 1. `Trit` radix-3 logic states: `Neg` (-1), `Zero` (0), `Pos` (+1).
//! 2. `Quat` radix-4 logic states: `Q0`, `Q1`, `Q2`, `Q3`.
//! 3. Fundamental ternary inverters: Simple Ternary Inverter (STI), Positive Ternary Inverter (PTI),
//!    and Negative Ternary Inverter (NTI).
//! 4. Canonical ternary logic gates: TNAND, TNOR, TMIN, TMAX.
//! 5. Balanced Ternary Full Adder (TFA) cell satisfying \(A + B + C_{in} = 3 \cdot C_{out} + S\)
//!    with exact arithmetic inversion symmetry.

use std::ops::Neg;

/// Balanced ternary digit (trit) with symmetric values {-1, 0, +1}.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Trit {
    /// Negative state (-1, or 'T' / '-')
    Neg = -1,
    /// Zero intermediate state (0)
    Zero = 0,
    /// Positive state (+1, or '+')
    Pos = 1,
}

impl Trit {
    /// Converts trit to integer value \(-1, 0, +1\).
    pub fn as_i8(&self) -> i8 {
        *self as i8
    }

    /// Converts an integer in \([-1, 1]\) to a Trit.
    pub fn from_i8(val: i8) -> Option<Self> {
        match val {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }

    /// Converts physical voltage to discrete Trit given supply voltage Vdd.
    ///
    /// Low: \(V < V_{dd} / 3\)
    /// Mid: \(V_{dd} / 3 \le V < 2 V_{dd} / 3\)
    /// High: \(V \ge 2 V_{dd} / 3\)
    pub fn from_voltage(v: f64, v_dd: f64) -> Self {
        let v_norm = v / v_dd.max(1e-3);
        if v_norm < 1.0 / 3.0 {
            Trit::Neg
        } else if v_norm < 2.0 / 3.0 {
            Trit::Zero
        } else {
            Trit::Pos
        }
    }

    /// Returns the nominal physical voltage for unipolar supply (0, Vdd/2, Vdd).
    pub fn nominal_voltage(&self, v_dd: f64) -> f64 {
        match self {
            Trit::Neg => 0.0,
            Trit::Zero => 0.5 * v_dd,
            Trit::Pos => v_dd,
        }
    }

    /// Returns the nominal physical voltage for bipolar supply (-Vdd/2, 0, +Vdd/2).
    pub fn nominal_bipolar_voltage(&self, v_dd: f64) -> f64 {
        match self {
            Trit::Neg => -0.5 * v_dd,
            Trit::Zero => 0.0,
            Trit::Pos => 0.5 * v_dd,
        }
    }

    /// Simple string symbol ('-', '0', '+').
    pub fn symbol(&self) -> char {
        match self {
            Trit::Neg => '-',
            Trit::Zero => '0',
            Trit::Pos => '+',
        }
    }
}

impl Neg for Trit {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Trit::Neg => Trit::Pos,
            Trit::Zero => Trit::Zero,
            Trit::Pos => Trit::Neg,
        }
    }
}

/// Quaternary (radix-4) logic states {0, 1, 2, 3}.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Quat {
    Q0 = 0,
    Q1 = 1,
    Q2 = 2,
    Q3 = 3,
}

impl Quat {
    /// Integer value 0..=3.
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Nominal voltage in unipolar supply [0, Vdd/3, 2Vdd/3, Vdd].
    pub fn nominal_voltage(&self, v_dd: f64) -> f64 {
        match self {
            Quat::Q0 => 0.0,
            Quat::Q1 => (1.0 / 3.0) * v_dd,
            Quat::Q2 => (2.0 / 3.0) * v_dd,
            Quat::Q3 => v_dd,
        }
    }

    /// Quantizes analog voltage into Quat state.
    pub fn from_voltage(v: f64, v_dd: f64) -> Self {
        let v_norm = v / v_dd.max(1e-3);
        if v_norm < 0.5 / 3.0 {
            Quat::Q0
        } else if v_norm < 1.5 / 3.0 {
            Quat::Q1
        } else if v_norm < 2.5 / 3.0 {
            Quat::Q2
        } else {
            Quat::Q3
        }
    }
}

/// Fundamental Ternary Inverters (STI, PTI, NTI).
pub struct TernaryInverters;

impl TernaryInverters {
    /// Simple Ternary Inverter (STI):
    /// Neg (-1) -> Pos (+1)
    /// Zero (0) -> Zero (0)
    /// Pos (+1) -> Neg (-1)
    pub fn sti(x: Trit) -> Trit {
        -x
    }

    /// Positive Ternary Inverter (PTI):
    /// Neg (-1) -> Pos (+1)
    /// Zero (0) -> Pos (+1)
    /// Pos (+1) -> Neg (-1)
    pub fn pti(x: Trit) -> Trit {
        match x {
            Trit::Neg | Trit::Zero => Trit::Pos,
            Trit::Pos => Trit::Neg,
        }
    }

    /// Negative Ternary Inverter (NTI):
    /// Neg (-1) -> Pos (+1)
    /// Zero (0) -> Neg (-1)
    /// Pos (+1) -> Neg (-1)
    pub fn nti(x: Trit) -> Trit {
        match x {
            Trit::Neg => Trit::Pos,
            Trit::Zero | Trit::Pos => Trit::Neg,
        }
    }
}

/// Canonical multi-valued logic gates.
pub struct TernaryGates;

impl TernaryGates {
    /// Ternary Minimum (conjunction / AND-equivalent).
    pub fn min(a: Trit, b: Trit) -> Trit {
        if a.as_i8() <= b.as_i8() {
            a
        } else {
            b
        }
    }

    /// Ternary Maximum (disjunction / OR-equivalent).
    pub fn max(a: Trit, b: Trit) -> Trit {
        if a.as_i8() >= b.as_i8() {
            a
        } else {
            b
        }
    }

    /// Ternary NAND: STI(min(A, B)).
    pub fn nand(a: Trit, b: Trit) -> Trit {
        TernaryInverters::sti(Self::min(a, b))
    }

    /// Ternary NOR: STI(max(A, B)).
    pub fn nor(a: Trit, b: Trit) -> Trit {
        TernaryInverters::sti(Self::max(a, b))
    }

    /// Ternary Consensus (consensus between three trits, majority voting).
    pub fn consensus(a: Trit, b: Trit, c: Trit) -> Trit {
        let sum = a.as_i8() + b.as_i8() + c.as_i8();
        if sum > 0 {
            Trit::Pos
        } else if sum < 0 {
            Trit::Neg
        } else {
            Trit::Zero
        }
    }
}

/// Balanced Ternary Full Adder (TFA) cell result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TfaOutput {
    /// Sum trit S
    pub sum: Trit,
    /// Carry out trit Cout
    pub carry_out: Trit,
}

/// Balanced Ternary Full Adder (TFA) cell.
///
/// Implements the exact balanced ternary arithmetic relation:
/// \[A + B + C_{in} = 3 \cdot C_{out} + S\]
/// where all signals are balanced trits in \(\{-1, 0, +1\}\).
#[derive(Debug, Clone, Copy)]
pub struct TernaryFullAdderCell {
    /// Transistor count in multi-threshold CMOS realization
    pub transistor_count: usize,
    /// Total active gate width [m]
    pub active_width_m: f64,
}

impl Default for TernaryFullAdderCell {
    fn default() -> Self {
        // High-density multi-threshold CMOS TFA implementation uses 14 transistors
        Self {
            transistor_count: 14,
            active_width_m: 14.0 * 100e-9,
        }
    }
}

impl TernaryFullAdderCell {
    /// Evaluates the 1-trit Balanced Ternary Full Adder cell for inputs A, B, and Cin.
    pub fn evaluate(&self, a: Trit, b: Trit, c_in: Trit) -> TfaOutput {
        let total_sum = a.as_i8() + b.as_i8() + c_in.as_i8();

        let (cout_val, sum_val) = match total_sum {
            3 => (1, 0),   // +3 = 3 * (+1) + 0
            2 => (1, -1),  // +2 = 3 * (+1) + (-1)
            1 => (0, 1),   // +1 = 3 * (0) + (+1)
            0 => (0, 0),   //  0 = 3 * (0) + (0)
            -1 => (0, -1), // -1 = 3 * (0) + (-1)
            -2 => (-1, 1), // -2 = 3 * (-1) + (+1)
            -3 => (-1, 0), // -3 = 3 * (-1) + 0
            _ => unreachable!("Total sum of three trits cannot exceed [-3, +3]"),
        };

        TfaOutput {
            sum: Trit::from_i8(sum_val).unwrap(),
            carry_out: Trit::from_i8(cout_val).unwrap(),
        }
    }

    /// Verifies the arithmetic inversion symmetry:
    /// \[TFA(-A, -B, -C_{in}) = (-C_{out}, -S)\]
    pub fn verify_inversion_symmetry(&self, a: Trit, b: Trit, c_in: Trit) -> bool {
        let fwd = self.evaluate(a, b, c_in);
        let rev = self.evaluate(-a, -b, -c_in);
        rev.sum == -fwd.sum && rev.carry_out == -fwd.carry_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trit_inverters() {
        assert_eq!(TernaryInverters::sti(Trit::Neg), Trit::Pos);
        assert_eq!(TernaryInverters::sti(Trit::Zero), Trit::Zero);
        assert_eq!(TernaryInverters::sti(Trit::Pos), Trit::Neg);

        assert_eq!(TernaryInverters::pti(Trit::Neg), Trit::Pos);
        assert_eq!(TernaryInverters::pti(Trit::Zero), Trit::Pos);
        assert_eq!(TernaryInverters::pti(Trit::Pos), Trit::Neg);

        assert_eq!(TernaryInverters::nti(Trit::Neg), Trit::Pos);
        assert_eq!(TernaryInverters::nti(Trit::Zero), Trit::Neg);
        assert_eq!(TernaryInverters::nti(Trit::Pos), Trit::Neg);
    }

    #[test]
    fn test_tfa_all_combinations_and_symmetry() {
        let tfa = TernaryFullAdderCell::default();
        let trits = [Trit::Neg, Trit::Zero, Trit::Pos];

        for &a in &trits {
            for &b in &trits {
                for &cin in &trits {
                    let out = tfa.evaluate(a, b, cin);
                    let lhs = a.as_i8() + b.as_i8() + cin.as_i8();
                    let rhs = 3 * out.carry_out.as_i8() + out.sum.as_i8();
                    assert_eq!(lhs, rhs, "Mismatch at A={a:?}, B={b:?}, Cin={cin:?}");
                    assert!(
                        tfa.verify_inversion_symmetry(a, b, cin),
                        "Inversion symmetry violated at A={a:?}, B={b:?}, Cin={cin:?}"
                    );
                }
            }
        }
    }
}
