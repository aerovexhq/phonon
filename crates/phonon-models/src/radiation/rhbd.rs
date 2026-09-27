//! Radiation-Hardened-by-Design (RHBD) architectures.
//!
//! Formulates:
//! - Dual-Interlocked Storage Cell (DICE) latch with 4-node cross-coupled regenerative topology.
//!   - Single-node strike immunity: Perturbation on any single internal node $X_i$ is isolated
//!     and dynamically restored by the orthogonal feedback paths.
//!   - Two-node charge sharing vulnerability modeling.
//! - Standard 6T SRAM cell critical charge $Q_{crit}$ and Single-Event Upset (SEU) comparison.
//! - Triple-Modular Redundancy (TMR) with majority voter logic and disagreeing channel detection:
//!   $$V_{out} = (A \cdot B) + (B \cdot C) + (A \cdot C)$$
//!   Masks single-event transient (SET) glitches in combinational and sequential data paths.

use phonon_core::TmrOutput;

/// Outcome of a radiation strike on a memory storage cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrikeOutcome {
    /// True if the logical stored bit flipped (Single-Event Upset).
    pub upset_occurred: bool,
    /// True if the cell dynamically self-restored without data corruption.
    pub self_restored: bool,
    /// Critical charge threshold of the cell in fC.
    pub q_crit_fc: f64,
    /// Actual collected charge in fC.
    pub q_coll_fc: f64,
}

/// Standard 6T SRAM cross-coupled inverter storage cell.
#[derive(Debug, Clone, PartialEq)]
pub struct StandardSramCell {
    /// Nominal supply voltage $V_{dd}$ in Volts ($V$).
    pub v_dd_v: f64,
    /// Storage node capacitance $C_{node}$ in Farads ($F$) (typically $0.5 - 2\text{ fF}$).
    pub c_node_f: f64,
    /// Restoring PMOS/NMOS pull-up/pull-down current $I_{restore}$ in Amperes ($A$).
    pub i_restore_a: f64,
    /// Inverter feedback loop delay $\tau_{delay}$ in seconds ($s$) (typically $10 - 50\text{ ps}$).
    pub tau_delay_s: f64,
    /// Current logical state ($Q \in \{0, 1\}$).
    pub state_q: bool,
}

impl StandardSramCell {
    /// Creates a 6T SRAM cell preset in deep-submicron CMOS (e.g. 28nm).
    pub fn new(initial_bit: bool, v_dd: f64) -> Self {
        Self {
            v_dd_v: v_dd.max(0.5),
            c_node_f: 1.0e-15,     // 1.0 fF node capacitance
            i_restore_a: 50.0e-6,  // 50 uA restoring drive current
            tau_delay_s: 20.0e-12, // 20 ps loop delay
            state_q: initial_bit,
        }
    }

    /// Evaluates critical charge $Q_{crit}$ in femtocoulombs ($\text{fC}$):
    ///
    /// $$Q_{crit} = C_{node} V_{dd} + I_{restore} \tau_{delay}$$
    #[inline]
    pub fn critical_charge_fc(&self) -> f64 {
        let q_coulombs = self.c_node_f * self.v_dd_v + self.i_restore_a * self.tau_delay_s;
        q_coulombs * 1.0e15 // convert C to fC
    }

    /// Applies a cosmic heavy-ion particle strike with deposited charge $Q_{coll}$ (fC).
    pub fn strike(&mut self, q_coll_fc: f64) -> StrikeOutcome {
        let q_crit = self.critical_charge_fc();
        let q_coll = q_coll_fc.max(0.0);

        if q_coll >= q_crit {
            // Overcomes restoring current and feedback margin -> flips state permanently
            self.state_q = !self.state_q;
            StrikeOutcome {
                upset_occurred: true,
                self_restored: false,
                q_crit_fc: q_crit,
                q_coll_fc: q_coll,
            }
        } else {
            // Sub-critical charge -> circuit transient dissipates, cell restores
            StrikeOutcome {
                upset_occurred: false,
                self_restored: true,
                q_crit_fc: q_crit,
                q_coll_fc: q_coll,
            }
        }
    }
}

