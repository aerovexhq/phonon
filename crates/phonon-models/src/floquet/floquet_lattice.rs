//! Floquet-Bloch Lattice Models & Light-Induced Topological Phase Transitions.
//!
//! Formulates time-periodic optical driving of 2D Dirac materials (graphene),
//! high-frequency Magnus expansion, light-induced mass gap opening, Floquet Chern
//! invariants, and chiral Floquet Hall conductance.

use phonon_core::constants::{
    ELEMENTARY_CHARGE, EPSILON_0, H_BAR, PLANCK_CONSTANT, SPEED_OF_LIGHT,
};
use std::f64::consts::PI;

/// Polarization state of the periodic optical drive field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloquetPolarization {
    /// Linearly polarized light with polarization angle in radians relative to the x-axis.
    Linear { angle_rad: f64 },
    /// Circularly polarized light with helicity (+1.0 for right-handed RCP, -1.0 for left-handed LCP).
    Circular { helicity: f64 },
    /// Elliptically polarized light with major axis angle and ellipticity parameter $\epsilon \in [-1.0, 1.0]$.
    Elliptical {
        major_axis_angle_rad: f64,
        ellipticity: f64,
    },
}

impl FloquetPolarization {
    /// Creates a right-circularly polarized (RCP) state ($\xi = +1.0$).
    pub fn rcp() -> Self {
        Self::Circular { helicity: 1.0 }
    }

    /// Creates a left-circularly polarized (LCP) state ($\xi = -1.0$).
    pub fn lcp() -> Self {
        Self::Circular { helicity: -1.0 }
    }

    /// Creates a linearly polarized state along the given angle in radians.
    pub fn linear(angle_rad: f64) -> Self {
        Self::Linear { angle_rad }
    }

    /// Returns the circular helicity $\xi \in [-1.0, 1.0]$ (0.0 for linear polarization).
    pub fn helicity(&self) -> f64 {
        match *self {
            Self::Linear { .. } => 0.0,
            Self::Circular { helicity } => helicity.clamp(-1.0, 1.0),
            Self::Elliptical { ellipticity, .. } => ellipticity.clamp(-1.0, 1.0),
        }
    }
}

/// Optical drive laser pulse parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetLaserPulse {
    /// Central wavelength $\lambda$ in meters (e.g. 3.2 um mid-IR or 800 nm).
    pub wavelength_m: f64,
    /// Peak electric field amplitude $E_0$ in V/m (e.g. 0.4 V/nm).
    pub peak_electric_field_v_per_m: f64,
    /// Pulse envelope duration (FWHM) in seconds (e.g. 60 fs).
    pub pulse_duration_seconds: f64,
}

impl Default for FloquetLaserPulse {
    fn default() -> Self {
        Self {
            wavelength_m: 3.2e-6,               // 3.2 um (mid-IR resonant Floquet drive)
            peak_electric_field_v_per_m: 4.0e8, // 0.4 V/nm
            pulse_duration_seconds: 60.0e-15,   // 60 fs
        }
    }
}

impl FloquetLaserPulse {
    /// Optical angular frequency $\Omega = 2\pi c / \lambda$ in rad/s.
    pub fn angular_frequency_rad(&self) -> f64 {
        2.0 * PI * SPEED_OF_LIGHT / self.wavelength_m
    }

    /// Photon energy $\hbar\Omega$ in Joules.
    pub fn photon_energy_joules(&self) -> f64 {
        H_BAR * self.angular_frequency_rad()
    }

    /// Photon energy $\hbar\Omega$ in electron-Volts (eV).
    pub fn photon_energy_ev(&self) -> f64 {
        self.photon_energy_joules() / ELEMENTARY_CHARGE
    }

    /// Driving optical cycle period $T = 2\pi / \Omega$ in seconds.
    pub fn optical_period_seconds(&self) -> f64 {
        self.wavelength_m / SPEED_OF_LIGHT
    }

    /// Vector potential amplitude $A_0 = E_0 / \Omega$ in $\text{V}\cdot\text{s/m}$.
    pub fn vector_potential_amplitude(&self) -> f64 {
        self.peak_electric_field_v_per_m / self.angular_frequency_rad()
    }

    /// Peak optical cycle intensity $I_0 = \frac{1}{2} c \epsilon_0 E_0^2$ in $\text{W/m}^2$.
    pub fn peak_intensity_w_per_m2(&self) -> f64 {
        0.5 * SPEED_OF_LIGHT
            * EPSILON_0
            * self.peak_electric_field_v_per_m
            * self.peak_electric_field_v_per_m
    }
}

/// 2D Honeycomb Floquet-Dirac lattice model for graphene under time-periodic drive.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetGrapheneLattice {
    /// Nearest-neighbor tight-binding hopping energy $t_0$ in eV (default 2.8 eV).
    pub hopping_energy_ev: f64,
    /// Carbon-carbon bond length $a_{cc}$ in meters (default 0.142 nm).
    pub carbon_bond_length_m: f64,
}

