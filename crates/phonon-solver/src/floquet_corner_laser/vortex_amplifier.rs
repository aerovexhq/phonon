#![deny(unsafe_code)]

//! Non-Hermitian Floquet Acoustic Vortex Beam Amplifier.
//!
//! Models spatio-temporal Floquet gain-loss modulation breaking time-reversal symmetry,
//! producing non-reciprocal chiral amplification of acoustic orbital angular momentum (OAM)
//! vortex beams with high mode purity (>= 90%) and forward gain >= 22.0 dB.

use std::f64::consts::PI;

/// Orbital angular momentum (OAM) topological charge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VortexOamCharge {
    MinusTwo,
    MinusOne,
    Zero,
    PlusOne,
    PlusTwo,
}

impl VortexOamCharge {
    pub fn charge_value(&self) -> i32 {
        match self {
            Self::MinusTwo => -2,
            Self::MinusOne => -1,
            Self::Zero => 0,
            Self::PlusOne => 1,
            Self::PlusTwo => 2,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::MinusTwo => "l = -2 (Left-Handed Double Vortex)",
            Self::MinusOne => "l = -1 (Left-Handed Chiral Vortex)",
            Self::Zero => "l = 0 (Fundamental Gaussian Beam)",
            Self::PlusOne => "l = +1 (Right-Handed Chiral Vortex)",
            Self::PlusTwo => "l = +2 (Right-Handed Double Vortex)",
        }
    }
}

/// Parameters for the Floquet non-Hermitian vortex amplifier.
#[derive(Debug, Clone)]
pub struct VortexAmplifierParams {
    /// Floquet modulation frequency in MHz (default ~85.0 MHz).
    pub modulation_freq_mhz: f64,
    /// Static background gain rate in MHz (default ~4.0 MHz).
    pub static_gain_mhz: f64,
    /// Spatio-temporal dynamic modulation amplitude in MHz (default ~8.5 MHz).
    pub dynamic_modulation_amplitude_mhz: f64,
    /// Target vortex topological charge l (default PlusOne).
    pub target_charge: VortexOamCharge,
    /// Amplifier acoustic interaction length in mm (default ~24.0 mm).
    pub interaction_length_mm: f64,
    /// Acoustic phase velocity in m/s (default ~3,400 m/s for LiNbO3/AlN).
    pub acoustic_velocity_ms: f64,
    /// Input vortex signal power in uW (default ~50.0 uW).
    pub input_signal_power_uw: f64,
}

impl Default for VortexAmplifierParams {
    fn default() -> Self {
        Self {
            modulation_freq_mhz: 85.0,
            static_gain_mhz: 4.0,
            dynamic_modulation_amplitude_mhz: 8.5,
            target_charge: VortexOamCharge::PlusOne,
            interaction_length_mm: 24.0,
            acoustic_velocity_ms: 3400.0,
            input_signal_power_uw: 50.0,
        }
    }
}

/// Amplification spectrum point across acoustic frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VortexAmplifierPoint {
    /// Frequency detuning in MHz.
    pub detuning_mhz: f64,
    /// Forward vortex power gain in dB.
    pub forward_gain_db: f64,
    /// Reverse vortex transmission / attenuation in dB.
    pub reverse_gain_db: f64,
    /// Directional isolation contrast in dB.
    pub isolation_contrast_db: f64,
}

/// Spatial 2D vortex profile point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VortexSpatialPoint {
    pub radius_mm: f64,
    pub azimuth_rad: f64,
    pub intensity: f64,
    pub phase_rad: f64,
}

/// Solver for Floquet non-Hermitian acoustic vortex beam amplification.
#[derive(Debug, Clone)]
pub struct VortexAmplifierSolver {
    pub params: VortexAmplifierParams,
}

impl VortexAmplifierSolver {
    pub fn new(params: VortexAmplifierParams) -> Self {
        Self { params }
    }

    /// Evaluates phase-matched forward power gain G_forward in dB.
    pub fn forward_power_gain_db(&self) -> f64 {
        let charge = self.params.target_charge.charge_value().abs() as f64;
        let effective_gain_mhz = self.params.static_gain_mhz + self.params.dynamic_modulation_amplitude_mhz * 0.85;
        let length_factor = self.params.interaction_length_mm / 10.0;
        let charge_degradation = if charge == 0.0 { 0.9 } else { 1.0 / (1.0 + 0.05 * charge) };
        let gain_linear = (effective_gain_mhz * 0.28 * length_factor * charge_degradation).exp();
        let gain_db = 10.0 * gain_linear.max(1.0).log10();
        gain_db.clamp(0.0, 35.0)
    }

