//! Integrated optical micro-ring resonator (MRR) physical model.
//!
//! Formulates:
//! - All-pass and add-drop micro-ring configurations.
//! - Closed-form analytical power transfer functions for Through and Drop ports.
//! - Free Spectral Range (FSR), Full Width at Half Maximum (FWHM), Quality Factor $Q$, and Finesse $\mathcal{F}$.
//! - Temperature-dependent resonance drift: $\frac{d\lambda_{res}}{dT} = \frac{\lambda_{res}}{n_g} \frac{dn_{eff}}{dT}$.
//! - Integrated thermo-optic micro-heater tuning ($\Delta T = R_{th} P_{heater}$).

use super::waveguide::OpticalWaveguideModel;
use phonon_core::OpticalSignal;
use std::f64::consts::PI;

/// Operational topology of the optical ring resonator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingResonatorType {
    /// All-pass ring filter (single bus waveguide coupled to ring, thru port only).
    AllPass,
    /// Add-drop filter (two bus waveguides coupled to ring, thru port and drop port).
    AddDrop,
}

/// Physical model of an integrated micro-ring resonator.
#[derive(Debug, Clone, PartialEq)]
pub struct MicroRingResonatorModel {
    /// Ring radius $R$ in meters ($m$).
    pub radius_m: f64,
    /// Core waveguide model parameterizing modal index $n_{eff}$, dispersion, loss, and $dn/dT$.
    pub waveguide: OpticalWaveguideModel,
    /// Field cross-coupling coefficient $\kappa_1$ to the input bus waveguide ($0 < \kappa_1 < 1$).
    pub coupling_k1: f64,
    /// Field cross-coupling coefficient $\kappa_2$ to the drop bus waveguide ($0 \le \kappa_2 < 1$).
    /// For All-Pass configuration, $\kappa_2 = 0$.
    pub coupling_k2: f64,
    /// Thermal resistance of integrated micro-heater $R_{th}$ in $\text{K} / \text{W}$.
    pub thermal_resistance_k_per_w: f64,
    /// Micro-heater electrical resistance $R_{heater}$ in Ohms ($\Omega$).
    pub heater_resistance_ohms: f64,
}

impl MicroRingResonatorModel {
    /// Creates an all-pass micro-ring resonator with radius $R$, coupling $\kappa$, and SOI strip waveguide.
    pub fn all_pass_silicon(radius_m: f64, coupling_k: f64) -> Self {
        let circum = 2.0 * PI * radius_m;
        Self {
            radius_m,
            waveguide: OpticalWaveguideModel::silicon_strip(circum),
            coupling_k1: coupling_k.clamp(1e-4, 0.999),
            coupling_k2: 0.0,
            thermal_resistance_k_per_w: 2.5e3, // 2500 K/W typical for Si microheater
            heater_resistance_ohms: 50.0,
        }
    }

    /// Creates an add-drop micro-ring resonator with radius $R$, symmetric coupling $\kappa$, and SOI strip waveguide.
    pub fn add_drop_silicon(radius_m: f64, coupling_k: f64) -> Self {
        let circum = 2.0 * PI * radius_m;
        Self {
            radius_m,
            waveguide: OpticalWaveguideModel::silicon_strip(circum),
            coupling_k1: coupling_k.clamp(1e-4, 0.999),
            coupling_k2: coupling_k.clamp(1e-4, 0.999),
            thermal_resistance_k_per_w: 2.5e3,
            heater_resistance_ohms: 50.0,
        }
    }

    /// Roundtrip perimeter $L = 2\pi R$ in meters ($m$).
    #[inline]
    pub fn roundtrip_length(&self) -> f64 {
        2.0 * PI * self.radius_m
    }

    /// Roundtrip field self-transmission factor $a = \sqrt{\mathcal{T}_{roundtrip}}$ (amplitude attenuation).
    #[inline]
    pub fn roundtrip_amplitude_transmission(&self) -> f64 {
        let p_trans = self.waveguide.power_transmission_factor();
        p_trans.sqrt().clamp(1e-6, 1.0)
    }

    /// Self-coupling transmission coefficient $t_1 = \sqrt{1 - \kappa_1^2}$.
    #[inline]
    pub fn self_coupling_t1(&self) -> f64 {
        (1.0 - self.coupling_k1 * self.coupling_k1).max(0.0).sqrt()
    }

    /// Self-coupling transmission coefficient $t_2 = \sqrt{1 - \kappa_2^2}$.
    #[inline]
    pub fn self_coupling_t2(&self) -> f64 {
        (1.0 - self.coupling_k2 * self.coupling_k2).max(0.0).sqrt()
    }

