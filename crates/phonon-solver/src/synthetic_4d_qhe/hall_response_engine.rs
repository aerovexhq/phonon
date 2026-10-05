#![deny(unsafe_code)]

//! Non-linear 4D Quantum Hall response and synthetic frequency ladder engine.
//!
//! Evaluates the quantized non-linear Hall current j_x = sigma_4D * E_y * B_zw,
//! synthetic frequency sidebands, boundary directivity, and disorder robustness.

use crate::synthetic_4d_qhe::four_dim_lattice::{FourDimLatticeSolver, Synthetic4dParams};

/// Parameters for evaluating the 4D Hall current response and synthetic frequency drive.
#[derive(Debug, Clone)]
pub struct SyntheticHallParams {
    /// 4D lattice parameters.
    pub lattice: Synthetic4dParams,
    /// Applied effective electric drive amplitude E_y in V/m or normalized units (default 1.0).
    pub drive_field_ey: f64,
    /// Applied synthetic magnetic field B_zw in Tesla or normalized units (default 1.0).
    pub synthetic_field_bzw: f64,
    /// Frequency modulation rate Omega_mod in Hz (default 250 Hz).
    pub modulation_freq_hz: f64,
    /// Number of synthetic frequency sidebands to resolve per side (default 3, -3..=+3).
    pub harmonic_span: usize,
}

impl Default for SyntheticHallParams {
    fn default() -> Self {
        Self {
            lattice: Synthetic4dParams::default(),
            drive_field_ey: 1.0,
            synthetic_field_bzw: 1.0,
            modulation_freq_hz: 250.0,
            harmonic_span: 3,
        }
    }
}

/// A synthetic frequency harmonic sideband in the modulation spectrum.
#[derive(Debug, Clone)]
pub struct SyntheticHarmonicPoint {
    /// Harmonic order index n (e.g. -3, -2, -1, 0, 1, 2, 3).
    pub harmonic_index: i32,
    /// Sideband frequency in Hz: f_0 + n * Omega_mod.
    pub frequency_hz: f64,
    /// Normalized modal power in linear fraction.
    pub power_fraction: f64,
    /// Modal power in dB.
    pub power_db: f64,
}

/// Metrics describing the 4D quantum Hall response and boundary chiral transport.
#[derive(Debug, Clone)]
pub struct SyntheticHallMetrics {
    /// Quantized non-linear 4D Hall current j_x (in normalized units).
    pub non_linear_hall_current: f64,
    /// Non-linear 4D Hall conductance sigma_4D = j_x / (E_y * B_zw).
    pub non_linear_hall_conductance: f64,
    /// Boundary chiral propagation directivity in dB.
    pub chiral_directivity_db: f64,
    /// Topological defect immunity retention ratio under disorder sigma(W) / sigma(0).
    pub disorder_retention_ratio: f64,
    /// Synthetic frequency modal harmonic distribution.
    pub synthetic_harmonics: Vec<SyntheticHarmonicPoint>,
}

/// Engine computing non-linear 4D quantum Hall transport and synthetic frequency dynamics.
#[derive(Debug, Clone)]
pub struct SyntheticHallEngine {
    pub params: SyntheticHallParams,
    pub lattice_solver: FourDimLatticeSolver,
    pub metrics: SyntheticHallMetrics,
}

impl SyntheticHallEngine {
    /// Construct a new synthetic Hall engine and solve response.
    pub fn new(params: SyntheticHallParams) -> Self {
        let lattice_solver = FourDimLatticeSolver::new(params.lattice.clone());
        let mut engine = Self {
            params,
            lattice_solver,
            metrics: SyntheticHallMetrics {
                non_linear_hall_current: 0.0,
                non_linear_hall_conductance: 0.0,
                chiral_directivity_db: 0.0,
                disorder_retention_ratio: 1.0,
                synthetic_harmonics: Vec::new(),
            },
        };
        engine.recompute();
        engine
    }

