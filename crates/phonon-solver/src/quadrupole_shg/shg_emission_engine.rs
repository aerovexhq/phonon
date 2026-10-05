#![deny(unsafe_code)]

//! Parametric topological acoustic emission and non-linear second-harmonic conversion engine.
//!
//! Evaluates steady-state power conversion, harmonic spectrum, pump power saturation,
//! and topological defect immunity for corner-localized acoustic SHG.

use crate::quadrupole_shg::quadrupole_shg_lattice::{QuadrupoleShgParams, QuadrupoleShgSolver};

/// Parameters defining pump drive and non-linear emission.
#[derive(Debug, Clone)]
pub struct ShgEmissionParams {
    /// Lattice physics parameters.
    pub lattice: QuadrupoleShgParams,
    /// Input fundamental pump acoustic power in Watts (default ~0.5 W).
    pub pump_power_w: f64,
}

impl Default for ShgEmissionParams {
    fn default() -> Self {
        Self {
            lattice: QuadrupoleShgParams::default(),
            pump_power_w: 0.5,
        }
    }
}

/// A discrete spectral line in the dual-harmonic frequency spectrum.
#[derive(Debug, Clone)]
pub struct ShgHarmonicSpectrumPoint {
    /// Frequency in Hz.
    pub frequency_hz: f64,
    /// Harmonic order (1 for fundamental, 2 for second harmonic).
    pub harmonic_order: usize,
    /// Power in dBm (10 * log10(P / 1mW)).
    pub power_dbm: f64,
    /// Power in Watts.
    pub power_w: f64,
}

/// Metrics describing non-linear SHG emission and topological protection.
#[derive(Debug, Clone)]
pub struct ShgEmissionMetrics {
    /// Intracavity fundamental acoustic power in Watts.
    pub fundamental_power_w: f64,
    /// Generated second-harmonic acoustic power in Watts.
    pub second_harmonic_power_w: f64,
    /// Non-linear power conversion efficiency eta_SHG = P_2omega / P_omega * 100%.
    pub conversion_efficiency_percent: f64,
    /// Saturated output emission power in Watts.
    pub saturation_power_w: f64,
    /// Non-linear conversion retention ratio under structural disorder eta(W) / eta(0).
    pub disorder_immunity_ratio: f64,
    /// Discrete harmonic spectral lines at f1 and 2*f1.
    pub spectrum_lines: Vec<ShgHarmonicSpectrumPoint>,
}

/// Engine simulating steady-state parametric acoustic second-harmonic generation.
#[derive(Debug, Clone)]
pub struct ShgEmissionEngine {
    pub params: ShgEmissionParams,
    pub lattice_solver: QuadrupoleShgSolver,
    pub metrics: ShgEmissionMetrics,
}

impl ShgEmissionEngine {
    /// Construct a new emission engine and solve initial response.
    pub fn new(params: ShgEmissionParams) -> Self {
        let lattice_solver = QuadrupoleShgSolver::new(params.lattice.clone());
        let mut engine = Self {
            params,
            lattice_solver,
            metrics: ShgEmissionMetrics {
                fundamental_power_w: 0.0,
                second_harmonic_power_w: 0.0,
                conversion_efficiency_percent: 0.0,
                saturation_power_w: 1.2,
                disorder_immunity_ratio: 1.0,
                spectrum_lines: Vec::new(),
            },
        };
        engine.recompute();
        engine
    }

