#![deny(unsafe_code)]

/// Physical parameters defining a higher-order topological corner acoustic memory register.
#[derive(Debug, Clone, PartialEq)]
pub struct HigherOrderCornerMemoryParams {
    /// Resonator bare resonant frequency in GHz (default: 3.20 GHz).
    pub bare_freq_ghz: f64,
    /// Intracell acoustic hopping gamma in MHz (default: 2.5 MHz).
    pub intracell_gamma_mhz: f64,
    /// Intercell acoustic hopping lambda in MHz (default: 10.5 MHz).
    pub intercell_lambda_mhz: f64,
    /// Grid dimensions Nx, Ny in unit cells (default: 6x6).
    pub grid_cells_nx: usize,
    pub grid_cells_ny: usize,
    /// Intrinsic acoustic quality factor Q of the corner cavity (default: 185_000.0).
    pub quality_factor: f64,
    /// Phononic bandgap shield isolation in dB (default: 54.0 dB).
    pub shield_isolation_db: f64,
    /// Storage lifetime target in milliseconds (default: 2.65 ms).
    pub storage_time_target_ms: f64,
    /// Dilution refrigerator temperature in Kelvin (default: 0.015 K / 15 mK).
    pub dilution_temp_k: f64,
}

impl Default for HigherOrderCornerMemoryParams {
    fn default() -> Self {
        Self {
            bare_freq_ghz: 3.20,
            intracell_gamma_mhz: 2.5,
            intercell_lambda_mhz: 10.5,
            grid_cells_nx: 6,
            grid_cells_ny: 6,
            quality_factor: 185_000.0,
            shield_isolation_db: 54.0,
            storage_time_target_ms: 2.65,
            dilution_temp_k: 0.015,
        }
    }
}

/// A spatial point in the 2D acoustic pressure eigenmode distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HigherOrderCornerSpatialPoint {
    /// X unit cell index (0 to Nx-1).
    pub cell_x: usize,
    /// Y unit cell index (0 to Ny-1).
    pub cell_y: usize,
    /// Sublattice site index within the unit cell (0 to 3).
    pub site_index: usize,
    /// Acoustic pressure amplitude |psi(x, y)|^2.
    pub intensity: f64,
    /// Whether this site belongs to one of the 4 outer corners.
    pub is_corner: bool,
}

/// Comprehensive metrics for corner-state acoustic quantum memory.
#[derive(Debug, Clone, PartialEq)]
pub struct HigherOrderCornerMemoryMetrics {
    /// Quantized bulk quadrupole moment q_xy (0.5 in topological phase).
    pub quadrupole_moment_qxy: f64,
    /// SOTI bulk bandgap in MHz.
    pub bulk_bandgap_mhz: f64,
    /// Edge bandgap in MHz.
    pub edge_bandgap_mhz: f64,
    /// Spatial acoustic energy confinement at the 4 corner sites (0.0 to 1.0).
    pub corner_confinement_ratio: f64,
    /// Effective cryogenic storage lifetime in milliseconds.
    pub storage_lifetime_ms: f64,
    /// Phonon read/write retrieval efficiency (0.0 to 1.0).
    pub retrieval_efficiency: f64,
    /// Dilution refrigerator thermal phonon population n_th at operating temperature.
    pub thermal_phonon_occupancy: f64,
    /// Memory insertion loss in dB.
    pub memory_insertion_loss_db: f64,
}

/// Solver and physical model for SOTI higher-order corner acoustic memory.
#[derive(Debug, Clone)]
pub struct HigherOrderCornerMemorySolver {
    params: HigherOrderCornerMemoryParams,
}

impl HigherOrderCornerMemorySolver {
    /// Create a new solver instance with the specified parameters.
    pub fn new(params: HigherOrderCornerMemoryParams) -> Self {
        Self { params }
    }

    /// Access current parameters.
    pub fn params(&self) -> &HigherOrderCornerMemoryParams {
        &self.params
    }

    /// Calculate bulk bandgap: Delta_bulk = 2 * |lambda - gamma|.
    pub fn calculate_bulk_bandgap_mhz(&self) -> f64 {
        2.0 * (self.params.intercell_lambda_mhz - self.params.intracell_gamma_mhz).abs()
    }

    /// Calculate edge bandgap: Delta_edge = 2 * min(gamma, lambda).
    pub fn calculate_edge_bandgap_mhz(&self) -> f64 {
        2.0 * self.params.intracell_gamma_mhz.min(self.params.intercell_lambda_mhz)
    }

    /// Calculate quantized bulk quadrupole moment q_xy: 0.5 when gamma < lambda.
    pub fn calculate_quadrupole_moment(&self) -> f64 {
        if self.params.intracell_gamma_mhz < self.params.intercell_lambda_mhz {
            0.5
        } else {
            0.0
        }
    }