impl Default for FloquetGrapheneLattice {
    fn default() -> Self {
        Self {
            hopping_energy_ev: 2.8,
            carbon_bond_length_m: 0.142e-9,
        }
    }
}

impl FloquetGrapheneLattice {
    /// Hexagonal lattice constant $a = \sqrt{3} a_{cc}$ in meters (~0.246 nm).
    pub fn lattice_constant_m(&self) -> f64 {
        3.0_f64.sqrt() * self.carbon_bond_length_m
    }

    /// Renormalized bare Fermi velocity $v_F = \frac{3 t_0 a_{cc}}{2 \hbar}$ in m/s.
    pub fn fermi_velocity_m_per_s(&self) -> f64 {
        let t0_joules = self.hopping_energy_ev * ELEMENTARY_CHARGE;
        (1.5 * t0_joules * self.carbon_bond_length_m) / H_BAR
    }

    /// Dimensionless laser drive amplitude:
    /// $$\mathcal{A} = \frac{e A_0 a_{cc}}{\hbar}$$
    pub fn dimensionless_drive_amplitude(&self, pulse: &FloquetLaserPulse) -> f64 {
        let a0 = pulse.vector_potential_amplitude();
        (ELEMENTARY_CHARGE * a0 * self.carbon_bond_length_m) / H_BAR
    }

    /// Floquet mass gap $\Delta_{Floquet}$ in eV opened at the Dirac points ($K, K'$ valleys)
    /// under circular polarization via the leading-order Magnus expansion:
    /// $$\Delta_{Floquet} = \frac{\sqrt{3} t_0^2 \mathcal{A}^2}{\hbar\Omega} |\xi|$$
    pub fn floquet_mass_gap_ev(
        &self,
        pulse: &FloquetLaserPulse,
        polarization: &FloquetPolarization,
    ) -> f64 {
        let hbar_omega_ev = pulse.photon_energy_ev();
        if hbar_omega_ev <= 0.0 {
            return 0.0;
        }

        let a_dim = self.dimensionless_drive_amplitude(pulse);
        let helicity = polarization.helicity().abs();

        if helicity < 1e-6 {
            // Linear polarization does not break time-reversal symmetry at order 1/(hbar Omega)
            0.0
        } else {
            let t0 = self.hopping_energy_ev;
            let gap = (3.0_f64.sqrt() * t0 * t0 * a_dim * a_dim) / hbar_omega_ev;
            gap * helicity
        }
    }

    /// Total full topological bandgap $\Delta_{gap} = 2 \Delta_{Floquet}$ in eV.
    pub fn total_topological_bandgap_ev(
        &self,
        pulse: &FloquetLaserPulse,
        polarization: &FloquetPolarization,
    ) -> f64 {
        2.0 * self.floquet_mass_gap_ev(pulse, polarization)
    }

    /// Floquet Chern number $\mathcal{C} = \text{sgn}(\xi) \in \{-1, 0, +1\}$.
    pub fn floquet_chern_number(&self, polarization: &FloquetPolarization) -> i32 {
        let h = polarization.helicity();
        if h > 1e-4 {
            1
        } else if h < -1e-4 {
            -1
        } else {
            0
        }
    }

    /// Quantized anomalous Floquet Hall conductance in Siemens (S):
    /// $$\sigma_{xy} = \mathcal{C} \frac{e^2}{h}$$
    pub fn floquet_hall_conductance_si(&self, polarization: &FloquetPolarization) -> f64 {
        let c = self.floquet_chern_number(polarization) as f64;
        let conductance_quantum = (ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / PLANCK_CONSTANT;
        c * conductance_quantum
    }

    /// Floquet quasi-energy dispersion at wavevector $(k_x, k_y)$ in $\text{m}^{-1}$:
    /// Returns conduction and valence band energies $(\epsilon_+, \epsilon_-)$ in eV.
    pub fn quasi_energy_dispersion_ev(
        &self,
        kx: f64,
        ky: f64,
        pulse: &FloquetLaserPulse,
        polarization: &FloquetPolarization,
    ) -> (f64, f64) {
        let acc = self.carbon_bond_length_m;
        // Tight-binding structure factor f(k)
        let kx_term = 0.5 * 3.0_f64.sqrt() * kx * acc;
        let ky_term = 1.5 * ky * acc;

        let cos_kx = kx_term.cos();
        let cos_ky = ky_term.cos();

        // |f(k)|^2 = 1 + 4 cos^2(kx_term) + 4 cos(kx_term) cos(ky_term)
        let f_mag_sq = (1.0 + 4.0 * cos_kx * cos_kx + 4.0 * cos_kx * cos_ky).max(0.0);
        let bare_energy = self.hopping_energy_ev * f_mag_sq.sqrt();

        let mass_gap = self.floquet_mass_gap_ev(pulse, polarization);
        let total_energy = (bare_energy * bare_energy + mass_gap * mass_gap).sqrt();

        (total_energy, -total_energy)
    }
}