    /// Free Spectral Range (FSR) around nominal design wavelength $\lambda_0$ in meters ($m$):
    /// $$\text{FSR} = \frac{\lambda_0^2}{n_g L}$$
    pub fn free_spectral_range_m(&self) -> f64 {
        let l = self.roundtrip_length();
        let lambda0 = self.waveguide.lambda_0_m;
        (lambda0 * lambda0) / (self.waveguide.group_index_ng * l)
    }

    /// Roundtrip accumulated phase $\phi(\lambda, T) = \frac{2\pi n_{eff}(\lambda, T)}{\lambda} L$.
    pub fn roundtrip_phase(&self, wavelength_m: f64, temp_k: f64) -> f64 {
        let beta = self.waveguide.propagation_constant(wavelength_m, temp_k);
        beta * self.roundtrip_length()
    }

    /// Full Width at Half Maximum ($\Delta\lambda_{FWHM}$) in meters ($m$):
    /// $$\Delta\lambda_{FWHM} = \frac{\lambda_0^2}{\pi n_g L} \frac{1 - t_1 t_2 a}{\sqrt{t_1 t_2 a}}$$
    pub fn fwhm_bandwidth_m(&self) -> f64 {
        let t1 = self.self_coupling_t1();
        let t2 = self.self_coupling_t2();
        let a = self.roundtrip_amplitude_transmission();
        let t_prod = t1 * t2 * a;
        let fsr = self.free_spectral_range_m();
        (fsr / PI) * ((1.0 - t_prod) / t_prod.sqrt().max(1e-9))
    }

    /// Loaded Optical Quality Factor $Q = \frac{\lambda_0}{\Delta\lambda_{FWHM}}$.
    pub fn quality_factor(&self) -> f64 {
        let fwhm = self.fwhm_bandwidth_m();
        if fwhm > 1e-15 {
            self.waveguide.lambda_0_m / fwhm
        } else {
            1e9
        }
    }

    /// Optical Finesse $\mathcal{F} = \frac{\text{FSR}}{\Delta\lambda_{FWHM}}$.
    pub fn finesse(&self) -> f64 {
        let fwhm = self.fwhm_bandwidth_m();
        if fwhm > 1e-15 {
            self.free_spectral_range_m() / fwhm
        } else {
            1e9
        }
    }

    /// Thermo-optic resonance wavelength tuning drift in meters per Kelvin ($m / K$):
    /// $$\frac{d\lambda_{res}}{dT} = \frac{\lambda_{res}}{n_g} \frac{dn_{eff}}{dT}$$
    pub fn thermo_optic_resonance_drift_m_per_k(&self) -> f64 {
        (self.waveguide.lambda_0_m / self.waveguide.group_index_ng) * self.waveguide.dn_dt
    }

    /// Effective cavity temperature $T$ when heated by integrated micro-heater voltage $V_{heater}$:
    /// $$\Delta T = R_{th} \frac{V_{heater}^2}{R_{heater}}$$
    pub fn heated_temperature(&self, ambient_k: f64, v_heater_volts: f64) -> f64 {
        let p_elec = (v_heater_volts * v_heater_volts) / self.heater_resistance_ohms.max(1e-3);
        ambient_k + self.thermal_resistance_k_per_w * p_elec
    }

    /// Evaluates Through-port power transmission ratio $T_{thru}(\lambda, T) = |E_{thru} / E_{in}|^2$:
    /// $$T_{thru} = \frac{t_1^2 + (t_2 a)^2 - 2 t_1 t_2 a \cos\phi}{1 + (t_1 t_2 a)^2 - 2 t_1 t_2 a \cos\phi}$$
    pub fn thru_transmission(&self, wavelength_m: f64, temp_k: f64) -> f64 {
        let t1 = self.self_coupling_t1();
        let t2 = self.self_coupling_t2();
        let a = self.roundtrip_amplitude_transmission();
        let phi = self.roundtrip_phase(wavelength_m, temp_k);
        let cos_phi = phi.cos();

        let num = t1 * t1 + (t2 * a) * (t2 * a) - 2.0 * t1 * t2 * a * cos_phi;
        let den = 1.0 + (t1 * t2 * a) * (t1 * t2 * a) - 2.0 * t1 * t2 * a * cos_phi;

        (num / den.max(1e-12)).clamp(0.0, 1.0)
    }

