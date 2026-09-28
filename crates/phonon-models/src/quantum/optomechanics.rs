//! Cavity Quantum Optomechanics, Piezoelectric Optomechanical Crystals, and Dynamical Backaction.
//!
//! Formulates:
//! - Coupled cavity optomechanical Hamiltonian:
//!   $$\hat{H} = \hbar \omega_c \hat{a}^\dagger \hat{a} + \hbar \Omega_m \hat{b}^\dagger \hat{b} - \hbar g_0 \hat{a}^\dagger \hat{a}(\hat{b} + \hat{b}^\dagger) + \hbar g_{em}(\hat{c}^\dagger \hat{b} + \hat{c}\hat{b}^\dagger)$$
//!   with zero-point fluctuation amplitude $x_{zpf} = \sqrt{\frac{\hbar}{2 m_{eff} \Omega_m}}$ and vacuum coupling $g_0 = -\frac{\omega_c}{L} x_{zpf}$.
//! - Piezoelectric optomechanical crystals (AlN, GaAs, LN, Si nanobeam) coupling optical telecommunication modes
//!   ($\lambda \approx 1550\text{ nm}$), gigahertz phononic acoustic breathing modes ($\Omega_m \approx 2-10\text{ GHz}$),
//!   and microwave superconducting coplanar waveguide resonators ($\omega_{mw} \approx 4-8\text{ GHz}$).
//! - Optomechanical dynamical backaction:
//!   - Optical spring effect: $\delta\Omega_m(\Delta)$.
//!   - Optomechanical damping rate: $\Gamma_{opt}(\Delta)$.
//!   - Dynamical sideband cooling to the phononic quantum ground state ($\bar{n}_{eff} < 0.1$) on the red sideband ($\Delta = -\Omega_m$).
//!   - Optomechanically Induced Transparency (OMIT): Narrowband transparency window in probe optical transmission spectrum.

use phonon_core::constants::{BOLTZMANN_CONSTANT, H_BAR, SPEED_OF_LIGHT};

/// Piezoelectric and optomechanical crystal material category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiezoCrystalMaterial {
    /// Aluminum Nitride (AlN): wide-bandgap wurtzite piezoelectric with high acoustic velocity.
    Aln,
    /// Gallium Arsenide (GaAs): direct bandgap zinc-blende with strong photoelastic and piezoelectric coupling.
    Gaas,
    /// Lithium Niobate ($\text{LiNbO}_3$): ferroelectric with giant electro-optic and piezoelectric coefficients ($k_{eff}^2 > 0.05$).
    LithiumNiobate,
    /// Silicon Nanobeam (Si): ultra-high optical $Q$ nanobeam with piezoelectric transducer overlayer.
    SiliconNanobeam,
}

impl PiezoCrystalMaterial {
    /// Nominal refractive index at optical telecommunication wavelengths ($\lambda \approx 1550\text{ nm}$).
    pub fn refractive_index(&self) -> f64 {
        match self {
            Self::Aln => 2.15,
            Self::Gaas => 3.48,
            Self::LithiumNiobate => 2.21,
            Self::SiliconNanobeam => 3.48,
        }
    }

    /// Speed of acoustic waves (bulk longitudinal velocity) in $\text{m/s}$.
    pub fn acoustic_velocity_m_s(&self) -> f64 {
        match self {
            Self::Aln => 10_400.0,
            Self::Gaas => 4_730.0,
            Self::LithiumNiobate => 6_570.0,
            Self::SiliconNanobeam => 8_430.0,
        }
    }

    /// Mass density $\rho$ in $\text{kg/m}^3$.
    pub fn mass_density_kg_per_m3(&self) -> f64 {
        match self {
            Self::Aln => 3_260.0,
            Self::Gaas => 5_320.0,
            Self::LithiumNiobate => 4_650.0,
            Self::SiliconNanobeam => 2_329.0,
        }
    }

    /// Effective electromechanical coupling coefficient $k_{eff}^2$.
    pub fn electromechanical_coupling_keff_sq(&self) -> f64 {
        match self {
            Self::Aln => 0.015,
            Self::Gaas => 0.002,
            Self::LithiumNiobate => 0.075,
            Self::SiliconNanobeam => 0.005,
        }
    }
}

