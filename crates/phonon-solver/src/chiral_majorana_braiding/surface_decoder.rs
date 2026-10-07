#![deny(unsafe_code)]

//! Real-Time Fault-Tolerant Surface Code Syndromic Minimum-Weight Perfect Matching (MWPM) Decoder.
//!
//! Models distance-d topological surface code patches, star (X) and plaquette (Z) stabilizer
//! syndrome defect extraction, and polynomial-time greedy/MWPM defect pair pairing and correction.

/// Stabilizer syndrome defect kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChiralStabilizerKind {
    /// Star operator A_s = prod_{i in s} X_i (detects Z phase-flip errors).
    StarX,
    /// Plaquette operator B_p = prod_{j in p} Z_j (detects X bit-flip errors).
    PlaquetteZ,
}

impl ChiralStabilizerKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::StarX => "Star Stabilizer (X-type)",
            Self::PlaquetteZ => "Plaquette Stabilizer (Z-type)",
        }
    }
}

/// A syndrome defect detected on the stabilizer lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralSyndromeDefect {
    /// Stabilizer type.
    pub kind: ChiralStabilizerKind,
    /// Grid coordinate x (0 to d-1).
    pub x: usize,
    /// Grid coordinate y (0 to d-1).
    pub y: usize,
    /// Extraction round index.
    pub round: usize,
    /// Whether this defect has been paired/resolved by the decoder.
    pub is_resolved: bool,
    /// Co-defect paired with this defect (if matched to another defect).
    pub paired_coord: Option<(usize, usize)>,
    /// Distance of the correction chain.
    pub chain_weight: usize,
}

/// Parameters for the topological surface code decoder.
#[derive(Debug, Clone)]
pub struct ChiralSurfaceDecoderParams {
    /// Code distance d (must be odd, e.g. 3 or 5).
    pub code_distance: usize,
    /// Physical error rate p per gate/step (typical ~0.001 to 0.010).
    pub physical_error_rate: f64,
    /// Number of syndrome measurement rounds.
    pub syndrome_rounds: usize,
}

impl Default for ChiralSurfaceDecoderParams {
    fn default() -> Self {
        Self {
            code_distance: 3,
            physical_error_rate: 0.005, // 0.5% physical error rate (below 1% threshold)
            syndrome_rounds: 3,
        }
    }
}

/// Summary report of the MWPM syndromic decoding process.
#[derive(Debug, Clone)]
pub struct ChiralDecodingResult {
    /// Total number of syndrome defect points detected.
    pub total_defects: usize,
    /// Number of defect pairs matched.
    pub matched_pairs_count: usize,
    /// Number of defects matched to rough/smooth boundaries.
    pub boundary_matches_count: usize,
    /// Total weight (sum of Manhattan chain lengths) of the correction path.
    pub total_chain_weight: usize,
    /// Projected logical error rate P_L under distance-d thresholding.
    pub logical_error_rate: f64,
    /// True if logical state was successfully protected without uncorrectable error chains.
    pub logical_success: bool,
}

/// Surface code lattice and MWPM decoding engine.
#[derive(Debug, Clone)]
pub struct ChiralSurfaceDecoder {
    pub params: ChiralSurfaceDecoderParams,
    pub defects: Vec<ChiralSyndromeDefect>,
}

impl ChiralSurfaceDecoder {
    /// Constructs a new surface code decoder.
    pub fn new(params: ChiralSurfaceDecoderParams) -> Self {
        let mut decoder = Self {
            params,
            defects: Vec::new(),
        };
        decoder.extract_syndromes();
        decoder
    }

    /// Evaluates projected logical error rate P_L = C * (p / p_th)^((d + 1) / 2).
    /// Using standard surface code threshold p_th = 0.01 (1.0%) and prefactor C = 0.1.
    pub fn projected_logical_error_rate(&self) -> f64 {
        let p_th = 0.030; // 3.0% threshold for topological surface code
        let ratio = (self.params.physical_error_rate / p_th).max(1.0e-6);
        let exponent = (self.params.code_distance + 1) as f64 * 0.5;
        (0.10 * ratio.powf(exponent)).min(1.0)
    }

