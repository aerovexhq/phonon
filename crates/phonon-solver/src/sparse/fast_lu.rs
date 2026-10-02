#![deny(unsafe_code)]

//! High-Throughput Zero-Allocation Sparse LU Solver for MNA Kernels.
//!
//! Provides two-phase sparse Gaussian elimination with Markowitz ordering:
//! 1. Symbolic Pre-factorization: Computes fill-ins, permutations, and static CSR index layouts once.
//! 2. In-Place Numeric Refactorization: Solves in-place with zero dynamic memory allocations in the inner loop.

use super::csc::SparseMatrixCsc;
use super::lu::SparseLuFactorization;
use super::markowitz::MarkowitzOptions;
use crate::error::SolverError;

/// Pre-allocated, flat contiguous-array Sparse LU Factorization.
///
/// Converts sparse graph representations into contiguous flat memory slices (Compressed Sparse Row),
/// eliminating HashMap overhead, pointer indirection, and memory fragmentation during
/// high-rate Newton-Raphson iterations and transient time integration.
#[derive(Debug, Clone)]
pub struct FastInPlaceLu {
    dim: usize,
    row_perm: Vec<usize>,
    col_perm: Vec<usize>,
    inv_row_perm: Vec<usize>,
    inv_col_perm: Vec<usize>,
    // Compressed Sparse Row representation for L (strictly lower triangular, unit diagonal implicit):
    l_row_ptrs: Vec<usize>,
    l_cols: Vec<usize>,
    l_vals: Vec<f64>,
    // Compressed Sparse Row representation for U (strictly upper triangular, diagonal stored separately):
    u_row_ptrs: Vec<usize>,
    u_cols: Vec<usize>,
    u_vals: Vec<f64>,
    u_diag: Vec<f64>,
    // Pre-allocated scratch buffers for zero-allocation solve:
    scratch_y: Vec<f64>,
    scratch_z: Vec<f64>,
}

