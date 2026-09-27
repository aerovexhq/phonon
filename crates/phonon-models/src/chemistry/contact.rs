//! Contact metallization, silicides, work functions, and Schottky-Mott / Bardeen barrier models.

use super::bandstructure::Bandstructure;
use phonon_core::constants::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, PLANCK_CONSTANT};

/// Classification of electrode contact materials and silicides.
#[derive(Debug, Clone, PartialEq)]
pub enum ContactSpecies {
    // Chemical Silicides formed by solid-state reaction with Silicon
    NickelSilicideNiSi,
    CobaltSilicideCoSi2,
    TitaniumSilicideTiSi2,
    PlatinumSilicidePtSi,
    // Elemental & Interconnect Metals
    Aluminum,
    Copper,
    Gold,
    Tungsten,
    Titanium,
    // Gate metals & Alloys
    TitaniumNitrideTiN,
    TantalumNitrideTaN,
    NPlusPolySilicon,
    PPlusPolySilicon,
    // Custom metal
    Custom {
        name: String,
        work_function_ev: f64,
        specific_resistivity_ohm_m2: f64,
    },
}

/// Chemical and electronic properties of an electrode contact or silicide interface.
#[derive(Debug, Clone, PartialEq)]
pub struct ContactMaterial {
    pub species: ContactSpecies,
    /// Work function $\Phi_m$ in electron-volts ($eV$).
    pub work_function_ev: f64,
    /// Native electron Schottky barrier height $\Phi_{Bn}$ on Silicon at 300K in $eV$.
    pub barrier_height_n_si_ev: f64,
    /// Native hole Schottky barrier height $\Phi_{Bp}$ on Silicon at 300K in $eV$.
    pub barrier_height_p_si_ev: f64,
    /// Specific contact resistivity $\rho_c$ in $\Omega \cdot \text{m}^2$.
    pub specific_resistivity_ohm_m2: f64,
    /// Bardeen interface pinning factor $\gamma \in [0, 1]$ (1 = Schottky-Mott ideal, ~0.27 = Silicon).
    pub pinning_factor_gamma: f64,
    /// Charge neutrality level $\Phi_0$ above valence band in $eV$.
    pub charge_neutrality_level_ev: f64,
}

impl ContactMaterial {
    /// Computes the effective electron Schottky barrier height $\Phi_{Bn}$ on an arbitrary semiconductor
    /// using the Cowley-Sze / Bardeen interface state pinning formulation:
    /// $$\Phi_{Bn} = \gamma (\Phi_m - \chi) + (1 - \gamma) (E_g - \Phi_0)$$
    pub fn electron_barrier_height(&self, band: &Bandstructure, temp_k: f64) -> f64 {
        let chi = band.electron_affinity_ev(temp_k);
        let eg = band.bandgap_ev(temp_k);
        let gamma = self.pinning_factor_gamma;
        let phi_bn = gamma * (self.work_function_ev - chi)
            + (1.0 - gamma) * (eg - self.charge_neutrality_level_ev);
        phi_bn.clamp(0.05, eg - 0.05)
    }

    /// Computes the effective hole Schottky barrier height $\Phi_{Bp}$ via the fundamental sum rule:
    /// $$\Phi_{Bp} = E_g - \Phi_{Bn}$$
    pub fn hole_barrier_height(&self, band: &Bandstructure, temp_k: f64) -> f64 {
        let eg = band.bandgap_ev(temp_k);
        let phi_bn = self.electron_barrier_height(band, temp_k);
        (eg - phi_bn).max(0.05)
    }

    /// Computes the thermionic emission saturation current density $J_s$ in $\text{A/m}^2$:
    /// $$J_s = A^* T^2 \exp\left( -\frac{q \Phi_B}{k_B T} \right)$$
    pub fn thermionic_saturation_current_density(
        &self,
        band: &Bandstructure,
        is_n_type: bool,
        temp_k: f64,
    ) -> f64 {
        let t = temp_k.max(1.0);
        let phi_b = if is_n_type {
            self.electron_barrier_height(band, t)
        } else {
            self.hole_barrier_height(band, t)
        };

        let me_cond = if is_n_type {
            band.me_transverse
        } else {
            band.mh_heavy
        };
        // Richardson constant A* = (4 * pi * q * m* * kB^2) / h^3
        let m_star = me_cond * crate::chemistry::bandstructure::ELECTRON_REST_MASS;
        let a_star = (4.0
            * std::f64::consts::PI
            * ELEMENTARY_CHARGE
            * m_star
            * BOLTZMANN_CONSTANT
            * BOLTZMANN_CONSTANT)
            / (PLANCK_CONSTANT * PLANCK_CONSTANT * PLANCK_CONSTANT);

        let exp_arg = -(phi_b * ELEMENTARY_CHARGE) / (BOLTZMANN_CONSTANT * t);
        a_star * t * t * exp_arg.clamp(-80.0, 0.0).exp()
    }