/// Dual-Interlocked Storage Cell (DICE) latch.
///
/// Features 4 internal nodes:
/// - State 0: $(0, 1, 0, 1)$
/// - State 1: $(1, 0, 1, 0)$
///
/// Node $i$ is driven by PMOS($i-1$) and NMOS($i+1$).
/// An upset on any single node cannot propagate around the feedback loop
/// because the orthogonal nodes remain stable and pull the struck node back.
#[derive(Debug, Clone, PartialEq)]
pub struct DiceCell {
    /// Nominal supply voltage $V_{dd}$ in Volts ($V$).
    pub v_dd_v: f64,
    /// Storage node capacitance per internal node in Farads ($F$).
    pub c_node_f: f64,
    /// Restoring transistor drive current in Amperes ($A$).
    pub i_restore_a: f64,
    /// Loop delay in seconds ($s$).
    pub tau_delay_s: f64,
    /// Internal node voltages in Volts: `[X0, X1, X2, X3]`.
    pub node_voltages: [f64; 4],
}

impl DiceCell {
    /// Creates a DICE cell initialized with the given logical bit.
    pub fn new(initial_bit: bool, v_dd: f64) -> Self {
        let vdd = v_dd.max(0.5);
        let node_voltages = if initial_bit {
            // State 1: [Vdd, 0, Vdd, 0]
            [vdd, 0.0, vdd, 0.0]
        } else {
            // State 0: [0, Vdd, 0, Vdd]
            [0.0, vdd, 0.0, vdd]
        };

        Self {
            v_dd_v: vdd,
            c_node_f: 1.0e-15,
            i_restore_a: 50.0e-6,
            tau_delay_s: 20.0e-12,
            node_voltages,
        }
    }

    /// Reads the stored logical bit via majority consensus across the 4 interlocked nodes.
    pub fn read_bit(&self) -> bool {
        let v_th = 0.5 * self.v_dd_v;
        // Nodes 0 and 2 encode the true bit; 1 and 3 encode the inverted bit
        let true_votes =
            (self.node_voltages[0] >= v_th) as u8 + (self.node_voltages[2] >= v_th) as u8;
        let inv_votes = (self.node_voltages[1] < v_th) as u8 + (self.node_voltages[3] < v_th) as u8;
        (true_votes + inv_votes) >= 3
    }

    /// Critical single-node charge threshold (fC) required for standard cells.
    #[inline]
    pub fn standard_q_crit_fc(&self) -> f64 {
        let q_coulombs = self.c_node_f * self.v_dd_v + self.i_restore_a * self.tau_delay_s;
        q_coulombs * 1.0e15
    }

    /// Simulates a single heavy-ion strike directly hitting one internal node `node_idx` $\in [0, 3]$.
    ///
    /// By DICE design, a single node perturbation cannot flip the complementary pair;
    /// the cell is topologically immune to single-node upsets!
    pub fn strike_single_node(&mut self, node_idx: usize, q_coll_fc: f64) -> StrikeOutcome {
        let idx = node_idx % 4;
        let q_crit = self.standard_q_crit_fc();
        let initial_bit = self.read_bit();

        // Perturb the target node
        let delta_v = (q_coll_fc * 1.0e-15) / self.c_node_f;
        if self.node_voltages[idx] > 0.5 * self.v_dd_v {
            self.node_voltages[idx] = (self.node_voltages[idx] - delta_v).max(0.0);
        } else {
            self.node_voltages[idx] = (self.node_voltages[idx] + delta_v).min(self.v_dd_v);
        }

        // Regenerative DICE recovery mechanism:
        // Orthogonal nodes (idx + 2) % 4 and adjacent nodes restore the perturbed node
        self.restore_cell(initial_bit);

        let final_bit = self.read_bit();
        StrikeOutcome {
            upset_occurred: final_bit != initial_bit,
            self_restored: final_bit == initial_bit,
            q_crit_fc: q_crit * 10.0, // DICE has effectively > 10x single-node immunity
            q_coll_fc,
        }
    }

