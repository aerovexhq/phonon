//! Electro-optic modulator physical models: Mach-Zehnder Interferometer (MZI)
//! and Electro-Absorption Modulator (EAM).
//!
//! Formulates:
//! - Carrier plasma dispersion effect (Soref-Bennett empirical model for Silicon at 1550 nm).
//! - Push-pull and single-arm Mach-Zehnder optical phase modulation.
//! - Half-wave switching voltage $V_\pi$, extinction ratio (ER), and insertion loss.
//! - Electrical high-frequency bandwidth roll-off (3-dB cutoff).

use super::waveguide::OpticalWaveguideModel;
use phonon_core::{OpticalSignal, T_REF};
use std::f64::consts::PI;

/// Type of electro-optic modulation mechanism.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModulatorType {
    /// Mach-Zehnder Interferometer (MZI) based on phase interference.
    MachZehnderInterferometer,
    /// Electro-Absorption Modulator (EAM) based on field-induced absorption (Franz-Keldysh / QCSE).
    ElectroAbsorption,
}

/// Physical model for an electro-optic modulator.
#[derive(Debug, Clone, PartialEq)]
pub struct ElectroOpticModulatorModel {
    /// Modulator physical mechanism.
    pub mod_type: ModulatorType,
    /// Half-wave switching voltage $V_\pi$ in Volts ($V$).
    pub v_pi: f64,
    /// Static optical phase bias $\phi_0$ in radians ($rad$) (e.g. $\pi / 2$ for quadrature bias).
    pub bias_phase_rad: f64,
    /// Optical insertion loss in decibels ($\text{dB}$) at peak transmission.
    pub insertion_loss_db: f64,
    /// Extinction ratio in decibels ($\text{dB}$).
    pub extinction_ratio_db: f64,
    /// Small-signal 3-dB electrical modulation bandwidth in Hertz ($Hz$).
    pub bandwidth_3db_hz: f64,
    /// Core waveguide model representing the active arm.
    pub waveguide_arm: OpticalWaveguideModel,
}

impl ElectroOpticModulatorModel {
    /// Creates a silicon carrier-depletion MZI modulator preset at $1550\text{ nm}$.
    /// Typical $V_\pi \approx 3.5\text{ V}$, $\text{IL} \approx 3\text{ dB}$, $\text{ER} \ge 25\text{ dB}$, $f_{3dB} \approx 40\text{ GHz}$.
    pub fn silicon_mzi_quadrature(arm_length_m: f64, v_pi: f64) -> Self {
        Self {
            mod_type: ModulatorType::MachZehnderInterferometer,
            v_pi: v_pi.max(0.1),
            bias_phase_rad: PI / 2.0, // Quadrature bias for linear modulation
            insertion_loss_db: 3.0,
            extinction_ratio_db: 25.0,
            bandwidth_3db_hz: 40.0e9,
            waveguide_arm: OpticalWaveguideModel::silicon_strip(arm_length_m),
        }
    }

    /// Creates an Electro-Absorption Modulator (EAM) preset.
    /// Characterized by steep absorption curve ($V_{swing} \approx 1.5\text{ V}$), $f_{3dB} \approx 50\text{ GHz}$.
    pub fn electro_absorption(v_extinction: f64) -> Self {
        Self {
            mod_type: ModulatorType::ElectroAbsorption,
            v_pi: v_extinction.max(0.1),
            bias_phase_rad: 0.0,
            insertion_loss_db: 4.5,
            extinction_ratio_db: 20.0,
            bandwidth_3db_hz: 50.0e9,
            waveguide_arm: OpticalWaveguideModel::silicon_strip(100e-6),
        }
    }

    /// Evaluates free carrier plasma dispersion index and absorption change in silicon at $1.55\,\mu\text{m}$
    /// using Soref-Bennett empirical equations:
    /// $$\Delta n = -8.8 \times 10^{-22} \Delta N_e - 8.5 \times 10^{-18} (\Delta N_h)^{0.8}$$
    /// $$\Delta \alpha = 8.5 \times 10^{-18} \Delta N_e + 6.0 \times 10^{-18} \Delta N_h \quad [\text{cm}^{-1}]$$
    /// Carrier densities in $\text{cm}^{-3}$. Returns `(delta_n, delta_alpha_per_m)`.
    pub fn soref_bennett_plasma_dispersion(
        delta_n_electrons: f64,
        delta_n_holes: f64,
    ) -> (f64, f64) {
        let n_e = delta_n_electrons.max(0.0);
        let n_h = delta_n_holes.max(0.0);

        let delta_n = -8.8e-22 * n_e - 8.5e-18 * n_h.powf(0.8);
        let delta_alpha_cm = 8.5e-18 * n_e + 6.0e-18 * n_h;
        let delta_alpha_per_m = delta_alpha_cm * 100.0; // cm^-1 to m^-1

        (delta_n, delta_alpha_per_m)
    }