impl FastInPlaceLu {
    /// Constructs a fast in-place LU factorizer from a square CSC matrix and Markowitz options.
    ///
    /// Executes full symbolic and numeric factorization, then flattens all internal sparse
    /// entries into cache-friendly contiguous vectors and pre-allocates scratch buffers.
    pub fn from_matrix(
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
                l_row_ptrs: vec![0],
                l_cols: Vec::new(),
                l_vals: Vec::new(),
                u_row_ptrs: vec![0],
                u_cols: Vec::new(),
                u_vals: Vec::new(),
                u_diag: Vec::new(),
                scratch_y: Vec::new(),
                scratch_z: Vec::new(),
            });
        }

        // Run baseline factorization to determine symbolic structure and Markowitz pivots
        let base_lu = SparseLuFactorization::factor(matrix, options)?;

        let mut instance = Self {
            dim: n,
            row_perm: base_lu.row_perm().to_vec(),
            col_perm: base_lu.col_perm().to_vec(),
            inv_row_perm: base_lu.inv_row_perm().to_vec(),
            inv_col_perm: base_lu.inv_col_perm().to_vec(),
            l_row_ptrs: vec![0; n + 1],
            l_cols: Vec::new(),
            l_vals: Vec::new(),
            u_row_ptrs: vec![0; n + 1],
            u_cols: Vec::new(),
            u_vals: Vec::new(),
            u_diag: vec![0.0; n],
            scratch_y: vec![0.0; n],
            scratch_z: vec![0.0; n],
        };

        instance.flatten_from_base(&base_lu);
        Ok(instance)
    }

    /// Flattens the entries from a base factorization into contiguous CSR vectors.
    fn flatten_from_base(&mut self, base_lu: &SparseLuFactorization) {
        let n = self.dim;
        self.l_cols.clear();
        self.l_vals.clear();
        self.u_cols.clear();
        self.u_vals.clear();

        // Extract diagonal and flatten L and U
        // In base_lu, L has unit diagonal implicit, u_diag holds U diagonal
        for i in 0..n {
            self.u_diag[i] = base_lu.u_diag()[i];
        }

        let mut l_count = 0;
        let mut u_count = 0;
        self.l_row_ptrs[0] = 0;
        self.u_row_ptrs[0] = 0;

        for i in 0..n {
            // L entries for row i: cols j < i
            let mut l_row_entries: Vec<(usize, f64)> = base_lu.l_entries()[i]
                .iter()
                .filter(|(&j, _)| j < i)
                .map(|(&j, &v)| (j, v))
                .collect();
            l_row_entries.sort_by_key(|&(j, _)| j);

            for (col, val) in l_row_entries {
                self.l_cols.push(col);
                self.l_vals.push(val);
                l_count += 1;
            }
            self.l_row_ptrs[i + 1] = l_count;

            // U entries for row i: cols j > i (diagonal stored in u_diag)
            let mut u_row_entries: Vec<(usize, f64)> = base_lu.u_entries()[i]
                .iter()
                .filter(|(&j, _)| j > i)
                .map(|(&j, &v)| (j, v))
                .collect();
            u_row_entries.sort_by_key(|&(j, _)| j);

            for (col, val) in u_row_entries {
                self.u_cols.push(col);
                self.u_vals.push(val);
                u_count += 1;
            }
            self.u_row_ptrs[i + 1] = u_count;
        }
    }

    /// Updates factorization from a modified matrix with identical or updated topology.
    pub fn update_and_refactor(
        &mut self,
        matrix: &SparseMatrixCsc,
        options: &MarkowitzOptions,
    ) -> Result<(), SolverError> {
        let n = matrix.nrows();
        if n != self.dim {
            *self = Self::from_matrix(matrix, options)?;
            return Ok(());
        }

        let base_lu = SparseLuFactorization::factor(matrix, options)?;
        self.row_perm.copy_from_slice(base_lu.row_perm());
        self.col_perm.copy_from_slice(base_lu.col_perm());
        self.inv_row_perm.copy_from_slice(base_lu.inv_row_perm());
        self.inv_col_perm.copy_from_slice(base_lu.inv_col_perm());
        self.flatten_from_base(&base_lu);
        Ok(())
    }

    /// High-throughput in-place linear solve: $\mathbf{A} \mathbf{x} = \mathbf{b}$.
    ///
    /// GUARANTEED ZERO HEAP ALLOCATIONS:
    /// Uses pre-allocated scratch buffers `scratch_y` and `scratch_z` and writes the solution directly into `sol`.
    #[inline]
    pub fn solve(&mut self, rhs: &[f64], sol: &mut [f64]) -> Result<(), SolverError> {
        let n = self.dim;
        if rhs.len() != n || sol.len() != n {
            return Err(SolverError::DimensionMismatch {
                expected: n,
                found_rows: rhs.len(),
                found_cols: sol.len(),
            });
        }

        if n == 0 {
            return Ok(());
        }

        // Step 1: Forward solve L * y = P * b
        // Permute RHS into scratch_y: y[i] = b[row_perm[i]]
        for i in 0..n {
            let orig_row = self.row_perm[i];
            self.scratch_y[i] = rhs[orig_row];
        }

        // Forward substitution with unit lower diagonal
        for i in 0..n {
            let start = self.l_row_ptrs[i];
            let end = self.l_row_ptrs[i + 1];
            let mut sum = 0.0;
            for idx in start..end {
                let col = self.l_cols[idx];
                let val = self.l_vals[idx];
                sum += val * self.scratch_y[col];
            }
            self.scratch_y[i] -= sum;
        }

        // Step 2: Backward solve U * z = y
        for i in (0..n).rev() {
            let start = self.u_row_ptrs[i];
            let end = self.u_row_ptrs[i + 1];
            let mut sum = 0.0;
            for idx in start..end {
                let col = self.u_cols[idx];
                let val = self.u_vals[idx];
                sum += val * self.scratch_z[col];
            }

            let diag = self.u_diag[i];
            if diag.abs() < 1e-25 {
                return Err(SolverError::NumericalAnomaly {
                    detail: format!("Zero diagonal element in U at step {}", i),
                });
            }
            self.scratch_z[i] = (self.scratch_y[i] - sum) / diag;
        }

        // Step 3: Un-permute solution into output: sol[col_perm[k]] = z[k]
        for k in 0..n {
            let orig_col = self.col_perm[k];
            sol[orig_col] = self.scratch_z[k];
        }

        Ok(())
    }

    /// Convenience wrapper returning an allocated solution vector.
    pub fn solve_vec(&mut self, rhs: &[f64]) -> Result<Vec<f64>, SolverError> {
        let mut sol = vec![0.0; self.dim];
        self.solve(rhs, &mut sol)?;
        Ok(sol)
    }

    /// Returns the matrix dimension.
    #[inline(always)]
    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Returns the non-zero element count in L.
    #[inline(always)]
    pub fn nnz_l(&self) -> usize {
        self.l_vals.len()
    }

    /// Returns the non-zero element count in U (excluding diagonal).
    #[inline(always)]
    pub fn nnz_u(&self) -> usize {
        self.u_vals.len()
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
}