    /// Simulates a dual-node strike (e.g. angled track causing charge sharing across adjacent nodes).
    /// If both adjacent nodes receive charge exceeding $Q_{crit}$, the DICE feedback can be overpowered.
    pub fn strike_adjacent_nodes(
        &mut self,
        node_a: usize,
        q_a_fc: f64,
        node_b: usize,
        q_b_fc: f64,
    ) -> StrikeOutcome {
        let initial_bit = self.read_bit();
        let q_crit = self.standard_q_crit_fc();

        let a = node_a % 4;
        let b = node_b % 4;

        let delta_va = (q_a_fc * 1.0e-15) / self.c_node_f;
        let delta_vb = (q_b_fc * 1.0e-15) / self.c_node_f;

        // Perturb both nodes
        self.node_voltages[a] = if self.node_voltages[a] > 0.5 * self.v_dd_v {
            (self.node_voltages[a] - delta_va).max(0.0)
        } else {
            (self.node_voltages[a] + delta_va).min(self.v_dd_v)
        };

        self.node_voltages[b] = if self.node_voltages[b] > 0.5 * self.v_dd_v {
            (self.node_voltages[b] - delta_vb).max(0.0)
        } else {
            (self.node_voltages[b] + delta_vb).min(self.v_dd_v)
        };

        // If two adjacent nodes both experience charge >= q_crit, state flips
        let adjacent = (a + 1) % 4 == b || (b + 1) % 4 == a;
        if adjacent && q_a_fc >= q_crit && q_b_fc >= q_crit {
            // Flip the entire cell to the inverted state
            self.restore_cell(!initial_bit);
            StrikeOutcome {
                upset_occurred: true,
                self_restored: false,
                q_crit_fc: q_crit,
                q_coll_fc: q_a_fc + q_b_fc,
            }
        } else {
            // Self-restores to original bit
            self.restore_cell(initial_bit);
            StrikeOutcome {
                upset_occurred: false,
                self_restored: true,
                q_crit_fc: q_crit,
                q_coll_fc: q_a_fc + q_b_fc,
            }
        }
    }

    /// Restores cell internal node voltages to canonical stable values for the given bit.
    fn restore_cell(&mut self, bit: bool) {
        if bit {
            self.node_voltages = [self.v_dd_v, 0.0, self.v_dd_v, 0.0];
        } else {
            self.node_voltages = [0.0, self.v_dd_v, 0.0, self.v_dd_v];
        }
    }
}

/// Triple-Modular Redundancy (TMR) Majority Voter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TmrVoter {
    /// Logic high threshold voltage in Volts ($V$) (typically $0.5 V_{dd}$).
    pub threshold_v: f64,
    /// High output voltage level in Volts ($V$).
    pub v_high_v: f64,
    /// Low output voltage level in Volts ($V$).
    pub v_low_v: f64,
}

impl TmrVoter {
    /// Creates a TMR voter for digital CMOS logic at $V_{dd}$.
    pub fn new(v_dd: f64) -> Self {
        Self {
            threshold_v: 0.5 * v_dd,
            v_high_v: v_dd,
            v_low_v: 0.0,
        }
    }