    /// Recompute non-linear 4D Hall transport, directivity, and frequency harmonics.
    pub fn recompute(&mut self) {
        self.lattice_solver.params = self.params.lattice.clone();
        self.lattice_solver.recompute();

        let c2 = self.lattice_solver.params.second_chern_number();
        let is_topo = self.lattice_solver.params.is_topological();
        let ey = self.params.drive_field_ey;
        let bzw = self.params.synthetic_field_bzw;
        let disorder = self.lattice_solver.params.disorder_w;

        // In 4D QHE: sigma_4D is strictly proportional to the second Chern number C2:
        // sigma_4D = C2 * (e^2 / 4*pi^2*hbar)
        let sigma_base = 0.50; // Normalized quantum of 4D Hall conductance
        let sigma_4d = (c2 as f64) * sigma_base;

        // Disorder retention: in topological phase, bulk gap protects C2 against moderate disorder
        let disorder_retention = if is_topo {
            (1.0 - 0.20 * (disorder / 0.5).powi(2)).clamp(0.80, 1.0)
        } else {
            (1.0 - 0.90 * (disorder / 0.5)).clamp(0.05, 1.0)
        };

        let effective_conductance = sigma_4d * disorder_retention;
        let hall_current = effective_conductance * ey * bzw;

        // Chiral directivity along 3D boundary hyper-surface:
        // Topological states exhibit backscattering immunity (> 30 dB isolation)
        let directivity_db = if is_topo {
            let base_dir = 32.0 - 15.0 * (disorder / 0.5);
            base_dir.clamp(20.0, 40.0)
        } else {
            0.5
        };

        // Synthetic frequency modal spectrum:
        // Harmonic sidebands n * Omega_mod with Bessel-like envelope
        let f0 = self.lattice_solver.params.resonance_freq_hz;
        let omega_mod = self.params.modulation_freq_hz;
        let m_synth = self.lattice_solver.params.synthetic_coupling.clamp(0.1, 0.9);
        let span = self.params.harmonic_span as i32;

        let mut harmonics = Vec::new();
        let mut total_power = 0.0;

        for n in -span..=span {
            let freq = f0 + (n as f64) * omega_mod;
            // Envelope decays with harmonic order |n|
            let weight = (m_synth / 2.0).powi(n.abs()) / factorial(n.abs() as usize);
            let p_lin = weight * weight;
            total_power += p_lin;
            harmonics.push((n, freq, p_lin));
        }

        let synthetic_points = harmonics
            .into_iter()
            .map(|(n, freq, p_lin)| {
                let norm_p = if total_power > 1e-12 {
                    p_lin / total_power
                } else {
                    0.0
                };
                let p_db = 10.0 * (norm_p.max(1e-6)).log10();
                SyntheticHarmonicPoint {
                    harmonic_index: n,
                    frequency_hz: freq,
                    power_fraction: norm_p,
                    power_db: p_db,
                }
            })
            .collect();

        self.metrics = SyntheticHallMetrics {
            non_linear_hall_current: hall_current,
            non_linear_hall_conductance: effective_conductance,
            chiral_directivity_db: directivity_db,
            disorder_retention_ratio: disorder_retention,
            synthetic_harmonics: synthetic_points,
        };
    }

    /// Generate 4D Hall current curve j_x vs field product (E_y * B_zw).
    pub fn hall_response_curve(&self, num_points: usize, max_field: f64) -> Vec<(f64, f64)> {
        let n = num_points.max(10);
        let mut curve = Vec::with_capacity(n);
        let sigma = self.metrics.non_linear_hall_conductance;

        for i in 0..n {
            let field_product = -max_field + 2.0 * max_field * (i as f64) / (n - 1) as f64;
            let current = sigma * field_product;
            curve.push((field_product, current));
        }

        curve
    }
}

/// Helper function for integer factorial.
#[inline]
fn factorial(n: usize) -> f64 {
    match n {
        0 | 1 => 1.0,
        2 => 2.0,
        3 => 6.0,
        4 => 24.0,
        5 => 120.0,
        _ => 720.0,
    }
}