/// Physical and electromagnetic parameters for a Piezoelectric Optomechanical Crystal (OMC).
#[derive(Debug, Clone, PartialEq)]
pub struct PiezoOptomechanicalCrystal {
    /// Crystal substrate material.
    pub material: PiezoCrystalMaterial,
    /// Resonant optical telecommunication wavelength $\lambda_c$ in meters ($m$).
    pub optical_wavelength_m: f64,
    /// Optical cavity angular frequency $\omega_c = 2\pi c / \lambda_c$ in $\text{rad/s}$.
    pub optical_frequency_rad_s: f64,
    /// Intrinsic optical cavity loss rate $\kappa_0$ in $\text{rad/s}$.
    pub optical_kappa_0: f64,
    /// External optical cavity coupling rate $\kappa_{ex}$ in $\text{rad/s}$.
    pub optical_kappa_ex: f64,
    /// Phononic acoustic breathing mode angular frequency $\Omega_m = 2\pi f_m$ in $\text{rad/s}$.
    pub mechanical_frequency_rad_s: f64,
    /// Intrinsic phononic mechanical damping rate $\gamma_m$ in $\text{rad/s}$.
    pub mechanical_gamma_m: f64,
    /// Effective motional mass of the localized phononic mode $m_{eff}$ in kilograms ($kg$).
    pub effective_mass_kg: f64,
    /// Effective optical cavity interaction length $L$ in meters ($m$).
    pub cavity_length_m: f64,
    /// Superconducting microwave resonator angular frequency $\omega_{mw} = 2\pi f_{mw}$ in $\text{rad/s}$.
    pub microwave_frequency_rad_s: f64,
    /// Intrinsic microwave resonator decay rate $\kappa_{mw,0}$ in $\text{rad/s}$.
    pub microwave_kappa_0: f64,
    /// External microwave resonator coupling rate $\kappa_{mw,ex}$ in $\text{rad/s}$.
    pub microwave_kappa_ex: f64,
    /// Single-photon optomechanical coupling rate $g_0$ in $\text{rad/s}$.
    pub single_photon_g0: f64,
    /// Piezoelectric electromechanical coupling rate $g_{em}$ in $\text{rad/s}$.
    pub electromechanical_gem: f64,
}

impl PiezoOptomechanicalCrystal {
    /// Computes the mechanical zero-point fluctuation amplitude:
    /// $$x_{zpf} = \sqrt{\frac{\hbar}{2 m_{eff} \Omega_m}}$$
    #[inline]
    pub fn compute_x_zpf(m_eff: f64, omega_m: f64) -> f64 {
        (H_BAR / (2.0 * m_eff * omega_m)).sqrt()
    }

    /// Computes the vacuum / single-photon optomechanical coupling rate:
    /// $$g_0 = \frac{\omega_c}{L} x_{zpf}$$
    #[inline]
    pub fn compute_g0(omega_c: f64, cavity_length_m: f64, x_zpf: f64) -> f64 {
        (omega_c / cavity_length_m) * x_zpf
    }

    /// Total optical cavity decay rate $\kappa = \kappa_0 + \kappa_{ex}$ in $\text{rad/s}$.
    #[inline]
    pub fn total_optical_kappa(&self) -> f64 {
        self.optical_kappa_0 + self.optical_kappa_ex
    }

    /// Optical outcoupling efficiency $\eta_{opt,ex} = \frac{\kappa_{ex}}{\kappa}$.
    #[inline]
    pub fn optical_outcoupling_efficiency(&self) -> f64 {
        self.optical_kappa_ex / self.total_optical_kappa()
    }

    /// Total microwave resonator decay rate $\kappa_{mw} = \kappa_{mw,0} + \kappa_{mw,ex}$ in $\text{rad/s}$.
    #[inline]
    pub fn total_microwave_kappa(&self) -> f64 {
        self.microwave_kappa_0 + self.microwave_kappa_ex
    }

    /// Microwave outcoupling efficiency $\eta_{mw,ex} = \frac{\kappa_{mw,ex}}{\kappa_{mw}}$.
    #[inline]
    pub fn microwave_outcoupling_efficiency(&self) -> f64 {
        self.microwave_kappa_ex / self.total_microwave_kappa()
    }

    /// Mechanical zero-point fluctuation amplitude $x_{zpf}$ in meters ($m$).
    #[inline]
    pub fn zero_point_fluctuation_m(&self) -> f64 {
        Self::compute_x_zpf(self.effective_mass_kg, self.mechanical_frequency_rad_s)
    }

    /// Optical cavity loaded quality factor $Q_{opt} = \frac{\omega_c}{\kappa}$.
    #[inline]
    pub fn optical_loaded_q(&self) -> f64 {
        self.optical_frequency_rad_s / self.total_optical_kappa()
    }

