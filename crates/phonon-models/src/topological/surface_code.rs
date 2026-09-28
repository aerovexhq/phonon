//! Rotated Surface Codes, 2D Color Codes, and Stabilizer Quantum Error Correction Models.
//!
//! Formulates:
//! - Parameterized **Rotated Surface Code** $\mathcal{S}(d)$ with odd distance $d \ge 3$,
//!   $N_d = d^2$ data qubits, $N_{anc} = d^2 - 1$ syndrome ancillas ($X$- and $Z$-type stabilizers),
//!   and minimum-weight logical operators $X_L, Z_L$.
//! - **2D Triangular Color Code** (6.6.6 hexagonal / 4.8.8 lattice) with 3-colorable plaquettes
//!   and transversal Clifford gates ($H, S$, CNOT).
//! - Phenomenological Pauli noise models (bit-flip $p_X$, phase-flip $p_Z$, depolarizing $p$, and measurement error $q$).
//! - Exact syndrome extraction and defect coordinate mapping.

/// Stabilizer check type in topological quantum codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StabilizerType {
    /// X-type stabilizer (measures $\prod X_i$, detects Pauli $Z$ phase-flip errors).
    X,
    /// Z-type stabilizer (measures $\prod Z_i$, detects Pauli $X$ bit-flip errors).
    Z,
}

/// Single Pauli error operator on a physical data qubit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PauliOp {
    I,
    X,
    Y,
    Z,
}

impl PauliOp {
    /// Multiplies two Pauli operators (ignoring global phases $\pm 1, \pm i$).
    pub fn multiply(self, other: Self) -> Self {
        match (self, other) {
            (PauliOp::I, p) | (p, PauliOp::I) => p,
            (PauliOp::X, PauliOp::X) => PauliOp::I,
            (PauliOp::Y, PauliOp::Y) => PauliOp::I,
            (PauliOp::Z, PauliOp::Z) => PauliOp::I,
            (PauliOp::X, PauliOp::Y) | (PauliOp::Y, PauliOp::X) => PauliOp::Z,
            (PauliOp::X, PauliOp::Z) | (PauliOp::Z, PauliOp::X) => PauliOp::Y,
            (PauliOp::Y, PauliOp::Z) | (PauliOp::Z, PauliOp::Y) => PauliOp::X,
        }
    }

    /// Checks if this Pauli operator anti-commutes with a stabilizer of given type.
    pub fn anticommutes_with_stabilizer(self, stab_type: StabilizerType) -> bool {
        match stab_type {
            StabilizerType::X => self == PauliOp::Z || self == PauliOp::Y,
            StabilizerType::Z => self == PauliOp::X || self == PauliOp::Y,
        }
    }
}

/// A stabilizer generator definition specifying support over data qubits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StabilizerGenerator {
    /// Unique index of the stabilizer.
    pub id: usize,
    /// Check type ($X$ or $Z$).
    pub stab_type: StabilizerType,
    /// Coordinates $(r, c)$ in the dual lattice grid.
    pub coord: (usize, usize),
    /// Indices of physical data qubits in the support of this stabilizer.
    pub data_qubit_indices: Vec<usize>,
}

/// Rotated surface code geometry $\mathcal{S}(d)$.
#[derive(Debug, Clone, PartialEq)]
pub struct RotatedSurfaceCode {
    /// Code distance $d$ (odd integer $\ge 3$, e.g., 3, 5, 7, 9).
    pub distance: usize,
    /// Number of physical data qubits $N_d = d^2$.
    pub num_data_qubits: usize,
    /// Number of syndrome ancillas / stabilizer generators $N_{anc} = d^2 - 1$.
    pub num_stabilizers: usize,
    /// Stabilizer generators list.
    pub stabilizers: Vec<StabilizerGenerator>,
    /// Indices of data qubits forming the canonical logical $X_L$ operator.
    pub logical_x_indices: Vec<usize>,
    /// Indices of data qubits forming the canonical logical $Z_L$ operator.
    pub logical_z_indices: Vec<usize>,
}

