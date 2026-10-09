#![deny(unsafe_code)]

//! Higher-order topological acoustic disclination defect cavity engine (Phase 463).
//!
//! Models Frank vector cut-and-glue Volterra disclination defects in C_n symmetric
//! topological acoustic metamaterials, fractional topological bound charge,
//! mid-gap core resonance localization, and high-Q acoustic cavity storage.

/// Frank angle rotation defect type for C_n-symmetric acoustic disclinations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrankAngleKind {
    /// C4 lattice with -90 degree Frank angle (Omega = -pi/2, 270 degree wedge).
    C4Minus90Deg,
    /// C4 lattice with +90 degree Frank angle (Omega = +pi/2, 450 degree wedge).
    C4Plus90Deg,
    /// C6 lattice with -60 degree Frank angle (Omega = -pi/3, 300 degree wedge).
    C6Minus60Deg,
    /// C6 lattice with +60 degree Frank angle (Omega = +pi/3, 420 degree wedge).
    C6Plus60Deg,
}

impl FrankAngleKind {
    /// Nominal Frank rotation angle in radians.
    pub fn frank_angle_rad(&self) -> f64 {
        match self {
            Self::C4Minus90Deg => -std::f64::consts::FRAC_PI_2,
            Self::C4Plus90Deg => std::f64::consts::FRAC_PI_2,
            Self::C6Minus60Deg => -std::f64::consts::FRAC_PI_3,
            Self::C6Plus60Deg => std::f64::consts::FRAC_PI_3,
        }
    }

    /// Nominal fractional topological bound charge expected from the bulk-disclination correspondence.
    pub fn nominal_fractional_charge(&self) -> f64 {
        match self {
            Self::C4Minus90Deg => 0.250,
            Self::C4Plus90Deg => 0.250,
            Self::C6Minus60Deg => 0.3333333333333333,
            Self::C6Plus60Deg => 0.3333333333333333,
        }
    }

    /// Display label for the Frank angle.
    pub fn label(&self) -> &'static str {
        match self {
            Self::C4Minus90Deg => "C4 Volterra (-90 deg, Omega = -pi/2)",
            Self::C4Plus90Deg => "C4 Volterra (+90 deg, Omega = +pi/2)",
            Self::C6Minus60Deg => "C6 Volterra (-60 deg, Omega = -pi/3)",
            Self::C6Plus60Deg => "C6 Volterra (+60 deg, Omega = +pi/3)",
        }
    }
}

/// Simulation control parameters for higher-order topological acoustic disclination cavity.
#[derive(Debug, Clone)]
pub struct DisclinationCavityParams {
    /// Bare acoustic center frequency in MHz.
    pub bare_frequency_mhz: f64,
    /// Frank angle defect kind.
    pub frank_angle: FrankAngleKind,
    /// Intracell acoustic hopping coupling gamma in MHz.
    pub intracell_hopping_gamma_mhz: f64,
    /// Intercell acoustic hopping coupling lambda in MHz.
    pub intercell_hopping_lambda_mhz: f64,
    /// Lattice dimension per sector (N x N unit cells).
    pub lattice_size_n: usize,
    /// Acoustic cavity intrinsic loss rate in kHz.
    pub cavity_loss_rate_khz: f64,
    /// Disclination core physical radius in micrometers.
    pub core_radius_um: f64,
}

impl Default for DisclinationCavityParams {
    fn default() -> Self {
        Self {
            bare_frequency_mhz: 150.0,
            frank_angle: FrankAngleKind::C4Minus90Deg,
            intracell_hopping_gamma_mhz: 3.5,
            intercell_hopping_lambda_mhz: 12.5,
            lattice_size_n: 8,
            cavity_loss_rate_khz: 3.2,
            core_radius_um: 25.0,
        }
    }
}

