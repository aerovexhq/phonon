//! Surface Acoustic Wave (SAW) and Bulk Acoustic Wave (BAW/FBAR) nanoresonator physics.
//!
//! Implements electromechanical transduction, Interdigital Transducer (IDT) arrays,
//! thin-film piezoelectric resonators in the hypersonic regime ($1 - 25 \text{ GHz}$),
//! and Modified Butterworth-Van Dyke (mBVD) equivalent circuit parameter extraction.

use super::piezoelectric::{PiezoelectricMaterial, EPSILON_0};

/// Modified Butterworth-Van Dyke (mBVD) lumped equivalent circuit parameters:
/// ```text
///            +--[ R_s ]--+-------[ R_m ]---[ L_m ]---[ C_m ]-------+-- Pin 2
///            |           |                                         |
///   Pin 1 ---+           +---------------[ C_0 ]-------------------+
///                        |                                         |
///                        +---------------[ R_0 ]-------------------+
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MbvdParameters {
    /// Series resonant frequency $f_s$ in Hz.
    pub f_s: f64,
    /// Parallel (anti-resonant) frequency $f_p$ in Hz.
    pub f_p: f64,
    /// Static clamp capacitance $C_0$ in Farads.
    pub c_0: f64,
    /// Motional capacitance $C_m$ in Farads.
    pub c_m: f64,
    /// Motional inductance $L_m$ in Henrys.
    pub l_m: f64,
    /// Motional resistance $R_m$ in Ohms.
    pub r_m: f64,
    /// Ohmic electrode contact series resistance $R_s$ in Ohms.
    pub r_s: f64,
    /// Substrate dielectric leakage resistance $R_0$ in Ohms.
    pub r_0: f64,
    /// Unloaded mechanical quality factor $Q_m$.
    pub q_m: f64,
    /// Effective electromechanical coupling coefficient $k_{eff}^2$.
    pub k_eff_sq: f64,
}

impl MbvdParameters {
    /// Evaluates the complex input impedance $Z(f) = R(f) + i X(f)$ in Ohms.
    pub fn impedance(&self, freq_hz: f64) -> (f64, f64) {
        let omega = 2.0 * std::f64::consts::PI * freq_hz;
        if omega <= 0.0 {
            return (self.r_s + self.r_0, 0.0);
        }

        // Motional branch: Z_m = R_m + i*(omega*L_m - 1/(omega*C_m))
        let x_m = omega * self.l_m - 1.0 / (omega * self.c_m);
        let r_m = self.r_m;
        let denom_m = r_m * r_m + x_m * x_m;
        // Admittance Y_m = G_m + i*B_m
        let g_m = r_m / denom_m;
        let b_m = -x_m / denom_m;

        // Static branch: Y_0 = 1/R_0 + i*omega*C_0
        let g_0 = if self.r_0 > 1e12 { 0.0 } else { 1.0 / self.r_0 };
        let b_0 = omega * self.c_0;

        // Total core admittance Y_core = Y_m + Y_0
        let g_tot = g_m + g_0;
        let b_tot = b_m + b_0;
        let denom_core = g_tot * g_tot + b_tot * b_tot;

        if denom_core < 1e-30 {
            return (self.r_s + 1e12, 0.0);
        }

        // Z_core = 1 / Y_core = (g_tot - i*b_tot) / denom_core
        let r_core = g_tot / denom_core;
        let x_core = -b_tot / denom_core;

        // Total impedance Z = R_s + Z_core
        (self.r_s + r_core, x_core)
    }

    /// Evaluates scattering parameter magnitude $|S_{11}|$ relative to reference impedance $Z_0$ (typically $50\,\Omega$).
    pub fn s11_magnitude(&self, freq_hz: f64, z0: f64) -> f64 {
        let (r, x) = self.impedance(freq_hz);
        // Gamma = (Z - Z0) / (Z + Z0) = ((R - Z0) + i*X) / ((R + Z0) + i*X)
        let num_r = r - z0;
        let num_i = x;
        let den_r = r + z0;
        let den_i = x;

        let num_sq = num_r * num_r + num_i * num_i;
        let den_sq = den_r * den_r + den_i * den_i;

        if den_sq < 1e-30 {
            1.0
        } else {
            (num_sq / den_sq).sqrt()
        }
    }
}

/// Surface Acoustic Wave (SAW) Resonator with Interdigital Transducers (IDT).
#[derive(Debug, Clone, PartialEq)]
pub struct SawResonator {
    /// Substrate piezoelectric material.
    pub material: PiezoelectricMaterial,
    /// IDT period $\lambda_0$ in meters.
    pub wavelength_m: f64,
    /// Acoustic aperture width $W$ in meters.
    pub aperture_m: f64,
    /// Number of finger pairs $N_p$.
    pub num_finger_pairs: usize,
    /// Metallization ratio $\eta = w / p$ (typically $0.5$).
    pub metallization_ratio: f64,
    /// Unloaded mechanical quality factor $Q_m$.
    pub quality_factor: f64,
    /// Electrode parasitic ohmic series resistance $R_s$ in Ohms.
    pub electrode_resistance_ohms: f64,
}

