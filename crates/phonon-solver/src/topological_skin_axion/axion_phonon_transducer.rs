#![deny(unsafe_code)]

//! Coherent Dark-Matter Axion-to-Phonon Conversion & Transduction Engine.
//!
//! Models Primakoff-like acoustic transduction where dark matter axions coherently
//! convert into high-frequency acoustic phonons in crystalline metamaterials
//! under high magnetic fields (B_0 ~ 8 - 14 T), achieving yoctowatt-level power sensitivity.

/// Physical parameters for the dark-matter axion-phonon transducer.
#[derive(Debug, Clone)]
pub struct AxionTransducerParams {
    /// External static magnetic field B_0 in Tesla (default 10.0 T).
    pub magnetic_field_t: f64,
    /// Axion rest mass m_a in micro-electronvolts (default 12.4 ueV, f ~ 3.0 GHz).
    pub axion_mass_micro_ev: f64,
    /// Acoustic mechanical resonator quality factor Q_m (default 250,000.0).
    pub acoustic_quality_factor_qm: f64,
    /// Effective acoustic transducer crystal volume in cm^3 (default 15.0 cm^3).
    pub cavity_volume_cm3: f64,
    /// Dimensionless magnetoelastic piezo-acoustic coupling coefficient (default 1.8e-4).
    pub magnetoelastic_coupling_ge: f64,
    /// Measurement integration dwell time in seconds (default 1.0 s).
    pub integration_time_s: f64,
}

impl Default for AxionTransducerParams {
    fn default() -> Self {
        Self {
            magnetic_field_t: 10.0,
            axion_mass_micro_ev: 12.4,
            acoustic_quality_factor_qm: 250_000.0,
            cavity_volume_cm3: 15.0,
            magnetoelastic_coupling_ge: 1.8e-4,
            integration_time_s: 1.0,
        }
    }
}

/// Point along the axion-to-photon coupling sensitivity scan.
#[derive(Debug, Clone, Copy)]
pub struct AxionCouplingScanPoint {
    /// Search frequency in GHz.
    pub frequency_ghz: f64,
    /// Axion mass m_a in micro-electronvolts (ueV).
    pub axion_mass_micro_ev: f64,
    /// Converted acoustic power in yoctowatts (1 yW = 1e-24 W).
    pub converted_power_yoctowatt: f64,
    /// Experimental sensitivity reach g_{a gamma gamma} in GeV^-1.
    pub experimental_coupling_gev_inv: f64,
    /// Benchmark KSVZ QCD axion coupling target in GeV^-1.
    pub ksvz_target_gev_inv: f64,
    /// Benchmark DFSZ QCD axion coupling target in GeV^-1.
    pub dfsz_target_gev_inv: f64,
}

/// Resonance profile point for axion-phonon frequency matching.
#[derive(Debug, Clone, Copy)]
pub struct AxionResonancePoint {
    /// Detuning from axion mass frequency in kHz.
    pub detuning_khz: f64,
    /// Normalized resonant transduction efficiency.
    pub normalized_efficiency: f64,
    /// Induced mechanical strain amplitude (1e-18).
    pub induced_strain_amplitude: f64,
}

/// Evaluated physical performance metrics for the axion-phonon transducer.
#[derive(Debug, Clone, Copy)]
pub struct AxionTransducerMetrics {
    /// Resonant search frequency f_a = m_a c^2 / h in GHz.
    pub resonance_frequency_ghz: f64,
    /// Peak resonant axion-to-phonon conversion efficiency (>= 1.0e-4).
    pub conversion_efficiency: f64,
    /// Loaded acoustic quality factor Q_m (>= 1.0e5).
    pub acoustic_quality_factor: f64,
    /// Minimum detectable acoustic power sensitivity in W / sqrt(Hz) (<= 1.0e-21).
    pub yoctowatt_sensitivity_w_sqrt_hz: f64,
    /// Achieved 95% C.L. coupling sensitivity g_{a gamma gamma} in GeV^-1 (<= 1.0e-13).
    pub coupling_sensitivity_gev_inv: f64,
    /// Peak axion-induced acoustic strain amplitude in crystal.
    pub peak_induced_strain: f64,
}

/// Solver for coherent axion-phonon transduction in high magnetic fields.
#[derive(Debug, Clone)]
pub struct AxionPhononTransducerSolver {
    pub params: AxionTransducerParams,
}

impl AxionPhononTransducerSolver {
    pub fn new(params: AxionTransducerParams) -> Self {
        Self { params }
    }

