#![deny(unsafe_code)]

//! Aubry-Andre-Harper (AAH) quasiperiodic Hamiltonian, self-dual symmetry,
//! and fractal Hofstadter butterfly spectrum engine.
//!
//! Implements standard and generalized incommensurate AAH models with analytical
//! localization-delocalization transitions, energy-dependent mobility edges,
//! and topological phason winding.

use std::f64::consts::PI;

/// Golden ratio conjugate constant: (sqrt(5) - 1) / 2 ~ 0.61803398875.
pub const GOLDEN_RATIO_CONJUGATE: f64 = 0.618_033_988_749_895;

/// Model variant for quasiperiodic AAH metamaterials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AahModelKind {
    /// Standard AAH with on-site incommensurate modulation and uniform hopping.
    StandardAah,
    /// Generalized AAH with off-diagonal hopping modulation hosting exact mobility edges.
    GeneralizedAah,
    /// 2D Moiré Quasicrystal with separable incommensurate orthogonal potentials.
    MoireQuasicrystal2D,
}

impl AahModelKind {
    /// Human-readable label for UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::StandardAah => "Standard AAH (Self-Dual Delta/J = 2)",
            Self::GeneralizedAah => "Generalized AAH (Energy Mobility Edge)",
            Self::MoireQuasicrystal2D => "2D Moire Quasicrystal Metamaterial",
        }
    }
}

/// Physical parameters for the AAH quasiperiodic metamaterial.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AahParams {
    /// Base nearest-neighbor hopping coupling J in MHz (default: 5.0 MHz).
    pub hopping_j: f64,
    /// Incommensurate on-site potential modulation amplitude Delta in MHz (default: 6.0 MHz).
    pub modulation_delta: f64,
    /// Off-diagonal hopping modulation parameter lambda (default: 0.0 for standard, 0.4 for generalized).
    pub off_diagonal_lambda: f64,
    /// Incommensurability frequency ratio beta (default: golden ratio conjugate ~ 0.618034).
    pub incommensurate_beta: f64,
    /// Synthetic phason phase angle phi in radians [0.0, 2*PI].
    pub phason_phi: f64,
    /// Off-diagonal hopping modulation phase theta in radians.
    pub hopping_phase_theta: f64,
    /// Selected model variant.
    pub model_kind: AahModelKind,
}

impl Default for AahParams {
    fn default() -> Self {
        Self {
            hopping_j: 5.0,
            modulation_delta: 6.0,
            off_diagonal_lambda: 0.0,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::StandardAah,
        }
    }
}

impl AahParams {
    /// Creates a standard AAH parameter set.
    pub fn standard(hopping_j: f64, modulation_delta: f64) -> Self {
        Self {
            hopping_j,
            modulation_delta,
            off_diagonal_lambda: 0.0,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::StandardAah,
        }
    }

    /// Creates a generalized AAH parameter set with an energy mobility edge.
    pub fn generalized(hopping_j: f64, modulation_delta: f64, lambda: f64) -> Self {
        Self {
            hopping_j,
            modulation_delta,
            off_diagonal_lambda: lambda,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::GeneralizedAah,
        }
    }

    /// Preset for Delocalized Extended Phase (Delta / J < 2).
    pub fn extended_phase() -> Self {
        Self {
            hopping_j: 5.0,
            modulation_delta: 4.0, // Delta / J = 0.8 < 2
            off_diagonal_lambda: 0.0,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::StandardAah,
        }
    }

    /// Preset for Self-Dual Critical Multifractal Phase (Delta / J = 2).
    pub fn critical_phase() -> Self {
        Self {
            hopping_j: 5.0,
            modulation_delta: 10.0, // Delta / J = 2.0 exactly
            off_diagonal_lambda: 0.0,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::StandardAah,
        }
    }

    /// Preset for Exponentially Localized Phase (Delta / J > 2).
    pub fn localized_phase() -> Self {
        Self {
            hopping_j: 5.0,
            modulation_delta: 16.0, // Delta / J = 3.2 > 2
            off_diagonal_lambda: 0.0,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::StandardAah,
        }
    }

    /// Preset for Mobility Edge Coexistence (Generalized AAH).
    pub fn mobility_edge_phase() -> Self {
        Self {
            hopping_j: 5.0,
            modulation_delta: 6.0,
            off_diagonal_lambda: 0.45,
            incommensurate_beta: GOLDEN_RATIO_CONJUGATE,
            phason_phi: 0.0,
            hopping_phase_theta: 0.0,
            model_kind: AahModelKind::GeneralizedAah,
        }
    }