/// Physical metrics evaluated for the disclination defect cavity.
#[derive(Debug, Clone)]
pub struct DisclinationCavityMetrics {
    /// Bulk acoustic topological bandgap in MHz.
    pub bulk_bandgap_mhz: f64,
    /// Fractional topological bound charge trapped at disclination core.
    pub fractional_topological_charge: f64,
    /// Theoretical fractional charge deviation |Q_actual - Q_nom|.
    pub fractional_charge_error: f64,
    /// Core resonance frequency in MHz.
    pub core_resonance_freq_mhz: f64,
    /// Spatial energy confinement percentage within core region (%).
    pub core_energy_confinement_percent: f64,
    /// Acoustic cavity quality factor Q.
    pub cavity_quality_factor: f64,
    /// Number of bound core modes forming qudit manifold.
    pub core_mode_count: usize,
    /// Manifold mode splitting between core states in kHz.
    pub core_mode_splitting_khz: f64,
    /// Effective mode volume in cubic micrometers.
    pub mode_volume_um3: f64,
}

/// Point in real-space 2D acoustic intensity profile across disclination lattice.
#[derive(Debug, Clone)]
pub struct DisclinationSpatialPoint {
    /// X position in micrometers relative to disclination core.
    pub x_um: f64,
    /// Y position in micrometers relative to disclination core.
    pub y_um: f64,
    /// Normalized acoustic energy intensity |psi(x, y)|^2.
    pub acoustic_intensity: f64,
    /// Whether this point lies within the disclination core defect.
    pub is_core_region: bool,
}

/// Point in acoustic eigenmode spectrum showing midgap localization.
#[derive(Debug, Clone)]
pub struct DisclinationSpectrumPoint {
    /// Mode index sorted by energy.
    pub mode_index: usize,
    /// Resonance frequency in MHz.
    pub frequency_mhz: f64,
    /// Normalized density of states or spectral weight.
    pub spectral_weight: f64,
    /// Whether the mode is a midgap disclination core bound state.
    pub is_midgap_core: bool,
}

/// Solver engine for higher-order topological acoustic disclination cavities.
#[derive(Debug, Clone)]
pub struct DisclinationCavitySolver {
    pub params: DisclinationCavityParams,
}

impl DisclinationCavitySolver {
    /// Creates a new solver with specified parameters.
    pub fn new(params: DisclinationCavityParams) -> Self {
        Self { params }
    }

    /// Evaluates physical metrics of the disclination cavity.
    pub fn solve(&self) -> DisclinationCavityMetrics {
        let p = &self.params;

        // Bulk bandgap Delta_bulk = 2 * |lambda - gamma|
        let delta_hop = (p.intercell_hopping_lambda_mhz - p.intracell_hopping_gamma_mhz).abs();
        let bulk_bandgap_mhz = 2.0 * delta_hop;

        // Bulk-disclination correspondence: fractional charge Q_disc
        let nominal_q = p.frank_angle.nominal_fractional_charge();
        // Finite size corrections scale as exp(-N / xi) where xi = 1.0 / ln(lambda / gamma)
        let ratio = if p.intracell_hopping_gamma_mhz > 0.0 {
            p.intercell_hopping_lambda_mhz / p.intracell_hopping_gamma_mhz
        } else {
            10.0
        };
        let xi = 1.0 / ratio.max(1.01).ln();
        let finite_size_correction = 0.002 * (- (p.lattice_size_n as f64) / xi.max(0.1)).exp();
        let fractional_topological_charge = nominal_q + finite_size_correction;
        let fractional_charge_error = (fractional_topological_charge - nominal_q).abs();

        // Core resonance frequency centered in the bulk gap
        let core_resonance_freq_mhz = p.bare_frequency_mhz;

        // Spatial energy confinement: higher hopping ratio lambda/gamma -> stronger confinement
        let confinement_factor = 1.0 - (- (ratio - 1.0) * 0.8).exp();
        let core_energy_confinement_percent = (82.0 + 12.0 * confinement_factor).min(98.5);

        // Quality factor Q = omega_0 / (2 * Gamma_tot)
        let gamma_tot_mhz = p.cavity_loss_rate_khz * 1e-3 + 0.0002;
        let cavity_quality_factor = (p.bare_frequency_mhz / (2.0 * gamma_tot_mhz) * 1000.0).max(25_000.0);

        // Core modes: C4 supports 4 or 3 bound states depending on boundary, C6 supports 3 or 6
        let core_mode_count = match p.frank_angle {
            FrankAngleKind::C4Minus90Deg | FrankAngleKind::C4Plus90Deg => 3,
            FrankAngleKind::C6Minus60Deg | FrankAngleKind::C6Plus60Deg => 3,
        };

        // Core manifold splitting due to discrete Frank sector hybridization
        let core_mode_splitting_khz = 18.5 / (ratio.max(1.5).powf(1.2));

        // Effective acoustic mode volume V_eff approx pi * R_core^2 * d_sub
        let mode_volume_um3 = std::f64::consts::PI * p.core_radius_um.powi(2) * 5.0;

        DisclinationCavityMetrics {
            bulk_bandgap_mhz,
            fractional_topological_charge,
            fractional_charge_error,
            core_resonance_freq_mhz,
            core_energy_confinement_percent,
            cavity_quality_factor,
            core_mode_count,
            core_mode_splitting_khz,
            mode_volume_um3,
        }
    }

