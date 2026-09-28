//! Quantum telemetry, internal detection efficiency, dark count rate, timing jitter, and g^(2)(tau).
//!
//! Formulates sigmoidal internal quantum efficiency \u{03b7}_int, Kramer phase-slip dark count rate (DCR),
//! geometric and slew-rate timing jitter (< 50 ps), Hanbury Brown-Twiss coincidence anti-bunching,
//! and quantum time-of-flight (ToF) ranging equations.

use super::nanowire_geometry::{NanowireGeometry, BOLTZMANN_K, SPEED_OF_LIGHT};

/// Quantum telemetry and detection statistics parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTelemetryModel {
    /// Attempt frequency for phase-slip dark counts \u{03bd}_0 in Hz (typically ~1.0e10 Hz).
    pub phase_slip_attempt_frequency_hz: f64,
    /// Phase slip energy barrier coefficient U_0 in Joules (typically ~50 - 100 * k_B * T_c).
    pub phase_slip_barrier_joules: f64,
    /// Electromagnetic slow-wave velocity ratio v_g / c along the nanowire (typically 0.03 - 0.08).
    pub slow_wave_velocity_ratio: f64,
    /// Electronic readout voltage noise RMS \u{03c3}_V in Volts (typically ~50 uV).
    pub voltage_noise_rms_volts: f64,
    /// Output pulse voltage slew rate dV/dt in V/s (typically ~5e9 V/s for 1 mV / 200 ps).
    pub pulse_slew_rate_v_s: f64,
    /// Intrinsic hotspot nucleation latency dispersion \u{03c3}_lat in picoseconds (typically 5 - 12 ps).
    pub latency_dispersion_ps: f64,
}

impl QuantumTelemetryModel {
    /// Standard NbN detector telemetry parameters.
    pub fn standard_nbn(geom: &NanowireGeometry) -> Self {
        // U_0 \u{2248} 80 * k_B * T_c
        let barrier_joules = 80.0 * BOLTZMANN_K * geom.critical_temperature_k;
        Self {
            phase_slip_attempt_frequency_hz: 1.0e10,
            phase_slip_barrier_joules: barrier_joules,
            slow_wave_velocity_ratio: 0.025,
            voltage_noise_rms_volts: 600.0e-6,
            pulse_slew_rate_v_s: 5.0e7, // 1.0 mV in 20 ps \u{2248} 5.0e7 V/s
            latency_dispersion_ps: 10.0,
        }
    }

    /// Internal quantum detection efficiency \u{03b7}_int(I_b, \u{03bb}) \u{2208} [0, 1]:
    /// \u{03b7}_int = 1 / (1 + exp(-(I_b - I_th(\u{03bb})) / \u{0394}I)).
    pub fn internal_quantum_efficiency(
        &self,
        geom: &NanowireGeometry,
        bias_current_ua: f64,
        wavelength_nm: f64,
    ) -> f64 {
        let i_c = geom.operational_critical_current_ua();
        // Threshold current depends on photon energy (longer wavelength requires higher bias)
        let ref_lambda_nm = 1550.0;
        let lambda_factor = (wavelength_nm / ref_lambda_nm).clamp(0.4, 2.0);
        let i_th = i_c * (0.65 * lambda_factor).clamp(0.40, 0.85);
        let delta_i = i_c * 0.05; // 5% transition width

        let exponent = -(bias_current_ua - i_th) / delta_i;
        let clamped_exp = exponent.clamp(-50.0, 50.0);
        1.0 / (1.0 + clamped_exp.exp())
    }