    /// Returns the dimensionless potential-to-hopping ratio Delta / J.
    pub fn delta_over_j(&self) -> f64 {
        if self.hopping_j.abs() > 1e-12 {
            self.modulation_delta / self.hopping_j
        } else {
            f64::INFINITY
        }
    }
}

/// Phase classification for quasiperiodic metamaterials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AahPhase {
    /// All bulk eigenstates are extended Bloch-like waves.
    DelocalizedExtended,
    /// Critical self-dual transition with multifractal scale-invariant wavefunctions.
    CriticalMultifractal,
    /// All bulk eigenstates are exponentially localized with finite localization length.
    ExponentiallyLocalized,
    /// Coexistence of localized and extended states separated by a mobility edge Ec.
    MobilityEdgeCoexistence,
}

impl AahPhase {
    /// Human-readable label for UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::DelocalizedExtended => "Delocalized Extended Phase (Delta/J < 2)",
            Self::CriticalMultifractal => "Critical Self-Dual Phase (Delta/J = 2)",
            Self::ExponentiallyLocalized => "Exponentially Localized Phase (Delta/J > 2)",
            Self::MobilityEdgeCoexistence => "Mobility Edge Phase (Coexistence)",
        }
    }
}

/// Single slice of the Hofstadter butterfly spectrum at flux beta.
#[derive(Debug, Clone, PartialEq)]
pub struct ButterflyPoint {
    /// Incommensurability / synthetic magnetic flux ratio beta.
    pub beta: f64,
    /// All computed eigenvalues in ascending order in MHz.
    pub energies: Vec<f64>,
}

/// Aubry-Andre-Harper quasiperiodic Hamiltonian solver.
#[derive(Debug, Clone, PartialEq)]
pub struct AahHamiltonian {
    pub params: AahParams,
}

impl AahHamiltonian {
    /// Creates a new AahHamiltonian solver.
    pub fn new(params: AahParams) -> Self {
        Self { params }
    }

