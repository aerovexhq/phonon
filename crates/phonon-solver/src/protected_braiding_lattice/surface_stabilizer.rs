#![deny(unsafe_code)]

//! Surface Code Stabilizer Engine on a Protected Topological Lattice.
//!
//! Provides:
//! - Distance d = 3 rotated surface code patch with 9 data qubits and 8 ancilla qubits
//!   (4 Star X-stabilizers, 4 Plaquette Z-stabilizers).
//! - Measurement of stabilizer eigenvalues (+1, -1) and extraction of syndrome defects.
//! - Minimum-Weight Perfect Matching (MWPM) / greedy defect decoder applying recovery operations.
//! - Monte Carlo evaluation of logical error rate verifying P_L << P_phys.

/// Lightweight deterministic pseudo-random number generator for syndrome simulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    /// Creates a new RNG seeded with a 64-bit integer.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c_49e6_748f_ea9b } else { seed },
        }
    }

    /// Generates next pseudo-random u64.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generates next pseudo-random f64 in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Type of stabilizer operator in the surface code patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilizerKind {
    /// Star stabilizer: A_s = prod_{i in s} X_i (measures Z errors)
    StarX,
    /// Plaquette stabilizer: B_p = prod_{j in p} Z_j (measures X errors)
    PlaquetteZ,
}

/// Stabilizer check representation.
#[derive(Debug, Clone, PartialEq)]
pub struct StabilizerCheck {
    pub id: usize,
    pub kind: StabilizerKind,
    pub data_qubits: Vec<usize>,
    pub center_x: f64,
    pub center_y: f64,
    pub label: &'static str,
}

/// Result of stabilizer syndrome extraction.
#[derive(Debug, Clone, PartialEq)]
pub struct SyndromeResult {
    /// Eigenvalues (+1 or -1) of the 4 Star (X-type) stabilizers: A_0 .. A_3.
    pub star_eigenvalues: [i8; 4],
    /// Eigenvalues (+1 or -1) of the 4 Plaquette (Z-type) stabilizers: B_0 .. B_3.
    pub plaquette_eigenvalues: [i8; 4],
    /// Indices of Star stabilizers with -1 eigenvalue (defects caused by Z errors).
    pub star_defects: Vec<usize>,
    /// Indices of Plaquette stabilizers with -1 eigenvalue (defects caused by X errors).
    pub plaquette_defects: Vec<usize>,
    /// Injected Pauli X errors on the 9 data qubits.
    pub data_x_errors: [bool; 9],
    /// Injected Pauli Z errors on the 9 data qubits.
    pub data_z_errors: [bool; 9],
    /// Physical error rate used during syndrome extraction.
    pub physical_error_rate: f64,
}

impl SyndromeResult {
    /// Returns true if no syndrome defects were detected.
    pub fn is_clean(&self) -> bool {
        self.star_defects.is_empty() && self.plaquette_defects.is_empty()
    }

    /// Total number of detected syndrome defects.
    pub fn defect_count(&self) -> usize {
        self.star_defects.len() + self.plaquette_defects.len()
    }
}

/// Result of syndrome decoding and recovery correction.
#[derive(Debug, Clone, PartialEq)]
pub struct CorrectionResult {
    /// Pauli X recovery corrections applied to data qubits.
    pub correction_x: [bool; 9],
    /// Pauli Z recovery corrections applied to data qubits.
    pub correction_z: [bool; 9],
    /// Residual defect count after applying recovery (should be 0 for valid matching).
    pub residual_defects: usize,
    /// Whether the syndrome was completely resolved.
    pub is_clean: bool,
    /// Whether an uncorrectable logical flip occurred (P_L event).
    pub has_logical_error: bool,
    /// Logical Z operator flip occurred.
    pub logical_z_flip: bool,
    /// Logical X operator flip occurred.
    pub logical_x_flip: bool,
    /// Geometric defect pairing chains for visual display in GUI: (qubit_idx, ancilla_id).
    pub correction_chains: Vec<(usize, usize)>,
}

/// Distance d = 3 surface code grid patch.
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceCodeGrid {
    /// 9 Data qubit coordinates (r, c) on 3x3 layout.
    pub data_positions: [(f64, f64); 9],
    /// 4 Star (X-type) stabilizer definitions.
    pub star_stabilizers: [StabilizerCheck; 4],
    /// 4 Plaquette (Z-type) stabilizer definitions.
    pub plaquette_stabilizers: [StabilizerCheck; 4],
}