impl SawResonator {
    pub fn new(
        material: PiezoelectricMaterial,
        wavelength_m: f64,
        aperture_m: f64,
        num_finger_pairs: usize,
        quality_factor: f64,
    ) -> Self {
        Self {
            material,
            wavelength_m,
            aperture_m,
            num_finger_pairs,
            metallization_ratio: 0.5,
            quality_factor,
            electrode_resistance_ohms: 1.5,
        }
    }

    /// Center synchronous resonant frequency: $f_0 = v_R / \lambda_0$ (Hz).
    pub fn resonant_frequency(&self) -> f64 {
        let vr = self.material.rayleigh_saw_velocity();
        vr / self.wavelength_m
    }

    /// Static dielectric capacitance $C_0 = N_p \epsilon_{eff} W$.
    pub fn static_capacitance(&self) -> f64 {
        let eps11 = self.material.epsilon_s[0][0];
        let eps33 = self.material.epsilon_s[2][2];
        let eps_eff = (eps11 * eps33).sqrt() + EPSILON_0; // substrate + air dielectric
        (self.num_finger_pairs as f64) * eps_eff * self.aperture_m
    }

    /// Extracts Modified Butterworth-Van Dyke (mBVD) equivalent circuit parameters.
    pub fn extract_mbvd(&self) -> MbvdParameters {
        let f_s = self.resonant_frequency();
        let k_eff_sq = self.material.electromechanical_coupling_kt2().min(0.20);
        let f_p =
            f_s * (1.0 + (4.0 / (std::f64::consts::PI * std::f64::consts::PI)) * k_eff_sq).sqrt();
        let c_0 = self.static_capacitance();
        let c_m = (8.0 / (std::f64::consts::PI * std::f64::consts::PI)) * k_eff_sq * c_0;
        let omega_s = 2.0 * std::f64::consts::PI * f_s;
        let l_m = 1.0 / (omega_s * omega_s * c_m);
        let r_m = omega_s * l_m / self.quality_factor;

        MbvdParameters {
            f_s,
            f_p,
            c_0,
            c_m,
            l_m,
            r_m,
            r_s: self.electrode_resistance_ohms,
            r_0: 1e10, // high dielectric isolation
            q_m: self.quality_factor,
            k_eff_sq,
        }
    }
}

/// Bulk Acoustic Wave (BAW) / Film Bulk Acoustic Resonator (FBAR).
/// Operates in hypersonic GHz regimes ($f \sim 1 - 25\text{ GHz}$).
#[derive(Debug, Clone, PartialEq)]
pub struct BawResonator {
    /// Active piezoelectric film material.
    pub material: PiezoelectricMaterial,
    /// Piezoelectric film thickness $d$ in meters.
    pub thickness_m: f64,
    /// Active resonator cross-sectional area $A$ in $\text{m}^2$.
    pub area_m2: f64,
    /// Unloaded mechanical quality factor $Q_m$.
    pub quality_factor: f64,
    /// Electrode parasitic series resistance $R_s$ in Ohms.
    pub electrode_resistance_ohms: f64,
}

impl BawResonator {
    pub fn new(
        material: PiezoelectricMaterial,
        thickness_m: f64,
        area_m2: f64,
        quality_factor: f64,
    ) -> Self {
        Self {
            material,
            thickness_m,
            area_m2,
            quality_factor,
            electrode_resistance_ohms: 0.8,
        }
    }

    /// Series acoustic resonant frequency: $f_s = v_{stiff} / (2d)$ (Hz).
    pub fn series_frequency(&self) -> f64 {
        let v = self.material.stiffened_longitudinal_velocity();
        v / (2.0 * self.thickness_m)
    }

    /// Parallel anti-resonant frequency: $f_p = f_s \sqrt{1 + \frac{8}{\pi^2} k_t^2}$ (Hz).
    pub fn parallel_frequency(&self) -> f64 {
        let fs = self.series_frequency();
        let kt2 = self.material.electromechanical_coupling_kt2();
        fs * (1.0 + (8.0 / (std::f64::consts::PI * std::f64::consts::PI)) * kt2).sqrt()
    }

    /// Static clamp capacitance: $C_0 = \frac{\epsilon_{33}^S A}{d}$ (Farads).
    pub fn static_capacitance(&self) -> f64 {
        let eps33 = self.material.epsilon_s[2][2];
        eps33 * self.area_m2 / self.thickness_m
    }

    /// Extracts Modified Butterworth-Van Dyke (mBVD) equivalent circuit parameters.
    pub fn extract_mbvd(&self) -> MbvdParameters {
        let f_s = self.series_frequency();
        let f_p = self.parallel_frequency();
        let c_0 = self.static_capacitance();
        let kt2 = self.material.electromechanical_coupling_kt2();
        let k_eff_sq = (std::f64::consts::PI * std::f64::consts::PI / 8.0)
            * ((f_p * f_p - f_s * f_s) / (f_p * f_p));
        let c_m = (8.0 / (std::f64::consts::PI * std::f64::consts::PI)) * kt2 * c_0;
        let omega_s = 2.0 * std::f64::consts::PI * f_s;
        let l_m = 1.0 / (omega_s * omega_s * c_m);
        let r_m = omega_s * l_m / self.quality_factor;

        MbvdParameters {
            f_s,
            f_p,
            c_0,
            c_m,
            l_m,
            r_m,
            r_s: self.electrode_resistance_ohms,
            r_0: 1e11,
            q_m: self.quality_factor,
            k_eff_sq,
        }
    }
}
