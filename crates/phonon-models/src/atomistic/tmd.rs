//! Transition Metal Dichalcogenide (TMD) monolayer 2D material models.
//!
//! Formulates:
//! - Direct bandgap at $K, K'$ valleys with massive Dirac dispersion and spin-orbit splitting $\Delta_{SO}$.
//! - Chalcogen vacancy defect scattering and substrate surface optical phonon (SOP) scattering.
//! - Quasi-ballistic transport with transmission coefficient $\mathcal{T} = \lambda_{mfp} / (L + \lambda_{mfp})$.

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, EPSILON_0, EPSILON_R_OX, H_BAR};

/// Free electron rest mass in kg.
const ELECTRON_MASS: f64 = 9.109_383_701_5e-31;

/// Monolayer Transition Metal Dichalcogenide (TMD) species.
#[derive(Debug, Clone, PartialEq)]
pub struct TmdMonolayer {
    /// Chemical identifier (e.g. "MoS2", "WS2", "MoSe2", "WSe2").
    pub species: String,
    /// In-plane hexagonal lattice constant $a$ in meters ($m$).
    pub lattice_constant_m: f64,
    /// Monolayer physical thickness $d_{tmd}$ in meters ($m$) ($\approx 0.65\text{ nm}$).
    pub thickness_m: f64,
    /// Direct electronic bandgap $E_g$ at $K$ valley in electron-volts ($eV$).
    pub bandgap_ev: f64,
    /// Valence band spin-orbit splitting $\Delta_{SO}$ at $K$ valley in $eV$.
    pub spin_orbit_splitting_ev: f64,
    /// Conduction band effective mass ratio $m_e^* / m_0$.
    pub me_ratio: f64,
    /// Valence band effective mass ratio $m_h^* / m_0$.
    pub mh_ratio: f64,
    /// Acoustic phonon mobility at 300K in $m^2 / (V \cdot s)$.
    pub mu_acoustic_300k: f64,
    /// Chalcogen vacancy defect areal density $N_{vac}$ in $m^{-2}$ (e.g. $10^{16} \text{ m}^{-2} = 10^{12} \text{ cm}^{-2}$).
    pub vacancy_density_m2: f64,
    /// Dielectric constant $\epsilon_r$ of the TMD monolayer.
    pub epsilon_r: f64,
}

impl TmdMonolayer {
    /// Molybdenum Disulfide ($\text{MoS}_2$) monolayer preset ($E_g = 1.82\text{ eV}$, $\Delta_{SO} = 0.15\text{ eV}$).
    pub fn mos2() -> Self {
        Self {
            species: "MoS2".to_string(),
            lattice_constant_m: 3.16e-10, // 3.16 A
            thickness_m: 0.65e-9,
            bandgap_ev: 1.82,
            spin_orbit_splitting_ev: 0.15,
            me_ratio: 0.45,
            mh_ratio: 0.54,
            mu_acoustic_300k: 0.020,  // 200 cm^2 / (V*s)
            vacancy_density_m2: 1e16, // 1e12 cm^-2
            epsilon_r: 4.0,
        }
    }

    /// Tungsten Disulfide ($\text{WS}_2$) monolayer preset ($E_g = 1.98\text{ eV}$, $\Delta_{SO} = 0.43\text{ eV}$).
    pub fn ws2() -> Self {
        Self {
            species: "WS2".to_string(),
            lattice_constant_m: 3.15e-10,
            thickness_m: 0.65e-9,
            bandgap_ev: 1.98,
            spin_orbit_splitting_ev: 0.43,
            me_ratio: 0.33,
            mh_ratio: 0.40,
            mu_acoustic_300k: 0.035, // 350 cm^2 / (V*s)
            vacancy_density_m2: 1e16,
            epsilon_r: 4.5,
        }
    }