    /// Mechanical quality factor $Q_m = \frac{\Omega_m}{\gamma_m}$.
    #[inline]
    pub fn mechanical_q(&self) -> f64 {
        self.mechanical_frequency_rad_s / self.mechanical_gamma_m
    }

    /// Microwave resonator loaded quality factor $Q_{mw} = \frac{\omega_{mw}}{\kappa_{mw}}$.
    #[inline]
    pub fn microwave_loaded_q(&self) -> f64 {
        self.microwave_frequency_rad_s / self.total_microwave_kappa()
    }

    /// Evaluates thermal phonon occupancy $\bar{n}_{th}(T)$ at bath temperature $T$ (Kelvin):
    /// $$\bar{n}_{th} = \frac{1}{\exp\left(\frac{\hbar \Omega_m}{k_B T}\right) - 1}$$
    pub fn thermal_phonon_occupancy(&self, temp_kelvin: f64) -> f64 {
        let t = temp_kelvin.max(1e-4);
        let x = (H_BAR * self.mechanical_frequency_rad_s) / (BOLTZMANN_CONSTANT * t);
        if x > 100.0 {
            0.0
        } else if x < 1e-4 {
            1.0 / x
        } else {
            1.0 / (x.exp() - 1.0)
        }
    }

    /// Evaluates thermal microwave photon occupancy $\bar{n}_{mw,th}(T)$ at bath temperature $T$ (Kelvin):
    /// $$\bar{n}_{mw,th} = \frac{1}{\exp\left(\frac{\hbar \omega_{mw}}{k_B T}\right) - 1}$$
    pub fn thermal_microwave_photon_occupancy(&self, temp_kelvin: f64) -> f64 {
        let t = temp_kelvin.max(1e-4);
        let x = (H_BAR * self.microwave_frequency_rad_s) / (BOLTZMANN_CONSTANT * t);
        if x > 100.0 {
            0.0
        } else if x < 1e-4 {
            1.0 / x
        } else {
            1.0 / (x.exp() - 1.0)
        }
    }

    /// Mean intracavity photon number $n_{cav} = |\alpha|^2$ under laser drive of power $P_{in}$ (Watts)
    /// at laser-cavity detuning $\Delta = \omega_l - \omega_c$ ($\text{rad/s}$):
    /// $$n_{cav} = \frac{\kappa_{ex} \frac{P_{in}}{\hbar \omega_l}}{\left(\frac{\kappa}{2}\right)^2 + \Delta^2}$$
    pub fn intracavity_photon_number(&self, input_power_watts: f64, detuning_rad_s: f64) -> f64 {
        let kappa = self.total_optical_kappa();
        let omega_l = (self.optical_frequency_rad_s + detuning_rad_s).max(1.0);
        let photon_flux_in = input_power_watts / (H_BAR * omega_l);
        let denom = (kappa * 0.5).powi(2) + detuning_rad_s.powi(2);
        (self.optical_kappa_ex * photon_flux_in) / denom
    }

    /// Linearized optomechanical coupling rate $g = g_0 \sqrt{n_{cav}}$ in $\text{rad/s}$.
    #[inline]
    pub fn linearized_coupling_g(&self, n_cav: f64) -> f64 {
        self.single_photon_g0 * n_cav.max(0.0).sqrt()
    }

    /// Creates an Aluminum Nitride (AlN) piezoelectric optomechanical crystal configuration.
    pub fn aln_crystal() -> Self {
        let lambda_c = 1550.0e-9;
        let omega_c = (2.0 * std::f64::consts::PI * SPEED_OF_LIGHT) / lambda_c;
        let f_m = 4.2e9; // 4.2 GHz breathing acoustic mode
        let omega_m = 2.0 * std::f64::consts::PI * f_m;
        let m_eff = 3.5e-16; // 350 femtograms
        let l_cav = 12.0e-6; // 12 um nanobeam
        let x_zpf = Self::compute_x_zpf(m_eff, omega_m);
        let g0 = Self::compute_g0(omega_c, l_cav, x_zpf);

        // Overcoupled ports for high-efficiency transduction: eta_opt_ex ~ 89%, eta_mw_ex ~ 88%
        let kappa_0 = omega_c / 800_000.0;
        let kappa_ex = omega_c / 100_000.0;
        let gamma_m = omega_m / 80_000.0;
        let omega_mw = omega_m; // Matched to acoustic mode
        let kappa_mw_0 = omega_mw / 60_000.0;
        let kappa_mw_ex = omega_mw / 8_000.0;
        let gem = 2.0 * std::f64::consts::PI * 550.0e3; // 550 kHz electromechanical coupling

        Self {
            material: PiezoCrystalMaterial::Aln,
            optical_wavelength_m: lambda_c,
            optical_frequency_rad_s: omega_c,
            optical_kappa_0: kappa_0,
            optical_kappa_ex: kappa_ex,
            mechanical_frequency_rad_s: omega_m,
            mechanical_gamma_m: gamma_m,
            effective_mass_kg: m_eff,
            cavity_length_m: l_cav,
            microwave_frequency_rad_s: omega_mw,
            microwave_kappa_0: kappa_mw_0,
            microwave_kappa_ex: kappa_mw_ex,
            single_photon_g0: g0,
            electromechanical_gem: gem,
        }
    }

