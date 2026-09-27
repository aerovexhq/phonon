//! Boolean Truth Table Specifications and Logical Correctness Verification.
//!
//! Provides exhaustive truth table representations for combinational primitives
//! (Inverter, NAND2, NOR2, XOR2, XNOR2) and arithmetic functional units (1-bit Full Adder).
//! Computes Hamming distance and normalized logic satisfaction scores.

/// A discrete Boolean vector test vector (inputs and expected outputs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthTableRow {
    pub inputs: Vec<bool>,
    pub outputs: Vec<bool>,
}

/// Truth table specification for a multi-input, multi-output logic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TruthTable {
    pub name: String,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub rows: Vec<TruthTableRow>,
}

impl TruthTable {
    pub fn new(name: impl Into<String>, num_inputs: usize, num_outputs: usize) -> Self {
        Self {
            name: name.into(),
            num_inputs,
            num_outputs,
            rows: Vec::new(),
        }
    }

    /// Appends an input-output evaluation row to the truth table.
    pub fn add_row(&mut self, inputs: Vec<bool>, outputs: Vec<bool>) {
        assert_eq!(inputs.len(), self.num_inputs);
        assert_eq!(outputs.len(), self.num_outputs);
        self.rows.push(TruthTableRow { inputs, outputs });
    }

    /// Evaluates total Hamming bit distance between expected and simulated outputs.
    /// Returns 0 if the circuit perfectly satisfies the truth table.
    pub fn compute_hamming_distance(&self, simulated_outputs: &[Vec<bool>]) -> usize {
        assert_eq!(
            simulated_outputs.len(),
            self.rows.len(),
            "Simulated output row count must match truth table"
        );

        let mut errors = 0;
        for (row_idx, row) in self.rows.iter().enumerate() {
            let sim = &simulated_outputs[row_idx];
            assert_eq!(sim.len(), self.num_outputs);
            for (&expected, &actual) in row.outputs.iter().zip(sim.iter()) {
                if expected != actual {
                    errors += 1;
                }
            }
        }
        errors
    }

    /// Computes normalized logical correctness score in $[0.0, 1.0]$.
    /// $1.0$ indicates 100% logic compliance across all vectors and outputs.
    pub fn normalized_correctness(&self, simulated_outputs: &[Vec<bool>]) -> f64 {
        let total_bits = self.rows.len() * self.num_outputs;
        if total_bits == 0 {
            return 1.0;
        }
        let errors = self.compute_hamming_distance(simulated_outputs);
        (total_bits.saturating_sub(errors)) as f64 / (total_bits as f64)
    }

    // --- Canonical Logic Primitives ---

    /// 1-input Inverter (NOT) gate truth table.
    pub fn inverter() -> Self {
        let mut tt = Self::new("Inverter", 1, 1);
        tt.add_row(vec![false], vec![true]);
        tt.add_row(vec![true], vec![false]);
        tt
    }

    /// 2-input NAND gate truth table.
    pub fn nand2() -> Self {
        let mut tt = Self::new("NAND2", 2, 1);
        tt.add_row(vec![false, false], vec![true]);
        tt.add_row(vec![false, true], vec![true]);
        tt.add_row(vec![true, false], vec![true]);
        tt.add_row(vec![true, true], vec![false]);
        tt
    }

    /// 2-input NOR gate truth table.
    pub fn nor2() -> Self {
        let mut tt = Self::new("NOR2", 2, 1);
        tt.add_row(vec![false, false], vec![true]);
        tt.add_row(vec![false, true], vec![false]);
        tt.add_row(vec![true, false], vec![false]);
        tt.add_row(vec![true, true], vec![false]);
        tt
    }

    /// 2-input XOR gate truth table ($Out = A \oplus B$).
    pub fn xor2() -> Self {
        let mut tt = Self::new("XOR2", 2, 1);
        tt.add_row(vec![false, false], vec![false]);
        tt.add_row(vec![false, true], vec![true]);
        tt.add_row(vec![true, false], vec![true]);
        tt.add_row(vec![true, true], vec![false]);
        tt
    }

    /// 2-input XNOR gate truth table ($Out = \overline{A \oplus B}$).
    pub fn xnor2() -> Self {
        let mut tt = Self::new("XNOR2", 2, 1);
        tt.add_row(vec![false, false], vec![true]);
        tt.add_row(vec![false, true], vec![false]);
        tt.add_row(vec![true, false], vec![false]);
        tt.add_row(vec![true, true], vec![true]);
        tt
    }

    /// 1-bit Full Adder truth table with inputs $[A, B, C_{in}]$ and outputs $[Sum, C_{out}]$.
    ///
    /// $$Sum = A \oplus B \oplus C_{in}$$
    /// $$C_{out} = (A \cdot B) + (C_{in} \cdot (A \oplus B))$$
    pub fn full_adder_1bit() -> Self {
        let mut tt = Self::new("FullAdder1Bit", 3, 2);
        // (A, B, Cin) -> (Sum, Cout)
        tt.add_row(vec![false, false, false], vec![false, false]); // 0+0+0 = 0, C=0
        tt.add_row(vec![false, false, true], vec![true, false]); // 0+0+1 = 1, C=0
        tt.add_row(vec![false, true, false], vec![true, false]); // 0+1+0 = 1, C=0
        tt.add_row(vec![false, true, true], vec![false, true]); // 0+1+1 = 0, C=1
        tt.add_row(vec![true, false, false], vec![true, false]); // 1+0+0 = 1, C=0
        tt.add_row(vec![true, false, true], vec![false, true]); // 1+0+1 = 0, C=1
        tt.add_row(vec![true, true, false], vec![false, true]); // 1+1+0 = 0, C=1
        tt.add_row(vec![true, true, true], vec![true, true]); // 1+1+1 = 1, C=1
        tt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truth_table_inverter_and_nand2() {
        let inv = TruthTable::inverter();
        assert_eq!(inv.rows.len(), 2);
        assert_eq!(inv.compute_hamming_distance(&[vec![true], vec![false]]), 0);
        assert_eq!(inv.normalized_correctness(&[vec![true], vec![false]]), 1.0);

        let nand = TruthTable::nand2();
        assert_eq!(nand.rows.len(), 4);
        assert_eq!(
            nand.compute_hamming_distance(&[vec![true], vec![true], vec![true], vec![false]]),
            0
        );
    }

    #[test]
    fn test_full_adder_truth_table() {
        let fa = TruthTable::full_adder_1bit();
        assert_eq!(fa.rows.len(), 8);

        // Correct outputs
        let correct = vec![
            vec![false, false],
            vec![true, false],
            vec![true, false],
            vec![false, true],
            vec![true, false],
            vec![false, true],
            vec![false, true],
            vec![true, true],
        ];
        assert_eq!(fa.compute_hamming_distance(&correct), 0);
        assert_eq!(fa.normalized_correctness(&correct), 1.0);

        // One bit error
        let mut faulty = correct.clone();
        faulty[0] = vec![true, false]; // Error on first row sum
        assert_eq!(fa.compute_hamming_distance(&faulty), 1);
        assert_eq!(fa.normalized_correctness(&faulty), 15.0 / 16.0);
    }
}