    /// Intrinsic dark count rate (DCR) in counts per second (cps) via thermal phase-slip barrier:
    /// DCR = \u{03bd}_0 * exp(- \u{0394}U(I_b) / (k_B * T_sub)),
    /// where \u{0394}U(I_b) = U_0 * (1 - I_b / I_c)^(5/4).
    pub fn dark_count_rate_cps(&self, geom: &NanowireGeometry, bias_current_ua: f64) -> f64 {
        let i_c = geom.operational_critical_current_ua();
        let bias_ratio = (bias_current_ua / i_c.max(1e-6)).clamp(0.0, 0.999);
        let barrier_factor = (1.0 - bias_ratio).powf(1.25);
        let delta_u = self.phase_slip_barrier_joules * barrier_factor;
        let thermal_energy = BOLTZMANN_K * geom.substrate_temperature_k.max(0.1);

        let exponent = -delta_u / thermal_energy;
        let clamped_exp = exponent.clamp(-100.0, 0.0);
        self.phase_slip_attempt_frequency_hz * clamped_exp.exp()
    }

    /// Geometric propagation delay jitter \u{03c3}_geom = L / (2 * v_g) in picoseconds.
    pub fn geometric_jitter_ps(&self, geom: &NanowireGeometry) -> f64 {
        let v_g = SPEED_OF_LIGHT * self.slow_wave_velocity_ratio;
        let length_m = geom.length_um * 1e-6;
        let dt_s = length_m / (2.0 * v_g);
        dt_s * 1e12
    }

    /// Electronic noise jitter \u{03c3}_noise = \u{03c3}_V / (dV/dt) in picoseconds.
    pub fn noise_jitter_ps(&self) -> f64 {
        let dt_s = self.voltage_noise_rms_volts / self.pulse_slew_rate_v_s.max(1.0);
        dt_s * 1e12
    }

    /// Total root-mean-square (RMS) detector timing jitter \u{03c3}_jitter in picoseconds:
    /// \u{03c3}_jitter = sqrt(\u{03c3}_geom^2 + \u{03c3}_noise^2 + \u{03c3}_lat^2).
    pub fn total_timing_jitter_ps(&self, geom: &NanowireGeometry) -> f64 {
        let s_geom = self.geometric_jitter_ps(geom);
        let s_noise = self.noise_jitter_ps();
        let s_lat = self.latency_dispersion_ps;

        (s_geom * s_geom + s_noise * s_noise + s_lat * s_lat).sqrt()
    }

    /// Evaluates second-order coherence g^(2)(\u{03c4}) for a single-photon emitter with background noise:
    /// g^(2)(\u{03c4}) = 1 - (1 - g0) * exp(-|\u{03c4}| / \u{03c4}_decay),
    /// where g0 < 0.5 indicates quantum anti-bunching.
    pub fn second_order_coherence_single_photon(
        &self,
        tau_ps: f64,
        g0: f64,
        tau_decay_ps: f64,
    ) -> f64 {
        let arg = -(tau_ps.abs()) / tau_decay_ps.max(1.0);
        1.0 - (1.0 - g0.clamp(0.0, 1.0)) * arg.exp()
    }

    /// Quantum Time-of-Flight (ToF) distance calculation: z = c * \u{0394}t_tof / 2 in meters.
    pub fn time_of_flight_distance_m(&self, time_of_flight_s: f64) -> f64 {
        (SPEED_OF_LIGHT * time_of_flight_s) / 2.0
    }

    /// Single-shot distance uncertainty \u{03b4}z = c * \u{03c3}_jitter / 2 in millimeters.
    pub fn single_shot_ranging_uncertainty_mm(&self, geom: &NanowireGeometry) -> f64 {
        let jitter_s = self.total_timing_jitter_ps(geom) * 1e-12;
        let delta_z_m = (SPEED_OF_LIGHT * jitter_s) / 2.0;
        delta_z_m * 1000.0
    }

    /// Averaged ranging precision over N_pulses: \u{03b4}z_avg = \u{03b4}z / sqrt(N_pulses) in millimeters.
    pub fn averaged_ranging_precision_mm(&self, geom: &NanowireGeometry, num_pulses: usize) -> f64 {
        let single_shot = self.single_shot_ranging_uncertainty_mm(geom);
        let n_sqrt = (num_pulses.max(1) as f64).sqrt();
        single_shot / n_sqrt
    }
}
