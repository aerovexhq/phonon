//! Markowitz minimum-degree ordering heuristic with threshold partial pivoting.

/// Markowitz search configuration options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MarkowitzOptions {
    /// Pivoting threshold $u \in (0, 1]$. Typically $0.01$ or $0.1$.
    /// A candidate pivot $a_{ij}$ must satisfy $|a_{ij}| \ge u \cdot \max_k |a_{kj}|$.
    pub threshold: f64,
    /// Absolute threshold below which an element is considered numerically zero.
    pub zero_tolerance: f64,
}

impl Default for MarkowitzOptions {
    fn default() -> Self {
        Self {
            threshold: 0.01,
            zero_tolerance: 1e-20,
        }
    }
}

/// Selected pivot location and value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PivotCandidate {
    pub row: usize,
    pub col: usize,
    pub value: f64,
    pub markowitz_cost: usize,
}

/// Finds the optimal pivot in the active submatrix using the Markowitz product heuristic
/// subject to the threshold stability condition.
pub fn find_markowitz_pivot(
    active_rows: &[usize],
    active_cols: &[usize],
    get_val: impl Fn(usize, usize) -> f64,
    options: &MarkowitzOptions,
) -> Option<PivotCandidate> {
    if active_rows.is_empty() || active_cols.is_empty() {
        return None;
    }

    // Step 1: Pre-calculate active row counts (r_i) and column maximums
    let mut row_counts: Vec<usize> = Vec::with_capacity(active_rows.len());
    for &r in active_rows {
        let mut count: usize = 0;
        for &c in active_cols {
            if get_val(r, c).abs() > options.zero_tolerance {
                count += 1;
            }
        }
        row_counts.push(count);
    }

    let mut col_counts: Vec<usize> = Vec::with_capacity(active_cols.len());
    let mut col_maxes = Vec::with_capacity(active_cols.len());

    for &c in active_cols {
        let mut count: usize = 0;
        let mut max_abs = 0.0;
        for &r in active_rows {
            let abs_val = get_val(r, c).abs();
            if abs_val > options.zero_tolerance {
                count += 1;
                if abs_val > max_abs {
                    max_abs = abs_val;
                }
            }
        }
        col_counts.push(count);
        col_maxes.push(max_abs);
    }

    let mut best_pivot: Option<PivotCandidate> = None;

    // Step 2: Search for candidate minimizing Markowitz product M_ij = (r_i - 1)(c_j - 1)
    for (c_idx, &c) in active_cols.iter().enumerate() {
        let max_val = col_maxes[c_idx];
        if max_val <= options.zero_tolerance {
            continue;
        }

        let min_acceptable_pivot = options.threshold * max_val;
        let col_count = col_counts[c_idx];

        for (r_idx, &r) in active_rows.iter().enumerate() {
            let val = get_val(r, c);
            let abs_val = val.abs();

            if abs_val >= min_acceptable_pivot && abs_val > options.zero_tolerance {
                let row_count = row_counts[r_idx];
                let cost = (row_count.saturating_sub(1)) * (col_count.saturating_sub(1));

                let is_better = match &best_pivot {
                    None => true,
                    Some(best) => {
                        if cost < best.markowitz_cost {
                            true
                        } else if cost == best.markowitz_cost {
                            abs_val > best.value.abs()
                        } else {
                            false
                        }
                    }
                };

                if is_better {
                    best_pivot = Some(PivotCandidate {
                        row: r,
                        col: c,
                        value: val,
                        markowitz_cost: cost,
                    });

                    // Optimal cost is 0 (singleton row or column) -> cannot do better
                    if cost == 0 {
                        return best_pivot;
                    }
                }
            }
        }
    }

    best_pivot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markowitz_singleton_pivot() {
        // 2x2 matrix: [[1, 0], [2, 3]]
        let rows = vec![0, 1];
        let cols = vec![0, 1];
        let get_val = |r, c| match (r, c) {
            (0, 0) => 1.0,
            (0, 1) => 0.0,
            (1, 0) => 2.0,
            (1, 1) => 3.0,
            _ => 0.0,
        };

        let options = MarkowitzOptions::default();
        let pivot = find_markowitz_pivot(&rows, &cols, get_val, &options).unwrap();
        // Element (0, 0) or (1, 1) has cost 0
        assert_eq!(pivot.markowitz_cost, 0);
    }
}
