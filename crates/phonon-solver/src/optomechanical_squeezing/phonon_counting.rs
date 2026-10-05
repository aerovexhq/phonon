#![deny(unsafe_code)]

//! Non-Classical Phonon Number Counting, Dispersive Cavity Spectroscopy & Fock Distributions.
//!
//! Provides:
//! - Phonon state classification (Fock |1>, Squeezed Vacuum, Ground State, Thermal, Coherent).
//! - Exact phonon number state distributions P(n) for n = 0..12.
//! - Strongly dispersive cavity number-resolved transmission / reflection spectrum S_21(omega).
//! - Non-classicality metrics: g^(2)(0) correlation, quantum purity Tr(rho^2), and sideband cooling limit.

use std::fmt;

/// Quantum phonon state classification for optomechanical resonators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PhononStateKind {
    #[default]
    GroundState,
    SinglePhononFock1,
    SqueezedVacuum,
    ThermalState,
    CoherentState,
}

impl PhononStateKind {
    /// Human-readable label for state selector in UI.
    pub fn label(&self) -> &'static str {
        match self {
            Self::GroundState => "Ground State |0>",
            Self::SinglePhononFock1 => "Single Phonon Fock State |1>",
            Self::SqueezedVacuum => "Quadrature Squeezed Vacuum",
            Self::ThermalState => "Thermal Phonon Bath",
            Self::CoherentState => "Coherent State |alpha>",
        }
    }

    /// Descriptive summary of the state physics.
    pub fn description(&self) -> &'static str {
        match self {
            Self::GroundState => "Pure vacuum ground state with zero average phonon occupancy.",
            Self::SinglePhononFock1 => {
                "Non-classical single-phonon Fock state with strict sub-Poissonian statistics and negative Wigner core."
            }
            Self::SqueezedVacuum => {
                "Continuous-variable non-classical Gaussian state with strictly even phonon parity and sub-SQL quadrature variance."
            }
            Self::ThermalState => {
                "Classical thermal Gaussian state governed by Bose-Einstein super-Poissonian statistics."
            }
            Self::CoherentState => {
                "Minimum-uncertainty classical coherent displacement state with Poissonian phonon counting statistics."
            }
        }
    }

    /// Returns true if the state exhibits non-classical quantum properties (sub-Poissonian or Wigner negative).
    pub fn is_nonclassical(&self) -> bool {
        matches!(self, Self::SinglePhononFock1 | Self::SqueezedVacuum)
    }
}

impl fmt::Display for PhononStateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// Phonon number state probability distribution P(n) evaluated up to maximum cutoff n_max.
#[derive(Debug, Clone, PartialEq)]
pub struct FockStateDistribution {
    /// Quantum state kind.
    pub state_kind: PhononStateKind,
    /// Maximum Fock index evaluated (typically 12).
    pub max_n: usize,
    /// Number state probabilities P(n) for n = 0..=max_n.
    pub probabilities: Vec<f64>,
    /// Mean phonon occupancy: sum_n n * P(n).
    pub mean_phonon_number: f64,
    /// Phonon number variance: sum_n (n - n_bar)^2 * P(n).
    pub variance: f64,
}

impl FockStateDistribution {
    /// Generates Fock state distribution for Single Phonon Fock State |1>:
    /// P(1) = 1.0, P(n != 1) = 0.0.
    pub fn for_single_fock1(max_n: usize) -> Self {
        let n_cut = max_n.max(1);
        let mut probs = vec![0.0; n_cut + 1];
        probs[1] = 1.0;

        Self {
            state_kind: PhononStateKind::SinglePhononFock1,
            max_n: n_cut,
            probabilities: probs,
            mean_phonon_number: 1.0,
            variance: 0.0,
        }
    }

    /// Generates Fock state distribution for Ground State |0>:
    /// P(0) = 1.0, P(n > 0) = 0.0.
    pub fn for_ground_state(max_n: usize) -> Self {
        let n_cut = max_n.max(1);
        let mut probs = vec![0.0; n_cut + 1];
        probs[0] = 1.0;

        Self {
            state_kind: PhononStateKind::GroundState,
            max_n: n_cut,
            probabilities: probs,
            mean_phonon_number: 0.0,
            variance: 0.0,
        }
    }

