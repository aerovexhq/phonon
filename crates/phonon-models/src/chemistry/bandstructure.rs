//! Multi-valley electronic bandstructure, effective density of states, and bandgap narrowing.

use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, PLANCK_CONSTANT, T_REF};

/// Rest mass of a free electron $m_0$ in kilograms ($kg$).
pub const ELECTRON_REST_MASS: f64 = 9.109_383_701_5e-31;

/// Electronic bandstructure characteristics of a semiconductor crystal.
#[derive(Debug, Clone, PartialEq)]
pub struct Bandstructure {
    /// Fundamental bandgap at 0 Kelvin $E_g(0)$ in electron-volts ($eV$).
    pub bandgap_0k_ev: f64,
    /// Varshni temperature parameter $\alpha$ in $eV/K$.
    pub varshni_alpha: f64,
    /// Varshni temperature parameter $\beta$ in $K$.
    pub varshni_beta: f64,
    /// Electron affinity $\chi$ at reference temperature in electron-volts ($eV$).
    pub electron_affinity_ev: f64,
    /// Number of equivalent conduction band minima valleys $M_c$ (e.g. 6 for Si, 1 for GaAs).
    pub conduction_valleys: usize,
    /// True if fundamental bandgap is direct (e.g. GaAs, GaN, InP), false if indirect (e.g. Si, Ge, SiC).
    pub is_direct: bool,
    /// Longitudinal electron effective mass $m_l^* / m_0$.
    pub me_longitudinal: f64,
    /// Transverse electron effective mass $m_t^* / m_0$.
    pub me_transverse: f64,
    /// Heavy hole effective mass $m_{hh}^* / m_0$.
    pub mh_heavy: f64,
    /// Light hole effective mass $m_{lh}^* / m_0$.
    pub mh_light: f64,
    /// Slotboom bandgap narrowing prefactor $\Delta E_{g0}$ in $eV$.
    pub bgn_prefactor_ev: f64,
    /// Slotboom reference doping concentration $N_{ref}$ in $\text{m}^{-3}$.
    pub bgn_n_ref: f64,
    /// Slotboom shape parameter $C$.
    pub bgn_c: f64,
}

impl Bandstructure {
    /// Computes the temperature-dependent fundamental bandgap $E_g(T)$ in $eV$ via Varshni's equation:
    /// $$E_g(T) = E_g(0) - \frac{\alpha T^2}{T + \beta}$$
    pub fn bandgap_ev(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(0.1);
        let eg = self.bandgap_0k_ev - (self.varshni_alpha * t * t) / (t + self.varshni_beta);
        eg.max(0.01)
    }

    /// Computes the temperature-dependent electron affinity $\chi(T)$ in $eV$:
    /// $$\chi(T) = \chi(300) - \frac{1}{2} [E_g(T) - E_g(300)]$$
    pub fn electron_affinity_ev(&self, temp_k: f64) -> f64 {
        let eg_t = self.bandgap_ev(temp_k);
        let eg_300 = self.bandgap_ev(T_REF);
        self.electron_affinity_ev - 0.5 * (eg_t - eg_300)
    }

    /// Computes the electron density-of-states effective mass ratio $m_{de}^* / m_0$:
    /// $$m_{de}^* = M_c^{2/3} (m_l^* m_t^{*2})^{1/3}$$
    pub fn me_dos(&self) -> f64 {
        let mc = self.conduction_valleys as f64;
        let valley_factor = mc.powf(2.0 / 3.0);
        let mass_product = (self.me_longitudinal * self.me_transverse * self.me_transverse).cbrt();
        valley_factor * mass_product
    }

    /// Computes the hole density-of-states effective mass ratio $m_{dh}^* / m_0$:
    /// $$m_{dh}^* = (m_{hh}^{*3/2} + m_{lh}^{*3/2})^{2/3}$$
    pub fn mh_dos(&self) -> f64 {
        let sum_mass = self.mh_heavy.powf(1.5) + self.mh_light.powf(1.5);
        sum_mass.powf(2.0 / 3.0)
    }

