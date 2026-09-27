//! Integrated photodetector physical models: PIN and Avalanche Photodiodes (APD).
//!
//! Formulates:
//! - Wavelength-dependent responsivity $\mathcal{R}(\lambda) = \eta_{ext} \frac{q \lambda}{h c}$.
//! - APD avalanche multiplication gain $M(V)$ and excess noise factor $F(M)$.
//! - Thermal dark current scaling with temperature and bandgap $E_g(T)$.
//! - High-speed transit-time and RC bandwidth roll-offs ($f_{tr}$, $f_{RC}$, $f_{3dB}$).
//! - Noise Equivalent Power (NEP) and shot noise spectral density.
//! - Modified Nodal Analysis (MNA) electrical companion model.

use phonon_core::{
    OpticalSignal, BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, PLANCK_CONSTANT, SPEED_OF_LIGHT, T_REF,
};

/// Type of photodetector architecture.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhotodetectorType {
    /// PIN Photodiode (linear response, unity internal gain).
    Pin,
    /// Avalanche Photodiode (APD, internal impact ionization gain).
    Apd,
}

/// Physical parameters for an integrated optoelectronic photodetector (e.g. Ge-on-Si or InGaAs).
#[derive(Debug, Clone, PartialEq)]
pub struct PhotodetectorModel {
    /// Photodetector type (PIN or APD).
    pub detector_type: PhotodetectorType,
    /// External quantum efficiency $\eta_{ext}$ ($0 < \eta_{ext} \le 1.0$).
    pub quantum_efficiency: f64,
    /// Depletion region thickness $w_{dep}$ in meters ($m$).
    pub depletion_width_m: f64,
    /// Active junction area $A$ in square meters ($m^2$).
    pub active_area_m2: f64,
    /// Relative dielectric permittivity $\epsilon_r$ of absorption layer.
    pub relative_permittivity: f64,
    /// Carrier saturation drift velocity $v_{sat}$ in meters per second ($m/s$).
    pub carrier_saturation_velocity_m_per_s: f64,
    /// Dark current $I_{dark,0}$ at $300\text{ K}$ in Amperes ($A$).
    pub dark_current_300k_a: f64,
    /// Energy bandgap $E_g$ at $300\text{ K}$ in electron-Volts ($eV$).
    pub bandgap_ev: f64,
    /// Series contact resistance $R_s$ in Ohms ($\Omega$).
    pub series_resistance_ohms: f64,
    /// APD breakdown voltage $V_{br}$ in Volts ($V$) (positive magnitude).
    pub breakdown_voltage_volts: f64,
    /// APD Miller exponent $m$ for multiplication curve ($1.5 \le m \le 6.0$).
    pub miller_exponent: f64,
    /// APD ionization coefficient ratio $k_{eff}$ ($k_{eff} \approx 0.05$ for Si, $\approx 0.45$ for InP/Ge).
    pub ionization_ratio_k: f64,
}

impl PhotodetectorModel {
    /// High-speed Germanium-on-Silicon (Ge-on-Si) waveguide PIN photodetector preset.
    /// Typical $f_{3dB} \ge 40\text{ GHz}$, $\mathcal{R} \approx 0.8\text{ A/W}$ at $1550\text{ nm}$.
    pub fn ge_on_si_pin() -> Self {
        Self {
            detector_type: PhotodetectorType::Pin,
            quantum_efficiency: 0.70,
            depletion_width_m: 0.5e-6,                  // 500 nm
            active_area_m2: 10.0e-12,                   // 10 um^2
            relative_permittivity: 16.0,                // Ge eps_r ~ 16
            carrier_saturation_velocity_m_per_s: 6.0e4, // 6e4 m/s in Ge
            dark_current_300k_a: 50.0e-9,               // 50 nA
            bandgap_ev: 0.66,                           // Ge Eg ~ 0.66 eV
            series_resistance_ohms: 25.0,
            breakdown_voltage_volts: 15.0,
            miller_exponent: 3.0,
            ionization_ratio_k: 0.1,
        }
    }