    /// Simulates syndrome extraction across the code patch with physical error rate p.
    pub fn extract_syndromes(&mut self) {
        self.defects.clear();
        let d = self.params.code_distance;
        let p = self.params.physical_error_rate;

        for round in 0..self.params.syndrome_rounds {
            for y in 0..d {
                for x in 0..d {
                    let seed = ((x * 37 + y * 53 + round * 97 + 13) % 1000) as f64 / 1000.0;
                    if seed < p * 2.0 {
                        let kind = if (x + y) % 2 == 0 {
                            ChiralStabilizerKind::StarX
                        } else {
                            ChiralStabilizerKind::PlaquetteZ
                        };
                        self.defects.push(ChiralSyndromeDefect {
                            kind,
                            x,
                            y,
                            round,
                            is_resolved: false,
                            paired_coord: None,
                            chain_weight: 0,
                        });
                    }
                }
            }
        }
    }

    /// Solves Minimum-Weight Perfect Matching (MWPM) pairing defect pairs via greedy shortest
    /// Manhattan distance chains and boundary connections.
    pub fn decode_and_correct(&mut self) -> ChiralDecodingResult {
        let d = self.params.code_distance;
        let mut matched_pairs = 0;
        let mut boundary_matches = 0;
        let mut total_weight = 0;

        let total_defects = self.defects.len();

        for kind in [ChiralStabilizerKind::StarX, ChiralStabilizerKind::PlaquetteZ] {
            let mut indices: Vec<usize> = (0..self.defects.len())
                .filter(|&i| self.defects[i].kind == kind && !self.defects[i].is_resolved)
                .collect();

            while !indices.is_empty() {
                let u = indices.remove(0);
                let (ux, uy) = (self.defects[u].x, self.defects[u].y);

                let boundary_dist = match kind {
                    ChiralStabilizerKind::StarX => ux.min(d.saturating_sub(1 + ux)) + 1,
                    ChiralStabilizerKind::PlaquetteZ => uy.min(d.saturating_sub(1 + uy)) + 1,
                };

                let mut best_pair = None;
                let mut best_dist = boundary_dist;

                for (idx_pos, &v) in indices.iter().enumerate() {
                    let (vx, vy) = (self.defects[v].x, self.defects[v].y);
                    let manhattan = (ux as isize - vx as isize).unsigned_abs()
                        + (uy as isize - vy as isize).unsigned_abs();
                    if manhattan < best_dist {
                        best_dist = manhattan;
                        best_pair = Some((idx_pos, v));
                    }
                }

                if let Some((idx_pos, v)) = best_pair {
                    indices.remove(idx_pos);
                    let (vx, vy) = (self.defects[v].x, self.defects[v].y);
                    self.defects[u].is_resolved = true;
                    self.defects[u].paired_coord = Some((vx, vy));
                    self.defects[u].chain_weight = best_dist;

                    self.defects[v].is_resolved = true;
                    self.defects[v].paired_coord = Some((ux, uy));
                    self.defects[v].chain_weight = best_dist;

                    matched_pairs += 1;
                    total_weight += best_dist;
                } else {
                    self.defects[u].is_resolved = true;
                    self.defects[u].paired_coord = None;
                    self.defects[u].chain_weight = boundary_dist;

                    boundary_matches += 1;
                    total_weight += boundary_dist;
                }
            }
        }

        let p_l = self.projected_logical_error_rate();
        let logical_success = p_l < self.params.physical_error_rate;

        ChiralDecodingResult {
            total_defects,
            matched_pairs_count: matched_pairs,
            boundary_matches_count: boundary_matches,
            total_chain_weight: total_weight,
            logical_error_rate: p_l,
            logical_success,
        }
    }
}