    /// Creates a Gallium Arsenide (GaAs) photoelastic optomechanical crystal configuration.
    pub fn gaas_crystal() -> Self {
        let lambda_c = 1550.0e-9;
        let omega_c = (2.0 * std::f64::consts::PI * SPEED_OF_LIGHT) / lambda_c;
        let f_m = 2.4e9; // 2.4 GHz breathing mode
        let omega_m = 2.0 * std::f64::consts::PI * f_m;
        let m_eff = 2.0e-16; // 200 femtograms
        let l_cav = 10.0e-6; // 10 um nanobeam
        let x_zpf = Self::compute_x_zpf(m_eff, omega_m);
        let g0 = Self::compute_g0(omega_c, l_cav, x_zpf);

        let kappa_0 = omega_c / 600_000.0;
        let kappa_ex = omega_c / 90_000.0;
        let gamma_m = omega_m / 70_000.0;
        let omega_mw = omega_m;
        let kappa_mw_0 = omega_mw / 50_000.0;
        let kappa_mw_ex = omega_mw / 7_000.0;
        let gem = 2.0 * std::f64::consts::PI * 450.0e3; // 450 kHz

        Self {
            material: PiezoCrystalMaterial::Gaas,
            optical_wavelength_m: lambda_c,
            optical_frequency_rad_s: omega_c,
            optical_kappa_0: kappa_0,
            optical_kappa_ex: kappa_ex,
            mechanical_frequency_rad_s: omega_m,
            mechanical_gamma_m: gamma_m,
            effective_mass_kg: m_eff,
            cavity_length_m: l_cav,
            microwave_frequency_rad_s: omega_mw,
            microwave_kappa_0: kappa_mw_0,
            microwave_kappa_ex: kappa_mw_ex,
            single_photon_g0: g0,
            electromechanical_gem: gem,
        }
    }

    /// Creates a Lithium Niobate ($\text{LiNbO}_3$) high-electromechanical-coupling transducer crystal.
    pub fn lithium_niobate_crystal() -> Self {
        let lambda_c = 1550.0e-9;
        let omega_c = (2.0 * std::f64::consts::PI * SPEED_OF_LIGHT) / lambda_c;
        let f_m = 5.0e9; // 5.0 GHz phononic breathing mode
        let omega_m = 2.0 * std::f64::consts::PI * f_m;
        let m_eff = 2.8e-16; // 280 femtograms
        let l_cav = 10.0e-6; // 10 um
        let x_zpf = Self::compute_x_zpf(m_eff, omega_m);
        let g0 = Self::compute_g0(omega_c, l_cav, x_zpf);

        // Overcoupled ports for high-efficiency transduction: eta_opt_ex ~ 89%, eta_mw_ex ~ 89%
        let kappa_0 = omega_c / 1_000_000.0;
        let kappa_ex = omega_c / 120_000.0;
        let gamma_m = omega_m / 100_000.0;
        let omega_mw = omega_m;
        let kappa_mw_0 = omega_mw / 80_000.0;
        let kappa_mw_ex = omega_mw / 10_000.0;
        let gem = 2.0 * std::f64::consts::PI * 650.0e3; // 650 kHz matched electromechanical coupling

        Self {
            material: PiezoCrystalMaterial::LithiumNiobate,
            optical_wavelength_m: lambda_c,
            optical_frequency_rad_s: omega_c,
            optical_kappa_0: kappa_0,
            optical_kappa_ex: kappa_ex,
            mechanical_frequency_rad_s: omega_m,
            mechanical_gamma_m: gamma_m,
            effective_mass_kg: m_eff,
            cavity_length_m: l_cav,
            microwave_frequency_rad_s: omega_mw,
            microwave_kappa_0: kappa_mw_0,
            microwave_kappa_ex: kappa_mw_ex,
            single_photon_g0: g0,
            electromechanical_gem: gem,
        }
    }

