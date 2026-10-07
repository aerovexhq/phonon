#![deny(unsafe_code)]

//! Second-order topological insulator (SOTI) corner-polariton micro-cavity modes.
//!
//! Models sub-diffraction 0D corner acoustic polariton modes in a 2D quadrupole
//! phononic crystal lattice with ultra-small mode volume and anomalous dispersion.

#[derive(Debug, Clone, PartialEq)]
pub struct CornerPolaritonParams {
    /// Number of unit cells along each dimension.
    pub lattice_size: usize,
    /// Intracell coupling strength in MHz (gamma).
    pub intracell_coupling_gamma_mhz: f64,
    /// Intercell coupling strength in MHz (lambda).
    pub intercell_coupling_lambda_mhz: f64,
    /// Bare cavity resonance frequency in GHz.
    pub cavity_resonance_ghz: f64,
    /// Intrinsic acoustic dissipation linewidth in kHz.
    pub intrinsic_loss_khz: f64,
    /// Effective sub-wavelength corner mode volume in cubic micrometers.
    pub corner_mode_volume_um3: f64,
    /// Second-order acoustic Kerr non-linear index n2 (m^2/W).
    pub kerr_index_n2_m2_w: f64,
    /// Group velocity dispersion beta2 in ps^2/mm (negative indicates anomalous dispersion).
    pub gvd_beta2_ps2_mm: f64,
}

impl Default for CornerPolaritonParams {
    fn default() -> Self {
        Self {
            lattice_size: 6,
            intracell_coupling_gamma_mhz: 2.0,
            intercell_coupling_lambda_mhz: 10.0,
            cavity_resonance_ghz: 2.45,
            intrinsic_loss_khz: 15.0,
            corner_mode_volume_um3: 0.045,
            kerr_index_n2_m2_w: 2.5e-17,
            gvd_beta2_ps2_mm: -0.35,
        }
    }
}

/// Modal profile and topological characteristics of a corner-polariton state.
#[derive(Debug, Clone, PartialEq)]
pub struct CornerModeProfile {
    /// Mode identification index.
    pub mode_index: usize,
    /// Resonant eigenfrequency in GHz.
    pub energy_ghz: f64,
    /// Fractional energy confined within the 4 outer corner unit cells.
    pub corner_confinement_ratio: f64,
    /// Loaded acoustic quality factor Q = omega / (2 * gamma).
    pub quality_factor: f64,
    /// Purcell non-linear enhancement factor Q / V_mode (normalized to cubic wavelengths).
    pub purcell_enhancement: f64,
    /// 2D normalized spatial amplitude distribution across the grid.
    pub spatial_amplitudes: Vec<Vec<f64>>,
}

/// SOTI Quadrupole corner-polariton cavity solver.
#[derive(Debug, Clone)]
pub struct CornerPolaritonCavityEngine {
    pub params: CornerPolaritonParams,
}

impl CornerPolaritonCavityEngine {
    pub fn new(params: CornerPolaritonParams) -> Self {
        Self { params }
    }

    /// Solves the 0D corner states of the 2D quadrupole phononic crystal.
    pub fn solve_corner_modes(&self) -> Vec<CornerModeProfile> {
        let n = self.params.lattice_size.max(4);
        let gamma = self.params.intracell_coupling_gamma_mhz;
        let lambda = self.params.intercell_coupling_lambda_mhz;

        // In the topological regime (lambda > gamma), the bulk bandgap is Delta = 2*(lambda - gamma).
        // 4 zero-energy (in-gap) corner modes emerge at the 4 corners of the square lattice.
        let decay_length = 1.0 / ((lambda / gamma.max(0.1)).ln().max(0.1));
        let quality_factor = (self.params.cavity_resonance_ghz * 1e6) / (2.0 * self.params.intrinsic_loss_khz);
        let wavelength_um = (3000.0 / (self.params.cavity_resonance_ghz * 1e3)) * 1000.0;
        let v_norm = self.params.corner_mode_volume_um3 / (wavelength_um * wavelength_um * wavelength_um).max(1e-12);
        let purcell = quality_factor / v_norm.max(1e-6);

        let mut modes = Vec::with_capacity(4);
        let corner_locations = [(0, 0), (0, n - 1), (n - 1, 0), (n - 1, n - 1)];

        for (idx, &(cx, cy)) in corner_locations.iter().enumerate() {
            let mut grid = vec![vec![0.0f64; n]; n];
            let mut corner_energy = 0.0f64;
            let mut total_energy = 0.0f64;

            for r in 0..n {
                for c in 0..n {
                    let dx = (r as f64 - cx as f64).abs();
                    let dy = (c as f64 - cy as f64).abs();
                    let dist = (dx * dx + dy * dy).sqrt();
                    let amp = (-dist / decay_length).exp();
                    let energy = amp * amp;
                    grid[r][c] = energy;
                    total_energy += energy;
                    if (r == 0 || r == n - 1) && (c == 0 || c == n - 1) {
                        corner_energy += energy;
                    }
                }
            }

            // Normalize grid
            if total_energy > 1e-12 {
                for r in 0..n {
                    for c in 0..n {
                        grid[r][c] /= total_energy;
                    }
                }
            }

            let corner_confinement_ratio = if total_energy > 0.0 {
                (corner_energy / total_energy).clamp(0.0, 1.0)
            } else {
                0.95
            };

            // Small hybridization frequency splitting between the 4 corners
            let splitting_ghz = 0.00015 * (-((n as f64) / decay_length)).exp() * (idx as f64 - 1.5);
            let energy_ghz = self.params.cavity_resonance_ghz + splitting_ghz;

            modes.push(CornerModeProfile {
                mode_index: idx,
                energy_ghz,
                corner_confinement_ratio,
                quality_factor,
                purcell_enhancement: purcell,
                spatial_amplitudes: grid,
            });
        }

        modes
    }

    /// Evaluates topological bulk bandgap in MHz.
    pub fn bulk_bandgap_mhz(&self) -> f64 {
        2.0 * (self.params.intercell_coupling_lambda_mhz - self.params.intracell_coupling_gamma_mhz).abs()
    }

    /// Verifies whether the system resides in the higher-order topological insulator phase.
    pub fn is_topological(&self) -> bool {
        self.params.intercell_coupling_lambda_mhz > self.params.intracell_coupling_gamma_mhz
    }

    /// Evaluates anomalous dispersion parameter D2 = -c/(2*pi*R) * beta2 in kHz.
    pub fn dispersion_parameter_d2_khz(&self) -> f64 {
        // Anomalous dispersion (beta2 < 0) corresponds to positive D2 > 0
        -self.params.gvd_beta2_ps2_mm * 1.5e3
    }
}
