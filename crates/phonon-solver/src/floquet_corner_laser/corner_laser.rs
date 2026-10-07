#![deny(unsafe_code)]

//! Topological Acoustic Higher-Order Corner-State Laser Engine.
//!
//! Models a non-Hermitian second-order topological insulator (SOTI) with corner-selective
//! acoustic gain and distributed bulk loss. Evaluates selective 0D topological corner mode
//! lasing, acoustic population inversion, threshold pump power, and high side-mode
//! suppression ratio (SMSR >= 35 dB).

use std::f64::consts::PI;

/// Parameters defining the topological acoustic corner laser.
#[derive(Debug, Clone)]
pub struct CornerLaserParams {
    /// Intracell hopping amplitude gamma in MHz (default ~2.0 MHz).
    pub intracell_gamma_mhz: f64,
    /// Intercell hopping amplitude lambda in MHz (default ~8.0 MHz, topological regime lambda > gamma).
    pub intercell_lambda_mhz: f64,
    /// Bare acoustic resonance frequency in GHz (default ~1.2 GHz).
    pub bare_frequency_ghz: f64,
    /// Corner localized acoustic pump gain rate in MHz (default ~6.0 MHz).
    pub corner_gain_mhz: f64,
    /// Distributed bulk acoustic loss rate in MHz (default ~3.5 MHz).
    pub bulk_loss_mhz: f64,
    /// Non-linear acoustic gain saturation power in mW (default ~15.0 mW).
    pub saturation_power_mw: f64,
    /// Input pump power in mW (default ~25.0 mW).
    pub pump_power_mw: f64,
    /// Cavity acoustic quality factor Q (default ~8,000).
    pub quality_factor: f64,
    /// Grid dimensions (Nx x Ny unit cells, default 6x6).
    pub grid_nx: usize,
    pub grid_ny: usize,
}

impl Default for CornerLaserParams {
    fn default() -> Self {
        Self {
            intracell_gamma_mhz: 2.0,
            intercell_lambda_mhz: 8.0,
            bare_frequency_ghz: 1.2,
            corner_gain_mhz: 6.0,
            bulk_loss_mhz: 3.5,
            saturation_power_mw: 15.0,
            pump_power_mw: 25.0,
            quality_factor: 8000.0,
            grid_nx: 6,
            grid_ny: 6,
        }
    }
}

/// Emission spectral point of the acoustic corner laser.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaserSpectralPoint {
    /// Detuning from bare resonance in MHz.
    pub detuning_mhz: f64,
    /// Spectral intensity in arbitrary units / dB.
    pub intensity_db: f64,
    /// Phase in radians.
    pub phase_rad: f64,
}

/// 0D Localized topological corner mode characteristics.
#[derive(Debug, Clone)]
pub struct CornerLasingMode {
    /// Corner identifier (Corner 1: (0,0), Corner 2: (Nx, 0), Corner 3: (0, Ny), Corner 4: (Nx, Ny)).
    pub corner_id: usize,
    /// Lasing frequency in GHz.
    pub frequency_ghz: f64,
    /// Net modal gain (linear gain minus intrinsic losses) in MHz.
    pub net_gain_mhz: f64,
    /// Spatial modal confinement ratio within corner unit cell (>= 85%).
    pub confinement_ratio: f64,
    /// Output acoustic coherent power in mW.
    pub output_power_mw: f64,
}

/// Solver for topological acoustic corner-state lasing.
#[derive(Debug, Clone)]
pub struct CornerLaserSolver {
    pub params: CornerLaserParams,
}

impl CornerLaserSolver {
    pub fn new(params: CornerLaserParams) -> Self {
        Self { params }
    }

    /// Evaluates whether the metamaterial lattice is in the topological higher-order phase (lambda > gamma).
    pub fn is_topological(&self) -> bool {
        self.params.intercell_lambda_mhz > self.params.intracell_gamma_mhz
    }

    /// Evaluates the bulk acoustic bandgap Delta_bulk = 2 * |lambda - gamma| in MHz.
    pub fn bulk_bandgap_mhz(&self) -> f64 {
        2.0 * (self.params.intercell_lambda_mhz - self.params.intracell_gamma_mhz).abs()
    }

    /// Evaluates the lasing threshold pump power P_th in mW.
    /// P_th = P_sat * (bulk_loss / corner_gain).
    pub fn threshold_pump_power_mw(&self) -> f64 {
        if self.params.corner_gain_mhz <= 1e-6 {
            return f64::INFINITY;
        }
        let intrinsic_cavity_loss = (self.params.bare_frequency_ghz * 1e3) / self.params.quality_factor;
        let total_loss = self.params.bulk_loss_mhz * 0.4 + intrinsic_cavity_loss;
        self.params.saturation_power_mw * (total_loss / self.params.corner_gain_mhz)
    }

    /// Evaluates steady-state coherent acoustic output power P_out in mW.
    /// Above threshold: P_out = P_sat * (P_pump / P_th - 1.0) * eta_slope.
    pub fn calculate_output_power_mw(&self, pump_power_mw: f64) -> f64 {
        let p_th = self.threshold_pump_power_mw();
        if pump_power_mw <= p_th {
            // Below threshold: spontaneous thermal acoustic emission
            0.02 * (pump_power_mw / (p_th + 1e-3))
        } else {
            let slope_efficiency = 0.45; // 45% acoustic differential quantum efficiency
            let stimulated = self.params.saturation_power_mw * (pump_power_mw / p_th - 1.0) * slope_efficiency;
            stimulated.max(0.0)
        }
    }