    /// Computes the effective conduction band density of states $N_c(T)$ in $\text{m}^{-3}$:
    /// $$N_c(T) = 2 \left( \frac{2\pi m_{de}^* m_0 k_B T}{h^2} \right)^{3/2}$$
    pub fn effective_nc(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(1.0);
        let m_de = self.me_dos() * ELECTRON_REST_MASS;
        let factor = (2.0 * std::f64::consts::PI * m_de * BOLTZMANN_CONSTANT * t)
            / (PLANCK_CONSTANT * PLANCK_CONSTANT);
        2.0 * factor.powf(1.5)
    }

    /// Computes the effective valence band density of states $N_v(T)$ in $\text{m}^{-3}$:
    /// $$N_v(T) = 2 \left( \frac{2\pi m_{dh}^* m_0 k_B T}{h^2} \right)^{3/2}$$
    pub fn effective_nv(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(1.0);
        let m_dh = self.mh_dos() * ELECTRON_REST_MASS;
        let factor = (2.0 * std::f64::consts::PI * m_dh * BOLTZMANN_CONSTANT * t)
            / (PLANCK_CONSTANT * PLANCK_CONSTANT);
        2.0 * factor.powf(1.5)
    }

    /// Computes the intrinsic carrier concentration $n_i(T)$ in $\text{m}^{-3}$:
    /// $$n_i(T) = \sqrt{N_c(T) N_v(T)} \exp\left( -\frac{q E_g(T)}{2 k_B T} \right)$$
    pub fn intrinsic_carrier_density(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(1.0);
        let nc = self.effective_nc(t);
        let nv = self.effective_nv(t);
        let eg_joules = self.bandgap_ev(t) * ELEMENTARY_CHARGE;
        let exp_arg = -eg_joules / (2.0 * BOLTZMANN_CONSTANT * t);
        // Clamp exponential argument to prevent floating-point underflow
        let clamped_exp = exp_arg.clamp(-80.0, 0.0);
        (nc * nv).sqrt() * clamped_exp.exp()
    }

    /// Computes the heavy-doping Bandgap Narrowing (BGN) $\Delta E_g(N)$ in $eV$
    /// via the Slotboom formulation:
    /// $$\Delta E_g = \Delta E_{g0} \left[ \ln\left( \frac{N}{N_{ref}} \right) + \sqrt{\ln^2\left(\frac{N}{N_{ref}}\right) + C} \right]$$
    pub fn bandgap_narrowing_ev(&self, total_doping_m3: f64) -> f64 {
        if total_doping_m3 <= 1.0e20 {
            // Unmeasurable narrowing below 1e14 cm^-3
            return 0.0;
        }
        let ratio = (total_doping_m3 / self.bgn_n_ref).max(1e-10);
        let ln_ratio = ratio.ln();
        let bgn = self.bgn_prefactor_ev * (ln_ratio + (ln_ratio * ln_ratio + self.bgn_c).sqrt());
        bgn.max(0.0)
    }

    /// Computes the effective intrinsic carrier concentration $n_{ie}(T, N)$ in $\text{m}^{-3}$
    /// accounting for bandgap narrowing:
    /// $$n_{ie}^2 = n_i^2 \exp\left( \frac{q \Delta E_g}{k_B T} \right)$$
    pub fn effective_intrinsic_carrier_density(&self, temp_k: f64, total_doping_m3: f64) -> f64 {
        let ni = self.intrinsic_carrier_density(temp_k);
        let delta_eg = self.bandgap_narrowing_ev(total_doping_m3);
        if delta_eg <= 0.0 {
            return ni;
        }
        let t = temp_k.max(1.0);
        let arg = (delta_eg * ELEMENTARY_CHARGE) / (2.0 * BOLTZMANN_CONSTANT * t);
        ni * arg.clamp(0.0, 50.0).exp()
    }

    // =========================================================================
    // Standard Bandstructure Presets
    // =========================================================================

    /// Silicon ($Si$): Indirect bandgap ($E_g(0) = 1.166 \text{ eV}$, $E_g(300) = 1.124 \text{ eV}$), $M_c = 6$.
    pub fn silicon() -> Self {
        Self {
            bandgap_0k_ev: 1.166,
            varshni_alpha: 4.73e-4,
            varshni_beta: 636.0,
            electron_affinity_ev: 4.05,
            conduction_valleys: 6,
            is_direct: false,
            me_longitudinal: 0.98,
            me_transverse: 0.19,
            mh_heavy: 0.49,
            mh_light: 0.16,
            bgn_prefactor_ev: 6.92e-3,
            bgn_n_ref: 1.3e23, // 1.3e17 cm^-3 in m^-3
            bgn_c: 0.5,
        }
    }

