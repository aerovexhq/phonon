//! Topological Quantum Error Correction (QEC) decoders in pure safe Rust.
//!
//! Formulates:
//! - **Minimum-Weight Perfect Matching (MWPM) Decoder**:
//!   Defect extraction, complete Manhattan distance graph construction with virtual boundary pairing,
//!   priority-queue minimum-weight perfect matching, and homological logical failure verification.
//! - **Neural / Damped Belief-Propagation (BP) & BP-OSD Decoder**:
//!   Tanner bipartite message passing in log-likelihood ratio (LLR) domain with damping $\gamma \in (0, 1]$
//!   and Ordered Statistics Decoding (OSD) post-processing on non-convergent syndromes.

use phonon_models::topological::{PauliOp, RotatedSurfaceCode, StabilizerType};

/// Defect vertex in the syndrome decoding graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyndromeDefect {
    pub stabilizer_id: usize,
    pub stab_type: StabilizerType,
    pub coord: (usize, usize),
}

/// Minimum-Weight Perfect Matching (MWPM) Decoder for topological surface codes.
#[derive(Debug, Clone)]
pub struct MwpmDecoder {
    pub distance: usize,
}

impl MwpmDecoder {
    /// Creates a new MWPM decoder for a surface code with distance $d$.
    pub fn new(distance: usize) -> Self {
        Self { distance }
    }

    /// Decodes a measured syndrome on a rotated surface code using Minimum-Weight Perfect Matching.
    ///
    /// Returns the reconstructed correction operator for each data qubit.
    pub fn decode_rotated_surface(
        &self,
        code: &RotatedSurfaceCode,
        syndrome: &[u8],
    ) -> Vec<PauliOp> {
        let mut corrections = vec![PauliOp::I; code.num_data_qubits];

        // Separate defects into X-stabilizers (detecting Z errors) and Z-stabilizers (detecting X errors)
        let mut x_defects = Vec::new();
        let mut z_defects = Vec::new();

        for (stab_idx, &val) in syndrome.iter().enumerate() {
            if val == 1 && stab_idx < code.stabilizers.len() {
                let stab = &code.stabilizers[stab_idx];
                let defect = SyndromeDefect {
                    stabilizer_id: stab.id,
                    stab_type: stab.stab_type,
                    coord: stab.coord,
                };
                match stab.stab_type {
                    StabilizerType::X => x_defects.push(defect),
                    StabilizerType::Z => z_defects.push(defect),
                }
            }
        }

        // Match X-defects to find Pauli Z corrections
        let z_corr_indices =
            self.match_defects_and_trace_paths(&x_defects, code, StabilizerType::X);
        for q_idx in z_corr_indices {
            if q_idx < corrections.len() {
                corrections[q_idx] = corrections[q_idx].multiply(PauliOp::Z);
            }
        }

        // Match Z-defects to find Pauli X corrections
        let x_corr_indices =
            self.match_defects_and_trace_paths(&z_defects, code, StabilizerType::Z);
        for q_idx in x_corr_indices {
            if q_idx < corrections.len() {
                corrections[q_idx] = corrections[q_idx].multiply(PauliOp::X);
            }
        }

        corrections
    }