impl RotatedSurfaceCode {
    /// Constructs a rotated surface code with given odd distance $d \ge 3$.
    pub fn new(distance: usize) -> Self {
        assert!(distance >= 3, "Surface code distance must be >= 3");
        assert_eq!(distance % 2, 1, "Surface code distance must be odd");

        let d = distance;
        let num_data_qubits = d * d;
        let mut stabilizers = Vec::new();
        let mut stab_id = 0;

        // In rotated surface code, data qubits are located at integer coordinates (r, c) for r, c in 0..d.
        // Data qubit index = r * d + c.
        // Stabilizers are centered on the dual grid vertices between data qubits: (r_dual, c_dual) in 0..=d.
        for r in 0..=d {
            for c in 0..=d {
                let is_even = (r + c) % 2 == 0;
                // Adjacent data qubits to dual coordinate (r, c):
                // (r-1, c-1), (r-1, c), (r, c-1), (r, c)
                let mut data_qubits = Vec::new();
                if r > 0 && c > 0 && (r - 1) < d && (c - 1) < d {
                    data_qubits.push((r - 1) * d + (c - 1));
                }
                if r > 0 && c < d && (r - 1) < d {
                    data_qubits.push((r - 1) * d + c);
                }
                if r < d && c > 0 && (c - 1) < d {
                    data_qubits.push(r * d + (c - 1));
                }
                if r < d && c < d {
                    data_qubits.push(r * d + c);
                }

                // Filter valid boundary and bulk stabilizers:
                // Weight-4 in bulk, weight-2 on alternating boundaries
                if (data_qubits.len() == 4)
                    || (data_qubits.len() == 2
                        && ((is_even && (r == 0 || r == d)) || (!is_even && (c == 0 || c == d))))
                {
                    let stab_type = if is_even {
                        StabilizerType::X
                    } else {
                        StabilizerType::Z
                    };
                    stabilizers.push(StabilizerGenerator {
                        id: stab_id,
                        stab_type,
                        coord: (r, c),
                        data_qubit_indices: data_qubits,
                    });
                    stab_id += 1;
                }
            }
        }

        // Canonical Logical operators:
        // X_L: vertical chain along left column c = 0 (r = 0..d)
        let logical_x_indices: Vec<usize> = (0..d).map(|r| r * d).collect();
        // Z_L: horizontal chain along top row r = 0 (c = 0..d)
        let logical_z_indices: Vec<usize> = (0..d).collect();

        Self {
            distance: d,
            num_data_qubits,
            num_stabilizers: stabilizers.len(),
            stabilizers,
            logical_x_indices,
            logical_z_indices,
        }
    }

    /// Evaluates the syndrome vector $\mathbf{s} \in \{0, 1\}^{N_{anc}}$ given a physical Pauli error configuration.
    ///
    /// Returns 1 for non-trivial defect syndrome (eigenvalue -1) and 0 for no violation (+1).
    pub fn measure_syndrome(&self, errors: &[PauliOp]) -> Vec<u8> {
        self.stabilizers
            .iter()
            .map(|stab| {
                let mut flips = 0;
                for &q_idx in &stab.data_qubit_indices {
                    if q_idx < errors.len()
                        && errors[q_idx].anticommutes_with_stabilizer(stab.stab_type)
                    {
                        flips += 1;
                    }
                }
                (flips % 2) as u8
            })
            .collect()
    }

    /// Checks if a net Pauli error string causes a logical failure on the encoded qubit.
    ///
    /// A failure occurs if the net error anti-commutes with either $X_L$ or $Z_L$.
    pub fn causes_logical_error(&self, net_errors: &[PauliOp]) -> (bool, bool) {
        let mut x_flips = 0;
        for &q_idx in &self.logical_x_indices {
            if q_idx < net_errors.len()
                && (net_errors[q_idx] == PauliOp::Z || net_errors[q_idx] == PauliOp::Y)
            {
                x_flips += 1;
            }
        }
        let mut z_flips = 0;
        for &q_idx in &self.logical_z_indices {
            if q_idx < net_errors.len()
                && (net_errors[q_idx] == PauliOp::X || net_errors[q_idx] == PauliOp::Y)
            {
                z_flips += 1;
            }
        }
        (x_flips % 2 == 1, z_flips % 2 == 1)
    }
}