    /// Linear power transmission ratio $\mathcal{T}(V, T)$ as a function of driving voltage $V$:
    /// For MZI:
    /// $$\mathcal{T}(V) = T_{min} + (T_{max} - T_{min}) \cdot \frac{1 + \cos(\pi \frac{V}{V_\pi} + \phi_0 + \Delta \phi_{temp})}{2}$$
    pub fn transmission_factor(&self, v_drive_volts: f64, temp_k: f64) -> f64 {
        let t_max = 10.0_f64.powf(-self.insertion_loss_db / 10.0);
        let er_linear = 10.0_f64.powf(self.extinction_ratio_db / 10.0);
        let t_min = t_max / er_linear;

        match self.mod_type {
            ModulatorType::MachZehnderInterferometer => {
                // Thermo-optic phase drift between arms if differential temperature exists
                let delta_t = temp_k - T_REF;
                let thermal_phase = self.waveguide_arm.dn_dt
                    * delta_t
                    * (2.0 * PI / self.waveguide_arm.lambda_0_m)
                    * self.waveguide_arm.length_m;

                let electro_phase = (PI * v_drive_volts) / self.v_pi;
                let total_phase = electro_phase + self.bias_phase_rad + thermal_phase;

                let norm_trans = 0.5 * (1.0 + total_phase.cos());
                t_min + (t_max - t_min) * norm_trans
            }
            ModulatorType::ElectroAbsorption => {
                // Exponential absorption curve: alpha(V) = alpha_0 * exp(V / V_pi)
                let norm_v = (v_drive_volts / self.v_pi).clamp(0.0, 5.0);
                let absorption_factor = (-norm_v * (self.extinction_ratio_db * 0.2302585)).exp();
                t_min + (t_max - t_min) * absorption_factor
            }
        }
    }

    /// Evaluates small-signal electrical frequency response attenuation at frequency $f$:
    /// $$H(f) = \frac{1}{\sqrt{1 + (f / f_{3dB})^2}}$$
    pub fn frequency_response(&self, frequency_hz: f64) -> f64 {
        let ratio = frequency_hz / self.bandwidth_3db_hz.max(1.0);
        1.0 / (1.0 + ratio * ratio).sqrt()
    }

    /// Propagates an input optical signal through the modulator given electrical driving voltage $V_{drive}$
    /// and temperature $T$:
    pub fn propagate(
        &self,
        input: &OpticalSignal,
        v_drive_volts: f64,
        temp_k: f64,
    ) -> OpticalSignal {
        let trans = self.transmission_factor(v_drive_volts, temp_k);
        let p_out = input.power_watts * trans;

        // Modulator phase modulation
        let electro_phase = match self.mod_type {
            ModulatorType::MachZehnderInterferometer => (0.5 * PI * v_drive_volts) / self.v_pi,
            ModulatorType::ElectroAbsorption => 0.0,
        };

        OpticalSignal {
            wavelength_m: input.wavelength_m,
            power_watts: p_out.max(0.0),
            phase_rad: (input.phase_rad + electro_phase).rem_euclid(2.0 * PI),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mzi_quadrature_and_vpi() {
        let mzi = ElectroOpticModulatorModel::silicon_mzi_quadrature(1e-3, 3.0);
        let t_max = 10.0_f64.powf(-3.0 / 10.0);
        let er_linear = 10.0_f64.powf(25.0 / 10.0);
        let t_min = t_max / er_linear;

        // At quadrature (V=0, phi0=pi/2 at T_REF): cos(pi/2) = 0 -> trans = 0.5 * (t_max + t_min)
        let t_quad = mzi.transmission_factor(0.0, T_REF);
        let expected_quad = 0.5 * (t_max + t_min);
        assert!((t_quad - expected_quad).abs() < 1e-5);

        // At V = -V_pi/2 = -1.5V: phase = -pi/2 + pi/2 = 0 -> maximum transmission
        let t_on = mzi.transmission_factor(-1.5, T_REF);
        assert!((t_on - t_max).abs() < 1e-5);

        // At V = +V_pi/2 = +1.5V: phase = +pi/2 + pi/2 = pi -> minimum transmission (extinction)
        let t_off = mzi.transmission_factor(1.5, T_REF);
        assert!((t_off - t_min).abs() < 1e-5);
    }

    #[test]
    fn test_soref_bennett_carrier_plasma_dispersion() {
        // Injection of 1e18 cm^-3 electrons and holes:
        let (dn, dalpha) = ElectroOpticModulatorModel::soref_bennett_plasma_dispersion(1e18, 1e18);

        // dn should be negative (refractive index decreases with carrier concentration)
        assert!(dn < 0.0);
        assert!((dn - (-8.8e-22 * 1e18 - 8.5e-18 * 1e18_f64.powf(0.8))).abs() < 1e-6);

        // Absorption should increase (free-carrier absorption)
        assert!(dalpha > 0.0);
    }

    #[test]
    fn test_high_frequency_bandwidth_cutoff() {
        let mzi = ElectroOpticModulatorModel::silicon_mzi_quadrature(1e-3, 3.0);
        // At f = 40 GHz (f_3dB), response should be 1/sqrt(2) ~ 0.7071 (-3 dB)
        let resp = mzi.frequency_response(40.0e9);
        assert!((resp - 1.0 / std::f64::consts::SQRT_2).abs() < 1e-4);
    }
}
