#![deny(unsafe_code)]

//! Non-Reciprocal Acoustic Traveling-Wave Parametric Edge Amplifier.
//!
//! Models directional parametric amplification along the topological boundary of the
//! quadrupole waveguide under a traveling-wave acoustic/microwave pump, achieving high
//! forward gain (>= 20 dB), high directional isolation (>= 25 dB), and quantum-limited added noise.

use std::f64::consts::PI;

/// Parameters governing the directional traveling-wave parametric edge amplifier.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricDriveParams {
    /// Bare acoustic signal resonance frequency omega_0 in GHz (default ~1.0 GHz).
    pub omega_0_ghz: f64,
    /// Traveling-wave pump frequency Omega_p in GHz: Omega_p = 2 * omega_0 (default ~2.0 GHz).
    pub pump_freq_ghz: f64,
    /// Parametric coupling rate g_param in MHz (default ~15.0 MHz).
    pub coupling_rate_mhz: f64,
    /// Modulation / pump wavevector k_p in rad/mm.
    pub modulation_km_rad_mm: f64,
    /// Waveguide propagation length L in mm (default ~50.0 mm).
    pub length_mm: f64,
    /// Waveguide acoustic sound speed / group velocity v_g in m/s (default 3000.0 m/s = 3.0 mm/us).
    pub sound_speed_m_s: f64,
    /// Waveguide linear acoustic loss alpha in dB/cm (default 0.2 dB/cm).
    pub alpha_db_cm: f64,
    /// Effective parametric drive velocity in m/s (default 1.25e6 m/s for electromagnetic/RF distributed drive).
    pub drive_velocity_m_s: f64,
}

impl Default for ParametricDriveParams {
    fn default() -> Self {
        let v_g = 3000.0; // m/s = 3.0e6 mm/s
        let f0 = 1.0; // GHz
        // Signal wavevector k0 = 2*pi*f0 / v_g in rad/mm
        let k0 = (2.0 * PI * f0 * 1.0e9) / (v_g * 1.0e3);
        let kp = 2.0 * k0;

        Self {
            omega_0_ghz: f0,
            pump_freq_ghz: 2.0 * f0,
            coupling_rate_mhz: 15.0,
            modulation_km_rad_mm: kp,
            length_mm: 50.0,
            sound_speed_m_s: v_g,
            alpha_db_cm: 0.2,
            drive_velocity_m_s: 1.25e6,
        }
    }
}

impl ParametricDriveParams {
    /// Acoustic signal wavevector k_s in rad/mm.
    #[inline]
    pub fn signal_wavevector_rad_mm(&self) -> f64 {
        let v_mm_s = self.sound_speed_m_s * 1.0e3;
        (2.0 * PI * self.omega_0_ghz * 1.0e9) / v_mm_s
    }

    /// Spatial parametric coupling coefficient g_spatial in rad/mm:
    /// g_spatial = 2 * pi * g_param / v_drive.
    #[inline]
    pub fn spatial_coupling_per_mm(&self) -> f64 {
        let v_drive_mm_s = self.drive_velocity_m_s * 1.0e3;
        (2.0 * PI * self.coupling_rate_mhz * 1.0e6) / v_drive_mm_s
    }

    /// Linear power transmission loss factor across waveguide length L:
    /// T_loss = 10^(-alpha_dB/cm * L_cm / 10).
    #[inline]
    pub fn power_loss_factor(&self) -> f64 {
        let l_cm = self.length_mm / 10.0;
        let total_loss_db = self.alpha_db_cm * l_cm;
        10.0f64.powf(-total_loss_db / 10.0)
    }

    /// Net phase mismatch for forward signal: Delta_k_fwd = k_p - k_s - k_idler = 0 under matched drive.
    #[inline]
    pub fn delta_k_forward(&self) -> f64 {
        let ks = self.signal_wavevector_rad_mm();
        (self.modulation_km_rad_mm - 2.0 * ks).abs()
    }

    /// Net phase mismatch for backward signal: Delta_k_bwd = k_p - (-k_s) - (-k_idler) = k_p + 2*k_s.
    #[inline]
    pub fn delta_k_backward(&self) -> f64 {
        let ks = self.signal_wavevector_rad_mm();
        self.modulation_km_rad_mm + 2.0 * ks
    }
}

/// Point on the parametric amplifier gain spectrum.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricGainSample {
    /// Signal frequency in GHz.
    pub freq_ghz: f64,
    /// Forward signal power gain in dB.
    pub gain_forward_db: f64,
    /// Backward signal power gain in dB.
    pub gain_backward_db: f64,
    /// Directional isolation in dB: Isolation = G_forward - G_backward.
    pub isolation_db: f64,
}

/// Key RF and quantum metrics for the parametric edge amplifier.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricAmplifierMetrics {
    /// Forward signal power gain in dB (achieves >= 20.0 dB).
    pub gain_forward_db: f64,
    /// Forward signal power gain in linear scale.
    pub gain_forward_linear: f64,
    /// Backward signal power gain in dB (severely phase-mismatched, <= 0.5 dB).
    pub gain_backward_db: f64,
    /// Backward signal power gain in linear scale.
    pub gain_backward_linear: f64,
    /// Directional isolation in dB: Isolation = G_forward - G_backward >= 25.0 dB.
    pub isolation_db: f64,
    /// Input-referred quantum added noise n_add in quanta: n_add = 0.5 * (1 - 1 / G_forward).
    pub added_noise_quanta: f64,
    /// Standard quantum limit for phase-insensitive parametric amplifier (0.5 quanta).
    pub quantum_limit_quanta: f64,
}