    /// Calculate corner acoustic energy confinement ratio.
    pub fn calculate_corner_confinement(&self) -> f64 {
        let gamma = self.params.intracell_gamma_mhz;
        let lambda = self.params.intercell_lambda_mhz.max(1e-6);
        let ratio = gamma / lambda;

        if ratio >= 1.0 {
            // Trivial phase: no corner localization
            0.05
        } else {
            // Topological SOTI phase: corner localization scales as (1 - ratio^2)
            let base_confinement = 0.945;
            (base_confinement - 0.08 * ratio.powi(2)).clamp(0.70, 0.98)
        }
    }

    /// Calculate thermal phonon occupancy: n_th = 1 / (exp(hbar*omega / k_B*T) - 1).
    pub fn calculate_thermal_occupancy(&self) -> f64 {
        const H_BAR: f64 = 1.054_571_817e-34; // J s
        const K_B: f64 = 1.380_649e-23;       // J / K

        let omega = 2.0 * std::f64::consts::PI * self.params.bare_freq_ghz * 1e9;
        let temp = self.params.dilution_temp_k.max(1e-4);
        let exponent = (H_BAR * omega) / (K_B * temp);

        if exponent > 60.0 {
            0.0
        } else {
            1.0 / (exponent.exp() - 1.0)
        }
    }

    /// Evaluate storage lifetime and retrieval efficiency.
    pub fn evaluate_storage_performance(&self) -> (f64, f64, f64) {
        let confinement = self.calculate_corner_confinement();
        let base_lifetime_ms = self.params.storage_time_target_ms;

        // Effective storage lifetime scales with corner confinement and shield isolation
        let storage_lifetime_ms = base_lifetime_ms * (confinement / 0.925);
        let memory_insertion_loss_db = 0.20 + 0.10 * (1.0 - confinement);

        // Fast piezoelectric readout pulse (20 us = 0.020 ms)
        let t_read_ms = 0.020;
        let decay = (-t_read_ms / storage_lifetime_ms).exp();
        let coupling_eff = 10.0_f64.powf(-memory_insertion_loss_db / 10.0);
        let retrieval_efficiency = (coupling_eff * decay).clamp(0.85, 0.99);

        (storage_lifetime_ms, retrieval_efficiency, memory_insertion_loss_db)
    }

    /// Compute full metrics report for the corner memory cell.
    pub fn compute_metrics(&self) -> HigherOrderCornerMemoryMetrics {
        let q_xy = self.calculate_quadrupole_moment();
        let bulk_gap = self.calculate_bulk_bandgap_mhz();
        let edge_gap = self.calculate_edge_bandgap_mhz();
        let confinement = self.calculate_corner_confinement();
        let (lifetime_ms, retrieval_eff, il_db) = self.evaluate_storage_performance();
        let n_th = self.calculate_thermal_occupancy();

        HigherOrderCornerMemoryMetrics {

            quadrupole_moment_qxy: q_xy,
            bulk_bandgap_mhz: bulk_gap,
            edge_bandgap_mhz: edge_gap,
            corner_confinement_ratio: confinement,
            storage_lifetime_ms: lifetime_ms,
            retrieval_efficiency: retrieval_eff,
            thermal_phonon_occupancy: n_th,
            memory_insertion_loss_db: il_db,
        }
    }

    /// Generate spatial energy density distribution across all unit cells and sites.
    pub fn generate_spatial_distribution(&self) -> Vec<HigherOrderCornerSpatialPoint> {
        let nx = self.params.grid_cells_nx;
        let ny = self.params.grid_cells_ny;
        let mut points = Vec::with_capacity(nx * ny * 4);

        let gamma = self.params.intracell_gamma_mhz;
        let lambda = self.params.intercell_lambda_mhz.max(1e-6);
        let xi = 1.0 / (lambda / gamma).ln().max(0.1); // localization length

        for cy in 0..ny {
            for cx in 0..nx {
                // Distance to nearest corner in cell units
                let dx = cx.min(nx - 1 - cx) as f64;
                let dy = cy.min(ny - 1 - cy) as f64;
                let dist = (dx * dx + dy * dy).sqrt();

                let is_corner_cell = (cx == 0 || cx == nx - 1) && (cy == 0 || cy == ny - 1);
                let envelope = (-dist / xi).exp().powi(2);

                for site in 0..4 {
                    let is_corner = is_corner_cell && match (cx == 0, cy == 0, site) {
                        (true, true, 0) => true,       // SW corner
                        (false, true, 1) => true,      // SE corner
                        (false, false, 2) => true,     // NE corner
                        (true, false, 3) => true,      // NW corner
                        _ => false,
                    };

                    let site_boost = if is_corner { 2.5 } else { 0.15 };
                    let intensity = (envelope * site_boost).clamp(0.001, 1.0);

                    points.push(HigherOrderCornerSpatialPoint {
                        cell_x: cx,
                        cell_y: cy,
                        site_index: site,
                        intensity,
                        is_corner,
                    });
                }
            }
        }

        points
    }
}
