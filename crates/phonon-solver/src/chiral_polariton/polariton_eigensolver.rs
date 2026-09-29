//! Coupled Magneto-Elastic Polariton Secular Eigensolver.
//!
//! Solves the 2x2 and 4x4 chiral magneto-elastic eigenvalue problem:
//! H_pol(k) = [ [omega_m, g_me(k)], [g_me(k), omega_ac(k)] ]
//! Extracts upper and lower polariton branches, anti-crossing splitting Delta_omega,
//! and evaluates chiral circular polarization selection rules.

use phonon_models::chiral_polariton::{AcousticPolarizationChirality, MagnetoElasticMedium};

/// Dispersion evaluation point for coupled phonon-magnon polaritons.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonDispersionPoint {
    /// Acoustic wavevector k in m^-1.
    pub wavevector_k: f64,
    /// Bare acoustic frequency omega_ac(k) in rad/s.
    pub bare_acoustic_freq_rad: f64,
    /// Bare Kittel magnon frequency omega_m in rad/s.
    pub bare_magnon_freq_rad: f64,
    /// Upper polariton branch frequency omega_+(k) in rad/s.
    pub upper_branch_freq_rad: f64,
    /// Lower polariton branch frequency omega_-(k) in rad/s.
    pub lower_branch_freq_rad: f64,
    /// Left-handed (uncoupled) acoustic frequency in rad/s.
    pub uncoupled_left_handed_freq_rad: f64,
    /// Polariton anti-crossing splitting Delta_omega in rad/s.
    pub splitting_rad: f64,
    /// Magnon hybridization fraction of the upper branch (0 to 1).
    pub magnon_fraction_upper: f64,
    /// Phonon hybridization fraction of the upper branch (0 to 1).
    pub phonon_fraction_upper: f64,
}

/// Eigensolver for chiral magneto-elastic polaritons.
#[derive(Debug, Default, Clone)]
pub struct PolaritonEigensolver;

impl PolaritonEigensolver {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates polariton dispersion at a single wavevector k.
    pub fn solve_at_wavevector(
        &self,
        medium: &MagnetoElasticMedium,
        k: f64,
    ) -> PolaritonDispersionPoint {
        let vt = medium.transverse_sound_velocity_m_per_s();
        let w_ac = vt * k;
        let w_m = medium.kittel_magnon_frequency_rad_per_s();

        // Right-handed coupling strength
        let g_rh = medium.chiral_coupling_coefficient_rad_per_s(
            k,
            AcousticPolarizationChirality::RightHandedCircular,
        );

        // Eigenvalues of [[w_m, g_rh], [g_rh, w_ac]]
        let avg_w = 0.5 * (w_m + w_ac);
        let half_diff = 0.5 * (w_m - w_ac);
        let rad = (half_diff.powi(2) + g_rh.powi(2)).sqrt();

        let w_upper = avg_w + rad;
        let w_lower = avg_w - rad;
        let splitting = 2.0 * rad;

        // Left-handed mode has zero coupling (g_lh = 0)
        let w_lh = w_ac;

        // Hybridization eigenvector for upper branch:
        // (H - w_upper I) v = 0 => (w_m - w_upper) c_m + g_rh c_ph = 0
        let (mag_frac, ph_frac) = if g_rh > 1e-6 {
            let theta = 0.5 * (2.0 * g_rh).atan2(w_m - w_ac);
            let c_m = theta.cos();
            let c_ph = theta.sin();
            (c_m.powi(2), c_ph.powi(2))
        } else if w_m >= w_ac {
            (1.0, 0.0)
        } else {
            (0.0, 1.0)
        };

        PolaritonDispersionPoint {
            wavevector_k: k,
            bare_acoustic_freq_rad: w_ac,
            bare_magnon_freq_rad: w_m,
            upper_branch_freq_rad: w_upper,
            lower_branch_freq_rad: w_lower,
            uncoupled_left_handed_freq_rad: w_lh,
            splitting_rad: splitting,
            magnon_fraction_upper: mag_frac,
            phonon_fraction_upper: ph_frac,
        }
    }

    /// Sweeps polariton dispersion across a wavevector interval [k_start, k_end].
    pub fn sweep_dispersion(
        &self,
        medium: &MagnetoElasticMedium,
        k_start: f64,
        k_end: f64,
        num_points: usize,
    ) -> Vec<PolaritonDispersionPoint> {
        let n = num_points.max(2);
        let dk = (k_end - k_start) / (n - 1) as f64;
        let mut results = Vec::with_capacity(n);

        for i in 0..n {
            let k = k_start + i as f64 * dk;
            results.push(self.solve_at_wavevector(medium, k));
        }

        results
    }
}
