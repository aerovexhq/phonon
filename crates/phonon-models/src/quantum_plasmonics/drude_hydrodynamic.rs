#![deny(unsafe_code)]

//! Non-Local Hydrodynamic Drude-Lorentz Electron Gas & Surface Plasmon Polaritons.
//!
//! Formulates complex permittivity of noble metals, non-local hydrodynamic quantum
//! electron pressure $\beta_{nl} = \sqrt{3/5} v_F$, surface plasmon resonance blueshift,
//! and Feibelman $d$-parameter surface charge centroid shifts.

/// Speed of light in vacuum $c$ in m/s.
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;
/// Reduced Planck constant $\hbar$ in J*s.
pub const HBAR: f64 = 1.054_571_817e-34;
/// Elementary charge $e$ in Coulombs.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;
/// Vacuum permittivity $\varepsilon_0$ in F/m.
pub const EPSILON_0: f64 = 8.854_187_812_8e-12;

/// Noble metal material for plasmonic nanocircuits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NobleMetal {
    /// Silver (Ag) - lowest optical loss in visible and near-IR.
    Silver,
    /// Gold (Au) - chemically inert, interband absorption below 550 nm.
    Gold,
    /// Aluminum (Al) - deep ultraviolet plasmonics, high plasma frequency.
    Aluminum,
    /// Copper (Cu) - CMOS-compatible, optical properties similar to gold.
    Copper,
}

impl NobleMetal {
    /// Plasma frequency $\omega_p$ in rad/s.
    pub fn plasma_frequency(&self) -> f64 {
        match self {
            Self::Silver => 1.37e16,   // ~9.0 eV
            Self::Gold => 1.37e16,     // ~9.0 eV
            Self::Aluminum => 2.28e16, // ~15.0 eV
            Self::Copper => 1.32e16,   // ~8.7 eV
        }
    }

    /// Drude collision damping rate $\gamma_d = 1 / \tau$ in rad/s.
    pub fn collision_damping_rate(&self) -> f64 {
        match self {
            Self::Silver => 3.2e13,    // ~0.021 eV
            Self::Gold => 1.08e14,     // ~0.071 eV
            Self::Aluminum => 1.22e14, // ~0.080 eV
            Self::Copper => 1.45e14,   // ~0.095 eV
        }
    }

    /// High-frequency dielectric constant $\varepsilon_\infty$ from interband core polarization.
    pub fn high_frequency_permittivity(&self) -> f64 {
        match self {
            Self::Silver => 3.7,
            Self::Gold => 9.0,
            Self::Aluminum => 1.0,
            Self::Copper => 6.7,
        }
    }

    /// Fermi velocity $v_F$ in m/s.
    pub fn fermi_velocity(&self) -> f64 {
        match self {
            Self::Silver => 1.39e6,
            Self::Gold => 1.40e6,
            Self::Aluminum => 2.03e6,
            Self::Copper => 1.57e6,
        }
    }

    /// Convective hydrodynamic velocity of degenerate electron gas $\beta_{nl} = \sqrt{\frac{3}{5}} v_F$ in m/s.
    pub fn hydrodynamic_velocity(&self) -> f64 {
        (3.0 / 5.0f64).sqrt() * self.fermi_velocity()
    }

    /// Complex Drude dielectric permittivity $\varepsilon_m(\omega) = \varepsilon' + i \varepsilon''$.
    pub fn dielectric_permittivity(&self, omega: f64) -> (f64, f64) {
        let eps_inf = self.high_frequency_permittivity();
        let wp = self.plasma_frequency();
        let gamma = self.collision_damping_rate();

        let denom = omega * omega + gamma * gamma;
        if denom <= 0.0 {
            return (eps_inf, 0.0);
        }

        let eps_real = eps_inf - (wp * wp) / denom;
        let eps_imag = (wp * wp * gamma) / (omega.max(1e-12) * denom);

        (eps_real, eps_imag)
    }
}

/// Surface Plasmon Polariton (SPP) Dispersion and Non-Local Hydrodynamic Properties.
#[derive(Debug, Clone, Copy)]
pub struct SppHydrodynamicModel {
    /// Metal material.
    pub metal: NobleMetal,
    /// Adjacent dielectric relative permittivity $\varepsilon_d$ (e.g. 1.0 for air, 2.25 for silica).
    pub dielectric_permittivity: f64,
    /// Characteristic nanostructure slot gap or diameter $d$ in meters.
    pub gap_width: f64,
    /// Feibelman surface charge centroid shift $d_\perp$ in meters (typically $0.1 - 0.3\text{ nm}$).
    pub feibelman_d_perp: f64,
}

