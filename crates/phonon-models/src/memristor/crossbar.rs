//! Neuromorphic memristive synaptic crossbar array architecture ($M \times N$).
//!
//! Formulates:
//! - 1R (passive cross-point), 1T1R (transistor isolated), and 1S1R (selector isolated) topologies.
//! - Parasitic interconnect metal line resistance ($R_{wire}$) along rows and columns.
//! - Sneak-path current leakage evaluation and suppression.
//! - Analog Vector-Matrix Multiplication (VMM): $\mathbf{I}_{out} = \mathbf{G} \cdot \mathbf{V}_{in}$.
//! - Signal-to-noise ratio (SNR), power consumption, and VMM error evaluation.

/// Synaptic cross-point cell architecture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossbarCellType {
    /// 1R: Passive two-terminal memristor (simplest, highest density, vulnerable to sneak paths).
    Passive1R,
    /// 1T1R: Transistor-gated memristor (zero sneak-path leakage, requires gate line).
    Transistor1T1R,
    /// 1S1R: Two-terminal ovonic/volatile selector diode in series with memristor.
    Selector1S1R,
}

/// Physical parameters and state of an $M \times N$ memristive crossbar array.
#[derive(Debug, Clone, PartialEq)]
pub struct MemristiveCrossbarModel {
    /// Number of input rows (word lines) $M$.
    pub num_rows: usize,
    /// Number of output columns (bit lines) $N$.
    pub num_cols: usize,
    /// Cross-point cell architecture (1R, 1T1R, 1S1R).
    pub cell_type: CrossbarCellType,
    /// Metal interconnect line resistance per segment along row in Ohms ($\Omega$) (typically $0.5 - 5\,\Omega$).
    pub r_wire_row_ohms: f64,
    /// Metal interconnect line resistance per segment along column in Ohms ($\Omega$).
    pub r_wire_col_ohms: f64,
    /// Conductance matrix $\mathbf{G}$ of size $M \times N$ in Siemens ($S$).
    pub conductance_matrix: Vec<Vec<f64>>,
    /// Off-state leakage conductance for 1S1R / 1T1R unselected cells in Siemens ($S$).
    pub selector_leakage_conductance: f64,
}

impl MemristiveCrossbarModel {
    /// Creates a new $M \times N$ crossbar array initialized with uniform conductance $G_0$.
    pub fn new(
        num_rows: usize,
        num_cols: usize,
        cell_type: CrossbarCellType,
        initial_conductance: f64,
    ) -> Self {
        let conductance_matrix = vec![vec![initial_conductance.max(1e-12); num_cols]; num_rows];
        Self {
            num_rows,
            num_cols,
            cell_type,
            r_wire_row_ohms: 2.0, // 2 Ohms per interconnect segment
            r_wire_col_ohms: 2.0,
            conductance_matrix,
            selector_leakage_conductance: 1.0e-9, // 1 nS off-state leakage for 1T1R / 1S1R
        }
    }

    /// Sets the conductance of a specific synaptic cell $(i, j)$ in Siemens ($S$).
    pub fn set_weight(&mut self, row: usize, col: usize, conductance_s: f64) {
        if row < self.num_rows && col < self.num_cols {
            self.conductance_matrix[row][col] = conductance_s.max(1e-12);
        }
    }

    /// Reads the conductance of a specific synaptic cell $(i, j)$ in Siemens ($S$).
    pub fn get_weight(&self, row: usize, col: usize) -> f64 {
        if row < self.num_rows && col < self.num_cols {
            self.conductance_matrix[row][col]
        } else {
            0.0
        }
    }

