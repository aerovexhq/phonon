#![deny(unsafe_code)]

//! Parity-Time (PT) symmetric non-Hermitian acoustic Hamiltonian and exceptional point solver.
//!
//! Models a coupled pair of acoustic resonators with balanced gain (+gamma) and loss (-gamma)
//! coupled by acoustic tunneling rate kappa. Computes eigenvalue bifurcation across the
//! exceptional point (EP) and eigenvector non-orthogonality (Petermann factor).

/// Parameters defining the 2-level PT-symmetric acoustic resonator system.
#[derive(Debug, Clone)]
pub struct PtAcousticParams {
    /// Bare acoustic resonance frequency in Hz (default ~3000 Hz).
    pub resonance_freq_hz: f64,
    /// Inter-cavity acoustic coupling tunneling rate kappa in Hz (default ~500 Hz).
    pub coupling_kappa_hz: f64,
    /// Balanced gain and loss rate gamma in Hz (default ~500 Hz, EP at gamma = kappa).
    pub gain_loss_gamma_hz: f64,
    /// Intrinsic background dissipative loss in Hz (default ~5 Hz).
    pub background_loss_hz: f64,
}

impl Default for PtAcousticParams {
    fn default() -> Self {
        Self {
            resonance_freq_hz: 3000.0,
            coupling_kappa_hz: 500.0,
            gain_loss_gamma_hz: 500.0,
            background_loss_hz: 5.0,
        }
    }
}

impl PtAcousticParams {
    /// Dimensionless non-Hermiticity ratio g = gamma / kappa.
    #[inline]
    pub fn non_hermiticity_ratio(&self) -> f64 {
        if self.coupling_kappa_hz.abs() < 1e-9 {
            1e6
        } else {
            self.gain_loss_gamma_hz / self.coupling_kappa_hz
        }
    }

    /// Classification of the non-Hermitian PT phase.
    #[inline]
    pub fn phase(&self) -> PtPhaseClassification {
        let g = self.non_hermiticity_ratio();
        let tol = 0.02;
        if (g - 1.0).abs() <= tol {
            PtPhaseClassification::ExceptionalPoint
        } else if g < 1.0 {
            PtPhaseClassification::ExactPtSymmetric
        } else {
            PtPhaseClassification::BrokenPtSymmetric
        }
    }
}

/// Classification of the PT-symmetric non-Hermitian phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtPhaseClassification {
    /// Exact PT-symmetric phase: real eigenvalues (gamma < kappa).
    ExactPtSymmetric,
    /// Exceptional point (EP2) singularity: coalesced eigenvalues and eigenvectors (gamma = kappa).
    ExceptionalPoint,
    /// Broken PT-symmetric phase: complex conjugate eigenvalues (gamma > kappa).
    BrokenPtSymmetric,
}

/// Complex eigenvalue of the non-Hermitian acoustic Hamiltonian.
#[derive(Debug, Clone, Copy)]
pub struct PtEigenvalue {
    /// Real frequency offset in Hz: Re(lambda - omega_0).
    pub real_offset_hz: f64,
    /// Imaginary dissipation / amplification rate in Hz: Im(lambda).
    pub imag_rate_hz: f64,
}

/// Modal metrics for the PT-symmetric acoustic system.
#[derive(Debug, Clone)]
pub struct PtModalMetrics {
    /// First complex eigenvalue lambda_+.
    pub eigenvalue_plus: PtEigenvalue,
    /// Second complex eigenvalue lambda_-.
    pub eigenvalue_minus: PtEigenvalue,
    /// Frequency splitting Delta f = |Re(lambda_+) - Re(lambda_-)| in Hz.
    pub frequency_splitting_hz: f64,
    /// Decay/amplification splitting Delta gamma = |Im(lambda_+) - Im(lambda_-)| in Hz.
    pub decay_splitting_hz: f64,
    /// Petermann factor K quantifying eigenvector non-orthogonality (diverges at EP).
    pub petermann_factor: f64,
}

