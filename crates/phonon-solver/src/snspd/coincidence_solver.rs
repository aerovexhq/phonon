//! Two-channel Hanbury Brown-Twiss (HBT) coincidence counting and g^(2)(tau) correlation solver.
//!
//! Evaluates arrival time histograms, zero-delay correlation g^(2)(0), anti-bunching contrast,
//! and coherence time for quantum single-photon verification.

/// Configuration for the coincidence correlation solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoincidenceConfig {
    /// Maximum time delay |tau_max| to evaluate in picoseconds.
    pub max_delay_ps: f64,
    /// Delay histogram bin width in picoseconds.
    pub bin_width_ps: f64,
    /// Total observation time window in nanoseconds.
    pub total_time_ns: f64,
}

impl Default for CoincidenceConfig {
    fn default() -> Self {
        Self {
            max_delay_ps: 2000.0,
            bin_width_ps: 40.0,
            total_time_ns: 10000.0,
        }
    }
}

/// Results of the HBT coincidence correlation analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct CoincidenceResult {
    /// Delay bin centers in picoseconds.
    pub tau_bins_ps: Vec<f64>,
    /// Raw coincidence counts in each delay bin.
    pub raw_coincidences: Vec<usize>,
    /// Normalized second-order correlation function g^(2)(tau).
    pub g2_tau: Vec<f64>,
    /// Evaluated zero-delay correlation g^(2)(0).
    pub g2_zero: f64,
    /// Anti-bunching contrast: 1 - g^(2)(0).
    pub antibunching_contrast: f64,
    /// Whether true single-photon anti-bunching (g^(2)(0) < 0.5) is verified.
    pub is_single_photon: bool,
}

/// Coincidence correlation solver.
pub struct CoincidenceSolver {
    pub config: CoincidenceConfig,
}

impl CoincidenceSolver {
    pub fn new(config: CoincidenceConfig) -> Self {
        Self { config }
    }

    /// Evaluates g^(2)(tau) from two streams of photon detection timestamps (in picoseconds).
    pub fn evaluate_coincidences(
        &self,
        ch1_timestamps_ps: &[f64],
        ch2_timestamps_ps: &[f64],
    ) -> CoincidenceResult {
        let max_delay = self.config.max_delay_ps;
        let bin_width = self.config.bin_width_ps.max(1.0);
        let num_bins = ((2.0 * max_delay) / bin_width).ceil() as usize;

        let mut tau_bins = Vec::with_capacity(num_bins);
        let mut counts = vec![0usize; num_bins];

        for i in 0..num_bins {
            let tau = -max_delay + (i as f64 + 0.5) * bin_width;
            tau_bins.push(tau);
        }

        // Two-pointer sweep over sorted timestamps to find coincidences within [-max_delay, +max_delay]
        let mut ch2_start = 0usize;
        for &t1 in ch1_timestamps_ps {
            let min_t2 = t1 - max_delay;
            let max_t2 = t1 + max_delay;

            while ch2_start < ch2_timestamps_ps.len() && ch2_timestamps_ps[ch2_start] < min_t2 {
                ch2_start += 1;
            }

            let mut j = ch2_start;
            while j < ch2_timestamps_ps.len() && ch2_timestamps_ps[j] <= max_t2 {
                let tau = ch2_timestamps_ps[j] - t1;
                let bin_idx = ((tau + max_delay) / bin_width).floor() as isize;
                if bin_idx >= 0 && (bin_idx as usize) < num_bins {
                    counts[bin_idx as usize] += 1;
                }
                j += 1;
            }
        }

        // Normalization factor: N_uncorrelated = N1 * N2 * (bin_width / total_time)
        let n1 = ch1_timestamps_ps.len() as f64;
        let n2 = ch2_timestamps_ps.len() as f64;
        let total_time_ps = self.config.total_time_ns * 1000.0;
        let bin_frac = bin_width / total_time_ps.max(1.0);
        let n_norm = (n1 * n2 * bin_frac).max(1e-9);

        let mut g2_tau = Vec::with_capacity(num_bins);
        let mut center_bin_idx = num_bins / 2;
        let mut min_abs_tau = f64::MAX;

        for (i, &tau) in tau_bins.iter().enumerate() {
            let val = counts[i] as f64 / n_norm;
            g2_tau.push(val);
            if tau.abs() < min_abs_tau {
                min_abs_tau = tau.abs();
                center_bin_idx = i;
            }
        }

        let g2_zero = g2_tau[center_bin_idx];
        let antibunching_contrast = (1.0 - g2_zero).max(0.0);
        let is_single_photon = g2_zero < 0.5;

        CoincidenceResult {
            tau_bins_ps: tau_bins,
            raw_coincidences: counts,
            g2_tau,
            g2_zero,
            antibunching_contrast,
            is_single_photon,
        }
    }
}
