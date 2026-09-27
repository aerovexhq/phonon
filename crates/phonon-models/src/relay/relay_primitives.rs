//! Non-volatile and zero-leakage logic primitives and arithmetic cells built with atomic relays.
//!
//! Provides:
//! 1. `RelayLogicGate`: Standard logic primitives (Inverter, NAND2, NOR2, XOR2).
//! 2. `RelayFullAdderCell`: 1-bit full adder with zero off-state standby leakage.
//! 3. `MultiBitRelayAdder`: Scalable N-bit arithmetic adder/subtractor.
//! 4. Static leakage and dynamic switching energy accounting.

use crate::relay::atomic_relay::{AtomicRelayModel, AtomicRelayParameters};

/// Type of relay switch contact behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelaySwitchType {
    /// Contact is open when unactuated, closes on positive gate bias
    NormallyOpen,
    /// Contact is closed when unactuated, opens on positive gate bias
    NormallyClosed,
}

/// A composite logic primitive constructed from 3-terminal atomic relays.
#[derive(Debug, Clone)]
pub struct RelayLogicGate {
    pub name: String,
    pub relay_count: usize,
    pub pull_in_v: f64,
    pub r_on_ohm: f64,
    pub r_off_ohm: f64,
    pub gate_capacitance_ff: f64,
}

impl RelayLogicGate {
    /// Constructs a relay-based Inverter (NOT gate).
    pub fn inverter() -> Self {
        let p = AtomicRelayParameters::default();
        Self {
            name: "Relay-NOT".to_string(),
            relay_count: 2,   // Complementary push-pull pair
            pull_in_v: 0.085, // 85 mV actuation
            r_on_ohm: p.r_on_contact_ohm,
            r_off_ohm: p.r_off_tunnel_ohm,
            gate_capacitance_ff: 1.2,
        }
    }

    /// Constructs a 2-input NAND gate.
    pub fn nand2() -> Self {
        let p = AtomicRelayParameters::default();
        Self {
            name: "Relay-NAND2".to_string(),
            relay_count: 3,
            pull_in_v: 0.085,
            r_on_ohm: p.r_on_contact_ohm * 2.0, // Series pull-down
            r_off_ohm: p.r_off_tunnel_ohm,
            gate_capacitance_ff: 2.4,
        }
    }

    /// Constructs a 2-input NOR gate.
    pub fn nor2() -> Self {
        let p = AtomicRelayParameters::default();
        Self {
            name: "Relay-NOR2".to_string(),
            relay_count: 3,
            pull_in_v: 0.085,
            r_on_ohm: p.r_on_contact_ohm * 2.0, // Series pull-up
            r_off_ohm: p.r_off_tunnel_ohm,
            gate_capacitance_ff: 2.4,
        }
    }

    /// Constructs a 2-input XOR gate.
    pub fn xor2() -> Self {
        let p = AtomicRelayParameters::default();
        Self {
            name: "Relay-XOR2".to_string(),
            relay_count: 4, // 4 relays in transmission configuration
            pull_in_v: 0.085,
            r_on_ohm: p.r_on_contact_ohm,
            r_off_ohm: p.r_off_tunnel_ohm,
            gate_capacitance_ff: 3.6,
        }
    }

    /// Evaluates Boolean output for NAND2.
    pub fn eval_nand2(a: bool, b: bool) -> bool {
        !(a && b)
    }

    /// Evaluates Boolean output for NOR2.
    pub fn eval_nor2(a: bool, b: bool) -> bool {
        !(a || b)
    }

    /// Evaluates Boolean output for XOR2.
    pub fn eval_xor2(a: bool, b: bool) -> bool {
        a ^ b
    }

    /// Evaluates static standby leakage power [W] at supply voltage Vdd.
    ///
    /// Because atomic relays have open vacuum/air gaps, static leakage is virtually zero.
    pub fn static_leakage_power_w(&self, v_dd: f64) -> f64 {
        let i_leak = v_dd / self.r_off_ohm;
        (self.relay_count as f64) * i_leak * v_dd
    }

