#![deny(unsafe_code)]

//! Dynamic Magnetoelastic Drive & Acoustic Wavepacket Absorption Engine.
//!
//! Models dynamic lattice strain tensor coupling to spin-wave precession,
//! effective dynamic RF magnetoelastic fields, and resonant acoustic attenuation
//! (non-reciprocal / resonant magnetoacoustic absorption depth).

use std::f64::consts::PI;

pub const _MU_0: f64 = 1.256_637_061_4e-6; // T * m / A

/// Parameters governing dynamic magnetoelastic drive in thin film heterostructures.
#[derive(Debug, Clone)]
pub struct MagnetoelasticDriveParams {
    /// Longitudinal magnetoelastic coupling coefficient B_1 in J / m^3 (e.g. -4.0e5 J/m^3 for YIG).
    pub b1_j_m3: f64,
    /// Shear magnetoelastic coupling coefficient B_2 in J / m^3 (e.g. 6.4e5 J/m^3 for YIG).
    pub b2_j_m3: f64,
    /// Saturation magnetization M_s in A / m (e.g. 1.4e5 A/m).
    pub ms_a_m: f64,
    /// Peak acoustic strain amplitude epsilon_0 (dimensionless, typically 1e-6 to 1e-4).
    pub peak_strain: f64,
    /// Acoustic phase velocity v_s in m / s (e.g. 3840.0 m/s).
    pub sound_velocity_m_s: f64,
    /// Non-resonant background acoustic attenuation alpha_0 in dB / cm (e.g. 0.5 dB/cm).
    pub background_attenuation_db_cm: f64,
}

impl Default for MagnetoelasticDriveParams {
    fn default() -> Self {
        Self {
            b1_j_m3: -4.0e5,
            b2_j_m3: 6.4e5,
            ms_a_m: 1.4e5,
            peak_strain: 2.5e-5, // 25 micro-strain
            sound_velocity_m_s: 3840.0,
            background_attenuation_db_cm: 0.8,
        }
    }
}

/// Dynamic snapshot of strain and magnetization along the 1D propagation track.
#[derive(Debug, Clone)]
pub struct MagnetoelasticTrackSnapshot {
    /// Spatial positions x along track in millimeters.
    pub x_positions_mm: Vec<f64>,
    /// Dynamic acoustic strain values epsilon(x).
    pub strain_values: Vec<f64>,
    /// Induced dynamic magnetization precession amplitude (m_x).
    pub dynamic_magnetization: Vec<f64>,
    /// Effective dynamic magnetoelastic field h_me in milliTesla (mT).
    pub effective_field_mt: Vec<f64>,
}

/// Engine evaluating magnetoelastic drive and resonant acoustic attenuation.
#[derive(Debug, Clone)]
pub struct MagnetoelasticDriveEngine {
    pub params: MagnetoelasticDriveParams,
}

impl MagnetoelasticDriveEngine {
    pub fn new(params: MagnetoelasticDriveParams) -> Self {
        Self { params }
    }

    /// Evaluates dynamic effective RF magnetoelastic field amplitude in Tesla:
    /// h_me = (2 * B_2 / M_s) * epsilon_0
    pub fn compute_effective_rf_field_tesla(&self, strain: f64) -> f64 {
        let b2 = self.params.b2_j_m3;
        let denom = self.params.ms_a_m.max(1.0);
        (2.0 * b2 / denom) * strain
    }

    /// Evaluates resonant acoustic attenuation alpha_ac(f) in dB / cm across frequency:
    ///
    /// alpha(f) = alpha_0 + Delta_alpha * [ (kappa_m/2)^2 / ((2*pi*(f - f_m))^2 + (kappa_m/2)^2) ]
    pub fn compute_resonant_attenuation_db_cm(
        &self,
        freq_hz: f64,
        resonance_freq_hz: f64,
        magnon_linewidth_hz: f64,
        peak_absorption_db_cm: f64,
    ) -> f64 {
        let delta_f = freq_hz - resonance_freq_hz;
        let gamma_half = 0.5 * magnon_linewidth_hz;
        let lorentzian = (gamma_half * gamma_half) / (delta_f * delta_f + gamma_half * gamma_half);
        self.params.background_attenuation_db_cm + peak_absorption_db_cm * lorentzian
    }

    /// Generates spatial track snapshot of propagating SAW wavepacket coupling to magnetization at time t.
    pub fn generate_track_snapshot(
        &self,
        track_length_mm: f64,
        wavelength_um: f64,
        time_s: f64,
        points: usize,
    ) -> MagnetoelasticTrackSnapshot {
        let mut x_positions = Vec::with_capacity(points);
        let mut strain_vals = Vec::with_capacity(points);
        let mut dyn_mag = Vec::with_capacity(points);
        let mut eff_fields = Vec::with_capacity(points);

        let k = 2.0 * PI / (wavelength_um * 1e-6);
        let omega = k * self.params.sound_velocity_m_s;
        let dx = track_length_mm / (points - 1).max(1) as f64;

        for i in 0..points {
            let x_mm = i as f64 * dx;
            let x_m = x_mm * 1e-3;

            // Gaussian wavepacket envelope centered at v_s * t
            let center_m = self.params.sound_velocity_m_s * time_s;
            let sigma_m = 4.0 * (wavelength_um * 1e-6);
            let dist = x_m - center_m;
            let envelope = (-dist * dist / (2.0 * sigma_m * sigma_m)).exp();

            let phase = k * x_m - omega * time_s;
            let strain = self.params.peak_strain * envelope * phase.cos();
            let h_me_t = self.compute_effective_rf_field_tesla(strain);
            let m_dyn = envelope * (phase - 0.5 * PI).cos() * 0.08; // 90 deg out-of-phase precession

            x_positions.push(x_mm);
            strain_vals.push(strain * 1e6); // in microstrain
            dyn_mag.push(m_dyn);
            eff_fields.push(h_me_t * 1e3); // in mT
        }

        MagnetoelasticTrackSnapshot {
            x_positions_mm: x_positions,
            strain_values: strain_vals,
            dynamic_magnetization: dyn_mag,
            effective_field_mt: eff_fields,
        }
    }
}
