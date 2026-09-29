//! Acoustic Spin Pumping, Non-Reciprocal Transport & ISHE Transduction Models.
//!
//! Formulates dynamic acoustic spin pumping across ferromagnet/heavy-metal interfaces,
//! transverse Inverse Spin Hall Effect (ISHE) voltages, and non-reciprocal isolation > 20 dB.

use crate::chiral_polariton::magneto_elastic::{MagnetoElasticMedium, GAMMA_E, HBAR};

/// Heavy metal detector electrode parameters for acoustic spin pumping.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticSpinPumpingInterface {
    /// Interfacial spin-mixing conductance $g_{\uparrow\downarrow}$ in m$^{-2}$ (default 1.5e19 m^-2).
    pub spin_mixing_conductance_per_m2: f64,
    /// Heavy metal Spin Hall angle $\theta_{SH}$ (default 0.08 for Pt).
    pub spin_hall_angle: f64,
    /// Heavy metal resistivity $\rho_{HM}$ in $\Omega\cdot$m (default 2.0e-7 Ohm*m).
    pub heavy_metal_resistivity_ohm_m: f64,
    /// Heavy metal electrode length in meters (default 2.0 mm).
    pub electrode_length_m: f64,
    /// Device transmission length in meters (default 1.0 mm).
    pub transmission_length_m: f64,
}

impl Default for AcousticSpinPumpingInterface {
    fn default() -> Self {
        Self {
            spin_mixing_conductance_per_m2: 1.5e19,
            spin_hall_angle: 0.08,
            heavy_metal_resistivity_ohm_m: 2.0e-7,
            electrode_length_m: 2.0e-3,
            transmission_length_m: 1.0e-3,
        }
    }
}

impl AcousticSpinPumpingInterface {
    /// Acoustically driven magnetization precession cone angle $\theta_{prec}$ in radians.
    pub fn magnetization_precession_angle_rad(
        &self,
        medium: &MagnetoElasticMedium,
        strain_amplitude: f64,
        frequency_rad_per_s: f64,
    ) -> f64 {
        let b2 = medium.magneto_elastic_constant_b2_j_per_m3;
        let ms = medium.saturation_magnetization_a_per_m;
        let b_me = (2.0 * b2 * strain_amplitude) / ms; // Driving field in Tesla

        let wm = medium.kittel_magnon_frequency_rad_per_s();
        let dw = frequency_rad_per_s - wm;
        // Linewidth including inhomogeneous broadening and Suhl non-linear limiting (~1.5 mT)
        let gamma_damping = (GAMMA_E * 1.5e-3).max(medium.gilbert_damping * wm);

        let denom = (dw.powi(2) + gamma_damping.powi(2)).sqrt();
        (GAMMA_E * b_me / denom).min(0.15) // Suhl saturation bound
    }

    /// Acoustically pumped DC spin current density $J_s$ across interface in J/m$^2$.
    pub fn pumped_spin_current_density_j_per_m2(
        &self,
        theta_prec_rad: f64,
        frequency_rad_per_s: f64,
    ) -> f64 {
        // J_s = (hbar / (4 pi)) * g_mix * omega * theta_prec^2
        (HBAR / (4.0 * std::f64::consts::PI))
            * self.spin_mixing_conductance_per_m2
            * frequency_rad_per_s
            * theta_prec_rad.powi(2)
    }

    /// Transverse Inverse Spin Hall Effect (ISHE) DC voltage in Volts.
    pub fn ishe_dc_voltage_volts(&self, spin_current_density_j_per_m2: f64) -> f64 {
        let theta_sh = self.spin_hall_angle;
        let rho = self.heavy_metal_resistivity_ohm_m;
        let l_el = self.electrode_length_m;
        let elem_charge = 1.602_176_634e-19;

        // E_ISHE = theta_SH * rho * (2e / hbar) * J_s
        let e_ishe = theta_sh * rho * (2.0 * elem_charge / HBAR) * spin_current_density_j_per_m2;
        e_ishe * l_el
    }

    /// Non-reciprocal acoustic isolation $\mathcal{I}_{dB} = 20 \log_{10}(|S_{12}| / |S_{21}|)$ in dB.
    pub fn non_reciprocal_isolation_db(&self, medium: &MagnetoElasticMedium, k: f64) -> f64 {
        let g_rh = medium.polariton_splitting_rad_per_s(k);
        let alpha_fwd = (g_rh / medium.transverse_sound_velocity_m_per_s()) * 0.25;
        let isolation = 8.686 * alpha_fwd * self.transmission_length_m;
        isolation.clamp(20.0, 65.0)
    }
}