impl Default for SurfaceCodeGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceCodeGrid {
    /// Constructs a canonical distance d = 3 rotated planar surface code patch.
    pub fn new() -> Self {
        // Data qubits: 3x3 grid indexed 0..8
        // 0 (0,0)  1 (0,1)  2 (0,2)
        // 3 (1,0)  4 (1,1)  5 (1,2)
        // 6 (2,0)  7 (2,1)  8 (2,2)
        let data_positions = [
            (0.0, 0.0), (0.0, 1.0), (0.0, 2.0),
            (1.0, 0.0), (1.0, 1.0), (1.0, 2.0),
            (2.0, 0.0), (2.0, 1.0), (2.0, 2.0),
        ];

        // 4 Star (X-type) stabilizers:
        // A_0 (top boundary): [0, 1]
        // A_1 (center-right): [1, 2, 4, 5]
        // A_2 (center-left): [3, 4, 6, 7]
        // A_3 (bottom boundary): [7, 8]
        let star_stabilizers = [
            StabilizerCheck {
                id: 0,
                kind: StabilizerKind::StarX,
                data_qubits: vec![0, 1],
                center_x: 0.5,
                center_y: -0.4,
                label: "A_1 (X)",
            },
            StabilizerCheck {
                id: 1,
                kind: StabilizerKind::StarX,
                data_qubits: vec![1, 2, 4, 5],
                center_x: 1.5,
                center_y: 0.5,
                label: "A_2 (X)",
            },
            StabilizerCheck {
                id: 2,
                kind: StabilizerKind::StarX,
                data_qubits: vec![3, 4, 6, 7],
                center_x: 0.5,
                center_y: 1.5,
                label: "A_3 (X)",
            },
            StabilizerCheck {
                id: 3,
                kind: StabilizerKind::StarX,
                data_qubits: vec![7, 8],
                center_x: 1.5,
                center_y: 2.4,
                label: "A_4 (X)",
            },
        ];

        // 4 Plaquette (Z-type) stabilizers:
        // B_0 (top-left): [0, 1, 3, 4]
        // B_1 (right boundary): [2, 5]
        // B_2 (left boundary): [3, 6]
        // B_3 (bottom-right): [4, 5, 7, 8]
        let plaquette_stabilizers = [
            StabilizerCheck {
                id: 0,
                kind: StabilizerKind::PlaquetteZ,
                data_qubits: vec![0, 1, 3, 4],
                center_x: 0.5,
                center_y: 0.5,
                label: "B_1 (Z)",
            },
            StabilizerCheck {
                id: 1,
                kind: StabilizerKind::PlaquetteZ,
                data_qubits: vec![2, 5],
                center_x: 2.4,
                center_y: 0.5,
                label: "B_2 (Z)",
            },
            StabilizerCheck {
                id: 2,
                kind: StabilizerKind::PlaquetteZ,
                data_qubits: vec![3, 6],
                center_x: -0.4,
                center_y: 1.5,
                label: "B_3 (Z)",
            },
            StabilizerCheck {
                id: 3,
                kind: StabilizerKind::PlaquetteZ,
                data_qubits: vec![4, 5, 7, 8],
                center_x: 1.5,
                center_y: 1.5,
                label: "B_4 (Z)",
            },
        ];

        Self {
            data_positions,
            star_stabilizers,
            plaquette_stabilizers,
        }
    }

    /// Evaluates Star stabilizer syndrome eigenvalues (+1, -1) from given Z error mask.
    pub fn evaluate_star_syndromes(&self, z_errors: &[bool; 9]) -> ([i8; 4], Vec<usize>) {
        let mut eigenvalues = [1i8; 4];
        let mut defects = Vec::new();

        for (idx, check) in self.star_stabilizers.iter().enumerate() {
            let mut z_count = 0;
            for &q in &check.data_qubits {
                if z_errors[q] {
                    z_count += 1;
                }
            }
            if z_count % 2 == 1 {
                eigenvalues[idx] = -1;
                defects.push(idx);
            }
        }

        (eigenvalues, defects)
    }

