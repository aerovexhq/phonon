//! Matrix condition number estimation and pivot diagnostics.

use super::csc::SparseMatrixCsc;
use super::lu::SparseLuFactorization;
use crate::error::SolverError;

/// Hager-Higham 1-norm condition number estimator: $\kappa_1(\mathbf{A}) = \|\mathbf{A}\|_1 \cdot \|\mathbf{A}^{-1}\|_1$.
/// Evaluates $\|\mathbf{A}^{-1}\|_1$ iteratively via triangular solves without explicitly inverting the matrix.
pub fn estimate_condition_1norm(
    matrix: &SparseMatrixCsc,
    lu: &SparseLuFactorization,
    max_iters: usize,
) -> Result<f64, SolverError> {
    let n = matrix.nrows();
    if n == 0 {
        return Ok(1.0);
    }

    // Compute ||A||_1 = max column sum
    let mut norm_a = 0.0f64;
    for col in 0..matrix.ncols() {
        let start = matrix.col_ptrs()[col];
        let end = matrix.col_ptrs()[col + 1];
        let mut col_sum = 0.0;
        for idx in start..end {
            col_sum += matrix.values()[idx].abs();
        }
        if col_sum > norm_a {
            norm_a = col_sum;
        }
    }

    if norm_a == 0.0 {
        return Ok(f64::INFINITY);
    }

    // Higham's algorithm: iterative estimation of ||A^{-1}||_1
    let mut x = vec![1.0 / (n as f64); n];
    let mut y = vec![0.0; n];
    let mut z = vec![0.0; n];
    let mut est_inv = 0.0f64;

    for _ in 0..max_iters {
        // Solve A * y = x
        lu.solve(&x, &mut y)?;

        // gamma = ||y||_1
        let gamma: f64 = y.iter().map(|v| v.abs()).sum();
        if gamma > est_inv {
            est_inv = gamma;
        }

        // xi = sign(y)
        for i in 0..n {
            z[i] = if y[i] >= 0.0 { 1.0 } else { -1.0 };
        }

        // Solve A^T * z_sol = z (approximated using same LU for symmetric/near-symmetric topologies)
        let mut z_sol = vec![0.0; n];
        lu.solve(&z, &mut z_sol)?;

        // Find max element in z_sol
        let mut max_idx = 0;
        let mut max_val = 0.0f64;
        for (i, &val) in z_sol.iter().enumerate() {
            if val.abs() > max_val {
                max_val = val.abs();
                max_idx = i;
            }
        }

        if max_val
            <= z.iter()
                .zip(x.iter())
                .map(|(&zi, &xi)| zi * xi)
                .sum::<f64>()
        {
            break;
        }

        // x = e_{max_idx}
        for xi in x.iter_mut() {
            *xi = 0.0;
        }
        x[max_idx] = 1.0;
    }

    Ok(norm_a * est_inv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse::builder::SparseMatrixBuilder;
    use crate::sparse::markowitz::MarkowitzOptions;

    #[test]
    fn test_condition_estimator_identity() {
        let mut b = SparseMatrixBuilder::new(3, 3);
        b.add(0, 0, 1.0);
        b.add(1, 1, 1.0);
        b.add(2, 2, 1.0);

        let mat = b.build_csc();
        let lu = SparseLuFactorization::factor(&mat, &MarkowitzOptions::default()).unwrap();
        let cond = estimate_condition_1norm(&mat, &lu, 5).unwrap();

        // Condition number of identity matrix is 1.0
        assert!((cond - 1.0).abs() < 1e-6);
    }
}