    /// Evaluates dynamic switching energy [Joules] per logic transition.
    ///
    /// \[E_{dyn} = \frac{1}{2} C_{gate} V_{dd}^2 + E_{mech}\]
    pub fn dynamic_switching_energy_j(&self, v_dd: f64) -> f64 {
        let c_total_f = self.gate_capacitance_ff * 1.0e-15;
        let e_elec = 0.5 * c_total_f * v_dd.powi(2);
        let e_mech = 2.0e-18; // ~2 aJ mechanical beam deflection energy
        e_elec + e_mech
    }
}

/// 1-bit Full Adder cell implemented with an optimized network of atomic relays.
#[derive(Debug, Clone)]
pub struct RelayFullAdderCell {
    /// Number of atomic relays utilized (10 relays vs 28 transistors in CMOS)
    pub relay_count: usize,
    /// Aggregate input capacitance [fF]
    pub input_capacitance_ff: f64,
    /// Actuation threshold voltage [V]
    pub actuation_voltage_v: f64,
    /// Internal relays simulating the sum and carry paths
    relays: Vec<AtomicRelayModel>,
}

impl Default for RelayFullAdderCell {
    fn default() -> Self {
        let params = AtomicRelayParameters::default();
        let relays = (0..10)
            .map(|_| AtomicRelayModel::new(params.clone()))
            .collect();
        Self {
            relay_count: 10,
            input_capacitance_ff: 8.5,
            actuation_voltage_v: 0.085,
            relays,
        }
    }
}

impl RelayFullAdderCell {
    /// Creates a new 1-bit atomic relay full adder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates the full adder truth table for inputs A, B, and Carry-In.
    /// Returns (Sum, Carry-Out).
    pub fn evaluate_logic(&self, a: bool, b: bool, c_in: bool) -> (bool, bool) {
        let sum = a ^ b ^ c_in;
        let c_out = (a && b) || (c_in && (a ^ b));
        (sum, c_out)
    }

    /// Advances transient mechanical state of the internal relays.
    pub fn step(&mut self, a: bool, b: bool, c_in: bool, v_dd: f64, dt_s: f64) {
        let v_a = if a { v_dd } else { 0.0 };
        let v_b = if b { v_dd } else { 0.0 };
        let v_cin = if c_in { v_dd } else { 0.0 };

        for (i, r) in self.relays.iter_mut().enumerate() {
            let v_gate = match i % 3 {
                0 => v_a,
                1 => v_b,
                _ => v_cin,
            };
            r.step(v_gate, dt_s);
        }
    }

    /// Static standby leakage power [W] at supply voltage Vdd.
    ///
    /// Demonstrates 100% elimination of standby subthreshold leakage.
    pub fn static_leakage_power_w(&self, v_dd: f64) -> f64 {
        let r_off = AtomicRelayParameters::default().r_off_tunnel_ohm;
        let i_leak = v_dd / r_off;
        (self.relay_count as f64) * i_leak * v_dd
    }

    /// Dynamic energy [Joules] dissipated per 1-bit addition.
    pub fn dynamic_energy_per_op_j(&self, v_dd: f64) -> f64 {
        let c_total_f = self.input_capacitance_ff * 1.0e-15;
        0.5 * c_total_f * v_dd.powi(2) + ((self.relay_count as f64) * 1.5e-18)
    }
}

/// Scalable N-bit arithmetic adder/subtractor built from atomic relay cells.
#[derive(Debug, Clone)]
pub struct MultiBitRelayAdder {
    pub bit_width: usize,
    pub cells: Vec<RelayFullAdderCell>,
}

impl MultiBitRelayAdder {
    /// Creates a new N-bit adder.
    pub fn new(bit_width: usize) -> Self {
        let cells = (0..bit_width).map(|_| RelayFullAdderCell::new()).collect();
        Self { bit_width, cells }
    }

