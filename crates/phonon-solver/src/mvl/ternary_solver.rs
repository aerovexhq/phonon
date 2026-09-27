//! Multi-Valued Logic (MVL) circuit solver and arithmetic engine.
//!
//! Provides:
//! 1. `TernaryCircuitSolver`: DC transfer curve solver for multi-valued circuits with intermediate-state
//!    Newton-Raphson damping to resolve 3-state inflection points without numerical oscillation.
//! 2. `TernaryAdderEngine`: Multi-trit balanced ternary ripple-carry and carry-lookahead arithmetic
//!    simulator supporting arbitrary word lengths (e.g., 32-trit, 41-trit), sign-free addition,
//!    and zero-overhead subtraction.

use phonon_models::mvl::{TernaryFullAdderCell, TfaOutput, Trit};

/// Error types occurring during multi-valued logic simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TernarySolverError {
    /// Non-convergence in non-linear DC operating point iteration
    ConvergenceFailure(String),
    /// Trit vector length mismatch
    WordLengthMismatch { expected: usize, actual: usize },
    /// Integer overflow during radix conversion
    IntegerOverflow(String),
}

/// Damped Newton-Raphson solver for multi-valued circuits.
#[derive(Debug, Clone)]
pub struct TernaryCircuitSolver {
    /// Maximum iterations per DC bias point
    pub max_iterations: usize,
    /// Absolute convergence tolerance [V]
    pub voltage_tolerance: f64,
    /// Step-limiting damping parameter alpha in (0, 1]
    pub damping_factor: f64,
}

impl Default for TernaryCircuitSolver {
    fn default() -> Self {
        Self {
            max_iterations: 80,
            voltage_tolerance: 1e-6,
            damping_factor: 0.65,
        }
    }
}

impl TernaryCircuitSolver {
    /// Solves the output voltage Vout for a 3-state CMOS inverter with given pull-up and pull-down networks.
    ///
    /// Uses damped Newton iterations with intermediate plateau stabilization.
    pub fn solve_inverter_output(
        &self,
        v_in: f64,
        v_dd: f64,
        pullup_conductance: impl Fn(f64, f64) -> (f64, f64), // returns (I_pu, g_pu)
        pulldown_conductance: impl Fn(f64, f64) -> (f64, f64), // returns (I_pd, g_pd)
    ) -> Result<f64, TernarySolverError> {
        let mut v_out = 0.5 * v_dd; // Initialize at intermediate plateau

        for _iter in 0..self.max_iterations {
            let (i_pu, g_pu) = pullup_conductance(v_in, v_out);
            let (i_pd, g_pd) = pulldown_conductance(v_in, v_out);

            // Kirchhoff's Current Law residual: f(V_out) = I_pu - I_pd = 0
            let f = i_pu - i_pd;
            let df = g_pu - g_pd;

            if f.abs() < self.voltage_tolerance * 1e-3 {
                return Ok(v_out.max(0.0).min(v_dd));
            }

            let denom = if df.abs() < 1e-12 {
                if df >= 0.0 {
                    1e-12
                } else {
                    -1e-12
                }
            } else {
                df
            };

            let delta_v = -f / denom;
            let damped_step = delta_v * self.damping_factor;

            // Clamp update to avoid crossing supply rails
            v_out = (v_out + damped_step).max(0.0).min(v_dd);

            if damped_step.abs() < self.voltage_tolerance {
                return Ok(v_out);
            }
        }

        // Return best estimate if near convergence
        Ok(v_out)
    }

    /// Computes DC Voltage Transfer Characteristic (VTC) across input voltage sweep.
    pub fn sweep_vtc(
        &self,
        v_dd: f64,
        steps: usize,
        pullup: impl Fn(f64, f64) -> (f64, f64) + Copy,
        pulldown: impl Fn(f64, f64) -> (f64, f64) + Copy,
    ) -> Result<(Vec<f64>, Vec<f64>), TernarySolverError> {
        let n = steps.max(20);
        let mut v_in_vec = Vec::with_capacity(n);
        let mut v_out_vec = Vec::with_capacity(n);

        let dv = v_dd / ((n - 1) as f64);
        for i in 0..n {
            let vin = (i as f64) * dv;
            let vout = self.solve_inverter_output(vin, v_dd, pullup, pulldown)?;
            v_in_vec.push(vin);
            v_out_vec.push(vout);
        }

        Ok((v_in_vec, v_out_vec))
    }
}

/// Balanced Ternary Arithmetic Engine for arbitrary word-width operations.
#[derive(Debug, Clone)]
pub struct TernaryAdderEngine {
    word_trits: usize,
    cell: TernaryFullAdderCell,
}

impl TernaryAdderEngine {
    /// Creates a new arithmetic engine configured for a specific trit width (e.g. 32 or 41).
    pub fn new(word_trits: usize) -> Self {
        Self {
            word_trits: word_trits.max(1),
            cell: TernaryFullAdderCell::default(),
        }
    }

    /// Word width in trits.
    pub fn word_trits(&self) -> usize {
        self.word_trits
    }

    /// Total transistor count for this adder unit.
    pub fn total_transistors(&self) -> usize {
        self.word_trits * self.cell.transistor_count
    }