    /// Molybdenum Diselenide ($\text{MoSe}_2$) monolayer preset ($E_g = 1.55\text{ eV}$, $\Delta_{SO} = 0.18\text{ eV}$).
    pub fn mose2() -> Self {
        Self {
            species: "MoSe2".to_string(),
            lattice_constant_m: 3.29e-10,
            thickness_m: 0.70e-9,
            bandgap_ev: 1.55,
            spin_orbit_splitting_ev: 0.18,
            me_ratio: 0.50,
            mh_ratio: 0.60,
            mu_acoustic_300k: 0.015,
            vacancy_density_m2: 1e16,
            epsilon_r: 4.2,
        }
    }

    /// Tungsten Diselenide ($\text{WSe}_2$) monolayer preset ($E_g = 1.65\text{ eV}$, $\Delta_{SO} = 0.46\text{ eV}$).
    pub fn wse2() -> Self {
        Self {
            species: "WSe2".to_string(),
            lattice_constant_m: 3.28e-10,
            thickness_m: 0.70e-9,
            bandgap_ev: 1.65,
            spin_orbit_splitting_ev: 0.46,
            me_ratio: 0.35,
            mh_ratio: 0.45,
            mu_acoustic_300k: 0.030,
            vacancy_density_m2: 1e16,
            epsilon_r: 4.6,
        }
    }

    /// Effective conduction band mass $m_e^* = (m_e^* / m_0) m_0$ in kg.
    #[inline]
    pub fn electron_mass_kg(&self) -> f64 {
        self.me_ratio * ELECTRON_MASS
    }

    /// Conduction band dispersion $E_c(q)$ (in $eV$) measured from midgap,
    /// where $q$ is the wavevector magnitude relative to the $K$ point in $\text{m}^{-1}$:
    /// $$E_c(q) = \frac{E_g}{2} + \frac{\hbar^2 q^2}{2 m_e^*}$$
    pub fn conduction_dispersion_ev(&self, q_mag: f64) -> f64 {
        let q_joules = (H_BAR * H_BAR * q_mag * q_mag) / (2.0 * self.electron_mass_kg());
        self.bandgap_ev / 2.0 + q_joules / ELEMENTARY_CHARGE
    }

    /// Valence band dispersion $E_v(q, s)$ (in $eV$) for spin $s \in \{+1, -1\}$:
    /// $$E_v(q, s) = -\frac{E_g}{2} + s \frac{\Delta_{SO}}{2} - \frac{\hbar^2 q^2}{2 m_h^*}$$
    pub fn valence_dispersion_ev(&self, q_mag: f64, spin_up: bool) -> f64 {
        let mh_kg = self.mh_ratio * ELECTRON_MASS;
        let q_joules = (H_BAR * H_BAR * q_mag * q_mag) / (2.0 * mh_kg);
        let s = if spin_up { 1.0 } else { -1.0 };
        -self.bandgap_ev / 2.0 + s * (self.spin_orbit_splitting_ev / 2.0)
            - q_joules / ELEMENTARY_CHARGE
    }

    /// Defect-limited carrier mobility $\mu_{def}$ in $m^2 / (V \cdot s)$
    /// due to neutral and charged chalcogen vacancy scattering:
    /// $$\mu_{def} \approx \frac{q \hbar}{m_e^* U_{scat}^2 N_{vac}}$$
    pub fn vacancy_mobility(&self) -> f64 {
        if self.vacancy_density_m2 <= 1e12 {
            return 1e4;
        }
        let q = ELEMENTARY_CHARGE;
        let m_eff = self.electron_mass_kg();
        // Scattering potential matrix element U_scat ~ 2.0 eV * Angstrom^2
        let u_scat = 2.0 * ELEMENTARY_CHARGE * 1e-20;
        let num = q * H_BAR;
        let den = m_eff * u_scat * self.vacancy_density_m2;
        (num / den).clamp(1e-4, 1e4)
    }

    /// Total effective carrier mobility $\mu_{eff}$ in $m^2 / (V \cdot s)$ at temperature $T$:
    /// $$\frac{1}{\mu_{eff}} = \frac{1}{\mu_{ph}(T)} + \frac{1}{\mu_{def}}$$
    pub fn effective_mobility(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(1.0);
        let mu_ph = self.mu_acoustic_300k * (300.0 / t).powf(1.4);
        let mu_def = self.vacancy_mobility();
        1.0 / (1.0 / mu_ph + 1.0 / mu_def)
    }

