//! Electro-thermal hotspot nucleation, Ginzburg-Landau resistive domain growth, and re-trapping current.
//!
//! Formulates single-photon absorption, quasiparticle diffusion, sideway depairing current breakdown,
//! Joule heating vs. phonon substrate cooling, and normal domain collapse at the re-trapping threshold.

use super::nanowire_geometry::{NanowireGeometry, ELEMENTARY_CHARGE, PLANCK_H, SPEED_OF_LIGHT};

/// Hotspot nucleation and thermal expansion parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotspotDynamicsModel {
    /// Photon energy conversion efficiency to quasiparticles \u{03b7}_dep (typically 0.25 - 0.40).
    pub energy_deposition_efficiency: f64,
    /// Quasiparticle diffusion coefficient D_qp in m^2/s (typically ~0.5e-4 m^2/s).
    pub quasiparticle_diffusivity_m2_s: f64,
    /// Electron-phonon thermalization time \u{03c4}_th in picoseconds (typically 5 - 15 ps).
    pub thermalization_time_ps: f64,
    /// Acoustic Kapitza boundary thermal conductance to substrate h_K in W/(m^2*K) (typically ~1.5e5 W/(m^2*K)).
    pub kapitza_conductance_w_m2_k: f64,
    /// Thermal relaxation cooling time constant \u{03c4}_cool in picoseconds (typically 50 - 200 ps).
    pub cooling_time_ps: f64,
    /// Domain expansion velocity parameter v_norm in m/s (typically 500 - 2000 m/s).
    pub normal_domain_velocity_m_s: f64,
    /// Volumetric normal-state heat capacity C_v in J/(m^3*K) at low temperature.
    pub volumetric_heat_capacity_j_m3_k: f64,
}

impl HotspotDynamicsModel {
    /// Standard NbN hotspot parameters on silicon/sapphire.
    pub fn standard_nbn() -> Self {
        Self {
            energy_deposition_efficiency: 0.30,
            quasiparticle_diffusivity_m2_s: 0.5e-4,
            thermalization_time_ps: 10.0,
            kapitza_conductance_w_m2_k: 1.5e5,
            cooling_time_ps: 100.0,
            normal_domain_velocity_m_s: 1200.0,
            volumetric_heat_capacity_j_m3_k: 1.2e3,
        }
    }

    /// Absorbed photon energy E_ph = h * c / \u{03bb} in Joules for wavelength in nanometers.
    pub fn photon_energy_joules(&self, wavelength_nm: f64) -> f64 {
        let lambda_m = wavelength_nm * 1e-9;
        (PLANCK_H * SPEED_OF_LIGHT) / lambda_m
    }

    /// Photon energy in electronVolts (eV).
    pub fn photon_energy_ev(&self, wavelength_nm: f64) -> f64 {
        self.photon_energy_joules(wavelength_nm) / ELEMENTARY_CHARGE
    }

    /// Number of primary quasiparticles generated: N_qp \u{2248} \u{03b7}_dep * E_ph / \u{0394}(0).
    pub fn generated_quasiparticles_count(
        &self,
        geom: &NanowireGeometry,
        wavelength_nm: f64,
    ) -> f64 {
        let e_ph = self.photon_energy_joules(wavelength_nm);
        let delta = geom.superconducting_gap_zero_t_joules();
        (self.energy_deposition_efficiency * e_ph) / delta.max(1e-25)
    }

    /// Initial hotspot radius r_hs0 = sqrt(D_qp * \u{03c4}_th / 4) in nanometers.
    pub fn initial_hotspot_radius_nm(&self) -> f64 {
        let tau_s = self.thermalization_time_ps * 1e-12;
        let r_m = (self.quasiparticle_diffusivity_m2_s * tau_s / 4.0).sqrt();
        r_m * 1e9
    }

    /// Sideway current density J_side in A/m^2 when a hotspot of radius r_hs_nm forms:
    /// J_side = I_b / ((w - 2 * r_hs) * d).
    /// If 2 * r_hs >= w, the hotspot immediately spans the full width.
    pub fn sideway_current_density_a_m2(
        &self,
        geom: &NanowireGeometry,
        bias_current_ua: f64,
        hotspot_radius_nm: f64,
    ) -> f64 {
        let remaining_width_nm = (geom.width_nm - 2.0 * hotspot_radius_nm).max(0.1);
        let remaining_area_m2 = (remaining_width_nm * 1e-9) * (geom.thickness_nm * 1e-9);
        let i_b_amps = bias_current_ua * 1e-6;
        i_b_amps / remaining_area_m2
    }

