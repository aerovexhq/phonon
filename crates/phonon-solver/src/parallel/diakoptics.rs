//! Multi-threaded Kron's Diakoptics / Node Tearing Block Bordered Diagonal Form (BBDF) solver.
//!
//! Solves partitioned circuit networks concurrently across thread pools via Schur complement condensation.

use super::node_tearing::PartitionedCircuit;
use crate::error::SolverError;
use crate::mna::assembler::{assemble_mna_dc, SolverOptions};
use crate::mna::linear_solver::DcSolution;
use crate::sparse::{MarkowitzOptions, SparseLuFactorization, SparseMatrixBuilder};
use rayon::prelude::*;
use std::collections::HashMap;

/// Local subcircuit factorization and sensitivity solution output.
pub struct SubcircuitLocalSolution {
    pub subcircuit_id: usize,
    /// Unconstrained local solution: y_k = A_k^{-1} * b_k
    pub y_k: Vec<f64>,
    /// Sensitivity matrix: W_k = A_k^{-1} * A_k0 (dim: n_k x n_0)
    pub w_k: Vec<Vec<f64>>,
    /// Return matrix: A_0k (dim: n_0 x n_k)
    pub a_0k: Vec<Vec<f64>>,
}

/// Diakoptics parallel linear solver.
pub struct DiakopticsSolver;

