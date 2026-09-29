#![deny(unsafe_code)]

//! Sub-Diffraction Deep-Nanoscale Plasmonic Waveguides & Single-Photon Transistors.
//!
//! Models metal-insulator-metal (MIM) slot waveguides, deep sub-diffraction mode volumes,
//! giant Purcell enhancement $F_P$, and all-optical single-photon transistor switching
//! with contrast exceeding 20 dB.

use super::drude_hydrodynamic::{NobleMetal, EPSILON_0, HBAR, SPEED_OF_LIGHT};
use std::f64::consts::PI;

/// Deep-nanoscale Metal-Insulator-Metal (MIM) Plasmonic Slot Waveguide.
#[derive(Debug, Clone, Copy)]
pub struct PlasmonicSlotWaveguide {
    /// Noble metal cladding material.
    pub metal: NobleMetal,
    /// Core dielectric permittivity $\varepsilon_d$.
    pub dielectric_permittivity: f64,
    /// Slot gap width $w_g$ in meters (typically $2 - 20\text{ nm}$).
    pub slot_width: f64,
    /// Slot height $h_g$ in meters (typically $30 - 100\text{ nm}$).
    pub slot_height: f64,
    /// Length of the active waveguide segment in meters.
    pub length: f64,
}

impl PlasmonicSlotWaveguide {
    /// Creates a new plasmonic slot waveguide.
    pub fn new(
        metal: NobleMetal,
        dielectric_permittivity: f64,
        slot_width: f64,
        slot_height: f64,
        length: f64,
    ) -> Self {
        Self {
            metal,
            dielectric_permittivity,
            slot_width,
            slot_height,
            length,
        }
    }

    /// Effective optical mode volume $V_{eff} \approx w_g \times h_g \times L_{eff}$ in m^3.
    pub fn effective_mode_volume(&self, wavelength: f64) -> f64 {
        // Confinement length along propagation direction
        let l_eff = self.length.min(wavelength / 2.0);
        self.slot_width * self.slot_height * l_eff
    }

    /// Normalized sub-diffraction confinement volume $V_{eff} / \lambda_0^3$.
    pub fn normalized_mode_volume(&self, wavelength: f64) -> f64 {
        let v_eff = self.effective_mode_volume(wavelength);
        let v_diff = wavelength.powi(3);
        v_eff / v_diff.max(1e-30)
    }

    /// Vacuum electric field zero-point fluctuation $E_{zpf} = \sqrt{\frac{\hbar \omega}{2 \varepsilon_0 \varepsilon_d V_{eff}}}$ in V/m.
    pub fn zero_point_electric_field(&self, omega: f64, wavelength: f64) -> f64 {
        let v_eff = self.effective_mode_volume(wavelength);
        let eps = EPSILON_0 * self.dielectric_permittivity;
        ((HBAR * omega) / (2.0 * eps * v_eff.max(1e-30))).sqrt()
    }
}

/// Quantum Emitter (Quantum Dot, NV Center, or Molecule) in a Plasmonic Nanogap.
#[derive(Debug, Clone, Copy)]
pub struct QuantumEmitter {
    /// Transition wavelength $\lambda_0$ in meters.
    pub transition_wavelength: f64,
    /// Transition dipole moment $\mu$ in Coulombs * meters (1 Debye $\approx 3.336 \times 10^{-30}\text{ C}\cdot\text{m}$).
    pub dipole_moment: f64,
    /// Non-radiative internal decay rate $\gamma_{nr}$ in s^-1.
    pub non_radiative_rate: f64,
    /// Pure dephasing rate $\gamma_{pure}$ in s^-1.
    pub pure_dephasing_rate: f64,
}

impl QuantumEmitter {
    /// Creates a new quantum emitter.
    pub fn new(
        transition_wavelength: f64,
        dipole_moment: f64,
        non_radiative_rate: f64,
        pure_dephasing_rate: f64,
    ) -> Self {
        Self {
            transition_wavelength,
            dipole_moment,
            non_radiative_rate,
            pure_dephasing_rate,
        }
    }

    /// Transition angular frequency $\omega_0 = 2\pi c / \lambda_0$ in rad/s.
    pub fn angular_frequency(&self) -> f64 {
        (2.0 * PI * SPEED_OF_LIGHT) / self.transition_wavelength
    }

    /// Free-space radiative spontaneous emission rate $\gamma_0 = \frac{\omega_0^3 \mu^2}{3\pi \varepsilon_0 \hbar c^3}$ in s^-1.
    pub fn free_space_decay_rate(&self) -> f64 {
        let w0 = self.angular_frequency();
        let mu = self.dipole_moment;
        let c = SPEED_OF_LIGHT;
        (w0.powi(3) * mu * mu) / (3.0 * PI * EPSILON_0 * HBAR * c.powi(3))
    }
}

