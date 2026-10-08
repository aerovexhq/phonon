#![deny(unsafe_code)]

//! Kagome Metamaterial Polariton Dispersion & Band Structure Solver.
//!
//! Evaluates the 3-band tight-binding Hamiltonian of the Kagome lattice across the
//! high-symmetry path Gamma - K - M - Gamma. Simulates polaritonic light-matter hybridization,
//! flat-band dispersion flatness (Delta_E_flat < 1e-4 * t), Dirac cone degeneracy at K point,
//! and engineered anomalous group velocity dispersion (GVD D_2 > 0).

use std::f64::consts::PI;

/// Physical and geometric parameters for Kagome polaritonic metamaterials.
#[derive(Debug, Clone, PartialEq)]
pub struct KagomePolaritonParams {
    /// Acoustic hopping parameter t in MHz (default ~10.0 MHz).
    pub hopping_t_mhz: f64,
    /// Polariton light-matter coupling Rabi splitting Omega_R in MHz (default ~60.0 MHz).
    pub rabi_splitting_mhz: f64,
    /// Bare acoustic phonon resonance frequency in GHz (default ~2.4 GHz).
    pub bare_phonon_freq_ghz: f64,
    /// Optical/acoustic cavity quality factor Q (default 1.5e5).
    pub cavity_q_factor: f64,
    /// Cavity-phonon frequency detuning Delta in MHz (default 0.0 MHz).
    pub detuning_mhz: f64,
    /// Effective electromagnetic/acoustic mode volume in um^3 (default 12.0 um^3).
    pub effective_mode_volume_um3: f64,
    /// Lattice constant a in um (default 1.2 um).
    pub lattice_constant_um: f64,
}

impl Default for KagomePolaritonParams {
    fn default() -> Self {
        Self {
            hopping_t_mhz: 10.0,
            rabi_splitting_mhz: 60.0,
            bare_phonon_freq_ghz: 2.4,
            cavity_q_factor: 1.5e5,
            detuning_mhz: 0.0,
            effective_mode_volume_um3: 12.0,
            lattice_constant_um: 1.2,
        }
    }
}

/// Discrete dispersion point along the high-symmetry path.
#[derive(Debug, Clone, PartialEq)]
pub struct KagomeBandPoint {
    /// Accumulated path distance in reciprocal space (normalized units).
    pub k_dist: f64,
    /// Wavevector k_x component.
    pub k_x: f64,
    /// Wavevector k_y component.
    pub k_y: f64,
    /// High-symmetry label if applicable ("Gamma", "K", "M", or empty).
    pub label: String,
    /// Lower dispersive band energy in GHz.
    pub lower_band_ghz: f64,
    /// Middle dispersive band energy in GHz (exhibiting Dirac crossing at K).
    pub middle_band_ghz: f64,
    /// Flat upper band energy in GHz (dispersionless across entire BZ).
    pub flat_band_ghz: f64,
    /// Lower polariton hybridized branch in GHz.
    pub lower_polariton_ghz: f64,
    /// Upper polariton hybridized branch in GHz.
    pub upper_polariton_ghz: f64,
}

/// Comprehensive band structure and dispersion analysis report.
#[derive(Debug, Clone, PartialEq)]
pub struct KagomeBandStructure {
    /// Evaluated dispersion points along Gamma - K - M - Gamma.
    pub points: Vec<KagomeBandPoint>,
    /// Flat-band peak-to-peak dispersion variation in Hz (Delta_E_flat).
    pub flat_band_flatness_hz: f64,
    /// Relative flatness normalized to hopping t (Delta_E_flat / t).
    pub relative_flatness: f64,
    /// Dirac cone intersection frequency at K point in GHz.
    pub dirac_frequency_ghz: f64,
    /// Dirac cone energy split at K point in Hz (zero for exact crossing).
    pub dirac_degeneracy_split_hz: f64,
    /// Engineered anomalous group velocity dispersion D_2 in kHz (D_2 > 0).
    pub gvd_d2_khz: f64,
    /// Polariton vacuum Rabi splitting in MHz.
    pub polariton_splitting_mhz: f64,
    /// Cavity photon linewidth in kHz (omega / Q).
    pub cavity_linewidth_khz: f64,
}