    /// Creates a Silicon Nanobeam crystal configuration with piezoelectric transducer overlayer.
    pub fn silicon_nanobeam_crystal() -> Self {
        let lambda_c = 1550.0e-9;
        let omega_c = (2.0 * std::f64::consts::PI * SPEED_OF_LIGHT) / lambda_c;
        let f_m = 5.2e9; // 5.2 GHz breathing acoustic mode
        let omega_m = 2.0 * std::f64::consts::PI * f_m;
        let m_eff = 1.8e-16; // 180 femtograms
        let l_cav = 10.0e-6; // 10 um
        let x_zpf = Self::compute_x_zpf(m_eff, omega_m);
        let g0 = Self::compute_g0(omega_c, l_cav, x_zpf);

        // Ultra-high optical Q in Si with overcoupled waveguide:
        let kappa_0 = omega_c / 1_500_000.0;
        let kappa_ex = omega_c / 150_000.0;
        let gamma_m = omega_m / 120_000.0;
        let omega_mw = omega_m;
        let kappa_mw_0 = omega_mw / 80_000.0;
        let kappa_mw_ex = omega_mw / 10_000.0;
        let gem = 2.0 * std::f64::consts::PI * 500.0e3; // 500 kHz with transducer

        Self {
            material: PiezoCrystalMaterial::SiliconNanobeam,
            optical_wavelength_m: lambda_c,
            optical_frequency_rad_s: omega_c,
            optical_kappa_0: kappa_0,
            optical_kappa_ex: kappa_ex,
            mechanical_frequency_rad_s: omega_m,
            mechanical_gamma_m: gamma_m,
            effective_mass_kg: m_eff,
            cavity_length_m: l_cav,
            microwave_frequency_rad_s: omega_mw,
            microwave_kappa_0: kappa_mw_0,
            microwave_kappa_ex: kappa_mw_ex,
            single_photon_g0: g0,
            electromechanical_gem: gem,
        }
    }
}

/// Coupled tripartite cavity optomechanical Hamiltonian system:
/// $$\hat{H} = \hbar \omega_c \hat{a}^\dagger \hat{a} + \hbar \Omega_m \hat{b}^\dagger \hat{b} - \hbar g_0 \hat{a}^\dagger \hat{a}(\hat{b} + \hat{b}^\dagger) + \hbar g_{em}(\hat{c}^\dagger \hat{b} + \hat{c}\hat{b}^\dagger)$$
#[derive(Debug, Clone, PartialEq)]
pub struct OptomechanicalHamiltonian {
    /// Optical cavity angular frequency $\omega_c$ ($\text{rad/s}$).
    pub omega_c: f64,
    /// Mechanical breathing mode angular frequency $\Omega_m$ ($\text{rad/s}$).
    pub omega_m: f64,
    /// Microwave coplanar waveguide angular frequency $\omega_{mw}$ ($\text{rad/s}$).
    pub omega_mw: f64,
    /// Single-photon optomechanical coupling $g_0$ ($\text{rad/s}$).
    pub g0: f64,
    /// Electromechanical piezoelectric coupling $g_{em}$ ($\text{rad/s}$).
    pub gem: f64,
    /// Mechanical zero-point fluctuation amplitude $x_{zpf}$ ($m$).
    pub x_zpf: f64,
}

impl OptomechanicalHamiltonian {
    /// Instantiates the Hamiltonian from a physical optomechanical crystal.
    pub fn from_crystal(crystal: &PiezoOptomechanicalCrystal) -> Self {
        Self {
            omega_c: crystal.optical_frequency_rad_s,
            omega_m: crystal.mechanical_frequency_rad_s,
            omega_mw: crystal.microwave_frequency_rad_s,
            g0: crystal.single_photon_g0,
            gem: crystal.electromechanical_gem,
            x_zpf: crystal.zero_point_fluctuation_m(),
        }
    }

    /// Evaluates the expectation value of the uncoupled Hamiltonian $\langle \hat{H}_0 \rangle$ (Joules):
    /// $$\langle \hat{H}_0 \rangle = \hbar \omega_c \langle \hat{a}^\dagger \hat{a} \rangle + \hbar \Omega_m \langle \hat{b}^\dagger \hat{b} \rangle + \hbar \omega_{mw} \langle \hat{c}^\dagger \hat{c} \rangle$$
    pub fn expectation_uncoupled(&self, n_opt: f64, n_mech: f64, n_mw: f64) -> f64 {
        H_BAR * (self.omega_c * n_opt + self.omega_m * n_mech + self.omega_mw * n_mw)
    }

