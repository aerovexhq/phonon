//! Atomistic reliability physics: Electromigration, TDDB, and Contact Resistance.
//!
//! Formulates:
//! - Electromigration (EM): Mass flux divergence $\nabla \cdot \mathbf{J}_a$, Black's equation, and void resistance degradation.
//! - Time-Dependent Dielectric Breakdown (TDDB): Percolation model of oxide defect generation and filament short-circuiting.
//! - Atomistic Contact Resistance ($R_c$): Transfer Length Method (TLM) with Schottky barrier de-pinning.

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Electromigration (EM) degradation and lifetime model for metallic interconnects.
#[derive(Debug, Clone, PartialEq)]
pub struct ElectromigrationModel {
    /// Initial undefected interconnect resistance $R_0$ in Ohms ($\Omega$).
    pub initial_resistance_ohms: f64,
    /// Interconnect cross-sectional area $A_{wire}$ in $m^2$.
    pub cross_section_area_m2: f64,
    /// Effective valence / charge number $Z^*$ (typically negative, e.g. $-5.0$ for Cu).
    pub z_star: f64,
    /// Activation energy for atomic diffusion $E_a$ in $eV$ (e.g. $0.9\text{ eV}$ for Cu grain boundary).
    pub activation_energy_ev: f64,
    /// Pre-exponential diffusion coefficient $D_0$ in $m^2 / s$.
    pub diffusion_prefactor_m2_s: f64,
    /// Atomic concentration $N_a$ in $m^{-3}$ (e.g. $8.49 \times 10^{28}\text{ m}^{-3}$ for Cu).
    pub atomic_density_m3: f64,
    /// Metal electrical resistivity $\rho$ in $\Omega \cdot m$.
    pub resistivity_ohm_m: f64,
    /// Black's law proportionality constant $A$ in $\text{hours} \cdot (A/m^2)^n$.
    pub blacks_constant_a: f64,
    /// Black's law current exponent $n$ (typically $1.8 - 2.0$).
    pub current_exponent_n: f64,
}

impl ElectromigrationModel {
    /// Copper (Cu) dual-damascene interconnect preset.
    pub fn copper(initial_resistance_ohms: f64, width_m: f64, height_m: f64) -> Self {
        Self {
            initial_resistance_ohms,
            cross_section_area_m2: width_m * height_m,
            z_star: -5.0,
            activation_energy_ev: 0.90, // Grain boundary / cap interface
            diffusion_prefactor_m2_s: 1.0e-5,
            atomic_density_m3: 8.49e28,
            resistivity_ohm_m: 1.68e-8,
            blacks_constant_a: 1.2e18,
            current_exponent_n: 2.0,
        }
    }

    /// Evaluates electromigration atomic mass flux $J_a$ (in $\text{atoms} / (m^2 \cdot s)$):
    /// $$J_a = \frac{N_a D_0}{k_B T} \exp\left(-\frac{E_a}{k_B T}\right) |Z^*| e \rho J$$
    pub fn atomic_mass_flux(&self, current_amperes: f64, temp_k: f64) -> f64 {
        let temp_safe = temp_k.max(1.0);
        let j_elec = (current_amperes / self.cross_section_area_m2).abs();
        let kb_t_j = BOLTZMANN_CONSTANT * temp_safe;
        let diff_coeff = self.diffusion_prefactor_m2_s
            * (-self.activation_energy_ev * ELEMENTARY_CHARGE / kb_t_j).exp();

        let force_wind = self.z_star.abs() * ELEMENTARY_CHARGE * self.resistivity_ohm_m * j_elec;
        (self.atomic_density_m3 * diff_coeff / kb_t_j) * force_wind
    }

    /// Predicts Mean Time to Failure (MTTF) in hours according to Black's Equation:
    /// $$\text{MTTF} = A J^{-n} \exp\left(\frac{E_a}{k_B T}\right)$$
    pub fn mean_time_to_failure_hours(&self, current_amperes: f64, temp_k: f64) -> f64 {
        let temp_safe = temp_k.max(1.0);
        let j_elec = (current_amperes / self.cross_section_area_m2)
            .abs()
            .max(1.0);
        let kb_t_j = BOLTZMANN_CONSTANT * temp_safe;
        let thermal_factor = (self.activation_energy_ev * ELEMENTARY_CHARGE / kb_t_j).exp();

        self.blacks_constant_a * j_elec.powf(-self.current_exponent_n) * thermal_factor
    }