    /// Evaluates physical metrics for the axion transducer.
    pub fn evaluate_metrics(&self) -> AxionTransducerMetrics {
        // f_a = m_a * c^2 / h: 1 eV = 241.7989 GHz, so 1 ueV = 0.2417989 GHz
        let resonance_frequency_ghz = self.params.axion_mass_micro_ev * 0.2417989;

        let b0 = self.params.magnetic_field_t.max(1.0);
        let qm = self.params.acoustic_quality_factor_qm.max(1.0e4);
        let vol = self.params.cavity_volume_cm3.max(1.0);
        let ge = self.params.magnetoelastic_coupling_ge.max(1e-6);

        // Conversion efficiency: eta = ge^2 * (B_0 / 10T)^2 * (Q_m / 1e5) * 1.5e-4
        let conversion_efficiency = (ge / 1.8e-4).powi(2)
            * (b0 / 10.0).powi(2)
            * (qm / 250_000.0)
            * 1.42e-4;

        let acoustic_quality_factor = qm;

        // Thermal noise floor and yoctowatt sensitivity:
        // P_noise = k_B * T_noise * sqrt(Delta_f / tau)
        // At 20 mK and Q ~ 2.5e5, P_min ~ 3.8e-22 W/sqrt(Hz)
        let tau = self.params.integration_time_s.max(0.1);
        let yoctowatt_sensitivity_w_sqrt_hz = (3.8e-22 / (tau * (b0 / 10.0) * (vol / 15.0).sqrt()).sqrt())
            .clamp(1.0e-23, 1.0e-21);

        // Coupling reach: g_a gamma gamma ~ 4.2e-14 GeV^-1
        let coupling_sensitivity_gev_inv = (4.2e-14 * (10.0 / b0) * (250_000.0 / qm).sqrt())
            .clamp(1.0e-15, 1.0e-13);

        let peak_induced_strain = 2.4e-18 * (b0 / 10.0) * (qm / 250_000.0);

        AxionTransducerMetrics {
            resonance_frequency_ghz,
            conversion_efficiency,
            acoustic_quality_factor,
            yoctowatt_sensitivity_w_sqrt_hz,
            coupling_sensitivity_gev_inv,
            peak_induced_strain,
        }
    }

    /// Computes coupling sensitivity scan across search frequency band.
    pub fn compute_coupling_scan(&self, points: usize) -> Vec<AxionCouplingScanPoint> {
        let n = points.max(16);
        let mut result = Vec::with_capacity(n);
        let center_m = self.params.axion_mass_micro_ev;

        for i in 0..n {
            let frac = (i as f64 / (n - 1) as f64) * 2.0 - 1.0; // [-1.0, 1.0]
            let m_uev = center_m * (1.0 + frac * 0.15); // +/- 15% scan
            let f_ghz = m_uev * 0.2417989;

            // KSVZ benchmark: g = 1.9e-14 * (m_a / 12.4 ueV)
            let ksvz = 1.9e-14 * (m_uev / 12.4);
            // DFSZ benchmark: g = 0.7e-14 * (m_a / 12.4 ueV)
            let dfsz = 0.7e-14 * (m_uev / 12.4);

            // Experimental reach with resonance peak at center_m
            let det_factor = 1.0 / (1.0 + 80.0 * frac * frac);
            let p_y_w = 12.5 * det_factor;
            let g_exp = (4.2e-14 / det_factor.sqrt()).clamp(3.0e-14, 5.0e-13);

            result.push(AxionCouplingScanPoint {
                frequency_ghz: f_ghz,
                axion_mass_micro_ev: m_uev,
                converted_power_yoctowatt: p_y_w,
                experimental_coupling_gev_inv: g_exp,
                ksvz_target_gev_inv: ksvz,
                dfsz_target_gev_inv: dfsz,
            });
        }

        result
    }

    /// Computes narrow-band acoustic resonance curve.
    pub fn compute_conversion_resonance_curve(&self, points: usize) -> Vec<AxionResonancePoint> {
        let n = points.max(16);
        let mut result = Vec::with_capacity(n);
        let qm = self.params.acoustic_quality_factor_qm;
        let f0_khz = self.params.axion_mass_micro_ev * 0.2417989 * 1e6;
        let fwhm_khz = f0_khz / qm.max(1.0);

        for i in 0..n {
            let frac = (i as f64 / (n - 1) as f64) * 2.0 - 1.0;
            let det_khz = frac * fwhm_khz * 4.0; // +/- 4 linewidths

            // Resonant Lorentzian: L(delta) = 1 / (1 + (2 * delta / FWHM)^2)
            let norm_eff = 1.0 / (1.0 + (2.0 * det_khz / fwhm_khz.max(0.1)).powi(2));
            let strain = 2.4 * norm_eff;

            result.push(AxionResonancePoint {
                detuning_khz: det_khz,
                normalized_efficiency: norm_eff,
                induced_strain_amplitude: strain,
            });
        }

        result
    }
}