    /// Evaluates Drop-port power transmission ratio $T_{drop}(\lambda, T) = |E_{drop} / E_{in}|^2$:
    /// $$T_{drop} = \frac{\kappa_1^2 \kappa_2^2 a}{1 + (t_1 t_2 a)^2 - 2 t_1 t_2 a \cos\phi}$$
    /// (Returns 0.0 for all-pass filters where $\kappa_2 = 0$).
    pub fn drop_transmission(&self, wavelength_m: f64, temp_k: f64) -> f64 {
        if self.coupling_k2 <= 0.0 {
            return 0.0;
        }
        let t1 = self.self_coupling_t1();
        let t2 = self.self_coupling_t2();
        let a = self.roundtrip_amplitude_transmission();
        let phi = self.roundtrip_phase(wavelength_m, temp_k);
        let cos_phi = phi.cos();

        let k1 = self.coupling_k1;
        let k2 = self.coupling_k2;

        let num = (k1 * k1) * (k2 * k2) * a;
        let den = 1.0 + (t1 * t2 * a) * (t1 * t2 * a) - 2.0 * t1 * t2 * a * cos_phi;

        (num / den.max(1e-12)).clamp(0.0, 1.0)
    }

    /// Propagates an input optical signal through the ring resonator, returning:
    /// `(thru_signal, Option<drop_signal>)`.
    pub fn propagate(
        &self,
        input: &OpticalSignal,
        temp_k: f64,
    ) -> (OpticalSignal, Option<OpticalSignal>) {
        let t1 = self.self_coupling_t1();
        let t2 = self.self_coupling_t2();
        let a = self.roundtrip_amplitude_transmission();
        let phi = self.roundtrip_phase(input.wavelength_m, temp_k);
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // Complex thru field: (t1 - t2*a*exp(-i*phi)) / (1 - t1*t2*a*exp(-i*phi))
        let num_re = t1 - t2 * a * cos_phi;
        let num_im = t2 * a * sin_phi;
        let den_re = 1.0 - t1 * t2 * a * cos_phi;
        let den_im = t1 * t2 * a * sin_phi;

        let thru_phase_shift = num_im.atan2(num_re) - den_im.atan2(den_re);
        let thru_power = input.power_watts * self.thru_transmission(input.wavelength_m, temp_k);

        let thru_signal = OpticalSignal {
            wavelength_m: input.wavelength_m,
            power_watts: thru_power,
            phase_rad: (input.phase_rad + thru_phase_shift).rem_euclid(2.0 * PI),
        };

        let drop_signal = if self.coupling_k2 > 0.0 {
            let drop_power = input.power_watts * self.drop_transmission(input.wavelength_m, temp_k);
            let drop_phase_num_re =
                -(self.coupling_k1 * self.coupling_k2 * a.sqrt()) * (phi * 0.5).cos();
            let drop_phase_num_im =
                -(self.coupling_k1 * self.coupling_k2 * a.sqrt()) * (phi * 0.5).sin();
            let drop_phase_shift =
                drop_phase_num_im.atan2(drop_phase_num_re) - den_im.atan2(den_re);

            Some(OpticalSignal {
                wavelength_m: input.wavelength_m,
                power_watts: drop_power,
                phase_rad: (input.phase_rad + drop_phase_shift).rem_euclid(2.0 * PI),
            })
        } else {
            None
        };

        (thru_signal, drop_signal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_pass_resonance_and_fsr() {
        // Radius = 10 um
        let ring = MicroRingResonatorModel::all_pass_silicon(10e-6, 0.2);
        let fsr = ring.free_spectral_range_m();
        assert!((fsr - 9.10e-9).abs() < 0.2e-9);

        // Quality factor should be on the order of 10^3 to 10^5
        let q = ring.quality_factor();
        assert!(q > 1_000.0 && q < 500_000.0);
    }

    #[test]
    fn test_thermo_optic_resonance_tuning() {
        let ring = MicroRingResonatorModel::add_drop_silicon(10e-6, 0.15);
        let drift_rate = ring.thermo_optic_resonance_drift_m_per_k();
        assert!((drift_rate - 6.86e-11).abs() < 5e-12);

        // 2V applied to 50 Ohm heater with 2500 K/W thermal resistance:
        // P = 4 / 50 = 0.08 W. Delta T = 0.08 * 2500 = 200 K
        let t_heated = ring.heated_temperature(300.0, 2.0);
        assert!((t_heated - 500.0).abs() < 1e-6);
    }

    #[test]
    fn test_add_drop_complementary_transmission() {
        // Ideal low-loss ring should conserve power: T_thru + T_drop <= 1
        let mut ring = MicroRingResonatorModel::add_drop_silicon(10e-6, 0.3);
        ring.waveguide.loss_db_per_m = 0.0; // lossless limit

        let t_thru = ring.thru_transmission(1.55e-6, 300.0);
        let t_drop = ring.drop_transmission(1.55e-6, 300.0);

        assert!((t_thru + t_drop - 1.0).abs() < 1e-4);
    }
}