    /// Evaluates the total Hamiltonian expectation value $\langle \hat{H} \rangle$ (Joules):
    /// $$\langle \hat{H} \rangle = \langle \hat{H}_0 \rangle - \hbar g_0 \langle \hat{a}^\dagger \hat{a} \rangle \langle \hat{b} + \hat{b}^\dagger \rangle + \hbar g_{em} \langle \hat{c}^\dagger \hat{b} + \hat{c} \hat{b}^\dagger \rangle$$
    pub fn expectation_total(
        &self,
        n_opt: f64,
        n_mech: f64,
        n_mw: f64,
        displacement_quadrature: f64, // <b + b^dagger>
        em_coherence: f64,            // <c^dagger b + c b^dagger>
    ) -> f64 {
        let h0 = self.expectation_uncoupled(n_opt, n_mech, n_mw);
        let h_om = -H_BAR * self.g0 * n_opt * displacement_quadrature;
        let h_em = H_BAR * self.gem * em_coherence;
        h0 + h_om + h_em
    }
}

// =========================================================================
// Optomechanical Dynamical Backaction & Sideband Cooling Physics
// =========================================================================

/// Calculates the optical spring effect frequency shift $\delta\Omega_m(\Delta)$ in $\text{rad/s}$:
/// $$\delta\Omega_m(\Delta) = g^2 \left( \frac{\Delta + \Omega_m}{\left(\frac{\kappa}{2}\right)^2 + (\Delta + \Omega_m)^2} + \frac{\Delta - \Omega_m}{\left(\frac{\kappa}{2}\right)^2 + (\Delta - \Omega_m)^2} \right)$$
/// where $\Delta = \omega_l - \omega_c$ is laser detuning and $g = g_0 \sqrt{n_{cav}}$.
pub fn optical_spring_shift(delta: f64, g: f64, kappa: f64, omega_m: f64) -> f64 {
    let half_k_sq = (kappa * 0.5).powi(2);
    let term_plus = (delta + omega_m) / (half_k_sq + (delta + omega_m).powi(2));
    let term_minus = (delta - omega_m) / (half_k_sq + (delta - omega_m).powi(2));
    g.powi(2) * (term_plus + term_minus)
}

/// Calculates the optomechanical dynamical damping rate $\Gamma_{opt}(\Delta)$ in $\text{rad/s}$:
/// $$\Gamma_{opt}(\Delta) = g^2 \left( \frac{\kappa}{\left(\frac{\kappa}{2}\right)^2 + (\Delta + \Omega_m)^2} - \frac{\kappa}{\left(\frac{\kappa}{2}\right)^2 + (\Delta - \Omega_m)^2} \right)$$
/// When driven on the red sideband ($\Delta = -\Omega_m$), $\Gamma_{opt} > 0$, cooling the phononic mode.
pub fn optomechanical_damping_rate(delta: f64, g: f64, kappa: f64, omega_m: f64) -> f64 {
    let half_k_sq = (kappa * 0.5).powi(2);
    let term_plus = kappa / (half_k_sq + (delta + omega_m).powi(2));
    let term_minus = kappa / (half_k_sq + (delta - omega_m).powi(2));
    g.powi(2) * (term_plus - term_minus)
}

/// Calculates the effective mechanical resonance frequency $\Omega_{m,eff} = \Omega_m + \delta\Omega_m(\Delta)$ in $\text{rad/s}$.
#[inline]
pub fn effective_mechanical_frequency(delta: f64, g: f64, kappa: f64, omega_m: f64) -> f64 {
    omega_m + optical_spring_shift(delta, g, kappa, omega_m)
}

/// Calculates the total effective mechanical damping rate $\Gamma_{eff} = \gamma_m + \Gamma_{opt}(\Delta)$ in $\text{rad/s}$.
#[inline]
pub fn effective_damping_rate(delta: f64, g: f64, kappa: f64, omega_m: f64, gamma_m: f64) -> f64 {
    gamma_m + optomechanical_damping_rate(delta, g, kappa, omega_m)
}

/// Optical cooperativity parameter $C_{opt}$:
/// $$C_{opt} = \frac{4 g^2}{\kappa \gamma_m}$$
#[inline]
pub fn optical_cooperativity(g: f64, kappa: f64, gamma_m: f64) -> f64 {
    (4.0 * g.powi(2)) / (kappa * gamma_m)
}