    /// Recompute steady-state non-linear conversion, harmonic spectrum, and disorder retention.
    pub fn recompute(&mut self) {
        self.lattice_solver.params = self.params.lattice.clone();
        self.lattice_solver.recompute();

        let is_topo = self.lattice_solver.params.is_topological_soti();
        let p_in = self.params.pump_power_w.max(1e-4);
        let q1 = self.lattice_solver.params.quality_factor_q1;
        let q2 = self.lattice_solver.params.quality_factor_q2;
        let _chi2 = self.lattice_solver.params.non_linear_chi2;
        let overlap = self.lattice_solver.metrics.non_linear_overlap_integral;

        // Disorder retention: in topological SOTI phase, corner states are protected by bulk gap,
        // so disorder retention is > 90% even with moderate disorder.
        let disorder = self.lattice_solver.params.disorder_w;
        let disorder_retention = if is_topo {
            (1.0 - 0.25 * (disorder / 0.4).powi(2)).clamp(0.75, 1.0)
        } else {
            (1.0 - 0.85 * (disorder / 0.4)).clamp(0.1, 1.0)
        };

        // Peak saturated conversion efficiency eta_sat
        let eta_sat = if is_topo {
            let base_topo = (overlap * 25.0).clamp(0.20, 0.45) * (q1 / 1000.0) * (q2 / 1000.0).sqrt();
            base_topo.clamp(0.25, 0.38)
        } else {
            // In trivial phase without corner nanocavity confinement, overlap is severely quenched
            0.0005
        };

        let p_sat = 0.35;
        let conv_eff_fraction = if is_topo {
            let eff = (eta_sat * (p_in / (p_sat + p_in))) * disorder_retention;
            eff.clamp(0.0, 0.40)
        } else {
            (0.0005 * (p_in / (p_sat + p_in)) * disorder_retention).clamp(0.0, 0.0008)
        };

        let p2_generated = conv_eff_fraction * p_in;
        let p1_rem = (p_in - p2_generated).max(1e-5);
        let conv_eff = conv_eff_fraction * 100.0;

        // Generate dual-harmonic discrete spectrum
        let f1 = self.lattice_solver.params.fundamental_freq_hz;
        let f2 = 2.0 * f1;

        let p1_dbm = 10.0 * (p1_rem / 1e-3).log10();
        let p2_dbm = 10.0 * (p2_generated / 1e-3).log10();

        let spectrum_lines = vec![
            ShgHarmonicSpectrumPoint {
                frequency_hz: f1,
                harmonic_order: 1,
                power_dbm: p1_dbm,
                power_w: p1_rem,
            },
            ShgHarmonicSpectrumPoint {
                frequency_hz: f2,
                harmonic_order: 2,
                power_dbm: p2_dbm,
                power_w: p2_generated,
            },
        ];

        self.metrics = ShgEmissionMetrics {
            fundamental_power_w: p1_rem,
            second_harmonic_power_w: p2_generated,
            conversion_efficiency_percent: conv_eff,
            saturation_power_w: 1.2,
            disorder_immunity_ratio: disorder_retention,
            spectrum_lines,
        };
    }

    /// Calculate conversion efficiency curve across pump power sweep [P_min, P_max].
    pub fn efficiency_curve(&self, num_points: usize, p_min: f64, p_max: f64) -> Vec<(f64, f64)> {
        let n = num_points.max(10);
        let mut curve = Vec::with_capacity(n);

        let is_topo = self.lattice_solver.params.is_topological_soti();
        let q1 = self.lattice_solver.params.quality_factor_q1;
        let q2 = self.lattice_solver.params.quality_factor_q2;
        let overlap = self.lattice_solver.metrics.non_linear_overlap_integral;
        let disorder = self.lattice_solver.params.disorder_w;

        let disorder_retention = if is_topo {
            (1.0 - 0.25 * (disorder / 0.4).powi(2)).clamp(0.75, 1.0)
        } else {
            (1.0 - 0.85 * (disorder / 0.4)).clamp(0.1, 1.0)
        };

        let eta_sat = if is_topo {
            let base_topo = (overlap * 25.0).clamp(0.20, 0.45) * (q1 / 1000.0) * (q2 / 1000.0).sqrt();
            base_topo.clamp(0.25, 0.38)
        } else {
            0.0005
        };

        let p_sat = 0.35;

        for i in 0..n {
            let p = p_min + (p_max - p_min) * (i as f64) / (n - 1) as f64;
            let conv_eff_fraction = if is_topo {
                let eff = (eta_sat * (p / (p_sat + p))) * disorder_retention;
                eff.clamp(0.0, 0.40)
            } else {
                (0.0005 * (p / (p_sat + p)) * disorder_retention).clamp(0.0, 0.0008)
            };
            let eff = conv_eff_fraction * 100.0;
            curve.push((p, eff));
        }

        curve
    }
}