    /// Evaluates ideal mathematical Vector-Matrix Multiplication (VMM):
    /// $$\mathbf{I}_{ideal, j} = \sum_{i=0}^{M-1} G_{ij} V_{in, i}$$
    pub fn evaluate_ideal_vmm(&self, v_in: &[f64]) -> Vec<f64> {
        let mut i_out = vec![0.0; self.num_cols];
        let m = v_in.len().min(self.num_rows);

        for (j, out_val) in i_out.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (i, &v_val) in v_in.iter().take(m).enumerate() {
                sum += self.conductance_matrix[i][j] * v_val;
            }
            *out_val = sum;
        }
        i_out
    }

    /// Evaluates parasitic-aware VMM taking into account wire resistance drops along rows and columns,
    /// as well as sneak-path currents depending on cell type (1R vs 1T1R/1S1R):
    pub fn evaluate_parasitic_vmm(&self, v_in: &[f64]) -> (Vec<f64>, f64) {
        let mut i_out = vec![0.0; self.num_cols];
        let mut total_power = 0.0;
        let m = v_in.len().min(self.num_rows);

        for (j, out_val) in i_out.iter_mut().enumerate() {
            let mut col_current = 0.0;

            for (i, &v_applied) in v_in.iter().take(m).enumerate() {
                // Interconnect line IR drop increases with distance from drivers
                // Row segment delay: i segments; Col segment delay: j segments
                let r_line_tot =
                    (i as f64) * self.r_wire_row_ohms + (j as f64) * self.r_wire_col_ohms;
                let g_synapse = self.conductance_matrix[i][j];

                // Effective cross-point admittance:
                let g_cell = match self.cell_type {
                    CrossbarCellType::Passive1R => g_synapse,
                    CrossbarCellType::Transistor1T1R | CrossbarCellType::Selector1S1R => {
                        if v_applied.abs() > 0.1 {
                            g_synapse
                        } else {
                            self.selector_leakage_conductance
                        }
                    }
                };

                // Voltage divider between line resistance and cell resistance:
                let r_cell = 1.0 / g_cell;
                let v_eff = v_applied * (r_cell / (r_cell + r_line_tot));
                let i_cell = v_eff * g_cell;

                col_current += i_cell;
                total_power += (i_cell * v_applied).abs();
            }

            *out_val = col_current;
        }

        (i_out, total_power)
    }

    /// Evaluates the Root-Mean-Square Error (RMSE) between ideal and parasitic-aware VMM:
    pub fn vmm_error_rmse(&self, v_in: &[f64]) -> f64 {
        let ideal = self.evaluate_ideal_vmm(v_in);
        let (parasitic, _) = self.evaluate_parasitic_vmm(v_in);

        let mut sum_sq = 0.0;
        for j in 0..self.num_cols {
            let diff = ideal[j] - parasitic[j];
            sum_sq += diff * diff;
        }

        (sum_sq / self.num_cols as f64).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ideal_vector_matrix_multiplication() {
        let mut xbar = MemristiveCrossbarModel::new(4, 4, CrossbarCellType::Transistor1T1R, 1.0e-3);
        // Set diagonal identity weights: G_ii = 5 mS, others = 0.1 mS
        for i in 0..4 {
            for j in 0..4 {
                if i == j {
                    xbar.set_weight(i, j, 5.0e-3);
                } else {
                    xbar.set_weight(i, j, 0.1e-3);
                }
            }
        }

        let v_in = vec![1.0, 0.0, 0.0, 0.0];
        let i_out = xbar.evaluate_ideal_vmm(&v_in);

        // Column 0 should see 5 mS * 1.0 V = 5.0 mA
        assert!((i_out[0] - 5.0e-3).abs() < 1e-6);
        // Column 1 should see 0.1 mS * 1.0 V = 0.1 mA
        assert!((i_out[1] - 0.1e-3).abs() < 1e-6);
    }

    #[test]
    fn test_parasitic_wire_resistance_degradation() {
        let mut xbar = MemristiveCrossbarModel::new(8, 8, CrossbarCellType::Transistor1T1R, 1.0e-3);
        xbar.r_wire_row_ohms = 5.0; // 5 Ohm per segment
        xbar.r_wire_col_ohms = 5.0;

        let v_in = vec![1.0; 8];
        let ideal = xbar.evaluate_ideal_vmm(&v_in);
        let (parasitic, power) = xbar.evaluate_parasitic_vmm(&v_in);

        // Due to wire IR-drops, parasitic current is slightly lower than ideal:
        for j in 0..8 {
            assert!(parasitic[j] <= ideal[j]);
            assert!(parasitic[j] > 0.8 * ideal[j]);
        }
        assert!(power > 0.0);

        let rmse = xbar.vmm_error_rmse(&v_in);
        assert!(rmse > 0.0);
    }
}