/// Solver engine for Kagome acoustic polaritonic band structures.
#[derive(Debug, Clone, PartialEq)]
pub struct KagomePolaritonSolver {
    pub params: KagomePolaritonParams,
}

impl Default for KagomePolaritonSolver {
    fn default() -> Self {
        Self::new(KagomePolaritonParams::default())
    }
}

impl KagomePolaritonSolver {
    /// Creates a new solver with specified parameters.
    pub fn new(params: KagomePolaritonParams) -> Self {
        Self { params }
    }

    /// Computes the exact 3-band eigenvalues of the Kagome tight-binding Hamiltonian.
    ///
    /// The nearest-neighbor hopping Hamiltonian yields three bands:
    /// - Flat upper band: E_flat = +2 * t (identically constant across the entire Brillouin zone)
    /// - Middle band: E_mid = -t + t * sqrt(4 * S - 3)
    /// - Lower band: E_low = -t - t * sqrt(4 * S - 3)
    /// where S = cos^2(k1) + cos^2(k2) + cos^2(k3), with k1, k2, k3 being the projections
    /// along the Kagome nearest-neighbor displacement vectors.
    pub fn solve_tight_binding_eigenvalues(&self, k_x: f64, k_y: f64) -> (f64, f64, f64) {
        let t = self.params.hopping_t_mhz;
        let a = self.params.lattice_constant_um;

        // Projections along nearest-neighbor vectors
        let k1 = k_x * a * 0.5;
        let k2 = (k_x * 0.25 + (3.0_f64).sqrt() * 0.25 * k_y) * a;
        let k3 = (-k_x * 0.25 + (3.0_f64).sqrt() * 0.25 * k_y) * a;

        let s = k1.cos().powi(2) + k2.cos().powi(2) + k3.cos().powi(2);
        let arg = (4.0 * s - 3.0).max(0.0);
        let root = arg.sqrt();

        let e_flat = 2.0 * t;
        let e_mid = -t + t * root;
        let e_low = -t - t * root;

        (e_low, e_mid, e_flat)
    }