    /// Evaluates the 2-out-of-3 majority vote across three input channel voltages $(V_A, V_B, V_C)$.
    ///
    /// Identifies:
    /// - Output voted level ($V_{high}$ or $V_{low}$)
    /// - Whether any channel faulted / disagreed
    /// - The specific faulted channel index (`Some(0)`, `Some(1)`, `Some(2)`, or `None`)
    pub fn vote(&self, v_a: f64, v_b: f64, v_c: f64) -> TmrOutput {
        let bit_a = v_a >= self.threshold_v;
        let bit_b = v_b >= self.threshold_v;
        let bit_c = v_c >= self.threshold_v;

        // Majority vote logic: (A & B) | (B & C) | (A & C)
        let majority_bit = (bit_a && bit_b) || (bit_b && bit_c) || (bit_a && bit_c);
        let voted_val = if majority_bit {
            self.v_high_v
        } else {
            self.v_low_v
        };

        // Identify faulty channel if any
        let faulted_channel = if bit_a != majority_bit {
            Some(0)
        } else if bit_b != majority_bit {
            Some(1)
        } else if bit_c != majority_bit {
            Some(2)
        } else {
            None
        };

        TmrOutput::new(voted_val, faulted_channel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sram_critical_charge_and_upset() {
        let mut sram = StandardSramCell::new(true, 1.0);
        let q_crit = sram.critical_charge_fc();
        assert!(q_crit > 1.0 && q_crit < 5.0); // Typically ~2 fC

        // Strike with sub-critical charge (1.0 fC)
        let outcome_sub = sram.strike(1.0);
        assert!(!outcome_sub.upset_occurred);
        assert!(outcome_sub.self_restored);
        assert!(sram.state_q); // Stored bit unchanged

        // Strike with supercritical charge (10.0 fC)
        let outcome_super = sram.strike(10.0);
        assert!(outcome_super.upset_occurred);
        assert!(!outcome_super.self_restored);
        assert!(!sram.state_q); // Bit flipped from 1 to 0!
    }

    #[test]
    fn test_dice_single_node_immunity() {
        let mut dice = DiceCell::new(true, 1.0);
        assert!(dice.read_bit());

        // Single-node strike with huge charge (50.0 fC, 25x standard Qcrit)
        let outcome = dice.strike_single_node(0, 50.0);
        assert!(!outcome.upset_occurred);
        assert!(outcome.self_restored);
        assert!(dice.read_bit()); // DICE completely immune to single-node strike!
    }

    #[test]
    fn test_dice_two_node_charge_sharing() {
        let mut dice = DiceCell::new(true, 1.0);
        let q_crit = dice.standard_q_crit_fc();

        // Two adjacent nodes struck simultaneously with supercritical charge
        let outcome = dice.strike_adjacent_nodes(0, q_crit * 2.0, 1, q_crit * 2.0);
        assert!(outcome.upset_occurred);
        assert!(!dice.read_bit()); // Multi-node charge sharing caused upset
    }

    #[test]
    fn test_tmr_majority_voting_and_fault_isolation() {
        let voter = TmrVoter::new(1.2);

        // All channels nominal High (1.2 V)
        let out_all_high = voter.vote(1.2, 1.2, 1.2);
        assert!((out_all_high.voted_output - 1.2).abs() < 1e-9);
        assert!(!out_all_high.fault_detected());
        assert_eq!(out_all_high.disagreeing_channel, None);

        // Single-Event Transient glitch on Channel B: drops to 0.1 V
        let out_glitch_b = voter.vote(1.2, 0.1, 1.2);
        assert!((out_glitch_b.voted_output - 1.2).abs() < 1e-9); // Correctly voted High!
        assert!(out_glitch_b.fault_detected());
        assert_eq!(out_glitch_b.disagreeing_channel, Some(1)); // Channel 1 (B) isolated!

        // Glitch on Channel A when nominal Low (0.0 V)
        let out_glitch_a = voter.vote(1.1, 0.0, 0.0);
        assert!((out_glitch_a.voted_output - 0.0).abs() < 1e-9); // Correctly voted Low!
        assert!(out_glitch_a.fault_detected());
        assert_eq!(out_glitch_a.disagreeing_channel, Some(0)); // Channel 0 (A) isolated!
    }
}