    /// Computes the metal-semiconductor flatband voltage offset $V_{fb}$ for gate stacks:
    /// $$V_{fb} = \Phi_m - \Phi_{semiconductor}$$
    pub fn flatband_offset_v(
        &self,
        band: &Bandstructure,
        fermi_level_from_ei_ev: f64,
        temp_k: f64,
    ) -> f64 {
        let chi = band.electron_affinity_ev(temp_k);
        let eg = band.bandgap_ev(temp_k);
        let phi_s = chi + 0.5 * eg - fermi_level_from_ei_ev;
        self.work_function_ev - phi_s
    }

    // =========================================================================
    // Standard Contact & Silicide Presets
    // =========================================================================

    /// Nickel Silicide ($NiSi$): Low resistance, $\Phi_{Bn} = 0.65 \text{ eV}, \Phi_{Bp} = 0.47 \text{ eV}, \rho_c = 1.5 \times 10^{-12} \ \Omega \cdot \text{m}^2$.
    pub fn nickel_silicide() -> Self {
        Self {
            species: ContactSpecies::NickelSilicideNiSi,
            work_function_ev: 4.67,
            barrier_height_n_si_ev: 0.65,
            barrier_height_p_si_ev: 0.47,
            specific_resistivity_ohm_m2: 1.5e-12, // 1.5e-8 Ohm cm^2
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }

    /// Cobalt Silicide ($CoSi_2$): $\Phi_{Bn} = 0.64 \text{ eV}, \Phi_{Bp} = 0.48 \text{ eV}$.
    pub fn cobalt_silicide() -> Self {
        Self {
            species: ContactSpecies::CobaltSilicideCoSi2,
            work_function_ev: 4.66,
            barrier_height_n_si_ev: 0.64,
            barrier_height_p_si_ev: 0.48,
            specific_resistivity_ohm_m2: 2.0e-12,
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }

    /// Platinum Silicide ($PtSi$): Ideal for p-type Ohmic contacts ($\Phi_{Bp} = 0.24 \text{ eV}$, $\Phi_{Bn} = 0.88 \text{ eV}$).
    pub fn platinum_silicide() -> Self {
        Self {
            species: ContactSpecies::PlatinumSilicidePtSi,
            work_function_ev: 4.95,
            barrier_height_n_si_ev: 0.88,
            barrier_height_p_si_ev: 0.24,
            specific_resistivity_ohm_m2: 1.0e-12,
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }

    /// Titanium Nitride ($TiN$): Standard mid-gap gate metal, $\Phi_m = 4.70 \text{ eV}$.
    pub fn titanium_nitride_gate() -> Self {
        Self {
            species: ContactSpecies::TitaniumNitrideTiN,
            work_function_ev: 4.70,
            barrier_height_n_si_ev: 0.60,
            barrier_height_p_si_ev: 0.52,
            specific_resistivity_ohm_m2: 5.0e-12,
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }

    /// Aluminum ($Al$): Low work function contact metal, $\Phi_m = 4.28 \text{ eV}$.
    pub fn aluminum() -> Self {
        Self {
            species: ContactSpecies::Aluminum,
            work_function_ev: 4.28,
            barrier_height_n_si_ev: 0.72,
            barrier_height_p_si_ev: 0.40,
            specific_resistivity_ohm_m2: 1.0e-11,
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }

    /// Copper ($Cu$): Interconnect metallization, $\Phi_m = 4.65 \text{ eV}$.
    pub fn copper() -> Self {
        Self {
            species: ContactSpecies::Copper,
            work_function_ev: 4.65,
            barrier_height_n_si_ev: 0.65,
            barrier_height_p_si_ev: 0.47,
            specific_resistivity_ohm_m2: 3.0e-12,
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }

    /// Gold ($Au$): High work function contact metal, $\Phi_m = 5.10 \text{ eV}$.
    pub fn gold() -> Self {
        Self {
            species: ContactSpecies::Gold,
            work_function_ev: 5.10,
            barrier_height_n_si_ev: 0.80,
            barrier_height_p_si_ev: 0.32,
            specific_resistivity_ohm_m2: 2.0e-12,
            pinning_factor_gamma: 0.27,
            charge_neutrality_level_ev: 0.36,
        }
    }
}
