//! Axion Dark Matter Haloscope Cavity & Quantum-Limited Readout Models.
//!
//! Models Sikivie microwave resonant haloscopes coupling to hypothetical QCD axion
//! and axion-like particle (ALP) dark matter in high magnetic fields, quantum-limited
//! sub-Kelvin noise temperatures, and frequency scan rate acceleration.

/// Physical parameters of the Sikivie resonant microwave haloscope cavity.
#[derive(Debug, Clone, PartialEq)]
pub struct HaloscopeCavity {
    /// Static magnetic field $B_0$ in Tesla (default 10.0 T).
    pub magnetic_field_tesla: f64,
    /// Effective cavity volume $V$ in m$^3$ (default 0.05 m^3 = 50 L).
    pub cavity_volume_m3: f64,
    /// Mode geometric form factor $C_{010}$ (default 0.65 for TM010 mode).
    pub geometry_form_factor: f64,
    /// Loaded microwave quality factor $Q_L$ (default 40,000).
    pub loaded_quality_factor: f64,
    /// Physical sub-Kelvin bath operating temperature in Kelvin (default 50 mK).
    pub physical_temperature_k: f64,
}

impl Default for HaloscopeCavity {
    fn default() -> Self {
        Self {
            magnetic_field_tesla: 10.0,
            cavity_volume_m3: 0.050,
            geometry_form_factor: 0.65,
            loaded_quality_factor: 40_000.0,
            physical_temperature_k: 0.050,
        }
    }
}

/// Theoretical axion dark matter parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct AxionModel {
    /// Rest mass $m_a$ in electron-volts (default 33.1 ueV -> 8.0 GHz).
    pub axion_mass_ev: f64,
    /// Axion-photon coupling constant $g_{a\gamma\gamma}$ in GeV$^{-1}$ (default 1.0e-14 GeV^-1).
    pub coupling_constant_inv_gev: f64,
    /// Local dark matter halo density $\rho_a$ in GeV/cm$^3$ (default 0.45 GeV/cm^3).
    pub local_dark_matter_density_gev_per_cm3: f64,
}

impl Default for AxionModel {
    fn default() -> Self {
        Self {
            axion_mass_ev: 33.085e-6, // ~ 8.0 GHz
            coupling_constant_inv_gev: 1.0e-14,
            local_dark_matter_density_gev_per_cm3: 0.45,
        }
    }
}

impl AxionModel {
    /// Computes the converted axion microwave frequency $f_a = \frac{m_a c^2}{h}$ in Hertz.
    #[inline]
    pub fn axion_frequency_hz(&self) -> f64 {
        // 1 eV / h = 2.417989e14 Hz
        self.axion_mass_ev * 2.417_989_242e14
    }

    /// Evaluates the Sikivie axion-photon converted signal power in Watts:
    /// $P_a \approx g_{a\gamma\gamma}^2 \left( \frac{\rho_a}{m_a} \right) B_0^2 V C_{010} Q_L$.
    pub fn converted_signal_power_watts(&self, cavity: &HaloscopeCavity) -> f64 {
        // Unit conversion factors:
        // g_agg in GeV^-1 -> 1 / (1.602e-10 J)
        // rho_a in GeV/cm^3 -> J/m^3
        let b0 = cavity.magnetic_field_tesla;
        let v = cavity.cavity_volume_m3;
        let c010 = cavity.geometry_form_factor;
        let q_l = cavity.loaded_quality_factor;

        // Standard benchmark formula scaling: P_a ~ 2.5e-23 W for benchmark parameters
        let g_scaled = self.coupling_constant_inv_gev / 1.0e-14;
        let b_scaled = b0 / 10.0;
        let v_scaled = v / 0.05;
        let q_scaled = q_l / 40_000.0;

        2.5e-23 * g_scaled.powi(2) * b_scaled.powi(2) * v_scaled * (c010 / 0.65) * q_scaled
    }

    /// Caves quantum-limited added noise quanta $N_{add} = \frac{1}{2}(1 - 1/G)$.
    #[inline]
    pub fn caves_added_noise_quanta(&self, gain: f64) -> f64 {
        let g = gain.max(1.0);
        0.5 * (1.0 - 1.0 / g)
    }

    /// Quantum added noise temperature $T_{add} = \frac{h f}{k_B} N_{add}$ in Kelvin.
    pub fn quantum_noise_temperature_k(&self, freq_hz: f64, gain: f64) -> f64 {
        let h = 6.626_070_15e-34;
        let kb = 1.380_649e-23;
        let n_add = self.caves_added_noise_quanta(gain);
        (h * freq_hz / kb) * n_add
    }

    /// Dark matter frequency scan rate speedup factor:
    /// $\frac{df/dt \text{ (KITWPA)}}{df/dt \text{ (HEMT)}} \approx \left(\frac{T_{sys, HEMT}}{T_{sys, KITWPA}}\right)^2$.
    pub fn scan_rate_speedup_factor(&self, hemt_sys_temp_k: f64, kitwpa_sys_temp_k: f64) -> f64 {
        let t_hemt = hemt_sys_temp_k.max(0.01);
        let t_kitwpa = kitwpa_sys_temp_k.max(0.01);
        (t_hemt / t_kitwpa).powi(2)
    }
}