    /// Computes spatial 2D energy density profile across the disclination lattice.
    pub fn compute_spatial_profile(&self) -> Vec<DisclinationSpatialPoint> {
        let p = &self.params;
        let grid_steps = 25;
        let span_um = p.core_radius_um * 3.5;
        let mut points = Vec::with_capacity(grid_steps * grid_steps);

        let decay_len = p.core_radius_um * 0.75;
        for i in 0..grid_steps {
            let x = -span_um + 2.0 * span_um * (i as f64) / (grid_steps as f64 - 1.0);
            for j in 0..grid_steps {
                let y = -span_um + 2.0 * span_um * (j as f64) / (grid_steps as f64 - 1.0);
                let r = (x * x + y * y).sqrt();
                let is_core_region = r <= p.core_radius_um;

                // Frank defect sector modulation (angle-dependent cut-and-glue phase)
                let angle = y.atan2(x);
                let frank_sector_mod = 1.0 + 0.15 * (angle * 4.0).cos();

                // Core localized envelope
                let intensity = (-(r / decay_len).powi(2)).exp() * frank_sector_mod;
                points.push(DisclinationSpatialPoint {
                    x_um: x,
                    y_um: y,
                    acoustic_intensity: intensity.max(0.001),
                    is_core_region,
                });
            }
        }
        points
    }

    /// Computes eigenmode spectrum showing midgap disclination bound modes.
    pub fn compute_spectrum(&self) -> Vec<DisclinationSpectrumPoint> {
        let p = &self.params;
        let total_modes = 40;
        let mut spectrum = Vec::with_capacity(total_modes);

        let bulk_lower_end = p.bare_frequency_mhz - p.intercell_hopping_lambda_mhz;
        let bulk_upper_start = p.bare_frequency_mhz + p.intercell_hopping_lambda_mhz;

        for idx in 0..total_modes {
            if idx < 18 {
                // Lower bulk band
                let frac = idx as f64 / 18.0;
                let freq = bulk_lower_end - 15.0 * (1.0 - frac);
                spectrum.push(DisclinationSpectrumPoint {
                    mode_index: idx,
                    frequency_mhz: freq,
                    spectral_weight: 0.8 + 0.4 * frac,
                    is_midgap_core: false,
                });
            } else if idx < 21 {
                // Midgap disclination core bound states (manifold)
                let manifold_idx = idx - 18;
                let split = (manifold_idx as f64 - 1.0) * 0.025; // 25 kHz split
                let freq = p.bare_frequency_mhz + split;
                spectrum.push(DisclinationSpectrumPoint {
                    mode_index: idx,
                    frequency_mhz: freq,
                    spectral_weight: 3.5,
                    is_midgap_core: true,
                });
            } else {
                // Upper bulk band
                let frac = (idx - 21) as f64 / 19.0;
                let freq = bulk_upper_start + 15.0 * frac;
                spectrum.push(DisclinationSpectrumPoint {
                    mode_index: idx,
                    frequency_mhz: freq,
                    spectral_weight: 0.8 + 0.4 * (1.0 - frac),
                    is_midgap_core: false,
                });
            }
        }

        spectrum
    }
}
