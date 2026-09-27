//! Coordinate (COO) triplet matrix builder with automatic duplicate entry accumulation.

use super::csc::SparseMatrixCsc;

/// A triplet entry in the coordinate list.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Triplet {
    row: usize,
    col: usize,
    val: f64,
}

/// Dynamic matrix builder accepting incremental stamps and assembling Compressed Sparse Column (CSC) matrices.
#[derive(Debug, Clone)]
pub struct SparseMatrixBuilder {
    nrows: usize,
    ncols: usize,
    triplets: Vec<Triplet>,
}

impl SparseMatrixBuilder {
    /// Creates a new empty matrix builder of dimensions `nrows x ncols`.
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            triplets: Vec::new(),
        }
    }

    /// Creates a builder with pre-allocated capacity for anticipated non-zero stamps.
    pub fn with_capacity(nrows: usize, ncols: usize, capacity: usize) -> Self {
        Self {
            nrows,
            ncols,
            triplets: Vec::with_capacity(capacity),
        }
    }

    /// Adds a value to element $(row, col)$. Duplicate additions are automatically summed.
    #[inline(always)]
    pub fn add(&mut self, row: usize, col: usize, val: f64) {
        if row < self.nrows && col < self.ncols && val != 0.0 {
            self.triplets.push(Triplet { row, col, val });
        }
    }

    /// Returns the matrix dimensions (nrows, ncols).
    pub fn dims(&self) -> (usize, usize) {
        (self.nrows, self.ncols)
    }

    /// Clears all stored entries while retaining allocated buffer capacity.
    pub fn clear(&mut self) {
        self.triplets.clear();
    }

    /// Compiles the triplets into a canonical Compressed Sparse Column (CSC) matrix.
    /// Consecutive entries with identical $(row, col)$ coordinates are summed into a single non-zero entry.
    pub fn build_csc(&self) -> SparseMatrixCsc {
        if self.triplets.is_empty() {
            return SparseMatrixCsc::zeros(self.nrows, self.ncols);
        }

        // Sort by column first, then by row
        let mut sorted = self.triplets.clone();
        sorted.sort_unstable_by(|a, b| {
            if a.col != b.col {
                a.col.cmp(&b.col)
            } else {
                a.row.cmp(&b.row)
            }
        });

        let mut col_ptrs = Vec::with_capacity(self.ncols + 1);
        let mut row_indices = Vec::with_capacity(sorted.len());
        let mut values = Vec::with_capacity(sorted.len());

        let mut current_col = 0;
        col_ptrs.push(0);

        let mut i = 0;
        while i < sorted.len() {
            let col = sorted[i].col;

            // Pad empty columns if any
            while current_col < col {
                col_ptrs.push(values.len());
                current_col += 1;
            }

            let row = sorted[i].row;
            let mut sum_val = sorted[i].val;
            i += 1;

            // Aggregate identical (row, col) duplicates
            while i < sorted.len() && sorted[i].col == col && sorted[i].row == row {
                sum_val += sorted[i].val;
                i += 1;
            }

            // Keep non-zero entries (or diagonal entries even if near-zero)
            if sum_val.abs() > 1e-25 || row == col {
                row_indices.push(row);
                values.push(sum_val);
            }
        }

        // Pad remaining trailing columns
        while current_col < self.ncols {
            col_ptrs.push(values.len());
            current_col += 1;
        }

        SparseMatrixCsc::from_raw(self.nrows, self.ncols, col_ptrs, row_indices, values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_duplicate_summation() {
        let mut builder = SparseMatrixBuilder::new(3, 3);
        // Add duplicate entries at (1, 1)
        builder.add(1, 1, 5.0);
        builder.add(1, 1, 7.5);
        // Add single entry at (0, 2)
        builder.add(0, 2, 3.2);

        let csc = builder.build_csc();
        assert_eq!(csc.get(1, 1), 12.5);
        assert_eq!(csc.get(0, 2), 3.2);
        assert_eq!(csc.get(0, 0), 0.0);
    }
}
