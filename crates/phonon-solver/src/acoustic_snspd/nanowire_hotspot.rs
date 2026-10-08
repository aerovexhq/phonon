#![deny(unsafe_code)]

//! Non-equilibrium electro-thermal hot-spot dynamics and single-phonon detection
//! in superconducting nanowire single-phonon detectors (SNSPDs).
//!
//! Models Cooper pair breaking by acoustic phonon energy absorption, hot-spot nucleation,
//! current crowding, resistive barrier formation, ultrafast electrical voltage pulses,
//! and acoustic timing jitter.

/// Parameters for the superconducting nanowire single-phonon detector (SNSPD).
#[derive(Debug, Clone, PartialEq)]
pub struct NanowireParams {
    /// Superconducting critical temperature Tc in Kelvin (e.g. 9.5 K for NbN).
    pub critical_temperature_tc_k: f64,
    /// Operating substrate base temperature in Kelvin (e.g. 1.2 K).
    pub base_temperature_k: f64,
    /// Superconducting critical current Ic in microamperes (e.g. 25.0 uA).
    pub critical_current_ic_ua: f64,
    /// Applied DC bias current in microamperes (e.g. 22.5 uA, 90% of Ic).
    pub bias_current_ib_ua: f64,
    /// Nanowire physical width in nanometers (e.g. 50.0 nm).
    pub nanowire_width_nm: f64,
    /// Nanowire physical thickness in nanometers (e.g. 5.0 nm).
    pub nanowire_thickness_nm: f64,
    /// Nanowire active strip length in micrometers (e.g. 10.0 um).
    pub nanowire_length_um: f64,
    /// Nanowire kinetic inductance in nanohenries (e.g. 45.0 nH).
    pub kinetic_inductance_nh: f64,
    /// Normal state sheet resistance in ohms per square (e.g. 500.0 Ohm/sq).
    pub sheet_resistance_ohm_sq: f64,
    /// High-speed transmission line load impedance in ohms (e.g. 50.0 Ohm).
    pub load_impedance_ohm: f64,
    /// Incident acoustic phonon center frequency in gigahertz (e.g. 10.0 GHz).
    pub phonon_frequency_ghz: f64,
    /// Acoustic strain coupling efficiency from topological waveguide into nanowire (0.0 to 1.0).
    pub acoustic_coupling_efficiency: f64,
}

impl Default for NanowireParams {
    fn default() -> Self {
        Self {
            critical_temperature_tc_k: 9.5,
            base_temperature_k: 1.2,
            critical_current_ic_ua: 25.0,
            bias_current_ib_ua: 22.5,
            nanowire_width_nm: 50.0,
            nanowire_thickness_nm: 5.0,
            nanowire_length_um: 10.0,
            kinetic_inductance_nh: 45.0,
            sheet_resistance_ohm_sq: 500.0,
            load_impedance_ohm: 50.0,
            phonon_frequency_ghz: 10.0,
            acoustic_coupling_efficiency: 0.92,
        }
    }
}

/// Instantaneous voltage and resistance pulse point over time.
#[derive(Debug, Clone, PartialEq)]
pub struct PulsePoint {
    /// Timestamp in picoseconds.
    pub time_ps: f64,
    /// Output voltage pulse amplitude in millivolts.
    pub voltage_mv: f64,
    /// Hot-spot resistance in ohms.
    pub resistance_ohm: f64,
    /// Current flowing through load in microamperes.
    pub load_current_ua: f64,
}

/// Performance telemetry metrics for the SNSPD detector.
#[derive(Debug, Clone, PartialEq)]
pub struct SnspdTelemetry {
    /// Bias current ratio (Ib / Ic).
    pub bias_ratio: f64,
    /// Peak voltage pulse height in millivolts.
    pub peak_voltage_mv: f64,
    /// 10%-90% electrical rise time in picoseconds.
    pub rise_time_ps: f64,
    /// 1/e electrical recovery / reset time in picoseconds (tau_reset = Lk / R_load).
    pub reset_time_ps: f64,
    /// Acoustic timing jitter FWHM in picoseconds.
    pub timing_jitter_fwhm_ps: f64,
    /// Internal single-phonon quantum detection efficiency in percent.
    pub internal_efficiency_percent: f64,
    /// Dark count rate in Hertz (spontaneous thermal/vortex fluctuations).
    pub dark_count_rate_hz: f64,
    /// Total normal-state resistance across the entire nanowire strip in kilohms.
    pub total_normal_resistance_kohm: f64,
    /// Single-phonon energy in attojoules (10^-18 J).
    pub single_phonon_energy_aj: f64,
    /// Quasiparticle hot-spot maximum radius in nanometers.
    pub hotspot_radius_nm: f64,
}

/// Solver for non-equilibrium electro-thermal hot-spot dynamics in SNSPDs.
#[derive(Debug, Clone)]
pub struct NanowireHotspotSolver {
    pub params: NanowireParams,
}

impl NanowireHotspotSolver {
    pub fn new(params: NanowireParams) -> Self {
        Self { params }
    }