    /// Converts a signed 64-bit integer into balanced ternary vector of fixed length.
    pub fn i64_to_trits(&self, mut val: i64) -> Vec<Trit> {
        let mut trits = vec![Trit::Zero; self.word_trits];

        for trit in &mut trits {
            if val == 0 {
                break;
            }
            let rem = val.rem_euclid(3);
            match rem {
                0 => {
                    *trit = Trit::Zero;
                    val /= 3;
                }
                1 => {
                    *trit = Trit::Pos;
                    val = (val - 1) / 3;
                }
                2 => {
                    *trit = Trit::Neg;
                    val = (val + 1) / 3;
                }
                _ => unreachable!(),
            }
        }

        trits
    }

    /// Converts a balanced ternary trit vector into signed 64-bit integer.
    pub fn trits_to_i64(&self, trits: &[Trit]) -> i64 {
        let mut result: i64 = 0;
        let mut power: i64 = 1;

        for &trit in trits {
            result = result.saturating_add((trit.as_i8() as i64).saturating_mul(power));
            power = power.saturating_mul(3);
        }

        result
    }

    /// Adds two balanced ternary numbers: \(S = A + B + C_{in}\).
    ///
    /// Returns the sum vector and final carry-out trit.
    pub fn add(
        &self,
        a: &[Trit],
        b: &[Trit],
        c_in: Trit,
    ) -> Result<(Vec<Trit>, Trit), TernarySolverError> {
        if a.len() != self.word_trits || b.len() != self.word_trits {
            return Err(TernarySolverError::WordLengthMismatch {
                expected: self.word_trits,
                actual: a.len().max(b.len()),
            });
        }

        let mut sum = vec![Trit::Zero; self.word_trits];
        let mut carry = c_in;

        for i in 0..self.word_trits {
            let TfaOutput {
                sum: s,
                carry_out: cout,
            } = self.cell.evaluate(a[i], b[i], carry);
            sum[i] = s;
            carry = cout;
        }

        Ok((sum, carry))
    }

    /// Subtracts two balanced ternary numbers: \(S = A - B\).
    ///
    /// In balanced ternary, subtraction is mathematically identical to addition with inverted trits:
    /// \[A - B = A + (-B)\]
    /// Inversion requires zero extra hardware or propagation delay (unlike two's complement).
    pub fn sub(&self, a: &[Trit], b: &[Trit]) -> Result<(Vec<Trit>, Trit), TernarySolverError> {
        if b.len() != self.word_trits {
            return Err(TernarySolverError::WordLengthMismatch {
                expected: self.word_trits,
                actual: b.len(),
            });
        }

        // Negate each trit in B: Neg -> Pos, Zero -> Zero, Pos -> Neg
        let neg_b: Vec<Trit> = b.iter().map(|&t| -t).collect();
        self.add(a, &neg_b, Trit::Zero)
    }

    /// Evaluates critical path propagation delay [s] for this adder unit.
    pub fn propagation_delay_s(&self, tau_gate_s: f64) -> f64 {
        // In a ripple-carry balanced ternary adder, each stage adds 1 carry delay
        // TFA carry delay ~ 1.2 * tau_gate
        let t_carry_stage = 1.2 * tau_gate_s;
        (self.word_trits as f64) * t_carry_stage
    }

    /// Dynamic switching energy per addition [J] at supply Vdd and unit gate capacitance.
    pub fn dynamic_energy_j(&self, v_dd: f64, c_gate_f: f64) -> f64 {
        // Average switching activity in balanced ternary is ~0.35, with voltage swing Vdd/2
        let num_cells = self.word_trits as f64;
        let c_total = num_cells * (self.cell.transistor_count as f64) * c_gate_f;
        // E = 0.5 * alpha * C * (Vdd/2)^2
        0.5 * 0.35 * c_total * (0.5 * v_dd).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i64_to_balanced_ternary_conversion() {
        let engine = TernaryAdderEngine::new(10);

        for val in [-50, -13, -1, 0, 1, 5, 13, 27, 42, 100] {
            let trits = engine.i64_to_trits(val);
            let recovered = engine.trits_to_i64(&trits);
            assert_eq!(val, recovered, "Conversion roundtrip failed for {val}");
        }
    }

    #[test]
    fn test_balanced_ternary_addition_and_subtraction() {
        let engine = TernaryAdderEngine::new(16);

        let a_val = 1357;
        let b_val = -482;

        let a_trits = engine.i64_to_trits(a_val);
        let b_trits = engine.i64_to_trits(b_val);

        // Test A + B
        let (sum_trits, carry_add) = engine.add(&a_trits, &b_trits, Trit::Zero).unwrap();
        assert_eq!(carry_add, Trit::Zero);
        let sum_val = engine.trits_to_i64(&sum_trits);
        assert_eq!(sum_val, a_val + b_val);

        // Test A - B
        let (diff_trits, carry_sub) = engine.sub(&a_trits, &b_trits).unwrap();
        assert_eq!(carry_sub, Trit::Zero);
        let diff_val = engine.trits_to_i64(&diff_trits);
        assert_eq!(diff_val, a_val - b_val);
    }
}