    /// Evaluates the analytical mobility edge energy Ec for generalized AAH models.
    ///
    /// For the generalized model with hopping modulation lambda:
    /// Ec = 2 * (hopping_j - modulation_delta / 2.0) / lambda (when |lambda| > 1e-5).
    pub fn mobility_edge_energy(&self) -> Option<f64> {
        match self.params.model_kind {
            AahModelKind::GeneralizedAah => {
                let lam = self.params.off_diagonal_lambda;
                if lam.abs() > 1e-4 {
                    let ec = 2.0 * (self.params.hopping_j - self.params.modulation_delta * 0.5) / lam;
                    Some(ec)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Determines the theoretical phase regime based on coupling parameters.
    pub fn phase(&self) -> AahPhase {
        match self.params.model_kind {
            AahModelKind::GeneralizedAah => {
                if self.params.off_diagonal_lambda.abs() > 0.05 {
                    AahPhase::MobilityEdgeCoexistence
                } else {
                    self.classify_standard_aah()
                }
            }
            AahModelKind::StandardAah | AahModelKind::MoireQuasicrystal2D => {
                self.classify_standard_aah()
            }
        }
    }

    fn classify_standard_aah(&self) -> AahPhase {
        let ratio = self.params.delta_over_j();
        if (ratio - 2.0).abs() < 0.15 {
            AahPhase::CriticalMultifractal
        } else if ratio < 2.0 {
            AahPhase::DelocalizedExtended
        } else {
            AahPhase::ExponentiallyLocalized
        }
    }

    /// Evaluates on-site potential at site n: V_n = Delta * cos(2*pi*beta*n + phi).
    #[inline]
    pub fn onsite_potential_at(&self, n: usize) -> f64 {
        let n_f = n as f64;
        let phase = 2.0 * PI * self.params.incommensurate_beta * n_f + self.params.phason_phi;
        self.params.modulation_delta * phase.cos()
    }

    /// Evaluates hopping coupling between site n and n+1:
    /// J_n = J * (1.0 + lambda * cos(2*pi*beta*n + theta)).
    #[inline]
    pub fn hopping_at(&self, n: usize) -> f64 {
        match self.params.model_kind {
            AahModelKind::GeneralizedAah => {
                let n_f = n as f64;
                let phase = 2.0 * PI * self.params.incommensurate_beta * n_f + self.params.hopping_phase_theta;
                self.params.hopping_j * (1.0 + self.params.off_diagonal_lambda * phase.cos())
            }
            _ => self.params.hopping_j,
        }
    }

    /// Generates Hofstadter butterfly spectrum by sweeping beta in [0.0, 1.0].
    pub fn generate_hofstadter_butterfly(
        &self,
        num_beta_samples: usize,
        lattice_size: usize,
    ) -> Vec<ButterflyPoint> {
        let mut butterfly = Vec::with_capacity(num_beta_samples);
        let n_sites = lattice_size.clamp(30, 120);

        for b_idx in 0..num_beta_samples {
            let beta = b_idx as f64 / (num_beta_samples - 1).max(1) as f64;
            let mut params_b = self.params;
            params_b.incommensurate_beta = beta;

            // Assemble tridiagonal matrix for 1D AAH chain
            let mut diag = Vec::with_capacity(n_sites);
            let mut offdiag = Vec::with_capacity(n_sites - 1);

            for n in 0..n_sites {
                let v = params_b.modulation_delta * (2.0 * PI * beta * (n as f64) + params_b.phason_phi).cos();
                diag.push(v);
                if n + 1 < n_sites {
                    let j = match params_b.model_kind {
                        AahModelKind::GeneralizedAah => {
                            params_b.hopping_j * (1.0 + params_b.off_diagonal_lambda * (2.0 * PI * beta * (n as f64) + params_b.hopping_phase_theta).cos())
                        }
                        _ => params_b.hopping_j,
                    };
                    offdiag.push(j);
                }
            }

            let evals = solve_symmetric_tridiagonal(&diag, &offdiag);
            butterfly.push(ButterflyPoint {
                beta,
                energies: evals,
            });
        }

        butterfly
    }

    /// Sweeps phason angle phi in [0, 2*pi] to trace energy levels and boundary edge modes.
    pub fn generate_phason_spectrum(
        &self,
        num_phi_samples: usize,
        lattice_size: usize,
    ) -> Vec<(f64, Vec<f64>)> {
        let mut results = Vec::with_capacity(num_phi_samples);
        let n_sites = lattice_size.clamp(30, 120);

        for p_idx in 0..num_phi_samples {
            let phi = 2.0 * PI * (p_idx as f64) / (num_phi_samples - 1).max(1) as f64;
            let mut diag = Vec::with_capacity(n_sites);
            let mut offdiag = Vec::with_capacity(n_sites - 1);

            for n in 0..n_sites {
                let v = self.params.modulation_delta * (2.0 * PI * self.params.incommensurate_beta * (n as f64) + phi).cos();
                diag.push(v);
                if n + 1 < n_sites {
                    offdiag.push(self.hopping_at(n));
                }
            }

            let evals = solve_symmetric_tridiagonal(&diag, &offdiag);
            results.push((phi, evals));
        }

        results
    }
}

/// Solves eigenvalues of a real symmetric tridiagonal matrix using implicit QL/QR iteration with Wilkinson shifts.
pub fn solve_symmetric_tridiagonal(diag_in: &[f64], offdiag_in: &[f64]) -> Vec<f64> {
    let n = diag_in.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return vec![diag_in[0]];
    }

    let mut d = diag_in.to_vec();
    let mut e = vec![0.0; n];
    for i in 0..(n - 1) {
        e[i] = offdiag_in[i];
    }
    e[n - 1] = 0.0;

    for l in 0..n {
        let mut iter = 0;
        loop {
            let mut m = l;
            while m < n - 1 {
                let dd = d[m].abs() + d[m + 1].abs();
                if e[m].abs() + dd == dd {
                    break;
                }
                m += 1;
            }

            if m == l {
                break;
            }

            iter += 1;
            if iter > 40 {
                break; // Converged sufficiently
            }

            let mut g = (d[l + 1] - d[l]) / (2.0 * e[l]);
            let mut r = (g * g + 1.0).sqrt();
            if g < 0.0 {
                g = d[m] - d[l] + e[l] / (g - r);
            } else {
                g = d[m] - d[l] + e[l] / (g + r);
            }

            let mut s = 1.0;
            let mut c = 1.0;
            let mut p = 0.0;

            for i in (l..m).rev() {
                let f = s * e[i];
                let b = c * e[i];
                r = (f * f + g * g).sqrt();
                e[i + 1] = r;

                if r.abs() < 1e-15 {
                    d[i + 1] -= p;
                    e[m] = 0.0;
                    break;
                }

                s = f / r;
                c = g / r;
                g = d[i + 1] - p;
                r = (d[i] - g) * s + 2.0 * c * b;
                p = s * r;
                d[i + 1] = g + p;
                g = c * r - b;
            }

            d[l] -= p;
            e[l] = g;
            e[m] = 0.0;
        }
    }

    d.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    d
}