    /// Germanium ($Ge$): Indirect bandgap ($E_g(0) = 0.7437 \text{ eV}$, $E_g(300) = 0.66 \text{ eV}$), $M_c = 4$.
    pub fn germanium() -> Self {
        Self {
            bandgap_0k_ev: 0.7437,
            varshni_alpha: 4.77e-4,
            varshni_beta: 235.0,
            electron_affinity_ev: 4.00,
            conduction_valleys: 4,
            is_direct: false,
            me_longitudinal: 1.59,
            me_transverse: 0.082,
            mh_heavy: 0.33,
            mh_light: 0.043,
            bgn_prefactor_ev: 5.0e-3,
            bgn_n_ref: 1.0e23,
            bgn_c: 0.5,
        }
    }

    /// Gallium Arsenide ($GaAs$): Direct bandgap ($E_g(0) = 1.519 \text{ eV}$, $E_g(300) = 1.424 \text{ eV}$), $M_c = 1$.
    pub fn gallium_arsenide() -> Self {
        Self {
            bandgap_0k_ev: 1.519,
            varshni_alpha: 5.405e-4,
            varshni_beta: 204.0,
            electron_affinity_ev: 4.07,
            conduction_valleys: 1,
            is_direct: true,
            me_longitudinal: 0.067,
            me_transverse: 0.067,
            mh_heavy: 0.45,
            mh_light: 0.082,
            bgn_prefactor_ev: 8.0e-3,
            bgn_n_ref: 5.0e22,
            bgn_c: 0.5,
        }
    }

    /// Gallium Nitride ($GaN$): Wide direct bandgap ($E_g(0) = 3.47 \text{ eV}$, $E_g(300) = 3.44 \text{ eV}$), $M_c = 1$.
    pub fn gallium_nitride() -> Self {
        Self {
            bandgap_0k_ev: 3.47,
            varshni_alpha: 9.09e-4,
            varshni_beta: 830.0,
            electron_affinity_ev: 4.10,
            conduction_valleys: 1,
            is_direct: true,
            me_longitudinal: 0.20,
            me_transverse: 0.20,
            mh_heavy: 0.80,
            mh_light: 0.30,
            bgn_prefactor_ev: 15.0e-3,
            bgn_n_ref: 1.0e24,
            bgn_c: 0.5,
        }
    }

    /// Silicon Carbide ($4H-SiC$): Wide indirect bandgap ($E_g(0) = 3.26 \text{ eV}$, $E_g(300) = 3.23 \text{ eV}$), $M_c = 3$.
    pub fn silicon_carbide_4h() -> Self {
        Self {
            bandgap_0k_ev: 3.26,
            varshni_alpha: 6.5e-4,
            varshni_beta: 1200.0,
            electron_affinity_ev: 3.70,
            conduction_valleys: 3,
            is_direct: false,
            me_longitudinal: 0.58,
            me_transverse: 0.31,
            mh_heavy: 1.00,
            mh_light: 0.45,
            bgn_prefactor_ev: 12.0e-3,
            bgn_n_ref: 5.0e23,
            bgn_c: 0.5,
        }
    }

    /// Indium Phosphide ($InP$): Direct bandgap ($E_g(0) = 1.424 \text{ eV}$, $E_g(300) = 1.344 \text{ eV}$), $M_c = 1$.
    pub fn indium_phosphide() -> Self {
        Self {
            bandgap_0k_ev: 1.424,
            varshni_alpha: 4.1e-4,
            varshni_beta: 136.0,
            electron_affinity_ev: 4.38,
            conduction_valleys: 1,
            is_direct: true,
            me_longitudinal: 0.08,
            me_transverse: 0.08,
            mh_heavy: 0.60,
            mh_light: 0.089,
            bgn_prefactor_ev: 7.0e-3,
            bgn_n_ref: 8.0e22,
            bgn_c: 0.5,
        }
    }
}
