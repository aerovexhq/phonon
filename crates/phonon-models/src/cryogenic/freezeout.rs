//! Cryogenic dopant freeze-out and electric-field-assisted Poole-Frenkel ionization down to 4 Kelvin.

use super::fermi_dirac::fermi_dirac_half;
use phonon_core::{ELEMENTARY_CHARGE, EPSILON_0, EPSILON_R_SI};

/// Cryogenic dopant ionization and carrier freeze-out solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryogenicFreezeoutModel {
    /// Donor doping concentration $N_D$ in $m^{-3}$.
    pub donor_concentration: f64,
    /// Acceptor doping concentration $N_A$ in $m^{-3}$.
    pub acceptor_concentration: f64,
    /// Donor ionization energy $\Delta E_D = E_c - E_D$ in $eV$ (e.g. 0.045 eV for P in Si).
    pub donor_ionization_energy_ev: f64,
    /// Acceptor ionization energy $\Delta E_A = E_A - E_v$ in $eV$ (e.g. 0.045 eV for B in Si).
    pub acceptor_ionization_energy_ev: f64,
    /// Relative dielectric permittivity of the semiconductor host.
    pub epsilon_r: f64,
}

impl Default for CryogenicFreezeoutModel {
    fn default() -> Self {
        Self {
            donor_concentration: 1e22, // 10^16 cm^-3
            acceptor_concentration: 1e20,
            donor_ionization_energy_ev: 0.045, // Phosphorus in Silicon
            acceptor_ionization_energy_ev: 0.045, // Boron in Silicon
            epsilon_r: EPSILON_R_SI,
        }
    }
}

impl CryogenicFreezeoutModel {
    /// Poole-Frenkel electric field barrier lowering $\Delta E_{PF}(\mathcal{E})$ in $eV$:
    /// $$\Delta E_{PF} = \sqrt{\frac{q^3 \mathcal{E}}{\pi \epsilon_0 \epsilon_r}}$$
    pub fn poole_frenkel_barrier_lowering_ev(&self, electric_field_v_per_m: f64) -> f64 {
        let field = electric_field_v_per_m.abs();
        if field < 1.0 {
            return 0.0;
        }
        let q = ELEMENTARY_CHARGE;
        let eps_s = self.epsilon_r * EPSILON_0;
        let delta_joules = ((q * q * q * field) / (std::f64::consts::PI * eps_s)).sqrt();
        delta_joules / q
    }

    /// Computes the ionized donor fraction $N_D^+ / N_D$ under temperature $T$ (Kelvin),
    /// reduced Fermi level $\eta_c = (E_F - E_c) / (k_B T)$, and electric field $\mathcal{E}$:
    /// $$\frac{N_D^+}{N_D} = \frac{1}{1 + g_D \exp\left( \frac{E_F - E_D + \Delta E_{PF}}{k_B T} \right)}$$
    pub fn ionized_donor_fraction(&self, temp_k: f64, eta_c: f64, electric_field: f64) -> f64 {
        let kb_ev = 8.617_333_262e-5;
        let kt = (kb_ev * temp_k).max(1e-5);
        let g_d = 2.0; // Ground state donor spin degeneracy

        let delta_pf = self.poole_frenkel_barrier_lowering_ev(electric_field);
        let effective_e_d = (self.donor_ionization_energy_ev - delta_pf).max(1e-4);

        // E_F - E_D = (E_F - E_c) + (E_c - E_D) = eta_c * k_B * T + effective_e_d
        let exponent = (eta_c + effective_e_d / kt).clamp(-40.0, 40.0);
        1.0 / (1.0 + g_d * exponent.exp())
    }

    /// Solves self-consistent equilibrium free electron concentration $n$ (in $m^{-3}$)
    /// and ionized donor concentration $N_D^+$ at cryogenic temperature $T$ (down to 4K).
    pub fn solve_equilibrium_electron_density(&self, temp_k: f64, nc: f64) -> (f64, f64) {
        if self.donor_concentration <= 0.0 {
            return (0.0, 0.0);
        }

        // Bisection on reduced Fermi level eta_c in range [-100, 20]
        let mut eta_min = -100.0;
        let mut eta_max = 20.0;

        for _ in 0..50 {
            let eta_mid = 0.5 * (eta_min + eta_max);
            let n_mid = nc * fermi_dirac_half(eta_mid);
            let n_d_plus =
                self.donor_concentration * self.ionized_donor_fraction(temp_k, eta_mid, 0.0);

            let res = n_mid - n_d_plus;
            if res > 0.0 {
                eta_max = eta_mid;
            } else {
                eta_min = eta_mid;
            }
        }

        let eta_sol = 0.5 * (eta_min + eta_max);
        let n_sol = nc * fermi_dirac_half(eta_sol);
        let n_d_plus = self.donor_concentration * self.ionized_donor_fraction(temp_k, eta_sol, 0.0);
        (n_sol, n_d_plus)
    }
}