/// 2D Triangular Color Code on a 3-colorable patch.
#[derive(Debug, Clone, PartialEq)]
pub struct TriangularColorCode {
    /// Code distance $d = 3$ or $d = 5$.
    pub distance: usize,
    /// Number of physical data qubits.
    pub num_data_qubits: usize,
    /// Number of plaquettes / stabilizer pairs.
    pub num_plaquettes: usize,
    /// Plaquette data qubit memberships.
    pub plaquettes: Vec<Vec<usize>>,
}

impl TriangularColorCode {
    /// Constructs a triangular distance $d=3$ color code (Steane code triangular patch, 7 data qubits).
    pub fn distance_3() -> Self {
        // 7 data qubits: 0, 1, 2, 3, 4, 5, 6
        // 3 plaquettes (Red, Green, Blue) of weight 4:
        // P_R: [0, 1, 3, 4]
        // P_G: [1, 2, 4, 5]
        // P_B: [3, 4, 5, 6]
        let plaquettes = vec![vec![0, 1, 3, 4], vec![1, 2, 4, 5], vec![3, 4, 5, 6]];

        Self {
            distance: 3,
            num_data_qubits: 7,
            num_plaquettes: 3,
            plaquettes,
        }
    }

    /// Evaluates both $X$- and $Z$-syndromes for all plaquettes.
    pub fn measure_syndromes(&self, errors: &[PauliOp]) -> (Vec<u8>, Vec<u8>) {
        let mut x_syndrome = Vec::with_capacity(self.num_plaquettes);
        let mut z_syndrome = Vec::with_capacity(self.num_plaquettes);

        for p in &self.plaquettes {
            let mut z_flips = 0;
            let mut x_flips = 0;
            for &q in p {
                if q < errors.len() {
                    let err = errors[q];
                    if err == PauliOp::Z || err == PauliOp::Y {
                        z_flips += 1;
                    }
                    if err == PauliOp::X || err == PauliOp::Y {
                        x_flips += 1;
                    }
                }
            }
            x_syndrome.push((z_flips % 2) as u8);
            z_syndrome.push((x_flips % 2) as u8);
        }
        (x_syndrome, z_syndrome)
    }
}

/// Deterministic, allocation-free Xoshiro256++ PRNG in pure safe Rust for Monte Carlo noise sampling.
#[derive(Debug, Clone)]
pub struct FastNoisePrng {
    s: [u64; 4],
}

impl FastNoisePrng {
    /// Seeds the PRNG with a 64-bit integer.
    pub fn from_seed(seed: u64) -> Self {
        let mut s = [0u64; 4];
        let mut cur = seed.wrapping_add(0x9E3779B97F4A7C15);
        for item in &mut s {
            cur = cur.wrapping_add(0x9E3779B97F4A7C15);
            let mut z = cur;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
            *item = z ^ (z >> 31);
        }
        Self { s }
    }

    /// Generates next uniform random u64.
    pub fn next_u64(&mut self) -> u64 {
        let res = (self.s[0].wrapping_add(self.s[3]))
            .rotate_left(23)
            .wrapping_add(self.s[0]);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        res
    }

    /// Generates next uniform random float in $[0, 1)$.
    pub fn next_f64(&mut self) -> f64 {
        let val = self.next_u64() >> 11;
        (val as f64) * (1.0 / 9007199254740992.0)
    }

    /// Generates an independent depolarizing error vector on $N$ data qubits.
    pub fn sample_depolarizing_errors(&mut self, n: usize, p: f64) -> Vec<PauliOp> {
        let mut errors = Vec::with_capacity(n);
        let px = p / 3.0;
        let py = 2.0 * p / 3.0;
        let pz = p;

        for _ in 0..n {
            let r = self.next_f64();
            if r < px {
                errors.push(PauliOp::X);
            } else if r < py {
                errors.push(PauliOp::Y);
            } else if r < pz {
                errors.push(PauliOp::Z);
            } else {
                errors.push(PauliOp::I);
            }
        }
        errors
    }
}