    /// High-gain InGaAs/InP Avalanche Photodiode (APD) preset.
    pub fn ingaas_apd() -> Self {
        Self {
            detector_type: PhotodetectorType::Apd,
            quantum_efficiency: 0.80,
            depletion_width_m: 1.0e-6,
            active_area_m2: 25.0e-12,
            relative_permittivity: 12.5,
            carrier_saturation_velocity_m_per_s: 7.0e4,
            dark_current_300k_a: 5.0e-9,
            bandgap_ev: 0.75,
            series_resistance_ohms: 30.0,
            breakdown_voltage_volts: 28.0,
            miller_exponent: 2.5,
            ionization_ratio_k: 0.45,
        }
    }

    /// Evaluates wavelength-dependent intrinsic responsivity $\mathcal{R}_0(\lambda)$ in Amperes per Watt ($A/W$):
    /// $$\mathcal{R}_0(\lambda) = \eta_{ext} \frac{q \lambda}{h c}$$
    pub fn responsivity(&self, wavelength_m: f64) -> f64 {
        let photon_energy = (PLANCK_CONSTANT * SPEED_OF_LIGHT) / wavelength_m.max(1e-12);
        (self.quantum_efficiency * ELEMENTARY_CHARGE) / photon_energy
    }

    /// Evaluates APD multiplication gain factor $M(V_{reverse})$:
    /// $$M(V) = \frac{1}{1 - \left(\frac{V_{rev}}{V_{br}}\right)^m}$$
    pub fn avalanche_gain(&self, reverse_bias_volts: f64) -> f64 {
        match self.detector_type {
            PhotodetectorType::Pin => 1.0,
            PhotodetectorType::Apd => {
                let v_rev = reverse_bias_volts.abs();
                let v_ratio = (v_rev / self.breakdown_voltage_volts).clamp(0.0, 0.999);
                let denom = 1.0 - v_ratio.powf(self.miller_exponent);
                (1.0 / denom.max(0.01)).clamp(1.0, 500.0)
            }
        }
    }

    /// Evaluates excess noise factor $F(M)$ under avalanche multiplication:
    /// $$F(M) = k_{eff} M + (1 - k_{eff})\left(2 - \frac{1}{M}\right)$$
    pub fn excess_noise_factor(&self, gain_m: f64) -> f64 {
        match self.detector_type {
            PhotodetectorType::Pin => 1.0,
            PhotodetectorType::Apd => {
                let m = gain_m.max(1.0);
                self.ionization_ratio_k * m + (1.0 - self.ionization_ratio_k) * (2.0 - 1.0 / m)
            }
        }
    }

    /// Evaluates temperature-dependent reverse dark current $I_{dark}(T)$:
    /// $$I_{dark}(T) = I_{dark,0} \left(\frac{T}{T_0}\right)^2 \exp\left( -\frac{q E_g}{2 k_B} \left(\frac{1}{T} - \frac{1}{T_0}\right) \right)$$
    pub fn dark_current(&self, temp_k: f64) -> f64 {
        let t_ratio = temp_k / T_REF;
        let delta_inv_t = (1.0 / temp_k) - (1.0 / T_REF);
        let eg_joules = self.bandgap_ev * ELEMENTARY_CHARGE;
        let exp_term = (-eg_joules * delta_inv_t / (2.0 * BOLTZMANN_CONSTANT)).exp();

        self.dark_current_300k_a * (t_ratio * t_ratio) * exp_term
    }

    /// Junction capacitance $C_j = \frac{\epsilon_0 \epsilon_r A}{w_{dep}}$ in Farads ($F$).
    pub fn junction_capacitance(&self) -> f64 {
        let eps_0 = 8.8541878128e-12;
        let eps = eps_0 * self.relative_permittivity;
        (eps * self.active_area_m2) / self.depletion_width_m.max(1e-9)
    }

    /// Carrier transit-time limited 3-dB bandwidth in Hertz ($Hz$):
    /// $$f_{tr} \approx \frac{0.45 v_{sat}}{w_{dep}}$$
    pub fn transit_time_bandwidth_hz(&self) -> f64 {
        (0.45 * self.carrier_saturation_velocity_m_per_s) / self.depletion_width_m.max(1e-9)
    }