impl SppHydrodynamicModel {
    /// Creates a new hydrodynamic SPP model.
    pub fn new(
        metal: NobleMetal,
        dielectric_permittivity: f64,
        gap_width: f64,
        feibelman_d_perp: f64,
    ) -> Self {
        Self {
            metal,
            dielectric_permittivity,
            gap_width,
            feibelman_d_perp,
        }
    }

    /// Classical local surface plasmon resonance frequency $\omega_{sp} = \frac{\omega_p}{\sqrt{\varepsilon_\infty + \varepsilon_d}}$ in rad/s.
    pub fn local_surface_plasmon_resonance(&self) -> f64 {
        let wp = self.metal.plasma_frequency();
        let eps_inf = self.metal.high_frequency_permittivity();
        let eps_d = self.dielectric_permittivity;
        wp / (eps_inf + eps_d).sqrt()
    }

    /// Non-local hydrodynamic surface plasmon resonance $\omega_{sp, nl}$ shifted by quantum electron pressure in a nanoscale gap $d$:
    ///
    /// $$\omega_{sp, nl} = \omega_{sp} \sqrt{1 + \frac{2 \beta_{nl}}{\omega_{sp} d}}$$
    pub fn non_local_surface_plasmon_resonance(&self) -> f64 {
        let w_sp = self.local_surface_plasmon_resonance();
        let beta = self.metal.hydrodynamic_velocity();
        let d = self.gap_width.max(1e-10);

        let correction = (2.0 * beta) / (w_sp * d);
        w_sp * (1.0 + correction).sqrt()
    }

    /// Quantum resonance blueshift $\Delta\omega_{nl} = \omega_{sp, nl} - \omega_{sp}$ in rad/s.
    pub fn resonance_blueshift(&self) -> f64 {
        self.non_local_surface_plasmon_resonance() - self.local_surface_plasmon_resonance()
    }

    /// Feibelman surface charge centroid frequency correction $\Delta\omega_{feibelman} / \omega_{sp} \approx -\frac{d_\perp}{d}$:
    pub fn feibelman_centroid_frequency_shift(&self) -> f64 {
        let w_sp = self.local_surface_plasmon_resonance();
        let d = self.gap_width.max(1e-10);
        -w_sp * (self.feibelman_d_perp / d)
    }

    /// Effective SPP wavevector $k_{spp} = k_{real} + i k_{imag}$ including Drude losses and non-local pressure.
    pub fn spp_wavevector(&self, omega: f64) -> (f64, f64) {
        let (eps_m_re, eps_m_im) = self.metal.dielectric_permittivity(omega);
        let eps_d = self.dielectric_permittivity;
        let k0 = omega / SPEED_OF_LIGHT;

        // k_spp = k0 * sqrt((eps_m * eps_d) / (eps_m + eps_d))
        let num_re = eps_m_re * eps_d;
        let num_im = eps_m_im * eps_d;

        let den_re = eps_m_re + eps_d;
        let den_im = eps_m_im;
        let den_sq = den_re * den_re + den_im * den_im;

        let div_re = (num_re * den_re + num_im * den_im) / den_sq.max(1e-18);
        let div_im = (num_im * den_re - num_re * den_im) / den_sq.max(1e-18);

        // Complex square root: sqrt(r * e^(i phi)) = sqrt(r) * (cos(phi/2) + i sin(phi/2))
        let r = (div_re * div_re + div_im * div_im).sqrt();
        let phi = div_im.atan2(div_re);

        let k_local_re = k0 * r.sqrt() * (phi / 2.0).cos();
        let k_local_im = k0 * r.sqrt() * (phi / 2.0).sin().abs();

        // Non-local hydrodynamic factor: 1 + beta_nl / c * sqrt(-eps_m * eps_d) / (|eps_m + eps_d|)
        let beta = self.metal.hydrodynamic_velocity();
        let nl_factor = 1.0 + (beta / SPEED_OF_LIGHT) * (r.sqrt() / den_sq.sqrt().max(0.1));

        (k_local_re * nl_factor, k_local_im * nl_factor)
    }

    /// SPP effective refractive index $n_{eff} = \text{Re}(k_{spp}) / k_0$.
    pub fn effective_index(&self, omega: f64) -> f64 {
        let (k_re, _) = self.spp_wavevector(omega);
        let k0 = omega / SPEED_OF_LIGHT;
        k_re / k0.max(1e-12)
    }

    /// SPP propagation length $L_{spp} = \frac{1}{2 \text{Im}(k_{spp})}$ in meters.
    pub fn propagation_length(&self, omega: f64) -> f64 {
        let (_, k_im) = self.spp_wavevector(omega);
        if k_im <= 1e-12 {
            1.0e-3
        } else {
            1.0 / (2.0 * k_im)
        }
    }
}