    /// Evaluates key detector telemetry and electro-thermal characteristics.
    pub fn evaluate_telemetry(&self) -> SnspdTelemetry {
        let p = &self.params;
        let bias_ratio = (p.bias_current_ib_ua / p.critical_current_ic_ua.max(1e-6)).clamp(0.0, 1.0);

        // Single phonon energy E = h * f
        const PLANCK_H: f64 = 6.62607015e-34; // J*s
        let phonon_freq_hz = p.phonon_frequency_ghz * 1e9;
        let single_phonon_energy_j = PLANCK_H * phonon_freq_hz;
        let single_phonon_energy_aj = single_phonon_energy_j * 1e18;

        // Superconducting energy gap Delta_0 approx 1.764 * k_B * Tc
        const BOLTZMANN_K: f64 = 1.380649e-23;
        let delta_0_j = 1.764 * BOLTZMANN_K * p.critical_temperature_tc_k;
        let quasiparticle_yield = (single_phonon_energy_j / delta_0_j.max(1e-26)).max(1.0);

        // Hotspot radius r_hs = xi_0 * sqrt(1 / (1 - I_b / I_c)) with Joule heating expansion
        let coherence_length_nm = 4.5; // typical for NbN
        let hotspot_radius_nm = (coherence_length_nm * (1.0 / (1.0 - bias_ratio).max(0.01)).sqrt() * (quasiparticle_yield * 0.5).sqrt().max(1.0))
            .clamp(6.0, p.nanowire_width_nm * 0.95);

        // Hotspot resistance R_hs
        let squares = p.nanowire_length_um * 1000.0 / p.nanowire_width_nm;
        let total_normal_resistance_kohm = (p.sheet_resistance_ohm_sq * squares) / 1000.0;
        let hotspot_length_nm = hotspot_radius_nm * 2.0;
        let hotspot_resistance_ohm = p.sheet_resistance_ohm_sq * (hotspot_length_nm / p.nanowire_width_nm).max(0.5);

        // Peak output voltage pulse V_peak = I_bias * (R_hs * R_L) / (R_hs + R_L)
        let i_bias_a = p.bias_current_ib_ua * 1e-6;
        let parallel_r = (hotspot_resistance_ohm * p.load_impedance_ohm)
            / (hotspot_resistance_ohm + p.load_impedance_ohm);
        let peak_voltage_mv = i_bias_a * parallel_r * 1e3;

        // Rise time approx 20-40 ps, limited by current diversion
        let rise_time_ps = 25.0 + 10.0 * (1.0 - bias_ratio);

        // Reset time tau = L_k / R_L
        let l_k_h = p.kinetic_inductance_nh * 1e-9;
        let tau_reset_s = l_k_h / p.load_impedance_ohm;
        let reset_time_ps = tau_reset_s * 1e12;

        // Acoustic timing jitter: sigma = sqrt(sigma_elec^2 + sigma_geo^2)
        // With high bias current, electronic jitter decreases significantly
        let electronic_jitter_ps = 1.2 / bias_ratio.max(0.1);
        let spot_size_um: f64 = 0.08; // 80 nm acoustic waveguide mode focus
        let geometric_jitter_ps: f64 = (spot_size_um * 1e-6 / 3400.0) * 1e12 * 0.05;
        let timing_jitter_fwhm_ps = 2.355 * (electronic_jitter_ps.powi(2) + geometric_jitter_ps.powi(2)).sqrt();

        // Internal quantum efficiency: saturates sigmoidally near Ib -> Ic
        let sigmoid_arg = 24.0 * (bias_ratio - 0.74);
        let sigmoid = 1.0 / (1.0 + (-sigmoid_arg).exp());
        let internal_efficiency_percent = (sigmoid * p.acoustic_coupling_efficiency * 100.0).clamp(0.0, 100.0);

        // Dark count rate: Kramers thermal vortex escape rate
        // DCR ~ Omega_0 * exp(-Delta_U(I) / k_B T)
        let barrier_factor = (1.0 - bias_ratio).powf(1.5) * 350.0 / p.base_temperature_k.max(0.1);
        let dark_count_rate_hz = (1e5 * (-barrier_factor).exp()).clamp(0.01, 50.0);

        SnspdTelemetry {
            bias_ratio,
            peak_voltage_mv,
            rise_time_ps,
            reset_time_ps,
            timing_jitter_fwhm_ps,
            internal_efficiency_percent,
            dark_count_rate_hz,
            total_normal_resistance_kohm,
            single_phonon_energy_aj,
            hotspot_radius_nm,
        }
    }

    /// Simulates the transient voltage pulse waveform over a 2.5 ns window.
    pub fn simulate_pulse_waveform(&self) -> Vec<PulsePoint> {
        let tele = self.evaluate_telemetry();
        let p = &self.params;
        let tau_rise = tele.rise_time_ps;
        let tau_reset = tele.reset_time_ps;
        let v_peak = tele.peak_voltage_mv;
        let i_b = p.bias_current_ib_ua;

        let total_time_ps = 2500.0;
        let steps = 250;
        let dt = total_time_ps / (steps as f64);

        (0..=steps).map(|i| {
            let t = (i as f64) * dt;
            let v = if t < 50.0 {
                0.0
            } else {
                let t_rel = t - 50.0;
                let rise_factor = 1.0 - (-t_rel / tau_rise).exp();
                let decay_factor = (-t_rel / tau_reset).exp();
                v_peak * rise_factor * decay_factor
            };

            let load_current_ua = (v / p.load_impedance_ohm) * 1000.0;
            let r_hs = if v > 0.05 {
                (v / 1e3) / (i_b * 1e-6 - (v / 1e3) / p.load_impedance_ohm).max(1e-9)
            } else {
                0.0
            };

            PulsePoint {
                time_ps: t,
                voltage_mv: v,
                resistance_ohm: r_hs.min(1500.0),
                load_current_ua,
            }
        }).collect()
    }
}