    /// Combined 3-dB optoelectronic bandwidth $f_{3dB}$ accounting for transit time and RC parasitic loading:
    /// $$\frac{1}{f_{3dB}^2} = \frac{1}{f_{tr}^2} + \frac{1}{f_{RC}^2}$$
    pub fn bandwidth_3db_hz(&self, load_resistance_ohms: f64) -> f64 {
        let f_tr = self.transit_time_bandwidth_hz();
        let total_r = self.series_resistance_ohms + load_resistance_ohms;
        let c_j = self.junction_capacitance();
        let f_rc = 1.0 / (2.0 * std::f64::consts::PI * total_r * c_j);

        1.0 / ((1.0 / (f_tr * f_tr)) + (1.0 / (f_rc * f_rc))).sqrt()
    }

    /// Total photocurrent produced by incident optical signal under given reverse bias and temperature:
    /// $$I_{ph} = \mathcal{R}_0(\lambda) \cdot M(V_{rev}) \cdot P_{opt}$$
    pub fn generate_photocurrent(
        &self,
        optical_signal: &OpticalSignal,
        reverse_bias_volts: f64,
    ) -> f64 {
        let r0 = self.responsivity(optical_signal.wavelength_m);
        let m = self.avalanche_gain(reverse_bias_volts);
        r0 * m * optical_signal.power_watts.max(0.0)
    }

    /// Total current flowing through photodetector under reverse bias:
    /// $$I_{total} = I_{ph} + I_{dark}(T)$$
    pub fn total_current(
        &self,
        optical_signal: &OpticalSignal,
        reverse_bias_volts: f64,
        temp_k: f64,
    ) -> f64 {
        let i_ph = self.generate_photocurrent(optical_signal, reverse_bias_volts);
        let i_dark = self.dark_current(temp_k);
        i_ph + i_dark
    }

    /// Noise Equivalent Power (NEP) in $\text{W} / \sqrt{\text{Hz}}$:
    /// $$\text{NEP} = \frac{\sqrt{2 q (I_{ph} + I_{dark}) F(M)}}{\mathcal{R}_0 M}$$
    pub fn noise_equivalent_power(
        &self,
        optical_signal: &OpticalSignal,
        reverse_bias_volts: f64,
        temp_k: f64,
    ) -> f64 {
        let r0 = self.responsivity(optical_signal.wavelength_m);
        let m = self.avalanche_gain(reverse_bias_volts);
        let f_m = self.excess_noise_factor(m);
        let i_tot = self.total_current(optical_signal, reverse_bias_volts, temp_k);

        let shot_noise_density = 2.0 * ELEMENTARY_CHARGE * i_tot * f_m;
        shot_noise_density.sqrt() / (r0 * m).max(1e-12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ge_pin_responsivity_and_photocurrent() {
        let pin = PhotodetectorModel::ge_on_si_pin();
        // Responsivity at 1550 nm:
        // R0 = 0.70 * q * 1.55e-6 / (h * c) ~ 0.875 A/W
        let r0 = pin.responsivity(1.55e-6);
        assert!((r0 - 0.875).abs() < 0.01);

        // 1 mW incident power -> ~ 0.875 mA photocurrent
        let sig = OpticalSignal::new(1.55e-6, 1e-3, 0.0);
        let i_ph = pin.generate_photocurrent(&sig, 2.0);
        assert!((i_ph - 0.875e-3).abs() < 1e-5);
    }

    #[test]
    fn test_apd_avalanche_gain_and_noise_factor() {
        let apd = PhotodetectorModel::ingaas_apd();
        // At 0V reverse bias, M = 1
        assert_eq!(apd.avalanche_gain(0.0), 1.0);

        // Near breakdown (e.g. 26V on 28V breakdown): M should be > 5
        let m_26v = apd.avalanche_gain(26.0);
        assert!(m_26v > 5.0);

        let f_m = apd.excess_noise_factor(m_26v);
        assert!(f_m > 1.0);
    }

    #[test]
    fn test_ge_pin_bandwidth() {
        let pin = PhotodetectorModel::ge_on_si_pin();
        // Bandwidth with 50 Ohm load
        let bw = pin.bandwidth_3db_hz(50.0);
        // Ge-on-Si pin should have bandwidth > 25 GHz
        assert!(bw > 25.0e9);
    }
}