    /// Generates Fock state distribution for Squeezed Vacuum with squeezing parameter r:
    /// Non-zero only for even n = 2k:
    /// P(2k) = (2k)! / (2^{2k} * (k!)^2) * tanh^{2k}(r) / cosh(r).
    /// For odd n: P(2k+1) = 0.0.
    pub fn for_squeezed_vacuum(r: f64, max_n: usize) -> Self {
        let n_cut = max_n.max(2);
        let mut probs = vec![0.0; n_cut + 1];

        if r <= 1e-12 {
            probs[0] = 1.0;
            return Self {
                state_kind: PhononStateKind::SqueezedVacuum,
                max_n: n_cut,
                probabilities: probs,
                mean_phonon_number: 0.0,
                variance: 0.0,
            };
        }

        let cosh_r = r.cosh();
        let tanh_sq = r.tanh().powi(2);

        // Base case: P(0) = 1 / cosh(r)
        let mut p_even = 1.0 / cosh_r;
        probs[0] = p_even;

        // Recurrence for even terms: P(2k) = P(2k-2) * ((2k - 1) / (2k)) * tanh^2(r)
        for k in 1..=(n_cut / 2) {
            let n = 2 * k;
            let ratio = ((2 * k - 1) as f64 / (2 * k) as f64) * tanh_sq;
            p_even *= ratio;
            probs[n] = p_even;
            // Odd numbers remain strictly 0.0
        }

        let mut mean = 0.0;
        let mut var_acc = 0.0;
        for (n, &p) in probs.iter().enumerate() {
            mean += n as f64 * p;
        }
        for (n, &p) in probs.iter().enumerate() {
            let diff = n as f64 - mean;
            var_acc += diff * diff * p;
        }

        Self {
            state_kind: PhononStateKind::SqueezedVacuum,
            max_n: n_cut,
            probabilities: probs,
            mean_phonon_number: mean,
            variance: var_acc,
        }
    }

    /// Generates Fock state distribution for Thermal State:
    /// P_th(n) = n_bar^n / (1 + n_bar)^{n+1}.
    pub fn for_thermal_state(n_bar: f64, max_n: usize) -> Self {
        let n_cut = max_n.max(1);
        let mut probs = vec![0.0; n_cut + 1];

        if n_bar <= 1e-12 {
            probs[0] = 1.0;
            return Self {
                state_kind: PhononStateKind::ThermalState,
                max_n: n_cut,
                probabilities: probs,
                mean_phonon_number: 0.0,
                variance: 0.0,
            };
        }

        let p0 = 1.0 / (1.0 + n_bar);
        let ratio = n_bar / (1.0 + n_bar);

        let mut curr_p = p0;
        probs[0] = curr_p;
        for n in 1..=n_cut {
            curr_p *= ratio;
            probs[n] = curr_p;
        }

        let mut mean = 0.0;
        let mut var_acc = 0.0;
        for (n, &p) in probs.iter().enumerate() {
            mean += n as f64 * p;
        }
        for (n, &p) in probs.iter().enumerate() {
            let diff = n as f64 - mean;
            var_acc += diff * diff * p;
        }

        Self {
            state_kind: PhononStateKind::ThermalState,
            max_n: n_cut,
            probabilities: probs,
            mean_phonon_number: mean,
            variance: var_acc,
        }
    }

    /// Generates Fock state distribution for Coherent State:
    /// P(n) = exp(-|alpha|^2) * |alpha|^{2n} / n!.
    pub fn for_coherent_state(alpha: f64, max_n: usize) -> Self {
        let n_cut = max_n.max(1);
        let mut probs = vec![0.0; n_cut + 1];

        let n_bar = alpha * alpha;
        if n_bar <= 1e-12 {
            probs[0] = 1.0;
            return Self {
                state_kind: PhononStateKind::CoherentState,
                max_n: n_cut,
                probabilities: probs,
                mean_phonon_number: 0.0,
                variance: 0.0,
            };
        }

        let mut curr_p = (-n_bar).exp();
        probs[0] = curr_p;
        for n in 1..=n_cut {
            curr_p *= n_bar / (n as f64);
            probs[n] = curr_p;
        }

        let mut mean = 0.0;
        let mut var_acc = 0.0;
        for (n, &p) in probs.iter().enumerate() {
            mean += n as f64 * p;
        }
        for (n, &p) in probs.iter().enumerate() {
            let diff = n as f64 - mean;
            var_acc += diff * diff * p;
        }

        Self {
            state_kind: PhononStateKind::CoherentState,
            max_n: n_cut,
            probabilities: probs,
            mean_phonon_number: mean,
            variance: var_acc,
        }
    }