    /// Evaluates phase-mismatched reverse transmission / gain G_reverse in dB.
    pub fn reverse_power_gain_db(&self) -> f64 {
        let mismatch_loss_db = -12.5;
        let residual_gain = (self.params.static_gain_mhz * 0.12 * (self.params.interaction_length_mm / 10.0)).exp();
        let net_db = 10.0 * residual_gain.log10() + mismatch_loss_db;
        net_db.clamp(-35.0, -0.5)
    }

    /// Evaluates directional non-reciprocal isolation contrast in dB.
    /// Contrast = G_forward - G_reverse >= 25.0 dB.
    pub fn isolation_contrast_db(&self) -> f64 {
        self.forward_power_gain_db() - self.reverse_power_gain_db()
    }

    /// Evaluates output orbital angular momentum (OAM) modal purity in percent (>= 90.0%).
    pub fn calculate_oam_purity_percent(&self) -> f64 {
        let charge = self.params.target_charge.charge_value().abs();
        let base_purity = match charge {
            0 => 96.5,
            1 => 94.2,
            2 => 91.5,
            _ => 88.0,
        };
        let length_bonus = (self.params.interaction_length_mm / 30.0).min(1.0) * 2.0;
        (base_purity + length_bonus).min(98.5)
    }

    /// Evaluates output acoustic amplified power in mW.
    pub fn calculate_output_power_mw(&self) -> f64 {
        let input_mw = self.params.input_signal_power_uw * 1e-3;
        let gain_linear = 10.0_f64.powf(self.forward_power_gain_db() / 10.0);
        input_mw * gain_linear
    }

    /// Generates forward and backward gain spectrum curves across detuning.
    pub fn generate_gain_spectrum(&self, num_points: usize) -> Vec<VortexAmplifierPoint> {
        let span_mhz = 40.0;
        let f_gain = self.forward_power_gain_db();
        let r_gain = self.reverse_power_gain_db();
        let bw_mhz = 12.0;
        let mut points = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let detuning = -span_mhz + (2.0 * span_mhz * i as f64) / (num_points - 1).max(1) as f64;
            // Lorentzian gain bandwidth profile
            let forward_db = f_gain / (1.0 + (detuning / (bw_mhz * 0.5)).powi(2));
            let reverse_db = r_gain - 2.0 * (detuning / bw_mhz).powi(2);
            let contrast_db = forward_db - reverse_db;

            points.push(VortexAmplifierPoint {
                detuning_mhz: detuning,
                forward_gain_db: forward_db,
                reverse_gain_db: reverse_db,
                isolation_contrast_db: contrast_db,
            });
        }

        points
    }

    /// Generates 2D cross-sectional donut intensity and spiral phase distribution.
    pub fn generate_spatial_vortex_slice(&self, grid_size: usize) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
        let charge = self.params.target_charge.charge_value();
        let mut intensity_grid = vec![vec![0.0; grid_size]; grid_size];
        let mut phase_grid = vec![vec![0.0; grid_size]; grid_size];
        let center = (grid_size as f64 - 1.0) * 0.5;
        let beam_waist = grid_size as f64 * 0.22;

        for y in 0..grid_size {
            for x in 0..grid_size {
                let dx = x as f64 - center;
                let dy = y as f64 - center;
                let r = (dx * dx + dy * dy).sqrt();
                let phi = dy.atan2(dx);

                // Laguerre-Gaussian vortex intensity I(r) ~ (r / w0)^(2|l|) * exp(-2 r^2 / w0^2)
                let r_norm = r / beam_waist;
                let intensity = if charge == 0 {
                    (-2.0 * r_norm.powi(2)).exp()
                } else {
                    let order = charge.abs() as f64;
                    r_norm.powf(2.0 * order) * (-2.0 * r_norm.powi(2)).exp() * 4.0
                };

                let spiral_phase = (charge as f64 * phi).rem_euclid(2.0 * PI);

                intensity_grid[y][x] = intensity.min(1.0);
                phase_grid[y][x] = spiral_phase;
            }
        }

        (intensity_grid, phase_grid)
    }
}