    /// Computes degraded interconnect resistance $R(t)$ under continuous electrical stress:
    /// $$\Delta R(t) = R_0 \left( 1 + \left( \frac{t}{\text{MTTF}} \right)^2 \right)$$
    pub fn degraded_resistance(&self, current_amperes: f64, temp_k: f64, time_hours: f64) -> f64 {
        let mttf = self.mean_time_to_failure_hours(current_amperes, temp_k);
        let stress_ratio = (time_hours / mttf.max(1e-6)).clamp(0.0, 100.0);
        self.initial_resistance_ohms * (1.0 + stress_ratio.powi(2))
    }
}

/// Time-Dependent Dielectric Breakdown (TDDB) percolation defect generation model.
#[derive(Debug, Clone, PartialEq)]
pub struct TddbPercolationModel {
    /// Physical oxide thickness $t_{ox}$ in meters ($m$).
    pub tox_m: f64,
    /// Critical defect density $N_{crit}$ in $m^{-3}$ required to form a percolation filament.
    pub critical_defect_density_m3: f64,
    /// Current defect density $N_{def}$ in $m^{-3}$.
    pub current_defect_density_m3: f64,
    /// Activation energy for oxygen vacancy / trap generation in $eV$.
    pub activation_energy_ev: f64,
    /// Electric field acceleration parameter $\gamma_{E}$ in $m/V$.
    pub field_acceleration_m_v: f64,
    /// Intrinsic pre-breakdown leakage conductance in Siemens ($S$).
    pub pre_breakdown_conductance_s: f64,
    /// Post-breakdown percolation filament conductance in Siemens ($S$).
    pub post_breakdown_conductance_s: f64,
    /// Breakdown flag indicating conductive percolation path has formed.
    pub has_broken_down: bool,
}

impl TddbPercolationModel {
    /// High-$\kappa$ dielectric $\text{HfO}_2$ ultra-thin gate oxide preset.
    pub fn hfo2(tox_m: f64) -> Self {
        // Critical defect density scales inversely with thickness t_ox: N_crit ~ 1 / t_ox^1.5
        let a0: f64 = 0.3e-9; // ~ 3 Angstrom defect size
        let n_crit = (1.0 / a0.powi(3)) * (a0 / tox_m.max(0.5e-9)).powf(1.5);
        Self {
            tox_m,
            critical_defect_density_m3: n_crit,
            current_defect_density_m3: 0.0,
            activation_energy_ev: 0.55,
            field_acceleration_m_v: 8.0e-9, // 8 nm / V
            pre_breakdown_conductance_s: 1.0e-11,
            post_breakdown_conductance_s: 1.0e-3, // ~ 1 kOhm filament
            has_broken_down: false,
        }
    }

    /// Advances the defect generation under electric field $E = V / t_{ox}$ for duration $\Delta t$:
    pub fn stress_step(&mut self, voltage_v: f64, temp_k: f64, dt_seconds: f64) {
        if self.has_broken_down {
            return;
        }

        let e_field = (voltage_v.abs() / self.tox_m.max(1e-10)).max(1.0);
        let kb_t_j = BOLTZMANN_CONSTANT * temp_k.max(1.0);

        // Defect generation rate dN/dt = G_0 * exp(gamma * E) * exp(-E_a / k_B T)
        let rate_prefactor = 1.0e28; // m^-3 s^-1
        let field_factor = (self.field_acceleration_m_v * e_field)
            .clamp(0.0, 50.0)
            .exp();

        let thermal_factor = (-self.activation_energy_ev * ELEMENTARY_CHARGE / kb_t_j).exp();

        let dn_dt = rate_prefactor * field_factor * thermal_factor;
        self.current_defect_density_m3 += dn_dt * dt_seconds;

        if self.current_defect_density_m3 >= self.critical_defect_density_m3 {
            self.has_broken_down = true;
        }
    }

    /// Returns effective oxide conductance (in Siemens) reflecting pre-breakdown or filamentary state.
    pub fn effective_conductance(&self) -> f64 {
        if self.has_broken_down {
            self.post_breakdown_conductance_s
        } else {
            let progress = (self.current_defect_density_m3
                / self.critical_defect_density_m3.max(1.0))
            .clamp(0.0, 0.999);
            // Soft trap-assisted tunneling increases smoothly before abrupt percolation
            self.pre_breakdown_conductance_s / (1.0 - progress)
        }
    }
}

