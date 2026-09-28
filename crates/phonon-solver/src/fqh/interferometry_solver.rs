//! Quantum Hall Fabry-P\u{00e9}rot interferometry solver.
//!
//! Generates 2D magnetoconductance maps G(B, V_g), extracts Aharonov-Bohm oscillation periods,
//! and verifies non-Abelian even-odd visibility collapse.

use phonon_models::fqh::{FabryPerotInterferometer, LuttingerEdgeModel};

/// Configuration for an interferometry scan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterferometryScanConfig {
    /// Minimum magnetic field in Tesla.
    pub b_min_tesla: f64,
    /// Maximum magnetic field in Tesla.
    pub b_max_tesla: f64,
    /// Number of B-field grid points.
    pub num_b_points: usize,
    /// Minimum gate voltage in milliVolts.
    pub vg_min_mv: f64,
    /// Maximum gate voltage in milliVolts.
    pub vg_max_mv: f64,
    /// Number of V_g grid points.
    pub num_vg_points: usize,
    /// Number of localized bulk anyons N_\u{03c3} enclosed by the interferometer.
    pub num_bulk_anyons: usize,
    /// Anyon qubit state (|0\u{27e9} if false, |1\u{27e9} if true).
    pub anyon_state_one: bool,
}

impl Default for InterferometryScanConfig {
    fn default() -> Self {
        Self {
            b_min_tesla: 3.0,
            b_max_tesla: 3.05,
            num_b_points: 100,
            vg_min_mv: 0.0,
            vg_max_mv: 5.0,
            num_vg_points: 50,
            num_bulk_anyons: 0,
            anyon_state_one: false,
        }
    }
}

/// 1D line scan result across magnetic field.
#[derive(Debug, Clone, PartialEq)]
pub struct LineScanResult {
    /// Magnetic field points in Tesla.
    pub b_points: Vec<f64>,
    /// Conductance in Siemens at each point.
    pub conductance_siemens: Vec<f64>,
    /// Minimum conductance in Siemens.
    pub min_conductance: f64,
    /// Maximum conductance in Siemens.
    pub max_conductance: f64,
    /// Interference visibility \u{03bd} = (G_max - G_min) / (G_max + G_min).
    pub visibility: f64,
}

/// 2D conductance map result.
#[derive(Debug, Clone, PartialEq)]
pub struct ConductanceMap2D {
    /// Magnetic field grid points in Tesla.
    pub b_grid: Vec<f64>,
    /// Gate voltage grid points in milliVolts.
    pub vg_grid: Vec<f64>,
    /// Flattened row-major 2D conductance values G(B, V_g) in Siemens.
    pub conductance_matrix: Vec<f64>,
    /// Global peak conductance in Siemens.
    pub max_conductance: f64,
    /// Global minimum conductance in Siemens.
    pub min_conductance: f64,
    /// Overall interference visibility.
    pub visibility: f64,
}

/// Interferometry solver.
pub struct InterferometrySolver {
    pub interferometer: FabryPerotInterferometer,
    pub edge: LuttingerEdgeModel,
}

impl InterferometrySolver {
    pub fn new(interferometer: FabryPerotInterferometer, edge: LuttingerEdgeModel) -> Self {
        Self {
            interferometer,
            edge,
        }
    }

    /// Evaluates a 1D magnetoconductance line scan across B at fixed V_g.
    pub fn scan_magnetic_field(
        &self,
        b_min_tesla: f64,
        b_max_tesla: f64,
        num_points: usize,
        fixed_vg_mv: f64,
        num_bulk_anyons: usize,
        anyon_state_one: bool,
    ) -> LineScanResult {
        let n = num_points.max(2);
        let step = (b_max_tesla - b_min_tesla) / (n - 1) as f64;

        let mut b_points = Vec::with_capacity(n);
        let mut conductance = Vec::with_capacity(n);

        let mut min_g = f64::INFINITY;
        let mut max_g = f64::NEG_INFINITY;

        for i in 0..n {
            let b = b_min_tesla + i as f64 * step;
            let g = self.interferometer.tunneling_conductance_siemens(
                &self.edge,
                b,
                fixed_vg_mv,
                num_bulk_anyons,
                anyon_state_one,
            );
            b_points.push(b);
            conductance.push(g);

            if g < min_g {
                min_g = g;
            }
            if g > max_g {
                max_g = g;
            }
        }

        let visibility = if max_g + min_g > 1e-12 {
            (max_g - min_g) / (max_g + min_g)
        } else {
            0.0
        };

        LineScanResult {
            b_points,
            conductance_siemens: conductance,
            min_conductance: min_g,
            max_conductance: max_g,
            visibility,
        }
    }

    /// Generates a full 2D conductance map G(B, V_g).
    pub fn generate_conductance_map(&self, config: &InterferometryScanConfig) -> ConductanceMap2D {
        let nb = config.num_b_points.max(2);
        let nvg = config.num_vg_points.max(2);

        let b_step = (config.b_max_tesla - config.b_min_tesla) / (nb - 1) as f64;
        let vg_step = (config.vg_max_mv - config.vg_min_mv) / (nvg - 1) as f64;

        let mut b_grid = Vec::with_capacity(nb);
        for i in 0..nb {
            b_grid.push(config.b_min_tesla + i as f64 * b_step);
        }

        let mut vg_grid = Vec::with_capacity(nvg);
        for j in 0..nvg {
            vg_grid.push(config.vg_min_mv + j as f64 * vg_step);
        }

        let mut matrix = Vec::with_capacity(nb * nvg);
        let mut min_g = f64::INFINITY;
        let mut max_g = f64::NEG_INFINITY;

        for &vg in &vg_grid {
            for &b in &b_grid {
                let g = self.interferometer.tunneling_conductance_siemens(
                    &self.edge,
                    b,
                    vg,
                    config.num_bulk_anyons,
                    config.anyon_state_one,
                );
                matrix.push(g);
                if g < min_g {
                    min_g = g;
                }
                if g > max_g {
                    max_g = g;
                }
            }
        }

        let visibility = if max_g + min_g > 1e-12 {
            (max_g - min_g) / (max_g + min_g)
        } else {
            0.0
        };

        ConductanceMap2D {
            b_grid,
            vg_grid,
            conductance_matrix: matrix,
            max_conductance: max_g,
            min_conductance: min_g,
            visibility,
        }
    }
}
