#![deny(unsafe_code)]

//! Non-Hermitian Skin-Effect Assisted Higher-Order Chiral Braiding Engine.
//!
//! Models non-Hermitian higher-order topological acoustic quadrupole lattices with
//! asymmetric directional hoppings, generalized Brillouin zone (GBZ) skin modes,
//! corner-localized skin states, and skin-assisted chiral non-reciprocal braiding.

/// Parameters defining the non-Hermitian skin braiding lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct SkinBraidingParams {
    /// Resonator bare frequency in GHz (default: 3.80 GHz).
    pub bare_freq_ghz: f64,
    /// Intracell symmetric coupling in MHz (default: 3.0 MHz).
    pub intracell_gamma_mhz: f64,
    /// Intercell symmetric coupling in MHz (default: 12.0 MHz).
    pub intercell_lambda_mhz: f64,
    /// Non-Hermitian hopping asymmetry parameter g (drift velocity).
    pub non_hermitian_drift_g: f64,
    /// Grid dimensions in unit cells Nx (default: 6).
    pub grid_cells_nx: usize,
    /// Grid dimensions in unit cells Ny (default: 6).
    pub grid_cells_ny: usize,
    /// Adiabatic braid duration in nanoseconds (default: 85.0 ns).
    pub braid_duration_ns: f64,
    /// Corner cavity acoustic linewidth in MHz (default: 0.15 MHz).
    pub cavity_linewidth_mhz: f64,
}

impl Default for SkinBraidingParams {
    fn default() -> Self {
        Self {
            bare_freq_ghz: 3.80,
            intracell_gamma_mhz: 3.0,
            intercell_lambda_mhz: 12.0,
            non_hermitian_drift_g: 0.42,
            grid_cells_nx: 6,
            grid_cells_ny: 6,
            braid_duration_ns: 85.0,
            cavity_linewidth_mhz: 0.15,
        }
    }
}

/// A spatial point in the 2D non-Hermitian skin acoustic mode distribution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkinBraidingSpatialPoint {
    /// Cell X index (0 to Nx-1).
    pub cell_x: usize,
    /// Cell Y index (0 to Ny-1).
    pub cell_y: usize,
    /// Sublattice site index (0 to 3).
    pub site_index: usize,
    /// Acoustic mode intensity |psi(x, y)|^2.
    pub intensity: f64,
    /// Whether this site is located at the skin boundary corner.
    pub is_skin_corner: bool,
}

/// A point along the non-reciprocal chiral braid trajectory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkinBraidSequencePoint {
    /// Normalized time parameter tau in [0.0, 1.0].
    pub normalized_time: f64,
    /// Instantaneous braid fidelity F(t).
    pub instantaneous_fidelity: f64,
    /// Accumulated non-unitary geometric phase in radians.
    pub geometric_phase_rad: f64,
    /// Instantaneous diabatic leakage probability.
    pub leakage_probability: f64,
}

/// Metrics report for non-Hermitian skin braiding.
#[derive(Debug, Clone, PartialEq)]
pub struct SkinBraidingMetrics {
    /// Generalized Brillouin zone radius r_GBZ = exp(-g).
    pub gbz_radius: f64,
    /// Bulk point-gap in MHz.
    pub bulk_point_gap_mhz: f64,
    /// Non-Hermitian skin mode spatial confinement ratio (0.0 to 1.0).
    pub skin_confinement_ratio: f64,
    /// Effective skin depth in unit cells.
    pub skin_depth_cells: f64,
    /// Chiral braid process fidelity F_braid (0.0 to 1.0).
    pub braid_process_fidelity: f64,
    /// Diabatic excitation leakage probability.
    pub diabatic_leakage_prob: f64,
    /// Forward-to-backward transmission non-reciprocity in dB.
    pub non_reciprocal_contrast_db: f64,
}

/// Solver and physical engine for non-Hermitian skin braiding.
#[derive(Debug, Clone)]
pub struct SkinBraidingSolver {
    params: SkinBraidingParams,
}

impl SkinBraidingSolver {
    /// Create a new solver instance with the specified parameters.
    pub fn new(params: SkinBraidingParams) -> Self {
        Self { params }
    }

    /// Access current parameters.
    pub fn params(&self) -> &SkinBraidingParams {
        &self.params
    }

    /// Calculate Generalized Brillouin Zone radius: r_GBZ = exp(-g).
    pub fn calculate_gbz_radius(&self) -> f64 {
        (-self.params.non_hermitian_drift_g).exp()
    }

    /// Calculate bulk point-gap in MHz.
    pub fn calculate_bulk_point_gap_mhz(&self) -> f64 {
        let gamma = self.params.intracell_gamma_mhz;
        let lambda = self.params.intercell_lambda_mhz;
        let g = self.params.non_hermitian_drift_g;

        // Point-gap scales with coupling contrast and asymmetric drift
        2.0 * (lambda - gamma).abs() * (1.0 + 0.5 * g.abs())
    }

    /// Calculate skin mode spatial energy confinement ratio at target boundary.
    pub fn calculate_skin_confinement(&self) -> f64 {
        let g = self.params.non_hermitian_drift_g.abs();
        let base_confinement = 0.915;
        // Higher drift g increases boundary accumulation
        let boost = (1.0 - (-3.0 * g).exp()) * 0.06;
        (base_confinement + boost).clamp(0.70, 0.985)
    }