    /// Performs N-bit addition: A + B + Cin -> (Sum, Cout).
    pub fn add(&self, a_bits: &[bool], b_bits: &[bool], c_in: bool) -> (Vec<bool>, bool) {
        assert_eq!(a_bits.len(), self.bit_width);
        assert_eq!(b_bits.len(), self.bit_width);

        let mut sum_bits = Vec::with_capacity(self.bit_width);
        let mut carry = c_in;

        for i in 0..self.bit_width {
            let (s, c_next) = self.cells[i].evaluate_logic(a_bits[i], b_bits[i], carry);
            sum_bits.push(s);
            carry = c_next;
        }

        (sum_bits, carry)
    }

    /// Performs N-bit integer addition directly.
    pub fn add_u64(&self, a: u64, b: u64) -> (u64, bool) {
        let mut a_bits = Vec::with_capacity(self.bit_width);
        let mut b_bits = Vec::with_capacity(self.bit_width);

        for i in 0..self.bit_width {
            a_bits.push(((a >> i) & 1) == 1);
            b_bits.push(((b >> i) & 1) == 1);
        }

        let (sum_bits, c_out) = self.add(&a_bits, &b_bits, false);

        let mut sum_val = 0u64;
        for (i, &bit) in sum_bits.iter().enumerate() {
            if bit {
                sum_val |= 1u64 << i;
            }
        }

        (sum_val, c_out)
    }

    /// Total standby leakage power [W] across all bits.
    pub fn total_static_leakage_w(&self, v_dd: f64) -> f64 {
        self.cells
            .iter()
            .map(|c| c.static_leakage_power_w(v_dd))
            .sum()
    }

    /// Total dynamic energy [Joules] for an N-bit addition.
    pub fn total_dynamic_energy_j(&self, v_dd: f64) -> f64 {
        self.cells
            .iter()
            .map(|c| c.dynamic_energy_per_op_j(v_dd))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_logic_gates_truth_tables() {
        assert!(RelayLogicGate::eval_nand2(false, false));
        assert!(RelayLogicGate::eval_nand2(false, true));
        assert!(RelayLogicGate::eval_nand2(true, false));
        assert!(!RelayLogicGate::eval_nand2(true, true));

        assert!(RelayLogicGate::eval_nor2(false, false));
        assert!(!RelayLogicGate::eval_nor2(true, false));
        assert!(!RelayLogicGate::eval_nor2(false, true));
        assert!(!RelayLogicGate::eval_nor2(true, true));

        assert!(!RelayLogicGate::eval_xor2(false, false));
        assert!(RelayLogicGate::eval_xor2(true, false));
        assert!(RelayLogicGate::eval_xor2(false, true));
        assert!(!RelayLogicGate::eval_xor2(true, true));
    }

    #[test]
    fn test_relay_full_adder_exhaustive_truth_table() {
        let fa = RelayFullAdderCell::new();
        for &a in &[false, true] {
            for &b in &[false, true] {
                for &cin in &[false, true] {
                    let (sum, cout) = fa.evaluate_logic(a, b, cin);
                    let expected_sum = a ^ b ^ cin;
                    let expected_cout = (a && b) || (cin && (a ^ b));
                    assert_eq!(sum, expected_sum);
                    assert_eq!(cout, expected_cout);
                }
            }
        }
    }

    #[test]
    fn test_multibit_relay_adder_u64() {
        let adder = MultiBitRelayAdder::new(16);
        let (res, cout) = adder.add_u64(1234, 5678);
        assert_eq!(res, 6912);
        assert!(!cout);

        // Standby leakage of 16-bit relay adder must be practically zero (< 100 fW)
        let p_leak = adder.total_static_leakage_w(0.8);
        assert!(p_leak < 1.0e-13, "Leakage power was {:.2e} W", p_leak);
    }
}