/// Calculates the dynamical sideband cooling steady-state effective phonon occupancy $\bar{n}_{eff}$
/// on the red sideband ($\Delta = -\Omega_m$):
/// $$\bar{n}_{eff} = \frac{\gamma_m \bar{n}_{th} + \Gamma_{opt} n_{min}}{\gamma_m + \Gamma_{opt}} = \frac{\bar{n}_{th} + C_{opt} n_{min}}{1 + C_{opt}}$$
/// with quantum backaction cooling limit $n_{min} = \left(\frac{\kappa}{4 \Omega_m}\right)^2$ in the resolved sideband regime ($\Omega_m \gg \kappa$).
pub fn sideband_cooling_phonon_occupancy(
    g: f64,
    kappa: f64,
    omega_m: f64,
    gamma_m: f64,
    n_th: f64,
) -> f64 {
    let c_opt = optical_cooperativity(g, kappa, gamma_m);
    let n_min = (kappa / (4.0 * omega_m)).powi(2);
    (n_th + c_opt * n_min) / (1.0 + c_opt)
}

/// Verifies whether the phononic breathing mode is cooled into the quantum ground state:
/// $$\bar{n}_{eff} < 0.1$$
#[inline]
pub fn is_ground_state_cooled(n_eff: f64) -> bool {
    n_eff < 0.1
}

// =========================================================================
// Optomechanically Induced Transparency (OMIT)
// =========================================================================

/// Result of Optomechanically Induced Transparency (OMIT) probe transmission evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OmitTransmissionResult {
    /// Normalized optical probe power transmission $|t_p|^2 \in [0, 1]$.
    pub transmission_power: f64,
    /// Probe transmission phase in radians ($-\pi$ to $\pi$).
    pub phase_rad: f64,
    /// OMIT transparency contrast / depth relative to zero-coupling baseline ($|t_p(g)|^2 - |t_p(0)|^2$).
    pub transparency_contrast: f64,
    /// Full-width at half-maximum (FWHM) linewidth of the OMIT transparency window in Hertz ($Hz$).
    pub fwhm_linewidth_hz: f64,
}