    /// Operating critical current density J_c = I_c / (w * d) in A/m^2.
    pub fn critical_current_density_a_m2(&self, geom: &NanowireGeometry) -> f64 {
        let i_c_amps = geom.operational_critical_current_ua() * 1e-6;
        i_c_amps / geom.cross_sectional_area_m2()
    }

    /// Determines whether a single absorbed photon triggers resistive barrier breakdown:
    /// Breakdown occurs when J_side >= J_c.
    pub fn triggers_resistive_barrier(
        &self,
        geom: &NanowireGeometry,
        bias_current_ua: f64,
        wavelength_nm: f64,
    ) -> bool {
        let r_hs = self.initial_hotspot_radius_nm();
        let j_side = self.sideway_current_density_a_m2(geom, bias_current_ua, r_hs);
        let j_c = self.critical_current_density_a_m2(geom);
        let e_ph_ev = self.photon_energy_ev(wavelength_nm);

        // Physical threshold: sideway depairing and sufficient photon energy to exceed gap
        j_side >= j_c && e_ph_ev > geom.superconducting_gap_zero_t_mev() * 1e-3
    }

    /// Normal domain resistance R_hs(x_norm) = R_sq * (x_norm / w) in Ohms.
    pub fn normal_domain_resistance_ohms(
        &self,
        geom: &NanowireGeometry,
        domain_length_nm: f64,
    ) -> f64 {
        geom.sheet_resistance_ohms_per_sq * (domain_length_nm / geom.width_nm)
    }

    /// Re-trapping current I_r in microAmperes below which the hotspot collapses back to superconducting state:
    /// I_r \u{2248} I_c * sqrt(2 * h_K * (T_c - T_sub) * w / (\u{03c1}_N * J_c^2 * d)).
    /// In typical NbN nanowires, I_r / I_c \u{2248} 0.20 - 0.30.
    pub fn retrapping_current_ua(&self, geom: &NanowireGeometry) -> f64 {
        let i_c = geom.operational_critical_current_ua();
        let delta_t = (geom.critical_temperature_k - geom.substrate_temperature_k).max(0.1);
        let rho_n = (geom.sheet_resistance_ohms_per_sq * (geom.thickness_nm * 1e-9)).max(1e-12);
        let j_c = self.critical_current_density_a_m2(geom);

        let numer = 2.0 * self.kapitza_conductance_w_m2_k * delta_t;
        let denom = (rho_n * j_c * j_c * (geom.thickness_nm * 1e-9)).max(1e-6);
        let ratio = (numer / denom).sqrt().clamp(0.15, 0.40);

        i_c * ratio
    }

    /// Thermal domain growth velocity dx_norm / dt in m/s:
    /// v_domain = v_norm * (I / I_r - 1).
    pub fn domain_growth_velocity_m_s(&self, current_ua: f64, retrapping_current_ua: f64) -> f64 {
        if retrapping_current_ua <= 0.0 {
            return 0.0;
        }
        let ratio = current_ua / retrapping_current_ua;
        self.normal_domain_velocity_m_s * (ratio - 1.0)
    }

    /// Joule heating power P_J = I^2 * R_hs in Watts.
    pub fn joule_heating_power_watts(&self, current_ua: f64, resistance_ohms: f64) -> f64 {
        let i_a = current_ua * 1e-6;
        i_a * i_a * resistance_ohms
    }

    /// Phonon substrate cooling power P_cool = h_K * A_domain * (T - T_sub) in Watts.
    pub fn substrate_cooling_power_watts(
        &self,
        geom: &NanowireGeometry,
        domain_length_nm: f64,
        hotspot_temp_k: f64,
    ) -> f64 {
        let domain_area_m2 = (domain_length_nm * 1e-9) * (geom.width_nm * 1e-9);
        let delta_t = (hotspot_temp_k - geom.substrate_temperature_k).max(0.0);
        self.kapitza_conductance_w_m2_k * domain_area_m2 * delta_t
    }
}
