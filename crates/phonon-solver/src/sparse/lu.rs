//! Sparse LU decomposition with Markowitz threshold pivoting and forward/backward solve.

use super::csc::SparseMatrixCsc;
use super::markowitz::{find_markowitz_pivot, MarkowitzOptions};
use crate::error::SolverError;
use std::collections::HashMap;

/// Sparse LU Factorization result: $\mathbf{P} \mathbf{A} \mathbf{Q}^T = \mathbf{L} \mathbf{U}$.
#[derive(Debug, Clone)]
pub struct SparseLuFactorization {
    dim: usize,
    /// Row permutation: row in original matrix corresponding to step $k$.
    row_perm: Vec<usize>,
    /// Column permutation: col in original matrix corresponding to step $k$.
    col_perm: Vec<usize>,
    /// Inverse row permutation mapping original row to step $k$.
    inv_row_perm: Vec<usize>,
    /// Inverse column permutation mapping original col to step $k$.
    inv_col_perm: Vec<usize>,
    /// Lower triangular entries (unit diagonal implicit).
    /// Maps permuted_row -> (permuted_col -> L_val) for row > col.
    l_entries: Vec<HashMap<usize, f64>>,
    /// Upper triangular entries (including diagonal).
    /// Maps permuted_row -> (permuted_col -> U_val) for row <= col.
    u_entries: Vec<HashMap<usize, f64>>,
    /// Diagonal elements of U: U[k, k].
    u_diag: Vec<f64>,
}