impl DiakopticsSolver {
    /// Solves the linear system defined by a partitioned circuit using parallel multi-threaded Diakoptics.
    ///
    /// # Arguments
    /// * `partitioned` - The partitioned circuit with graph, subcircuits, and torn indices
    /// * `options` - Solver operational options (gmin, etc.)
    ///
    /// # Returns
    /// Exact `DcSolution` matching monolithic MNA to machine precision.
    pub fn solve_linear(
        partitioned: &PartitionedCircuit,
        options: &SolverOptions,
    ) -> Result<DcSolution, SolverError> {
        let mna = assemble_mna_dc(&partitioned.graph, options)?;
        let total_dim = mna.total_dim;
        let active_nodes = mna.active_nodes;
        let total_branches = mna.total_branches;

        if total_dim == 0 {
            return Ok(DcSolution {
                node_voltages: vec![0.0],
                branch_currents: Vec::new(),
                condition_ratio: 1.0,
            });
        }

        let a_global = &mna.g_matrix;
        let b_global = &mna.rhs;

        let torn_indices = &partitioned.torn_indices;
        let n0 = torn_indices.len();

        let mut torn_map = HashMap::new();
        for (local_idx, &global_idx) in torn_indices.iter().enumerate() {
            torn_map.insert(global_idx, local_idx);
        }

        // Build mapping from global MNA index to (subcircuit_id, local_index)
        let num_subcircuits = partitioned.subcircuits.len();
        let mut sub_maps: Vec<HashMap<usize, usize>> = Vec::with_capacity(num_subcircuits);
        for sub_vars in &partitioned.subcircuits {
            let mut map = HashMap::new();
            for (local_idx, &global_idx) in sub_vars.iter().enumerate() {
                map.insert(global_idx, local_idx);
            }
            sub_maps.push(map);
        }

        // Extract A_00 (n0 x n0) and b_0 (n0)
        let mut a00 = vec![vec![0.0; n0]; n0];
        let mut b0 = vec![0.0; n0];

        for (col_0, &g_col) in torn_indices.iter().enumerate() {
            b0[col_0] = b_global[g_col];
            let start = a_global.col_ptrs()[g_col];
            let end = a_global.col_ptrs()[g_col + 1];
            for k in start..end {
                let g_row = a_global.row_indices()[k];
                let val = a_global.values()[k];
                if let Some(&row_0) = torn_map.get(&g_row) {
                    a00[row_0][col_0] += val;
                }
            }
        }

        let markowitz_opts = MarkowitzOptions::default();

        // 1. Parallel Local Subcircuit Solves (Across all CPU cores with Rayon)
        let local_results: Result<Vec<SubcircuitLocalSolution>, SolverError> = (0..num_subcircuits)
            .into_par_iter()
            .map(|sub_idx| {
                let sub_vars = &partitioned.subcircuits[sub_idx];
                let sub_map = &sub_maps[sub_idx];
                let n_k = sub_vars.len();

                if n_k == 0 {
                    return Ok(SubcircuitLocalSolution {
                        subcircuit_id: sub_idx,
                        y_k: Vec::new(),
                        w_k: Vec::new(),
                        a_0k: vec![Vec::new(); n0],
                    });
                }

                // Extract A_kk (n_k x n_k), b_k (n_k), A_k0 (n_k x n0), A_0k (n0 x n_k)
                let mut a_kk_builder = SparseMatrixBuilder::with_capacity(n_k, n_k, n_k * 4);
                let mut b_k = vec![0.0; n_k];
                let mut a_0k = vec![vec![0.0; n_k]; n0];

                for (local_col, &g_col) in sub_vars.iter().enumerate() {
                    b_k[local_col] = b_global[g_col];
                    let start = a_global.col_ptrs()[g_col];
                    let end = a_global.col_ptrs()[g_col + 1];
                    for k in start..end {
                        let g_row = a_global.row_indices()[k];
                        let val = a_global.values()[k];

                        if let Some(&local_row) = sub_map.get(&g_row) {
                            a_kk_builder.add(local_row, local_col, val);
                        } else if let Some(&torn_row) = torn_map.get(&g_row) {
                            a_0k[torn_row][local_col] += val;
                        }
                    }
                }

                // Extract A_k0 (n_k x n0)
                let mut a_k0 = vec![vec![0.0; n0]; n_k];
                for (col_0, &g_col) in torn_indices.iter().enumerate() {
                    let start = a_global.col_ptrs()[g_col];
                    let end = a_global.col_ptrs()[g_col + 1];
                    for k in start..end {
                        let g_row = a_global.row_indices()[k];
                        let val = a_global.values()[k];
                        if let Some(&local_row) = sub_map.get(&g_row) {
                            a_k0[local_row][col_0] += val;
                        }
                    }
                }

                // Factor local matrix A_kk
                let csc_kk = a_kk_builder.build_csc();
                let lu_k = match SparseLuFactorization::factor(&csc_kk, &markowitz_opts) {
                    Ok(lu) => lu,
                    Err(e) => return Err(e),
                };

                // Solve y_k = A_kk^{-1} * b_k
                let mut y_k = vec![0.0; n_k];
                lu_k.solve(&b_k, &mut y_k)?;

                // Solve W_k = A_kk^{-1} * A_k0 column by column
                let mut w_k = vec![vec![0.0; n0]; n_k];
                for col in 0..n0 {
                    let mut rhs_col = vec![0.0; n_k];
                    for row in 0..n_k {
                        rhs_col[row] = a_k0[row][col];
                    }
                    let mut w_col = vec![0.0; n_k];
                    lu_k.solve(&rhs_col, &mut w_col)?;
                    for row in 0..n_k {
                        w_k[row][col] = w_col[row];
                    }
                }

                Ok(SubcircuitLocalSolution {
                    subcircuit_id: sub_idx,
                    y_k,
                    w_k,
                    a_0k,
                })
            })
            .collect();

        let local_solutions = local_results?;

        // 2. Form Condensed Schur Complement: S_0 = A_00 - sum(A_0k * W_k)
        // and Condensed RHS: r_0 = b_0 - sum(A_0k * y_k)
        let mut s0 = a00;
        let mut r0 = b0;

        for sol in &local_solutions {
            let n_k = sol.y_k.len();
            if n_k == 0 {
                continue;
            }

            // Subtract A_0k * y_k from r_0
            for (i, r_val) in r0.iter_mut().enumerate().take(n0) {
                let mut sum_ay = 0.0;
                for k in 0..n_k {
                    sum_ay += sol.a_0k[i][k] * sol.y_k[k];
                }
                *r_val -= sum_ay;
            }

            // Subtract A_0k * W_k from s0
            for (i, s0_row) in s0.iter_mut().enumerate().take(n0) {
                for (j, s0_elem) in s0_row.iter_mut().enumerate().take(n0) {
                    let mut sum_aw = 0.0;
                    for k in 0..n_k {
                        sum_aw += sol.a_0k[i][k] * sol.w_k[k][j];
                    }
                    *s0_elem -= sum_aw;
                }
            }
        }

        // 3. Solve Condensed Boundary System: S_0 * x_0 = r_0
        let x0 = if n0 > 0 {
            solve_dense_system(&s0, &r0)?
        } else {
            Vec::new()
        };

        // 4. Parallel Back-Substitution: x_k = y_k - W_k * x_0
        let back_sub_results: Vec<Vec<f64>> = local_solutions
            .par_iter()
            .map(|sol| {
                let mut x_k = sol.y_k.clone();
                for (i, x_val) in x_k.iter_mut().enumerate() {
                    for (j, &x0_val) in x0.iter().enumerate() {
                        *x_val -= sol.w_k[i][j] * x0_val;
                    }
                }
                x_k
            })
            .collect();

        // 5. Assemble global solution vector x of dimension total_dim
        let mut x_global = vec![0.0; total_dim];

        // Insert boundary torn variables
        for (col_0, &g_col) in torn_indices.iter().enumerate() {
            x_global[g_col] = x0[col_0];
        }

        // Insert local subcircuit variables
        for (sub_idx, sub_vars) in partitioned.subcircuits.iter().enumerate() {
            let x_k = &back_sub_results[sub_idx];
            for (local_idx, &g_idx) in sub_vars.iter().enumerate() {
                x_global[g_idx] = x_k[local_idx];
            }
        }

        // Pack into standard DcSolution
        let mut node_voltages = vec![0.0; active_nodes + 1];
        node_voltages[1..(active_nodes + 1)].copy_from_slice(&x_global[..active_nodes]);

        let mut branch_currents = vec![0.0; total_branches];
        branch_currents[..total_branches]
            .copy_from_slice(&x_global[active_nodes..(active_nodes + total_branches)]);

        Ok(DcSolution {
            node_voltages,
            branch_currents,
            condition_ratio: 1.0,
        })
    }
}