/// Solver for the 2-level PT-symmetric acoustic Hamiltonian.
#[derive(Debug, Clone)]
pub struct PtHamiltonianSolver {
    pub params: PtAcousticParams,
    pub metrics: PtModalMetrics,
    /// Eigenvalue bifurcation trajectory across gamma/kappa sweep.
    pub bifurcation_curve: Vec<(f64, f64, f64)>, // (gamma/kappa, Re(delta_lambda), Im(delta_lambda))
}

impl PtHamiltonianSolver {
    /// Construct a new PT Hamiltonian solver and compute initial eigenvalues.
    pub fn new(params: PtAcousticParams) -> Self {
        let mut solver = Self {
            params,
            metrics: PtModalMetrics {
                eigenvalue_plus: PtEigenvalue { real_offset_hz: 0.0, imag_rate_hz: 0.0 },
                eigenvalue_minus: PtEigenvalue { real_offset_hz: 0.0, imag_rate_hz: 0.0 },
                frequency_splitting_hz: 0.0,
                decay_splitting_hz: 0.0,
                petermann_factor: 1.0,
            },
            bifurcation_curve: Vec::new(),
        };
        solver.recompute();
        solver
    }

    /// Recompute eigenvalues, Petermann factor, and bifurcation curve.
    pub fn recompute(&mut self) {
        let kappa = self.params.coupling_kappa_hz;
        let gamma = self.params.gain_loss_gamma_hz;
        let bg_loss = self.params.background_loss_hz;

        // Effective 2x2 Hamiltonian:
        // H = [[omega_0 + i*gamma - i*bg_loss, kappa],
        //      [kappa, omega_0 - i*gamma - i*bg_loss]]
        // Eigenvalues: lambda_pm = omega_0 - i*bg_loss pm sqrt(kappa^2 - gamma^2)
        let discriminant = kappa * kappa - gamma * gamma;

        let (e_plus, e_minus, petermann) = if discriminant >= 0.0 {
            // Exact PT phase: real splitting, identical imaginary damping
            let delta = discriminant.sqrt();
            let p_plus = PtEigenvalue {
                real_offset_hz: delta,
                imag_rate_hz: -bg_loss,
            };
            let p_minus = PtEigenvalue {
                real_offset_hz: -delta,
                imag_rate_hz: -bg_loss,
            };

            // Petermann factor: K = 1 / (1 - (gamma/kappa)^2)
            let g = (gamma / kappa.max(1e-6)).clamp(0.0, 0.999);
            let k_factor = (1.0 / (1.0 - g * g)).min(1e4);
            (p_plus, p_minus, k_factor)
        } else {
            // Broken PT phase: degenerate real frequencies, imaginary bifurcation
            let delta = (-discriminant).sqrt();
            let p_plus = PtEigenvalue {
                real_offset_hz: 0.0,
                imag_rate_hz: delta - bg_loss,
            };
            let p_minus = PtEigenvalue {
                real_offset_hz: 0.0,
                imag_rate_hz: -delta - bg_loss,
            };

            // In broken phase, Petermann factor decreases from EP peak
            let g = gamma / kappa.max(1e-6);
            let k_factor = (g * g / (g * g - 1.0)).min(1e4);
            (p_plus, p_minus, k_factor)
        };

        let freq_split = (e_plus.real_offset_hz - e_minus.real_offset_hz).abs();
        let decay_split = (e_plus.imag_rate_hz - e_minus.imag_rate_hz).abs();

        self.metrics = PtModalMetrics {
            eigenvalue_plus: e_plus,
            eigenvalue_minus: e_minus,
            frequency_splitting_hz: freq_split,
            decay_splitting_hz: decay_split,
            petermann_factor: petermann,
        };

        // Generate bifurcation curve across gamma/kappa in [0.0, 2.0]
        let num_pts = 80;
        let mut curve = Vec::with_capacity(num_pts);
        for i in 0..num_pts {
            let g = 2.0 * (i as f64) / (num_pts - 1) as f64;
            let disc = 1.0 - g * g;
            let (re, im) = if disc >= 0.0 {
                (disc.sqrt() * kappa, 0.0)
            } else {
                (0.0, (-disc).sqrt() * kappa)
            };
            curve.push((g, re, im));
        }
        self.bifurcation_curve = curve;
    }
}