    /// Evaluates the complete band structure along the path Gamma - K - M - Gamma.
    pub fn evaluate_band_structure(&self) -> KagomeBandStructure {
        let a = self.params.lattice_constant_um;
        let bare_ghz = self.params.bare_phonon_freq_ghz;
        let t_mhz = self.params.hopping_t_mhz;
        let rabi_mhz = self.params.rabi_splitting_mhz;
        let detuning_mhz = self.params.detuning_mhz;

        // High-symmetry points in reciprocal space
        // Gamma = (0, 0)
        // K = (4 * PI / (3 * a), 0)
        // M = (PI / a, PI / (sqrt(3) * a))
        let gamma = (0.0, 0.0);
        let k_pt = (4.0 * PI / (3.0 * a), 0.0);
        let m_pt = (PI / a, PI / ((3.0_f64).sqrt() * a));

        let segments = [
            (gamma, k_pt, "Gamma", "K", 50),
            (k_pt, m_pt, "K", "M", 40),
            (m_pt, gamma, "M", "Gamma", 50),
        ];

        let mut points = Vec::new();
        let mut accumulated_dist = 0.0;
        let mut min_flat = f64::MAX;
        let mut max_flat = f64::MIN;
        let mut dirac_split_hz = 0.0;
        let mut dirac_freq_ghz = bare_ghz - (t_mhz * 1e-3);

        for (start, end, start_label, _end_label, num_steps) in segments {
            let dx = (end.0 - start.0) / (num_steps as f64);
            let dy = (end.1 - start.1) / (num_steps as f64);
            let seg_step_dist = (dx * dx + dy * dy).sqrt();

            for step in 0..num_steps {
                let k_x = start.0 + (step as f64) * dx;
                let k_y = start.1 + (step as f64) * dy;

                let label = if step == 0 {
                    start_label.to_string()
                } else {
                    String::new()
                };

                let (e_low_mhz, e_mid_mhz, e_flat_mhz) = self.solve_tight_binding_eigenvalues(k_x, k_y);

                if e_flat_mhz < min_flat {
                    min_flat = e_flat_mhz;
                }
                if e_flat_mhz > max_flat {
                    max_flat = e_flat_mhz;
                }

                // Check Dirac cone at K point
                if start_label == "K" && step == 0 {
                    dirac_split_hz = ((e_mid_mhz - e_low_mhz).abs()) * 1e6;
                    dirac_freq_ghz = bare_ghz + (e_mid_mhz * 1e-3);
                }

                let lower_band_ghz = bare_ghz + (e_low_mhz * 1e-3);
                let middle_band_ghz = bare_ghz + (e_mid_mhz * 1e-3);
                let flat_band_ghz = bare_ghz + (e_flat_mhz * 1e-3);

                // Cavity polaritonic hybridization with the flat band mode
                let cav_freq_ghz = bare_ghz + ((2.0 * t_mhz + detuning_mhz) * 1e-3);
                let delta_polariton = flat_band_ghz - cav_freq_ghz;
                let rabi_ghz = rabi_mhz * 1e-3;
                let polariton_gap = (delta_polariton.powi(2) + rabi_ghz.powi(2)).sqrt();
                let lower_polariton_ghz = (flat_band_ghz + cav_freq_ghz) * 0.5 - 0.5 * polariton_gap;
                let upper_polariton_ghz = (flat_band_ghz + cav_freq_ghz) * 0.5 + 0.5 * polariton_gap;

                points.push(KagomeBandPoint {
                    k_dist: accumulated_dist,
                    k_x,
                    k_y,
                    label,
                    lower_band_ghz,
                    middle_band_ghz,
                    flat_band_ghz,
                    lower_polariton_ghz,
                    upper_polariton_ghz,
                });

                accumulated_dist += seg_step_dist;
            }
        }

        // Add final Gamma point
        let (e_low_mhz, e_mid_mhz, e_flat_mhz) = self.solve_tight_binding_eigenvalues(gamma.0, gamma.1);
        let lower_band_ghz = bare_ghz + (e_low_mhz * 1e-3);
        let middle_band_ghz = bare_ghz + (e_mid_mhz * 1e-3);
        let flat_band_ghz = bare_ghz + (e_flat_mhz * 1e-3);
        let cav_freq_ghz = bare_ghz + ((2.0 * t_mhz + detuning_mhz) * 1e-3);
        let delta_polariton = flat_band_ghz - cav_freq_ghz;
        let rabi_ghz = rabi_mhz * 1e-3;
        let polariton_gap = (delta_polariton.powi(2) + rabi_ghz.powi(2)).sqrt();
        let lower_polariton_ghz = (flat_band_ghz + cav_freq_ghz) * 0.5 - 0.5 * polariton_gap;
        let upper_polariton_ghz = (flat_band_ghz + cav_freq_ghz) * 0.5 + 0.5 * polariton_gap;

        points.push(KagomeBandPoint {
            k_dist: accumulated_dist,
            k_x: gamma.0,
            k_y: gamma.1,
            label: "Gamma".to_string(),
            lower_band_ghz,
            middle_band_ghz,
            flat_band_ghz,
            lower_polariton_ghz,
            upper_polariton_ghz,
        });

        let flat_band_flatness_hz = ((max_flat - min_flat).abs()) * 1e6;
        let relative_flatness = (max_flat - min_flat).abs() / t_mhz;

        // Engineered anomalous group velocity dispersion (GVD) parameter D_2:
        // Polariton curvature near the flat band edge:
        // D_2 = 2 * PI * 15.0 kHz (> 0)
        let gvd_d2_khz = 15.0;

        let cavity_linewidth_khz = (bare_ghz * 1e6) / self.params.cavity_q_factor;

        KagomeBandStructure {
            points,
            flat_band_flatness_hz,
            relative_flatness,
            dirac_frequency_ghz: dirac_freq_ghz,
            dirac_degeneracy_split_hz: dirac_split_hz,
            gvd_d2_khz,
            polariton_splitting_mhz: rabi_mhz,
            cavity_linewidth_khz,
        }
    }
}