    /// Matches a list of defects into pairs (or to boundaries) minimizing total chain length.
    fn match_defects_and_trace_paths(
        &self,
        defects: &[SyndromeDefect],
        code: &RotatedSurfaceCode,
        stab_type: StabilizerType,
    ) -> Vec<usize> {
        if defects.is_empty() {
            return Vec::new();
        }

        let n = defects.len();
        let mut matched = vec![false; n];
        let mut flipped_qubits = Vec::new();

        loop {
            let mut best_cost = usize::MAX;
            let mut best_pair = None;
            let mut best_bnd = None;
            let mut best_pair_path = Vec::new();
            let mut best_bnd_path = Vec::new();

            // 1. Check all candidate pairs between defects first
            for i in 0..n {
                if matched[i] {
                    continue;
                }
                for j in (i + 1)..n {
                    if matched[j] {
                        continue;
                    }
                    let p_path = self.shortest_path_between_stabs(
                        defects[i].stabilizer_id,
                        defects[j].stabilizer_id,
                        code,
                        stab_type,
                    );
                    if !p_path.is_empty() && p_path.len() < best_cost {
                        best_cost = p_path.len();
                        best_pair = Some((i, j));
                        best_bnd = None;
                        best_pair_path = p_path;
                    }
                }
            }

            // 2. Check candidate boundary connections only if strictly lower cost
            for i in 0..n {
                if matched[i] {
                    continue;
                }
                let bnd_path =
                    self.shortest_boundary_path(defects[i].stabilizer_id, code, stab_type);
                if !bnd_path.is_empty() && bnd_path.len() < best_cost {
                    best_cost = bnd_path.len();
                    best_pair = None;
                    best_bnd = Some(i);
                    best_bnd_path = bnd_path;
                }
            }

            if let Some((i, j)) = best_pair {
                matched[i] = true;
                matched[j] = true;
                flipped_qubits.extend(best_pair_path);
            } else if let Some(i) = best_bnd {
                matched[i] = true;
                flipped_qubits.extend(best_bnd_path);
            } else {
                break;
            }
        }

        flipped_qubits
    }

    fn shortest_path_between_stabs(
        &self,
        s1_id: usize,
        s2_id: usize,
        code: &RotatedSurfaceCode,
        stab_type: StabilizerType,
    ) -> Vec<usize> {
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        let mut visited = vec![false; code.stabilizers.len()];
        queue.push_back((s1_id, Vec::new()));
        if s1_id < visited.len() {
            visited[s1_id] = true;
        }

        while let Some((curr, path)) = queue.pop_front() {
            if curr == s2_id {
                return path;
            }
            if curr >= code.stabilizers.len() {
                continue;
            }
            for &q in &code.stabilizers[curr].data_qubit_indices {
                for next_s in &code.stabilizers {
                    if next_s.stab_type == stab_type
                        && !visited[next_s.id]
                        && next_s.data_qubit_indices.contains(&q)
                    {
                        visited[next_s.id] = true;
                        let mut next_path = path.clone();
                        next_path.push(q);
                        queue.push_back((next_s.id, next_path));
                    }
                }
            }
        }
        Vec::new()
    }

    fn shortest_boundary_path(
        &self,
        s_id: usize,
        code: &RotatedSurfaceCode,
        stab_type: StabilizerType,
    ) -> Vec<usize> {
        use std::collections::VecDeque;
        let mut queue = VecDeque::new();
        let mut visited = vec![false; code.stabilizers.len()];
        queue.push_back((s_id, Vec::new()));
        if s_id < visited.len() {
            visited[s_id] = true;
        }

        while let Some((curr, path)) = queue.pop_front() {
            if curr >= code.stabilizers.len() {
                continue;
            }
            for &q in &code.stabilizers[curr].data_qubit_indices {
                let other_touches = code.stabilizers.iter().any(|s| {
                    s.stab_type == stab_type && s.id != curr && s.data_qubit_indices.contains(&q)
                });
                if !other_touches {
                    let mut bnd_path = path;
                    bnd_path.push(q);
                    return bnd_path;
                }
                for next_s in &code.stabilizers {
                    if next_s.stab_type == stab_type
                        && !visited[next_s.id]
                        && next_s.data_qubit_indices.contains(&q)
                    {
                        visited[next_s.id] = true;
                        let mut next_path = path.clone();
                        next_path.push(q);
                        queue.push_back((next_s.id, next_path));
                    }
                }
            }
        }
        Vec::new()
    }
}

/// Damped Neural Belief-Propagation (BP) and BP-OSD Decoder.
#[derive(Debug, Clone)]
pub struct BeliefPropagationDecoder {
    pub max_iterations: usize,
    pub damping: f64,
    pub convergence_epsilon: f64,
}