    /// Computes field-effect drain current $I_{DS}$ (in Amperes) in a monolayer TMD FET
    /// with channel width $W$, channel length $L$, equivalent oxide thickness $t_{ox}$,
    /// and gate/drain voltages $(V_{GS}, V_{DS})$:
    #[allow(clippy::too_many_arguments)]
    pub fn evaluate_fet_current(
        &self,
        v_gs: f64,
        v_ds: f64,
        v_th: f64,
        width_m: f64,
        length_m: f64,
        tox_m: f64,
        temp_k: f64,
    ) -> f64 {
        if v_ds.abs() < 1e-12 {
            return 0.0;
        }

        let is_neg = v_ds < 0.0;
        let vds_mag = v_ds.abs();
        let mu_eff = self.effective_mobility(temp_k);
        let cox = (EPSILON_R_OX * EPSILON_0) / tox_m.max(0.5e-9);

        let vov = v_gs - v_th;
        let kb_t = BOLTZMANN_CONSTANT * temp_k.max(1.0);
        let vt = kb_t / ELEMENTARY_CHARGE;
        let ss = 1.1 * vt * std::f64::consts::LN_10;

        // Smooth overdrive voltage
        let vgst_eff = ss * (1.0 + (vov / ss).clamp(-40.0, 40.0).exp()).ln();

        // 2D electron density n_2D
        let n_2d = (cox * vgst_eff / ELEMENTARY_CHARGE).max(1e14);

        // Mean free path: lambda_mfp = (2 * hbar * mu / q) * sqrt(pi * n_2D)
        let lambda_mfp =
            (2.0 * H_BAR * mu_eff / ELEMENTARY_CHARGE) * (std::f64::consts::PI * n_2d).sqrt();
        let transmission = (lambda_mfp / (length_m + lambda_mfp)).clamp(0.01, 1.0);

        // Saturation voltage
        let vsat_velocity = 1.5e5; // 1.5e7 cm/s saturation velocity in TMDs
        let vdsat =
            (vgst_eff * vsat_velocity * length_m) / (vgst_eff * mu_eff + vsat_velocity * length_m);

        let vds_eff = vds_mag / (1.0 + (vds_mag / vdsat.max(0.01)).powi(2)).sqrt();
        let linear_factor = vgst_eff * vds_eff - 0.5 * vds_eff * vds_eff;

        let ids_mag = transmission * (width_m / length_m) * mu_eff * cox * linear_factor;

        if is_neg {
            -ids_mag
        } else {
            ids_mag
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmd_monolayer_spin_orbit_splitting() {
        let mos2 = TmdMonolayer::mos2();
        let ws2 = TmdMonolayer::ws2();

        // WS2 has heavier tungsten atoms, resulting in much larger spin-orbit coupling than MoS2
        assert_eq!(mos2.spin_orbit_splitting_ev, 0.15);
        assert_eq!(ws2.spin_orbit_splitting_ev, 0.43);
        assert!(ws2.spin_orbit_splitting_ev > mos2.spin_orbit_splitting_ev * 2.5);

        // Valence band edge at K point differs by spin-orbit splitting
        let ev_up = ws2.valence_dispersion_ev(0.0, true);
        let ev_down = ws2.valence_dispersion_ev(0.0, false);
        assert!((ev_up - ev_down - ws2.spin_orbit_splitting_ev).abs() < 1e-6);
    }

    #[test]
    fn test_tmd_fet_conduction() {
        let mos2 = TmdMonolayer::mos2();
        let i_off = mos2.evaluate_fet_current(0.1, 0.5, 0.35, 1e-6, 30e-9, 1.2e-9, 300.0);
        let i_on = mos2.evaluate_fet_current(1.0, 0.5, 0.35, 1e-6, 30e-9, 1.2e-9, 300.0);

        assert!(
            i_on > i_off * 100.0,
            "FET on-current must be much higher than off-current: {} vs {}",
            i_on,
            i_off
        );
        assert!(i_on > 0.0);
    }
}