/// Solves a small dense linear system A * x = b via Gaussian elimination with partial pivoting.
#[allow(clippy::needless_range_loop)]
fn solve_dense_system(a: &[Vec<f64>], b: &[f64]) -> Result<Vec<f64>, SolverError> {
    let n = b.len();
    if n == 0 {
        return Ok(Vec::new());
    }

    let mut aug = vec![vec![0.0; n + 1]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = a[i][j];
        }
        aug[i][n] = b[i];
    }

    for col in 0..n {
        let mut pivot_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            if aug[row][col].abs() > max_val {
                max_val = aug[row][col].abs();
                pivot_row = row;
            }
        }

        if max_val < 1e-15 {
            return Err(SolverError::SingularMatrix {
                step: 0,
                row: col,
                col,
                pivot_value: max_val,
                entity_diagnostic: format!("Torn boundary system pivot too small: {:.3e}", max_val),
            });
        }

        if pivot_row != col {
            aug.swap(col, pivot_row);
        }

        let pivot = aug[col][col];
        for j in col..=n {
            aug[col][j] /= pivot;
        }

        for row in 0..n {
            if row != col {
                let factor = aug[row][col];
                if factor.abs() > 1e-20 {
                    for j in col..=n {
                        aug[row][j] -= factor * aug[col][j];
                    }
                }
            }
        }
    }

    let mut x = vec![0.0; n];
    for i in 0..n {
        x[i] = aug[i][n];
    }

    Ok(x)
}