    /// Generates distribution for arbitrary state kind with relevant parameterization.
    pub fn for_state(
        kind: PhononStateKind,
        squeezing_r: f64,
        n_th: f64,
        alpha: f64,
        max_n: usize,
    ) -> Self {
        match kind {
            PhononStateKind::GroundState => Self::for_ground_state(max_n),
            PhononStateKind::SinglePhononFock1 => Self::for_single_fock1(max_n),
            PhononStateKind::SqueezedVacuum => Self::for_squeezed_vacuum(squeezing_r, max_n),
            PhononStateKind::ThermalState => Self::for_thermal_state(n_th, max_n),
            PhononStateKind::CoherentState => Self::for_coherent_state(alpha, max_n),
        }
    }

    /// Returns probability for specific Fock state n.
    pub fn probability(&self, n: usize) -> f64 {
        if n < self.probabilities.len() {
            self.probabilities[n]
        } else {
            0.0
        }
    }

    /// Computes parity expectation value <(-1)^n> = sum_n (-1)^n * P(n) / sum_n P(n).
    pub fn parity(&self) -> f64 {
        let total: f64 = self.probabilities.iter().sum();
        if total <= 1e-12 {
            return 1.0;
        }
        let alt_sum: f64 = self
            .probabilities
            .iter()
            .enumerate()
            .map(|(n, &p)| if n % 2 == 0 { p } else { -p })
            .sum();
        alt_sum / total
    }
}

/// Resolved spectral peak of dispersive optomechanical cavity.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPeak {
    /// Phonon number state n.
    pub phonon_number: usize,
    /// Absolute cavity frequency in GHz for this number state: omega_c - 2 * n * chi.
    pub frequency_ghz: f64,
    /// Frequency detuning relative to bare cavity in MHz: -2 * n * chi.
    pub detuning_mhz: f64,
    /// Number state probability P(n).
    pub probability: f64,
    /// Peak transmission amplitude.
    pub peak_amplitude: f64,
}

/// Strong dispersive optomechanical number-resolved spectrum S_21(omega).
#[derive(Debug, Clone, PartialEq)]
pub struct PhononCountingResolvedSpectrum {
    /// Cavity frequency in GHz.
    pub cavity_freq_ghz: f64,
    /// Dispersive shift rate chi in MHz.
    pub dispersive_shift_chi_mhz: f64,
    /// Cavity linewidth kappa in MHz.
    pub cavity_linewidth_kappa_mhz: f64,
    /// Resolved individual number state peaks.
    pub peaks: Vec<ResolvedPeak>,
    /// Detuning axis values in MHz.
    pub detunings_mhz: Vec<f64>,
    /// Optical transmission spectrum intensity curve T(omega).
    pub spectrum_intensity: Vec<f64>,
    /// Transmission spectrum in dB.
    pub spectrum_db: Vec<f64>,
    /// True if dispersive shift strongly resolves peaks (2 * chi >> kappa).
    pub is_resolved: bool,
}

impl PhononCountingResolvedSpectrum {
    /// Synthesizes dispersive cavity spectrum weighted by phonon distribution P(n).
    /// Peaks appear at omega_c - 2 * n * chi.
    pub fn compute(
        cavity_freq_ghz: f64,
        dispersive_shift_chi_mhz: f64,
        cavity_linewidth_kappa_mhz: f64,
        distribution: &FockStateDistribution,
        num_frequency_points: usize,
    ) -> Self {
        let chi = dispersive_shift_chi_mhz.max(0.1);
        let kappa = cavity_linewidth_kappa_mhz.max(0.01);
        let is_resolved = (2.0 * chi) >= kappa;

        let max_n = distribution.probabilities.len().min(12);
        let mut peaks = Vec::with_capacity(max_n);

        for n in 0..max_n {
            let p_n = distribution.probability(n);
            let detuning_mhz = -2.0 * (n as f64) * chi;
            let freq_ghz = cavity_freq_ghz + detuning_mhz * 1e-3;
            peaks.push(ResolvedPeak {
                phonon_number: n,
                frequency_ghz: freq_ghz,
                detuning_mhz,
                probability: p_n,
                peak_amplitude: p_n,
            });
        }

        // Frequency sweep range covering all active number states
        let min_detuning = -2.0 * (max_n as f64) * chi - 2.0 * kappa;
        let max_detuning = 2.0 * chi + 2.0 * kappa;
        let n_pts = num_frequency_points.max(100);
        let step = (max_detuning - min_detuning) / (n_pts - 1) as f64;

        let mut detunings_mhz = Vec::with_capacity(n_pts);
        let mut spectrum_intensity = Vec::with_capacity(n_pts);
        let mut spectrum_db = Vec::with_capacity(n_pts);

        let half_kappa_sq = (0.5 * kappa).powi(2);

        for i in 0..n_pts {
            let delta = min_detuning + i as f64 * step;
            detunings_mhz.push(delta);

            // Multimode Lorentzian transmission superposition: sum_n P(n) * (kappa/2)^2 / ((delta - delta_n)^2 + (kappa/2)^2)
            let mut intensity = 0.0;
            for peak in &peaks {
                if peak.probability > 1e-6 {
                    let d = delta - peak.detuning_mhz;
                    let lorentzian = half_kappa_sq / (d * d + half_kappa_sq);
                    intensity += peak.probability * lorentzian;
                }
            }

            spectrum_intensity.push(intensity);
            let db = 10.0 * (intensity.max(1e-12)).log10();
            spectrum_db.push(db);
        }

        Self {
            cavity_freq_ghz,
            dispersive_shift_chi_mhz: chi,
            cavity_linewidth_kappa_mhz: kappa,
            peaks,
            detunings_mhz,
            spectrum_intensity,
            spectrum_db,
            is_resolved,
        }
    }
}