impl SparseLuFactorization {
    /// Factors the given square CSC matrix $\mathbf{A} \in \mathbb{R}^{n \times n}$.
    pub fn factor(
        matrix: &SparseMatrixCsc,
        options: &MarkowitzOptions,
    ) -> Result<Self, SolverError> {
        let n = matrix.nrows();
        if n != matrix.ncols() {
            return Err(SolverError::DimensionMismatch {
                expected: n,
                found_rows: n,
                found_cols: matrix.ncols(),
            });
        }

        if n == 0 {
            return Ok(Self {
                dim: 0,
                row_perm: Vec::new(),
                col_perm: Vec::new(),
                inv_row_perm: Vec::new(),
                inv_col_perm: Vec::new(),
                l_entries: Vec::new(),
                u_entries: Vec::new(),
                u_diag: Vec::new(),
            });
        }

        // Maintain working entries in sparse row-indexed maps
        let mut a_rows: Vec<HashMap<usize, f64>> = vec![HashMap::new(); n];
        for col in 0..n {
            let start = matrix.col_ptrs()[col];
            let end = matrix.col_ptrs()[col + 1];
            for idx in start..end {
                let row = matrix.row_indices()[idx];
                let val = matrix.values()[idx];
                if val.abs() > options.zero_tolerance {
                    a_rows[row].insert(col, val);
                }
            }
        }

        let mut active_rows: Vec<usize> = (0..n).collect();
        let mut active_cols: Vec<usize> = (0..n).collect();

        let mut row_perm = Vec::with_capacity(n);
        let mut col_perm = Vec::with_capacity(n);

        // Store multipliers per original row: l_multipliers[orig_row] = [(orig_col, m)]
        let mut l_multipliers: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        // Store final pivot row entries: u_orig_rows[orig_row] = [(orig_col, val)]
        let mut u_orig_rows: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        let mut u_diag_vals = Vec::with_capacity(n);

        for step in 0..n {
            let get_val = |r: usize, c: usize| -> f64 { a_rows[r].get(&c).copied().unwrap_or(0.0) };

            let pivot = match find_markowitz_pivot(&active_rows, &active_cols, get_val, options) {
                Some(p) => p,
                None => {
                    let offending_row = active_rows.first().copied().unwrap_or(step);
                    let offending_col = active_cols.first().copied().unwrap_or(step);
                    return Err(SolverError::SingularMatrix {
                        step,
                        row: offending_row,
                        col: offending_col,
                        pivot_value: 0.0,
                        entity_diagnostic: format!("Zero pivot at submatrix step {}", step),
                    });
                }
            };

            if pivot.value.abs() <= options.zero_tolerance {
                return Err(SolverError::SingularMatrix {
                    step,
                    row: pivot.row,
                    col: pivot.col,
                    pivot_value: pivot.value,
                    entity_diagnostic: format!(
                        "Near-zero pivot ({:e}) at step {}",
                        pivot.value, step
                    ),
                });
            }

            let p_row = pivot.row;
            let p_col = pivot.col;
            let pivot_val = pivot.value;

            row_perm.push(p_row);
            col_perm.push(p_col);
            u_diag_vals.push(pivot_val);

            // Remove pivot row & col from active sets
            active_rows.retain(|&r| r != p_row);
            active_cols.retain(|&c| c != p_col);

            // Freeze the pivot row as row `step` in U
            let pivot_row_entries: Vec<(usize, f64)> = a_rows[p_row]
                .iter()
                .filter(|(_, &v)| v.abs() > options.zero_tolerance)
                .map(|(&c, &v)| (c, v))
                .collect();
            u_orig_rows[p_row] = pivot_row_entries.clone();

            // Gaussian elimination across remaining active rows
            for &r in &active_rows {
                if let Some(&val_rc) = a_rows[r].get(&p_col) {
                    if val_rc.abs() > options.zero_tolerance {
                        let multiplier = val_rc / pivot_val;
                        l_multipliers[r].push((p_col, multiplier));

                        // Row[r] = Row[r] - multiplier * Row[p_row]
                        for &(c, p_val) in &pivot_row_entries {
                            if c == p_col {
                                a_rows[r].remove(&c);
                            } else {
                                let new_val =
                                    a_rows[r].get(&c).copied().unwrap_or(0.0) - multiplier * p_val;
                                if new_val.abs() > options.zero_tolerance {
                                    a_rows[r].insert(c, new_val);
                                } else {
                                    a_rows[r].remove(&c);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Invert permutation vectors for O(1) coordinate mapping
        let mut inv_row_perm = vec![0; n];
        let mut inv_col_perm = vec![0; n];
        for (step, &r) in row_perm.iter().enumerate() {
            inv_row_perm[r] = step;
        }
        for (step, &c) in col_perm.iter().enumerate() {
            inv_col_perm[c] = step;
        }

        // Build step-indexed L and U entries
        let mut l_entries: Vec<HashMap<usize, f64>> = vec![HashMap::new(); n];
        let mut u_entries: Vec<HashMap<usize, f64>> = vec![HashMap::new(); n];

        for r in 0..n {
            let step_row = inv_row_perm[r];

            // Map L entries
            for &(c, m) in &l_multipliers[r] {
                let step_col = inv_col_perm[c];
                l_entries[step_row].insert(step_col, m);
            }

            // Map U entries
            for &(c, val) in &u_orig_rows[r] {
                let step_col = inv_col_perm[c];
                u_entries[step_row].insert(step_col, val);
            }
        }

        Ok(Self {
            dim: n,
            row_perm,
            col_perm,
            inv_row_perm,
            inv_col_perm,
            l_entries,
            u_entries,
            u_diag: u_diag_vals,
        })
    }

    /// Solves the linear system $\mathbf{A} \mathbf{x} = \mathbf{b}$ using the factored $\mathbf{L}$ and $\mathbf{U}$.
    pub fn solve(&self, b: &[f64], x: &mut [f64]) -> Result<(), SolverError> {
        if b.len() != self.dim {
            return Err(SolverError::VectorDimensionMismatch {
                expected: self.dim,
                found: b.len(),
            });
        }
        if x.len() != self.dim {
            return Err(SolverError::VectorDimensionMismatch {
                expected: self.dim,
                found: x.len(),
            });
        }

        if self.dim == 0 {
            return Ok(());
        }

        let n = self.dim;

        // Step 1: Permute RHS according to row permutation P: y[k] = b[row_perm[k]]
        let mut y = vec![0.0; n];
        for k in 0..n {
            y[k] = b[self.row_perm[k]];
        }

        // Step 2: Forward solve L * y_sol = y (L has implicit unit diagonal)
        for i in 0..n {
            let mut sum = 0.0;
            for (&j, &l_val) in &self.l_entries[i] {
                if j < i {
                    sum += l_val * y[j];
                }
            }
            y[i] -= sum;
        }

        // Step 3: Backward solve U * z = y
        let mut z = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = 0.0;
            for (&j, &u_val) in &self.u_entries[i] {
                if j > i {
                    sum += u_val * z[j];
                }
            }
            let diag = self.u_diag[i];
            if diag.abs() < 1e-25 {
                return Err(SolverError::NumericalAnomaly {
                    detail: format!("Zero diagonal element in U at step {}", i),
                });
            }
            z[i] = (y[i] - sum) / diag;
        }

        // Step 4: Un-permute solution vector: x[col_perm[k]] = z[k]
        for (k, &zk) in z.iter().enumerate().take(n) {
            let orig_col = self.col_perm[k];
            x[orig_col] = zk;
        }

        Ok(())
    }

    /// Returns the condition ratio: $\max_k |U_{kk}| / \min_k |U_{kk}|$.
    pub fn pivot_condition_ratio(&self) -> f64 {
        if self.u_diag.is_empty() {
            return 1.0;
        }
        let mut max_abs = 0.0f64;
        let mut min_abs = f64::INFINITY;
        for &d in &self.u_diag {
            let abs_d = d.abs();
            if abs_d > max_abs {
                max_abs = abs_d;
            }
            if abs_d < min_abs {
                min_abs = abs_d;
            }
        }
        if min_abs <= 1e-25 {
            f64::INFINITY
        } else {
            max_abs / min_abs
        }
    }

    /// Returns the matrix dimension.
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Returns the row permutation vector.
    pub fn row_perm(&self) -> &[usize] {
        &self.row_perm
    }

    /// Returns the column permutation vector.
    pub fn col_perm(&self) -> &[usize] {
        &self.col_perm
    }

    /// Returns the inverse row permutation mapping original row to step.
    pub fn inv_row_perm(&self) -> &[usize] {
        &self.inv_row_perm
    }

    /// Returns the inverse column permutation mapping original col to step.
    pub fn inv_col_perm(&self) -> &[usize] {
        &self.inv_col_perm
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse::builder::SparseMatrixBuilder;

    #[test]
    fn test_lu_solve_2x2() {
        // [ 2  1 ] [ x0 ] = [ 5 ]
        // [ 1  3 ] [ x1 ] = [ 5 ]
        // Solution: x0 = 2, x1 = 1
        let mut b = SparseMatrixBuilder::new(2, 2);
        b.add(0, 0, 2.0);
        b.add(0, 1, 1.0);
        b.add(1, 0, 1.0);
        b.add(1, 1, 3.0);

        let mat = b.build_csc();
        let lu = SparseLuFactorization::factor(&mat, &MarkowitzOptions::default()).unwrap();

        let rhs = vec![5.0, 5.0];
        let mut sol = vec![0.0; 2];
        lu.solve(&rhs, &mut sol).unwrap();

        assert!((sol[0] - 2.0).abs() < 1e-12);
        assert!((sol[1] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_lu_solve_3x3() {
        // [ 3  2 -1 ] [ x0 ]   [ 1 ]
        // [ 2 -2  4 ] [ x1 ] = [-2 ]
        // [-1 0.5 -1] [ x2 ]   [ 0 ]
        // Solution: x0 = 1, x1 = -2, x2 = -2
        let mut b = SparseMatrixBuilder::new(3, 3);
        b.add(0, 0, 3.0);
        b.add(0, 1, 2.0);
        b.add(0, 2, -1.0);

        b.add(1, 0, 2.0);
        b.add(1, 1, -2.0);
        b.add(1, 2, 4.0);

        b.add(2, 0, -1.0);
        b.add(2, 1, 0.5);
        b.add(2, 2, -1.0);

        let mat = b.build_csc();
        let lu = SparseLuFactorization::factor(&mat, &MarkowitzOptions::default()).unwrap();

        let rhs = vec![1.0, -2.0, 0.0];
        let mut sol = vec![0.0; 3];
        lu.solve(&rhs, &mut sol).unwrap();

        assert!((sol[0] - 1.0).abs() < 1e-10);
        assert!((sol[1] - (-2.0)).abs() < 1e-10);
        assert!((sol[2] - (-2.0)).abs() < 1e-10);
    }
}