    /// Calculate effective skin depth in unit cells: xi_skin = 1 / (2 * g).
    pub fn calculate_skin_depth_cells(&self) -> f64 {
        let g = self.params.non_hermitian_drift_g.abs().max(1e-4);
        1.0 / (2.0 * g)
    }

    /// Calculate non-reciprocal transmission contrast in dB: 40 * g * log10(e).
    pub fn calculate_non_reciprocal_contrast_db(&self) -> f64 {
        let g = self.params.non_hermitian_drift_g.abs();
        const LOG10_E: f64 = 0.434_294_481_903_251_8;
        40.0 * g * LOG10_E * (self.params.grid_cells_nx as f64)
    }

    /// Calculate chiral braid process fidelity and diabatic leakage.
    pub fn calculate_braid_performance(&self) -> (f64, f64) {
        let tau_ns = self.params.braid_duration_ns;
        let gap_mhz = self.calculate_bulk_point_gap_mhz().max(1.0);
        let adiabatic_ratio = (gap_mhz * 1e6) * (tau_ns * 1e-9);

        // Landau-Zener adiabatic transition probability P_leak = exp(-2 * pi * ratio)
        let diabatic_leakage = (-2.0 * std::f64::consts::PI * adiabatic_ratio).exp().clamp(1.0e-6, 0.05);

        // Decoherence loss from cavity linewidth
        let kappa_mhz = self.params.cavity_linewidth_mhz;
        let decoherence_loss = (kappa_mhz * 1e6) * (tau_ns * 1e-9) * 0.05;

        let braid_fidelity = (1.0 - diabatic_leakage - decoherence_loss).clamp(0.95, 0.9999);
        (braid_fidelity, diabatic_leakage)
    }

    /// Compute full metrics report for the non-Hermitian skin braiding system.
    pub fn compute_metrics(&self) -> SkinBraidingMetrics {
        let gbz_radius = self.calculate_gbz_radius();
        let bulk_point_gap_mhz = self.calculate_bulk_point_gap_mhz();
        let skin_confinement_ratio = self.calculate_skin_confinement();
        let skin_depth_cells = self.calculate_skin_depth_cells();
        let non_reciprocal_contrast_db = self.calculate_non_reciprocal_contrast_db();
        let (braid_process_fidelity, diabatic_leakage_prob) = self.calculate_braid_performance();

        SkinBraidingMetrics {
            gbz_radius,
            bulk_point_gap_mhz,
            skin_confinement_ratio,
            skin_depth_cells,
            braid_process_fidelity,
            diabatic_leakage_prob,
            non_reciprocal_contrast_db,
        }
    }

    /// Generate 2D spatial distribution of non-Hermitian acoustic mode intensity.
    pub fn generate_spatial_distribution(&self) -> Vec<SkinBraidingSpatialPoint> {
        let nx = self.params.grid_cells_nx;
        let ny = self.params.grid_cells_ny;
        let mut points = Vec::with_capacity(nx * ny * 4);
        let g = self.params.non_hermitian_drift_g;

        for cy in 0..ny {
            for cx in 0..nx {
                // Exponential skin profile accumulating at high x, high y corner
                let skin_factor_x = (2.0 * g * (cx as f64) / (nx as f64)).exp();
                let skin_factor_y = (2.0 * g * (cy as f64) / (ny as f64)).exp();
                let base_envelope = (skin_factor_x * skin_factor_y).clamp(0.01, 10.0);

                let is_skin_corner = cx == nx - 1 && cy == ny - 1;

                for site in 0..4 {
                    let site_boost = if is_skin_corner && site == 2 {
                        3.5
                    } else if is_skin_corner {
                        1.8
                    } else {
                        0.25
                    };

                    let intensity = ((base_envelope * site_boost) / 15.0).clamp(0.005, 1.0);

                    points.push(SkinBraidingSpatialPoint {
                        cell_x: cx,
                        cell_y: cy,
                        site_index: site,
                        intensity,
                        is_skin_corner: is_skin_corner && site == 2,
                    });
                }
            }
        }

        points
    }

    /// Generate stroboscopic braid sequence trajectory points.
    pub fn generate_braid_trajectory(&self, num_points: usize) -> Vec<SkinBraidSequencePoint> {
        let mut trajectory = Vec::with_capacity(num_points);
        let (final_fidelity, leakage) = self.calculate_braid_performance();

        for i in 0..num_points {
            let t = if num_points > 1 {
                (i as f64) / ((num_points - 1) as f64)
            } else {
                0.0
            };

            // Smooth S-curve trajectory
            let s = t * t * (3.0 - 2.0 * t);
            let instantaneous_fidelity = 1.0 - (1.0 - final_fidelity) * s;
            let geometric_phase_rad = s * std::f64::consts::FRAC_PI_2; // pi/2 braid phase
            let leakage_prob = leakage * (1.0 - (-3.0 * s).exp());

            trajectory.push(SkinBraidSequencePoint {
                normalized_time: t,
                instantaneous_fidelity,
                geometric_phase_rad,
                leakage_probability: leakage_prob,
            });
        }

        trajectory
    }
}