/// Non-classicality metrics, correlation statistics, and cooling limits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonClassicalityMetrics {
    /// Mean phonon number n_bar = sum_n n * P(n).
    pub mean_phonon_number: f64,
    /// Second-order phonon auto-correlation function g^(2)(0).
    pub second_order_correlation_g2: f64,
    /// Quantum state purity Tr(rho^2) in [0.0, 1.0].
    pub quantum_purity: f64,
    /// Final sideband cooling occupancy n_final.
    pub sideband_cooled_n_final: f64,
    /// True if state exhibits non-classical sub-Poissonian statistics (g^(2)(0) < 1.0).
    pub is_sub_poissonian: bool,
    /// True if quantum state is strictly pure (Tr(rho^2) >= 0.999).
    pub is_quantum_pure: bool,
}

impl NonClassicalityMetrics {
    /// Computes full set of non-classical metrics from phonon distribution and optomechanical parameters.
    pub fn evaluate(
        dist: &FockStateDistribution,
        gamma_m_hz: f64,
        n_th: f64,
        gamma_opt_hz: f64,
        kappa_mhz: f64,
        omega_m_mhz: f64,
    ) -> Self {
        let n_bar = dist.mean_phonon_number;

        // Second-order correlation g^(2)(0) = sum_n n*(n-1)*P(n) / (n_bar^2)
        let g2 = if n_bar <= 1e-12 {
            0.0
        } else {
            let mut second_moment = 0.0;
            for (n, &p) in dist.probabilities.iter().enumerate() {
                if n >= 2 {
                    second_moment += (n * (n - 1)) as f64 * p;
                }
            }
            second_moment / (n_bar * n_bar)
        };

        // Quantum purity Tr(rho^2)
        let purity = match dist.state_kind {
            PhononStateKind::GroundState
            | PhononStateKind::SinglePhononFock1
            | PhononStateKind::SqueezedVacuum
            | PhononStateKind::CoherentState => 1.0,
            PhononStateKind::ThermalState => {
                let p_sum: f64 = dist.probabilities.iter().map(|&p| p * p).sum();
                p_sum.clamp(0.0, 1.0)
            }
        };

        // Sideband cooling final occupancy:
        // n_final = (gamma_m * n_th) / (gamma_m + Gamma_opt) + (kappa / (4 * Omega_m))^2
        let n_final = Self::compute_sideband_cooling(
            gamma_m_hz,
            n_th,
            gamma_opt_hz,
            kappa_mhz,
            omega_m_mhz,
        );

        let is_sub_poissonian = g2 < 0.999 && n_bar > 1e-6;
        let is_quantum_pure = purity >= 0.999;

        Self {
            mean_phonon_number: n_bar,
            second_order_correlation_g2: g2,
            quantum_purity: purity,
            sideband_cooled_n_final: n_final,
            is_sub_poissonian,
            is_quantum_pure,
        }
    }

    /// Evaluates final sideband cooling phonon occupation limit:
    /// n_final = (gamma_m * n_th) / (gamma_m + Gamma_opt) + (kappa / (4 * Omega_m))^2.
    pub fn compute_sideband_cooling(
        gamma_m_hz: f64,
        n_th: f64,
        gamma_opt_hz: f64,
        kappa_mhz: f64,
        omega_m_mhz: f64,
    ) -> f64 {
        let total_damping = (gamma_m_hz + gamma_opt_hz).max(1e-6);
        let classical_limit = (gamma_m_hz * n_th) / total_damping;

        let omega_m = omega_m_mhz.max(0.1);
        let quantum_backaction_limit = (kappa_mhz / (4.0 * omega_m)).powi(2);

        classical_limit + quantum_backaction_limit
    }
}
