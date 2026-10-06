#![deny(unsafe_code)]

//! Microscopic Acoustic Phonon-Electron Scattering and Lattice Deformation Potential Engine.
//!
//! Evaluates the spatio-temporal lattice deformation potential:
//! $$V_{\mathrm{def}}(x, t) = \Xi \cdot \frac{\partial u(x, t)}{\partial x} = \Xi \cdot S_0 \cos(q x - \omega_q t)$$
//! and computes momentum-energy conservation kinematics for acoustic phonon emission and absorption.

use super::schroedinger_stepper::{ELEMENTARY_CHARGE, HBAR};

/// Parameters defining a propagating longitudinal acoustic (LA) phonon mode.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticPhononMode {
    /// Phonon wavevector q in rad/nm (e.g. 0.4 rad/nm).
    pub q_rad_nm: f64,
    /// Longitudinal sound velocity in m/s (e.g. 5,240 m/s for GaAs).
    pub sound_velocity_m_s: f64,
    /// Peak acoustic strain amplitude S0 = du/dx (dimensionless, e.g. 1.0e-3).
    pub strain_amplitude: f64,
    /// Acoustic deformation potential constant Xi in eV (e.g. 8.6 eV for GaAs, 12.0 eV for Si).
    pub deformation_potential_ev: f64,
    /// Acoustic medium mass density in kg/m^3 (e.g. 5,317 kg/m^3 for GaAs).
    pub material_density_kg_m3: f64,
}

impl Default for AcousticPhononMode {
    fn default() -> Self {
        Self {
            q_rad_nm: 0.4,
            sound_velocity_m_s: 5240.0,
            strain_amplitude: 1.5e-3,
            deformation_potential_ev: 9.0,
            material_density_kg_m3: 5317.0,
        }
    }
}

impl AcousticPhononMode {
    /// Computes the phonon angular frequency omega_q in rad/s: omega_q = v_s * q.
    pub fn angular_frequency_rad_s(&self) -> f64 {
        let q_m = self.q_rad_nm * 1e9;
        self.sound_velocity_m_s * q_m
    }

    /// Computes the linear acoustic phonon frequency in GHz.
    pub fn frequency_ghz(&self) -> f64 {
        self.angular_frequency_rad_s() / (2.0 * std::f64::consts::PI * 1e9)
    }

    /// Computes the quantum phonon energy hbar * omega_q in meV.
    pub fn phonon_energy_mev(&self) -> f64 {
        let energy_j = HBAR * self.angular_frequency_rad_s();
        (energy_j / ELEMENTARY_CHARGE) * 1e3
    }

    /// Computes the spatial acoustic deformation potential energy profile V_def(x, t) in eV.
    pub fn deformation_potential_at(&self, x_coords_nm: &[f64], time_fs: f64) -> Vec<f64> {
        let omega = self.angular_frequency_rad_s();
        let t_s = time_fs * 1e-15;
        let q_nm = self.q_rad_nm;
        let v_amp_ev = self.deformation_potential_ev * self.strain_amplitude;

        let mut v_def = Vec::with_capacity(x_coords_nm.len());
        for &x in x_coords_nm {
            let phase = q_nm * x - omega * t_s;
            v_def.push(v_amp_ev * phase.cos());
        }
        v_def
    }

    /// Computes quantum scattering kinematics for an electron of wavevector k0 and energy E0.
    pub fn evaluate_scattering_kinematics(
        &self,
        k0_rad_nm: f64,
        electron_energy_ev: f64,
        effective_mass_ratio: f64,
    ) -> InelasticScatteringKinematics {
        let q = self.q_rad_nm;
        let hbar_omega_ev = self.phonon_energy_mev() * 1e-3;

        // Phonon absorption: k_f = k0 + q, E_f = E0 + hbar * omega
        let k_abs = k0_rad_nm + q;
        let e_abs = electron_energy_ev + hbar_omega_ev;

        // Phonon emission: k_f = k0 - q, E_f = E0 - hbar * omega
        let k_em = k0_rad_nm - q;
        let e_em = (electron_energy_ev - hbar_omega_ev).max(0.0);

        // Transition matrix element |M|^2 = Xi^2 * hbar * q^2 / (2 * rho * omega)
        let q_m = q * 1e9;
        let omega = self.angular_frequency_rad_s();
        let rho = self.material_density_kg_m3;
        let xi_j = self.deformation_potential_ev * ELEMENTARY_CHARGE;
        let matrix_element_sq = (xi_j * xi_j * HBAR * q_m * q_m) / (2.0 * rho * omega);

        // Density of states prefactor in 1D: D(E) = (1 / (pi * hbar)) * sqrt(m* / (2 E))
        let m_eff = effective_mass_ratio * super::schroedinger_stepper::ELECTRON_MASS_KG;
        let dos_1d = if electron_energy_ev > 1e-6 {
            (1.0 / (std::f64::consts::PI * HBAR))
                * (m_eff / (2.0 * electron_energy_ev * ELEMENTARY_CHARGE)).sqrt()
        } else {
            1e18
        };

        let rate_s_inv = (2.0 * std::f64::consts::PI / HBAR) * matrix_element_sq * dos_1d;
        let scattering_time_fs = (1.0 / rate_s_inv.max(1e10)) * 1e15;

        InelasticScatteringKinematics {
            initial_k_rad_nm: k0_rad_nm,
            initial_energy_ev: electron_energy_ev,
            phonon_wavevector_q_rad_nm: q,
            phonon_energy_mev: self.phonon_energy_mev(),
            absorption_k_rad_nm: k_abs,
            absorption_energy_ev: e_abs,
            emission_k_rad_nm: k_em,
            emission_energy_ev: e_em,
            scattering_rate_s_inv: rate_s_inv,
            scattering_lifetime_fs: scattering_time_fs,
        }
    }
}

/// Kinematic output of acoustic phonon emission and absorption channels.
#[derive(Debug, Clone, PartialEq)]
pub struct InelasticScatteringKinematics {
    pub initial_k_rad_nm: f64,
    pub initial_energy_ev: f64,
    pub phonon_wavevector_q_rad_nm: f64,
    pub phonon_energy_mev: f64,
    pub absorption_k_rad_nm: f64,
    pub absorption_energy_ev: f64,
    pub emission_k_rad_nm: f64,
    pub emission_energy_ev: f64,
    pub scattering_rate_s_inv: f64,
    pub scattering_lifetime_fs: f64,
}