/// Traveling-wave Parametric Edge Amplifier Engine.
#[derive(Debug, Clone)]
pub struct ParametricEdgeAmplifier {
    pub params: ParametricDriveParams,
    pub metrics: ParametricAmplifierMetrics,
    /// Discretized gain spectrum across frequency band.
    pub gain_spectrum: Vec<ParametricGainSample>,
}

impl ParametricEdgeAmplifier {
    /// Construct and solve a new parametric edge amplifier.
    pub fn new(params: ParametricDriveParams) -> Self {
        let mut amp = Self {
            params,
            metrics: ParametricAmplifierMetrics {
                gain_forward_db: 0.0,
                gain_forward_linear: 1.0,
                gain_backward_db: 0.0,
                gain_backward_linear: 1.0,
                isolation_db: 0.0,
                added_noise_quanta: 0.5,
                quantum_limit_quanta: 0.5,
            },
            gain_spectrum: Vec::new(),
        };
        amp.recompute();
        amp
    }

    /// Recomputes forward gain, backward gain, isolation, added noise, and gain spectrum.
    pub fn recompute(&mut self) {
        let dk_fwd = self.params.delta_k_forward();
        let (g_fwd_lin, g_fwd_db) = self.compute_gain_at_delta_k(dk_fwd);

        let dk_bwd = self.params.delta_k_backward();
        let (g_bwd_lin, g_bwd_db) = self.compute_gain_at_delta_k(dk_bwd);

        let isolation_db = g_fwd_db - g_bwd_db;
        let added_noise = Self::compute_added_noise(g_fwd_lin);

        self.metrics = ParametricAmplifierMetrics {
            gain_forward_db: g_fwd_db,
            gain_forward_linear: g_fwd_lin,
            gain_backward_db: g_bwd_db,
            gain_backward_linear: g_bwd_lin,
            isolation_db,
            added_noise_quanta: added_noise,
            quantum_limit_quanta: 0.5,
        };

        // Sweep gain spectrum across +/- 40 MHz around resonance frequency
        self.gain_spectrum = self.sweep_gain_spectrum(40.0, 65);
    }

    /// Computes signal power gain (linear, dB) for a given phase mismatch Delta_k in rad/mm:
    ///
    /// If |Delta_k| / 2 < g_spatial:
    /// s = sqrt(g_spatial^2 - (Delta_k / 2)^2)
    /// G = T_loss * [1 + (g_spatial / s)^2 * sinh^2(s * L)]
    ///
    /// If |Delta_k| / 2 >= g_spatial:
    /// q = sqrt((Delta_k / 2)^2 - g_spatial^2)
    /// G = T_loss * [1 + (g_spatial / q)^2 * sin^2(q * L)]
    pub fn compute_gain_at_delta_k(&self, delta_k: f64) -> (f64, f64) {
        let g = self.params.spatial_coupling_per_mm();
        let l = self.params.length_mm;
        let t_loss = self.params.power_loss_factor();

        let half_dk = 0.5 * delta_k.abs();

        let g_linear = if half_dk < g {
            let s = (g * g - half_dk * half_dk).sqrt();
            let sinh_val = (s * l).sinh();
            let gain_param = (g / s) * (g / s) * sinh_val * sinh_val;
            t_loss * (1.0 + gain_param)
        } else {
            let q = (half_dk * half_dk - g * g).sqrt();
            let sin_val = (q * l).sin();
            let gain_param = if q < 1e-12 {
                0.0
            } else {
                (g / q) * (g / q) * sin_val * sin_val
            };
            t_loss * (1.0 + gain_param)
        };

        let g_linear_clamped = g_linear.max(1e-12);
        let g_db = 10.0 * g_linear_clamped.log10();
        (g_linear_clamped, g_db)
    }

    /// Computes Caves input-referred added quantum noise in quanta:
    ///
    /// n_add = 0.5 * (1 - 1 / G_forward)
    #[inline]
    pub fn compute_added_noise(gain_forward_linear: f64) -> f64 {
        let g = gain_forward_linear.max(1.0);
        0.5 * (1.0 - 1.0 / g)
    }

    /// Sweeps signal frequency across +/- bandwidth_mhz around resonance frequency omega_0.
    pub fn sweep_gain_spectrum(&self, bandwidth_mhz: f64, num_points: usize) -> Vec<ParametricGainSample> {
        let n = num_points.max(16);
        let mut samples = Vec::with_capacity(n);

        let f0 = self.params.omega_0_ghz;
        let v_mm_s = self.params.sound_speed_m_s * 1.0e3;
        let span_ghz = (bandwidth_mhz * 1.0e-3).max(1.0e-4);

        for i in 0..n {
            let frac = (i as f64) / ((n - 1) as f64);
            let f = (f0 - span_ghz) + frac * (2.0 * span_ghz);
            let delta_f = f - f0;

            // Phase mismatch induced by frequency detuning: Delta_k_detuning = 2 * pi * delta_f / v_g
            let delta_k_detune = (2.0 * PI * delta_f * 1.0e9) / v_mm_s;

            let dk_fwd = (self.params.delta_k_forward() + delta_k_detune).abs();
            let (_, g_fwd_db) = self.compute_gain_at_delta_k(dk_fwd);

            let dk_bwd = (self.params.delta_k_backward() + delta_k_detune).abs();
            let (_, g_bwd_db) = self.compute_gain_at_delta_k(dk_bwd);

            samples.push(ParametricGainSample {
                freq_ghz: f,
                gain_forward_db: g_fwd_db,
                gain_backward_db: g_bwd_db,
                isolation_db: g_fwd_db - g_bwd_db,
            });
        }

        samples
    }
}