/// Evaluates Optomechanically Induced Transparency (OMIT) probe transmission amplitude and spectrum:
/// $$t_p(\delta_p) = 1 - \frac{\kappa_{ex}}{-i(\delta_p + \Delta) + \frac{\kappa}{2} + \frac{g^2}{-i(\delta_p - \Omega_m) + \frac{\gamma_m}{2}}}$$
/// where:
/// - $\delta_p = \omega_{probe} - \omega_{control}$ is probe detuning relative to control laser.
/// - $\Delta = \omega_{control} - \omega_c$ is control laser detuning relative to cavity resonance (typically red sideband $\Delta = -\Omega_m$).
/// - At probe resonance $\delta_p \approx \Omega_m$, destructive quantum interference between the probe field and the anti-Stokes
///   scattered control field opens a narrowband transparency window.
pub fn omit_probe_transmission(
    delta_probe: f64,
    control_detuning: f64,
    g: f64,
    kappa: f64,
    kappa_ex: f64,
    omega_m: f64,
    gamma_m: f64,
) -> OmitTransmissionResult {
    // Mechanical susceptibility denominator: -i(\delta_p - \Omega_m) + \gamma_m/2
    let d_mech_real = gamma_m * 0.5;
    let d_mech_imag = -(delta_probe - omega_m);
    let d_mech_mag_sq = d_mech_real.powi(2) + d_mech_imag.powi(2);

    // Mechanical term: g^2 / (-i(delta_p - omega_m) + gamma_m/2)
    let g_sq = g.powi(2);
    let term_mech_real = (g_sq * d_mech_real) / d_mech_mag_sq;
    let term_mech_imag = -(g_sq * d_mech_imag) / d_mech_mag_sq;

    // Total cavity denominator: -i(delta_p + Delta) + kappa/2 + term_mech
    let denom_real = (kappa * 0.5) + term_mech_real;
    let denom_imag = -(delta_probe + control_detuning) + term_mech_imag;
    let denom_mag_sq = denom_real.powi(2) + denom_imag.powi(2);

    // Cavity response fraction: kappa_ex / denom
    let frac_real = (kappa_ex * denom_real) / denom_mag_sq;
    let frac_imag = -(kappa_ex * denom_imag) / denom_mag_sq;

    // Probe transmission: t_p = 1 - fraction
    let tp_real = 1.0 - frac_real;
    let tp_imag = -frac_imag;

    let transmission_power = (tp_real.powi(2) + tp_imag.powi(2)).clamp(0.0, 1.0);
    let phase_rad = tp_imag.atan2(tp_real);

    // Baseline transmission with g = 0 (unperturbed cavity Lorentz dip)
    let denom0_real = kappa * 0.5;
    let denom0_imag = -(delta_probe + control_detuning);
    let denom0_mag_sq = denom0_real.powi(2) + denom0_imag.powi(2);
    let tp0_real = 1.0 - (kappa_ex * denom0_real) / denom0_mag_sq;
    let tp0_imag = (kappa_ex * denom0_imag) / denom0_mag_sq;
    let power_baseline = (tp0_real.powi(2) + tp0_imag.powi(2)).clamp(0.0, 1.0);

    let contrast = (transmission_power - power_baseline).max(0.0);

    // OMIT transparency linewidth: Gamma_omit = gamma_m * (1 + C_opt) in Hz
    let c_opt = optical_cooperativity(g, kappa, gamma_m);
    let linewidth_rad_s = gamma_m * (1.0 + c_opt);
    let fwhm_linewidth_hz = linewidth_rad_s / (2.0 * std::f64::consts::PI);

    OmitTransmissionResult {
        transmission_power,
        phase_rad,
        transparency_contrast: contrast,
        fwhm_linewidth_hz,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_point_fluctuations() {
        let crystal = PiezoOptomechanicalCrystal::aln_crystal();
        let x_zpf = crystal.zero_point_fluctuation_m();
        // For m_eff ~ 350 fg and omega_m ~ 2*pi*4.2 GHz:
        // x_zpf should be around ~ 2-5 femtometers (10^-15 m)
        assert!(x_zpf > 1e-16 && x_zpf < 1e-13);
        assert!(crystal.single_photon_g0 > 1e3); // g0 > kHz
    }

    #[test]
    fn test_sideband_cooling_to_ground_state() {
        let crystal = PiezoOptomechanicalCrystal::lithium_niobate_crystal();
        let t_bath = 0.020; // 20 mK dilution fridge temperature
        let n_th = crystal.thermal_phonon_occupancy(t_bath);
        assert!(n_th < 1.0); // At 20 mK and 5 GHz, n_th is << 1

        // Test at higher temperature (e.g. 4.2 K) where n_th >> 1
        let n_th_init = crystal.thermal_phonon_occupancy(0.8);
        assert!(n_th_init > 1.0);

        // Drive with optical power to achieve high cooperativity
        let p_in = 5.0e-3; // 5 mW
        let n_cav = crystal.intracavity_photon_number(p_in, -crystal.mechanical_frequency_rad_s);
        let g = crystal.linearized_coupling_g(n_cav);

        let n_eff = sideband_cooling_phonon_occupancy(
            g,
            crystal.total_optical_kappa(),
            crystal.mechanical_frequency_rad_s,
            crystal.mechanical_gamma_m,
            n_th_init,
        );

        // Strong cooling suppresses phonon occupancy below ground-state threshold
        assert!(n_eff < n_th_init);
        assert!(is_ground_state_cooled(n_eff));
    }

    #[test]
    fn test_omit_transparency_peak() {
        let crystal = PiezoOptomechanicalCrystal::silicon_nanobeam_crystal();
        let omega_m = crystal.mechanical_frequency_rad_s;
        let kappa = crystal.total_optical_kappa();
        let kappa_ex = crystal.optical_kappa_ex;
        let gamma_m = crystal.mechanical_gamma_m;

        let delta_control = -omega_m; // Red sideband pump
        let g = 2.0 * std::f64::consts::PI * 15.0e6; // 15 MHz coupling

        // On resonance probe: delta_probe = omega_m
        let res_on =
            omit_probe_transmission(omega_m, delta_control, g, kappa, kappa_ex, omega_m, gamma_m);

        // Off resonance probe: delta_probe = omega_m + 50 MHz
        let res_off = omit_probe_transmission(
            omega_m + 2.0 * std::f64::consts::PI * 50.0e6,
            delta_control,
            g,
            kappa,
            kappa_ex,
            omega_m,
            gamma_m,
        );

        // On resonance, OMIT produces high transmission / transparency window
        assert!(res_on.transmission_power > res_off.transmission_power);
        assert!(res_on.transparency_contrast > 0.1);
    }
}