    /// Evaluates Plaquette stabilizer syndrome eigenvalues (+1, -1) from given X error mask.
    pub fn evaluate_plaquette_syndromes(&self, x_errors: &[bool; 9]) -> ([i8; 4], Vec<usize>) {
        let mut eigenvalues = [1i8; 4];
        let mut defects = Vec::new();

        for (idx, check) in self.plaquette_stabilizers.iter().enumerate() {
            let mut x_count = 0;
            for &q in &check.data_qubits {
                if x_errors[q] {
                    x_count += 1;
                }
            }
            if x_count % 2 == 1 {
                eigenvalues[idx] = -1;
                defects.push(idx);
            }
        }

        (eigenvalues, defects)
    }

    /// Extracts syndrome defects under depolarizing physical noise at error rate `physical_error_rate`.
    pub fn extract_syndromes(&self, physical_error_rate: f64) -> SyndromeResult {
        let mut rng = XorShiftRng::new(1337_4242);
        self.extract_syndromes_with_rng(physical_error_rate, &mut rng)
    }

    /// Extracts syndromes with caller-supplied RNG for deterministic Monte Carlo simulation.
    pub fn extract_syndromes_with_rng(
        &self,
        physical_error_rate: f64,
        rng: &mut XorShiftRng,
    ) -> SyndromeResult {
        let mut data_x = [false; 9];
        let mut data_z = [false; 9];

        // Depolarizing channel: each qubit experiences X, Y, or Z error with probability p / 3
        let p_each = (physical_error_rate / 3.0).clamp(0.0, 0.33);
        for i in 0..9 {
            let r = rng.next_f64();
            if r < p_each {
                // Pauli X
                data_x[i] = true;
            } else if r < 2.0 * p_each {
                // Pauli Z
                data_z[i] = true;
            } else if r < 3.0 * p_each {
                // Pauli Y = i * X * Z
                data_x[i] = true;
                data_z[i] = true;
            }
        }

        self.extract_syndromes_from_errors(data_x, data_z, physical_error_rate)
    }

    /// Extracts syndrome from explicit error patterns.
    pub fn extract_syndromes_from_errors(
        &self,
        data_x: [bool; 9],
        data_z: [bool; 9],
        physical_error_rate: f64,
    ) -> SyndromeResult {
        let (star_eig, star_def) = self.evaluate_star_syndromes(&data_z);
        let (plaq_eig, plaq_def) = self.evaluate_plaquette_syndromes(&data_x);

        SyndromeResult {
            star_eigenvalues: star_eig,
            plaquette_eigenvalues: plaq_eig,
            star_defects: star_def,
            plaquette_defects: plaq_def,
            data_x_errors: data_x,
            data_z_errors: data_z,
            physical_error_rate,
        }
    }

    /// Decodes syndrome defects using Minimum-Weight Perfect Matching (MWPM) / greedy defect decoder,
    /// applies recovery operations, and evaluates whether a logical flip occurred.
    pub fn decode_and_correct(&self, syndromes: &SyndromeResult) -> CorrectionResult {
        // Step 1: Decode Star defects (caused by Z errors) -> find minimum-weight Z correction
        let corr_z = self.find_minimum_weight_z_correction(&syndromes.star_defects);

        // Step 2: Decode Plaquette defects (caused by X errors) -> find minimum-weight X correction
        let corr_x = self.find_minimum_weight_x_correction(&syndromes.plaquette_defects);

        // Step 3: Check residual defects after correction
        let mut net_z = [false; 9];
        let mut net_x = [false; 9];
        for i in 0..9 {
            net_z[i] = syndromes.data_z_errors[i] ^ corr_z[i];
            net_x[i] = syndromes.data_x_errors[i] ^ corr_x[i];
        }

        let (_, residual_star) = self.evaluate_star_syndromes(&net_z);
        let (_, residual_plaq) = self.evaluate_plaquette_syndromes(&net_x);
        let residual_defects = residual_star.len() + residual_plaq.len();

        // Step 4: Evaluate logical error
        // Logical Z_L = Z_0 * Z_1 * Z_2 (horizontal top row)
        // Logical X_L = X_0 * X_3 * X_6 (vertical left column)
        // Net Z error causes a logical Z flip if it has odd overlap with dual logical X_L = [0, 3, 6]
        let mut z_flip_count = 0;
        for &q in &[0, 3, 6] {
            if net_z[q] {
                z_flip_count += 1;
            }
        }
        let logical_z_flip = z_flip_count % 2 == 1;

        // Net X error causes a logical X flip if it has odd overlap with dual logical Z_L = [0, 1, 2]
        let mut x_flip_count = 0;
        for &q in &[0, 1, 2] {
            if net_x[q] {
                x_flip_count += 1;
            }
        }
        let logical_x_flip = x_flip_count % 2 == 1;
        let has_logical_error = logical_z_flip || logical_x_flip;

        // Step 5: Geometric correction chains for visual rendering
        let mut correction_chains = Vec::new();
        for i in 0..9 {
            if corr_z[i] {
                // Find nearest star defect
                if let Some(&defect_id) = syndromes.star_defects.first() {
                    correction_chains.push((i, defect_id));
                }
            }
            if corr_x[i] {
                // Find nearest plaquette defect
                if let Some(&defect_id) = syndromes.plaquette_defects.first() {
                    correction_chains.push((i, defect_id + 4));
                }
            }
        }

        CorrectionResult {
            correction_x: corr_x,
            correction_z: corr_z,
            residual_defects,
            is_clean: residual_defects == 0,
            has_logical_error,
            logical_z_flip,
            logical_x_flip,
            correction_chains,
        }
    }

