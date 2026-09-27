//! Compressed Sparse Column (CSC) matrix format for circuit linear algebra.

/// Compressed Sparse Column (CSC) sparse matrix representation.
#[derive(Debug, Clone, PartialEq)]
pub struct SparseMatrixCsc {
    nrows: usize,
    ncols: usize,
    col_ptrs: Vec<usize>,
    row_indices: Vec<usize>,
    values: Vec<f64>,
}

impl SparseMatrixCsc {
    /// Creates an empty sparse matrix with zero entries.
    pub fn zeros(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            col_ptrs: vec![0; ncols + 1],
            row_indices: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Constructs a CSC matrix directly from raw arrays without validation.
    pub fn from_raw(
        nrows: usize,
        ncols: usize,
        col_ptrs: Vec<usize>,
        row_indices: Vec<usize>,
        values: Vec<f64>,
    ) -> Self {
        Self {
            nrows,
            ncols,
            col_ptrs,
            row_indices,
            values,
        }
    }

    #[inline(always)]
    pub fn nrows(&self) -> usize {
        self.nrows
    }

    #[inline(always)]
    pub fn ncols(&self) -> usize {
        self.ncols
    }

    #[inline(always)]
    pub fn nnz(&self) -> usize {
        self.values.len()
    }

    #[inline(always)]
    pub fn col_ptrs(&self) -> &[usize] {
        &self.col_ptrs
    }

    #[inline(always)]
    pub fn row_indices(&self) -> &[usize] {
        &self.row_indices
    }

    #[inline(always)]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    #[inline(always)]
    pub fn values_mut(&mut self) -> &mut [f64] {
        &mut self.values
    }

    /// Retrieves the matrix element $A(row, col)$ via binary search on column rows.
    pub fn get(&self, row: usize, col: usize) -> f64 {
        if row >= self.nrows || col >= self.ncols {
            return 0.0;
        }

        let start = self.col_ptrs[col];
        let end = self.col_ptrs[col + 1];

        if start == end {
            return 0.0;
        }

        let rows = &self.row_indices[start..end];
        match rows.binary_search(&row) {
            Ok(idx) => self.values[start + idx],
            Err(_) => 0.0,
        }
    }

    /// Performs sparse matrix-vector multiplication $\mathbf{y} = \mathbf{A} \mathbf{x}$.
    pub fn matvec(&self, x: &[f64], y: &mut [f64]) {
        assert_eq!(x.len(), self.ncols, "x length mismatch");
        assert_eq!(y.len(), self.nrows, "y length mismatch");

        for yi in y.iter_mut() {
            *yi = 0.0;
        }

        for (j, &xj) in x.iter().enumerate().take(self.ncols) {
            if xj == 0.0 {
                continue;
            }
            let start = self.col_ptrs[j];
            let end = self.col_ptrs[j + 1];
            for k in start..end {
                let row = self.row_indices[k];
                let val = self.values[k];
                y[row] += val * xj;
            }
        }
    }

    /// Computes the residual vector $\mathbf{r} = \mathbf{b} - \mathbf{A} \mathbf{x}$.
    pub fn residual(&self, x: &[f64], b: &[f64], r: &mut [f64]) {
        assert_eq!(b.len(), self.nrows, "b length mismatch");
        assert_eq!(r.len(), self.nrows, "r length mismatch");

        self.matvec(x, r);
        for (ri, &bi) in r.iter_mut().zip(b.iter()) {
            *ri = bi - *ri;
        }
    }

    /// Computes the infinity-norm (maximum absolute row sum).
    pub fn norm_inf(&self) -> f64 {
        let mut row_sums = vec![0.0; self.nrows];
        for j in 0..self.ncols {
            let start = self.col_ptrs[j];
            let end = self.col_ptrs[j + 1];
            for k in start..end {
                let row = self.row_indices[k];
                row_sums[row] += self.values[k].abs();
            }
        }
        row_sums.into_iter().fold(0.0, f64::max)
    }

    /// Converts the sparse matrix into a dense 2D vector for small matrix inspection.
    #[allow(clippy::needless_range_loop)]
    pub fn to_dense(&self) -> Vec<Vec<f64>> {
        let mut dense = vec![vec![0.0; self.ncols]; self.nrows];
        for j in 0..self.ncols {
            let start = self.col_ptrs[j];
            let end = self.col_ptrs[j + 1];
            for k in start..end {
                let row = self.row_indices[k];
                dense[row][j] = self.values[k];
            }
        }
        dense
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse::builder::SparseMatrixBuilder;

    #[test]
    fn test_matvec() {
        let mut builder = SparseMatrixBuilder::new(3, 3);
        builder.add(0, 0, 2.0);
        builder.add(0, 1, 1.0);
        builder.add(1, 1, 4.0);
        builder.add(2, 2, 3.0);

        let csc = builder.build_csc();
        let x = vec![1.0, 2.0, 3.0];
        let mut y = vec![0.0; 3];
        csc.matvec(&x, &mut y);

        assert_eq!(y, vec![4.0, 8.0, 9.0]);

        let b = vec![4.0, 8.0, 9.0];
        let mut res = vec![0.0; 3];
        csc.residual(&x, &b, &mut res);
        assert_eq!(res, vec![0.0, 0.0, 0.0]);

        assert_eq!(csc.norm_inf(), 4.0);

        let dense = csc.to_dense();
        assert_eq!(dense[0], vec![2.0, 1.0, 0.0]);
        assert_eq!(dense[1], vec![0.0, 4.0, 0.0]);
        assert_eq!(dense[2], vec![0.0, 0.0, 3.0]);

        let empty = SparseMatrixCsc::zeros(2, 2);
        assert_eq!(empty.nnz(), 0);
    }
}