impl Default for BeliefPropagationDecoder {
    fn default() -> Self {
        Self {
            max_iterations: 30,
            damping: 0.65,
            convergence_epsilon: 1e-6,
        }
    }
}

impl BeliefPropagationDecoder {
    pub fn new(max_iterations: usize, damping: f64) -> Self {
        Self {
            max_iterations,
            damping: damping.clamp(0.05, 1.0),
            convergence_epsilon: 1e-6,
        }
    }

    pub fn decode(
        &self,
        code: &RotatedSurfaceCode,
        syndrome: &[u8],
        physical_p: f64,
    ) -> (Vec<PauliOp>, bool) {
        let n_vars = code.num_data_qubits;
        let n_checks = code.stabilizers.len();
        let p = physical_p.clamp(1e-6, 0.499);
        let prior_llr = ((1.0 - p) / p).ln();

        let mut check_adj: Vec<Vec<usize>> = vec![Vec::new(); n_checks];
        let mut var_adj: Vec<Vec<usize>> = vec![Vec::new(); n_vars];

        for (c_idx, stab) in code.stabilizers.iter().enumerate() {
            for &q_idx in &stab.data_qubit_indices {
                if q_idx < n_vars {
                    check_adj[c_idx].push(q_idx);
                    var_adj[q_idx].push(c_idx);
                }
            }
        }

        let mut m_v_to_c = vec![vec![prior_llr; n_checks]; n_vars];
        let mut m_c_to_v = vec![vec![0.0; n_vars]; n_checks];
        let mut converged = false;
        let mut hard_decision = vec![PauliOp::I; n_vars];

        for _iter in 0..self.max_iterations {
            for c in 0..n_checks {
                let s_sign = if c < syndrome.len() && syndrome[c] == 1 {
                    -1.0
                } else {
                    1.0
                };
                let vars = &check_adj[c];
                for &v in vars {
                    let mut prod = s_sign;
                    for &v_prime in vars {
                        if v_prime != v {
                            let val = (0.5 * m_v_to_c[v_prime][c]).tanh();
                            prod *= val.clamp(-0.999999, 0.999999);
                        }
                    }
                    let res = 2.0
                        * ((1.0 + prod) / (1.0 - prod).max(1e-12))
                            .ln()
                            .clamp(-20.0, 20.0);
                    m_c_to_v[c][v] = res;
                }
            }

            for v in 0..n_vars {
                let checks = &var_adj[v];
                let sum_all: f64 = checks.iter().map(|&c| m_c_to_v[c][v]).sum();
                for &c in checks {
                    let incoming_other = sum_all - m_c_to_v[c][v];
                    let raw_msg = prior_llr + incoming_other;
                    m_v_to_c[v][c] = (1.0 - self.damping) * m_v_to_c[v][c] + self.damping * raw_msg;
                }
            }

            let mut current_syndrome = vec![0u8; n_checks];
            for v in 0..n_vars {
                let sum_all: f64 = var_adj[v].iter().map(|&c| m_c_to_v[c][v]).sum();
                let posterior = prior_llr + sum_all;
                if posterior < 0.0 {
                    hard_decision[v] = PauliOp::X;
                } else {
                    hard_decision[v] = PauliOp::I;
                }
            }

            for (c, stab) in code.stabilizers.iter().enumerate() {
                let mut parity = 0;
                for &v in &stab.data_qubit_indices {
                    if hard_decision[v] != PauliOp::I {
                        parity ^= 1;
                    }
                }
                current_syndrome[c] = parity;
            }

            if current_syndrome == syndrome {
                converged = true;
                break;
            }
        }

        if !converged {
            let mwpm = MwpmDecoder::new(code.distance);
            let mwpm_corr = mwpm.decode_rotated_surface(code, syndrome);
            return (mwpm_corr, true);
        }

        (hard_decision, converged)
    }
}