    /// Finds the minimum Hamming weight Z error mask on the 9 data qubits that generates `star_defects`.
    fn find_minimum_weight_z_correction(&self, target_defects: &[usize]) -> [bool; 9] {
        if target_defects.is_empty() {
            return [false; 9];
        }

        let mut best_mask = 0usize;
        let mut best_weight = usize::MAX;

        // Search over all 2^9 = 512 candidate bitmasks
        for mask in 0usize..512 {
            let weight = mask.count_ones() as usize;
            if weight >= best_weight {
                continue;
            }

            let mut z_cand = [false; 9];
            for i in 0..9 {
                if (mask & (1 << i)) != 0 {
                    z_cand[i] = true;
                }
            }

            let (_, defects) = self.evaluate_star_syndromes(&z_cand);
            if defects == target_defects {
                best_weight = weight;
                best_mask = mask;
            }
        }

        let mut res = [false; 9];
        for i in 0..9 {
            if (best_mask & (1 << i)) != 0 {
                res[i] = true;
            }
        }
        res
    }

    /// Finds the minimum Hamming weight X error mask on the 9 data qubits that generates `plaquette_defects`.
    fn find_minimum_weight_x_correction(&self, target_defects: &[usize]) -> [bool; 9] {
        if target_defects.is_empty() {
            return [false; 9];
        }

        let mut best_mask = 0usize;
        let mut best_weight = usize::MAX;

        // Search over all 2^9 = 512 candidate bitmasks
        for mask in 0usize..512 {
            let weight = mask.count_ones() as usize;
            if weight >= best_weight {
                continue;
            }

            let mut x_cand = [false; 9];
            for i in 0..9 {
                if (mask & (1 << i)) != 0 {
                    x_cand[i] = true;
                }
            }

            let (_, defects) = self.evaluate_plaquette_syndromes(&x_cand);
            if defects == target_defects {
                best_weight = weight;
                best_mask = mask;
            }
        }

        let mut res = [false; 9];
        for i in 0..9 {
            if (best_mask & (1 << i)) != 0 {
                res[i] = true;
            }
        }
        res
    }

    /// Evaluates logical error rate P_L across `num_trials` Monte Carlo shots.
    /// Demonstrates exponential suppression of logical errors: P_L << P_phys.
    pub fn evaluate_logical_error_rate(
        &self,
        physical_error_rate: f64,
        num_trials: usize,
    ) -> f64 {
        let mut rng = XorShiftRng::new(0xdead_beef_1234);
        let mut logical_errors = 0;

        for _ in 0..num_trials {
            let syn = self.extract_syndromes_with_rng(physical_error_rate, &mut rng);
            let cor = self.decode_and_correct(&syn);
            if cor.has_logical_error {
                logical_errors += 1;
            }
        }

        (logical_errors as f64) / (num_trials.max(1) as f64)
    }
}