    /// Evaluates the Side-Mode Suppression Ratio (SMSR) in dB.
    /// Topological corner mode has preferential gain while bulk modes experience bulk loss.
    pub fn calculate_smsr_db(&self) -> f64 {
        let p_th = self.threshold_pump_power_mw();
        if self.params.pump_power_mw <= p_th {
            return 8.0; // Spontaneous regime
        }
        let ratio = self.params.pump_power_mw / p_th;
        let base_smsr = 32.0;
        let dynamic_boost = 6.0 * ratio.ln().max(0.0);
        (base_smsr + dynamic_boost).min(55.0)
    }

    /// Evaluates the Schawlow-Townes linewidth of the acoustic corner laser in kHz.
    pub fn calculate_laser_linewidth_khz(&self) -> f64 {
        let p_out = self.calculate_output_power_mw(self.params.pump_power_mw);
        let passive_linewidth_khz = (self.params.bare_frequency_ghz * 1e6) / self.params.quality_factor;
        if p_out <= 0.05 {
            passive_linewidth_khz
        } else {
            let st_linewidth = passive_linewidth_khz / (1.0 + p_out * 12.0);
            st_linewidth.max(0.12)
        }
    }

    /// Solves the 4 localized 0D corner modes.
    pub fn solve_corner_modes(&self) -> Vec<CornerLasingMode> {
        let p_th = self.threshold_pump_power_mw();
        let p_out = self.calculate_output_power_mw(self.params.pump_power_mw);
        let net_gain = if self.params.pump_power_mw > p_th {
            self.params.corner_gain_mhz - self.params.bulk_loss_mhz * 0.2
        } else {
            -self.params.bulk_loss_mhz * 0.5
        };

        let confinement = if self.is_topological() {
            let ratio = self.params.intercell_lambda_mhz / self.params.intracell_gamma_mhz.max(1e-3);
            (0.85 + 0.04 * ratio.min(4.0)).min(0.98)
        } else {
            0.25 // Extended / bulk mode in trivial regime
        };

        let mut modes = Vec::with_capacity(4);
        for id in 0..4 {
            // Small symmetric frequency split among the 4 corners due to finite-size coupling
            let split_ghz = match id {
                0 => 0.0,
                1 => 0.0001,
                2 => -0.0001,
                _ => 0.00005,
            };
            modes.push(CornerLasingMode {
                corner_id: id + 1,
                frequency_ghz: self.params.bare_frequency_ghz + split_ghz,
                net_gain_mhz: net_gain,
                confinement_ratio: confinement,
                output_power_mw: p_out * 0.25, // Divided equally among active corners
            });
        }
        modes
    }

    /// Generates high-resolution emission spectrum around bare resonance.
    pub fn generate_emission_spectrum(&self, num_points: usize) -> Vec<LaserSpectralPoint> {
        let span_mhz = 25.0;
        let p_out = self.calculate_output_power_mw(self.params.pump_power_mw);
        let lw_mhz = self.calculate_laser_linewidth_khz() * 1e-3;
        let smsr_db = self.calculate_smsr_db();
        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let detuning = -span_mhz + (2.0 * span_mhz * i as f64) / (num_points - 1).max(1) as f64;
            // Central lasing peak (Lorentzian profile)
            let peak_intensity = p_out / (1.0 + (detuning / (lw_mhz * 0.5)).powi(2));
            // Bulk sub-threshold modes at +/- 8 MHz
            let bulk_peak1 = (p_out * 10.0_f64.powf(-smsr_db / 10.0)) / (1.0 + ((detuning - 8.5) / 1.5).powi(2));
            let bulk_peak2 = (p_out * 10.0_f64.powf(-smsr_db / 10.0)) / (1.0 + ((detuning + 8.5) / 1.5).powi(2));
            let noise_floor = 1e-5;

            let total_linear = peak_intensity + bulk_peak1 + bulk_peak2 + noise_floor;
            let intensity_db = 10.0 * total_linear.max(1e-8).log10();
            let phase_rad = (-detuning / 3.0).atan();

            points.push(LaserSpectralPoint {
                detuning_mhz: detuning,
                intensity_db,
                phase_rad,
            });
        }

        points
    }

    /// Evaluates 2D real-space acoustic intensity field across Nx x Ny unit cells (4 sites per unit cell).
    pub fn generate_realspace_intensity(&self) -> Vec<Vec<f64>> {
        let total_nx = self.params.grid_nx * 2;
        let total_ny = self.params.grid_ny * 2;
        let mut field = vec![vec![0.0; total_nx]; total_ny];

        let decay_length = if self.is_topological() {
            1.2 // Tight corner decay
        } else {
            8.0 // Delocalized
        };

        for y in 0..total_ny {
            for x in 0..total_nx {
                // Distance to nearest of 4 corners
                let d_c1 = ((x as f64).powi(2) + (y as f64).powi(2)).sqrt();
                let d_c2 = (((total_nx - 1 - x) as f64).powi(2) + (y as f64).powi(2)).sqrt();
                let d_c3 = ((x as f64).powi(2) + ((total_ny - 1 - y) as f64).powi(2)).sqrt();
                let d_c4 = (((total_nx - 1 - x) as f64).powi(2) + ((total_ny - 1 - y) as f64).powi(2)).sqrt();

                let val1 = (-d_c1 / decay_length).exp();
                let val2 = (-d_c2 / decay_length).exp();
                let val3 = (-d_c3 / decay_length).exp();
                let val4 = (-d_c4 / decay_length).exp();

                let combined = (val1 + val2 + val3 + val4).powi(2);
                field[y][x] = combined.min(1.0);
            }
        }

        field
    }
}