/// Atomistic Contact Resistance model using Transfer Length Method (TLM).
#[derive(Debug, Clone, PartialEq)]
pub struct ContactResistanceModel {
    /// Specific contact resistivity $\rho_c$ in $\Omega \cdot m^2$.
    pub specific_resistivity_ohm_m2: f64,
    /// Channel sheet resistance $R_{sheet}$ under contact in $\Omega / \Box$.
    pub sheet_resistance_ohms_sq: f64,
    /// Physical contact length $L_c$ in meters ($m$).
    pub contact_length_m: f64,
    /// Contact width $W$ in meters ($m$).
    pub contact_width_m: f64,
    /// Fermi level pinning factor $S = \frac{d\Phi_B}{d\Phi_M}$ ($0 \le S \le 1$).
    pub pinning_factor_s: f64,
}

impl ContactResistanceModel {
    /// Monolayer 2D material metal contact preset (e.g. Ti or Au on $\text{MoS}_2$).
    pub fn tmd_metal_contact(contact_width_m: f64, contact_length_m: f64) -> Self {
        Self {
            specific_resistivity_ohm_m2: 1.0e-11, // 10^-7 Ohm*cm^2 = 10^-11 Ohm*m^2
            sheet_resistance_ohms_sq: 1000.0,     // 1 kOhm / sq
            contact_length_m,
            contact_width_m,
            pinning_factor_s: 0.1, // Strong Fermi level pinning
        }
    }

    /// Evaluates transfer length $L_T = \sqrt{\frac{\rho_c}{R_{sheet}}}$ in meters ($m$).
    pub fn transfer_length_m(&self) -> f64 {
        (self.specific_resistivity_ohm_m2 / self.sheet_resistance_ohms_sq.max(1e-12)).sqrt()
    }

    /// Evaluates total atomistic contact resistance $R_c$ (in Ohms):
    /// $$R_c = \frac{\sqrt{\rho_c R_{sheet}}}{W} \coth\left(\frac{L_c}{L_T}\right)$$
    pub fn evaluate_contact_resistance(&self) -> f64 {
        let lt = self.transfer_length_m();
        let characteristic_rc = (self.specific_resistivity_ohm_m2 * self.sheet_resistance_ohms_sq)
            .sqrt()
            / self.contact_width_m.max(1e-12);

        let arg = (self.contact_length_m / lt.max(1e-12)).clamp(0.01, 50.0);
        let coth = 1.0 / arg.tanh();

        characteristic_rc * coth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_electromigration_black_law_scaling() {
        let em = ElectromigrationModel::copper(10.0, 50e-9, 50e-9);

        let mttf_low_j = em.mean_time_to_failure_hours(1e-4, 350.0);
        let mttf_high_j = em.mean_time_to_failure_hours(1e-3, 350.0);

        // Current increased by 10x, with n=2 MTTF must decrease by ~100x
        let ratio = mttf_low_j / mttf_high_j;
        assert!(
            (ratio - 100.0).abs() < 1.0,
            "Expected 100x ratio, got {}",
            ratio
        );

        // Resistance degradation over time
        let r_init = em.degraded_resistance(1e-4, 350.0, 0.0);
        let r_stressed = em.degraded_resistance(1e-4, 350.0, mttf_low_j);
        assert_eq!(r_init, 10.0);
        assert!((r_stressed - 20.0).abs() < 1e-4); // 1 + (1)^2 = 2x
    }

    #[test]
    fn test_tddb_percolation_breakdown() {
        let mut tddb = TddbPercolationModel::hfo2(2.0e-9);
        assert!(!tddb.has_broken_down);
        let g_initial = tddb.effective_conductance();

        // High field stress: 2.5 V across 2 nm = 12.5 MV/cm
        for _ in 0..100 {
            tddb.stress_step(2.5, 400.0, 1.0);
        }

        assert!(
            tddb.has_broken_down,
            "current: {:e}, crit: {:e}",
            tddb.current_defect_density_m3, tddb.critical_defect_density_m3
        );

        let g_post = tddb.effective_conductance();
        assert!(
            g_post > g_initial * 1e6,
            "Post-breakdown conductance surge: {} vs {}",
            g_post,
            g_initial
        );
    }

    #[test]
    fn test_contact_resistance_tlm() {
        let contact = ContactResistanceModel::tmd_metal_contact(1.0e-6, 50.0e-9);
        let rc = contact.evaluate_contact_resistance();
        assert!(rc > 0.0);
        // Transfer length
        let lt = contact.transfer_length_m();
        assert!(lt > 1e-9 && lt < 1e-6);
    }
}