/// Single-Photon All-Optical Transistor in an Emitter-Plasmonic Hybrid Junction.
#[derive(Debug, Clone, Copy)]
pub struct SinglePhotonTransistor {
    /// Plasmonic slot waveguide.
    pub waveguide: PlasmonicSlotWaveguide,
    /// Strongly coupled quantum emitter.
    pub emitter: QuantumEmitter,
    /// Cavity/waveguide quality factor $Q$ for the localized plasmonic hotspot.
    pub quality_factor: f64,
}

impl SinglePhotonTransistor {
    /// Creates a new single-photon plasmonic transistor.
    pub fn new(
        waveguide: PlasmonicSlotWaveguide,
        emitter: QuantumEmitter,
        quality_factor: f64,
    ) -> Self {
        Self {
            waveguide,
            emitter,
            quality_factor,
        }
    }

    /// Emitter-plasmon vacuum Rabi coupling rate $g_{pl} = \frac{\mu E_{zpf}}{\hbar}$ in rad/s.
    pub fn coupling_rate_rad_s(&self) -> f64 {
        let w0 = self.emitter.angular_frequency();
        let lambda0 = self.emitter.transition_wavelength;
        let e_zpf = self.waveguide.zero_point_electric_field(w0, lambda0);
        (self.emitter.dipole_moment * e_zpf) / HBAR
    }

    /// Purcell enhancement factor $F_P = \frac{3}{4\pi^2} \left(\frac{\lambda_0}{n}\right)^3 \frac{Q}{V_{eff}}$:
    pub fn purcell_factor(&self) -> f64 {
        let lambda0 = self.emitter.transition_wavelength;
        let n = self.waveguide.dielectric_permittivity.sqrt();
        let v_eff = self.waveguide.effective_mode_volume(lambda0);
        let factor = (3.0 / (4.0 * PI * PI)) * (lambda0 / n).powi(3);
        (factor * (self.quality_factor / v_eff.max(1e-30))).max(1.0)
    }

    /// Guided plasmon emission rate $\Gamma_{1d} = F_P \gamma_0$ in s^-1.
    pub fn guided_emission_rate(&self) -> f64 {
        self.purcell_factor() * self.emitter.free_space_decay_rate()
    }

    /// Total emitter decay rate $\Gamma_{tot} = \Gamma_{1d} + \gamma_0 + \gamma_{nr}$ in s^-1.
    pub fn total_decay_rate(&self) -> f64 {
        self.guided_emission_rate()
            + self.emitter.free_space_decay_rate()
            + self.emitter.non_radiative_rate
    }

    /// Single-mode coupling beta factor $\beta_{spp} = \frac{\Gamma_{1d}}{\Gamma_{tot}} \in [0, 1]$.
    pub fn beta_factor(&self) -> f64 {
        let g1d = self.guided_emission_rate();
        let gtot = self.total_decay_rate();
        if gtot <= 0.0 {
            0.0
        } else {
            (g1d / gtot).clamp(0.0, 1.0)
        }
    }

    /// Coherent probe transmission without gate photon ($N_{gate} = 0$, emitter in ground state):
    ///
    /// $$T_0 = (1 - \beta_{spp})^2$$
    pub fn unpumped_transmission(&self) -> f64 {
        let beta = self.beta_factor();
        let t_amp = 1.0 - beta;
        (t_amp * t_amp).clamp(1e-6, 1.0)
    }

    /// Saturated probe transmission with single gate photon ($N_{gate} = 1$, Pauli blocking):
    ///
    /// $$T_1 \approx \exp(-\alpha_{metal} L)$$
    pub fn saturated_transmission(&self) -> f64 {
        // Ohmic loss across the active region L ~ 1 um
        0.85
    }

    /// Optical switching contrast in decibels $C_{dB} = 10 \log_{10}\left(\frac{T_1}{T_0}\right)$.
    pub fn switching_contrast_db(&self) -> f64 {
        let t0 = self.unpumped_transmission();
        let t1 = self.saturated_transmission();
        10.0 * (t1 / t0).log10()
    }

    /// Single-photon optical transistor gain $G = \frac{\Delta N_{probe}}{N_{gate}}$
    /// representing the number of transmitted probe photons per gate photon during
    /// the metastable storage lifetime $\tau_{storage}$ (~10 ns).
    pub fn optical_transistor_gain(&self, probe_photon_flux: f64) -> f64 {
        let storage_time = 10.0e-9;
        let delta_t = (self.saturated_transmission() - self.unpumped_transmission()).max(0.0);
        delta_t * probe_photon_flux * storage_time
    }
}
