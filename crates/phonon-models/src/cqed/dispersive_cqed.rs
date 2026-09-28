//! Jaynes-Cummings dispersive Hamiltonian, AC Stark shift, state-dependent cavity pull,
//! and photon shot noise dephasing in circuit QED.

use crate::cqed::{MicrowaveCavity, TransmonParams};

/// Coupled Transmon-Cavity Circuit QED system operating in the dispersive regime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispersiveCqedSystem {
    /// Transmon physical parameters.
    pub transmon: TransmonParams,
    /// Readout cavity parameters.
    pub cavity: MicrowaveCavity,
    /// Transmon-cavity vacuum Rabi coupling rate $g / (2\pi)$ in MHz.
    pub coupling_g_mhz: f64,
}

impl DispersiveCqedSystem {
    /// Constructs a coupled cQED system.
    pub fn new(transmon: TransmonParams, cavity: MicrowaveCavity, coupling_g_mhz: f64) -> Self {
        Self {
            transmon,
            cavity,
            coupling_g_mhz,
        }
    }

    /// Standard baseline: 5 GHz transmon coupled to 7 GHz cavity with $g / (2\pi) = 70\text{ MHz}$.
    pub fn standard_baseline() -> Self {
        Self::new(
            TransmonParams::standard_5ghz(),
            MicrowaveCavity::standard_7ghz(),
            70.0,
        )
    }

    /// Detuning between transmon qubit and cavity $\Delta / (2\pi) = \omega_{01} - \omega_r$ in GHz.
    pub fn detuning_ghz(&self) -> f64 {
        self.transmon.omega_01_ghz() - self.cavity.resonance_frequency_ghz
    }

    /// Verifies if system operates in the valid dispersive regime ($|\Delta| > 5 g$).
    pub fn is_dispersive(&self) -> bool {
        let delta_mhz = self.detuning_ghz().abs() * 1.0e3;
        delta_mhz > 5.0 * self.coupling_g_mhz
    }

    /// Effective dispersive shift $\chi / (2\pi)$ in MHz including transmon higher levels ($|2\rangle$):
    /// $$\chi = \frac{g^2 \alpha}{\Delta (\Delta + \alpha)}$$
    /// where $\alpha$ is the transmon negative anharmonicity and $\Delta$ is qubit-cavity detuning.
    pub fn dispersive_shift_chi_mhz(&self) -> f64 {
        let delta = self.detuning_ghz(); // GHz
        let alpha = self.transmon.anharmonicity_ghz(); // GHz (negative)
        let g_ghz = self.coupling_g_mhz * 1.0e-3;

        // chi in GHz = (g^2 * alpha) / (delta * (delta + alpha))
        let denom = delta * (delta + alpha);
        let chi_ghz = (g_ghz.powi(2) * alpha) / denom.max(1e-6);
        chi_ghz * 1.0e3 // MHz
    }

    /// State-dependent cavity resonance frequency when qubit is in ground state $|0\rangle$:
    /// $$\omega_r(|0\rangle) = \omega_r + \chi$$
    pub fn cavity_freq_ground_ghz(&self) -> f64 {
        self.cavity.resonance_frequency_ghz + (self.dispersive_shift_chi_mhz() * 1.0e-3)
    }

    /// State-dependent cavity resonance frequency when qubit is in excited state $|1\rangle$:
    /// $$\omega_r(|1\rangle) = \omega_r - \chi$$
    pub fn cavity_freq_excited_ghz(&self) -> f64 {
        self.cavity.resonance_frequency_ghz - (self.dispersive_shift_chi_mhz() * 1.0e-3)
    }

    /// Frequency pull separation $2\chi / (2\pi)$ between qubit state responses in MHz.
    pub fn cavity_frequency_pull_mhz(&self) -> f64 {
        2.0 * self.dispersive_shift_chi_mhz().abs()
    }

    /// AC Stark shift on transmon qubit per intra-cavity photon in MHz:
    /// $$\delta\omega_q = 2 \chi \bar{n}$$
    pub fn ac_stark_shift_mhz(&self, mean_photon_number: f64) -> f64 {
        2.0 * self.dispersive_shift_chi_mhz() * mean_photon_number
    }

    /// Dephasing rate due to intra-cavity photon shot noise $\Gamma_\phi$ in kHz:
    /// $$\Gamma_\phi \approx \frac{4 \chi^2}{\kappa_r} \bar{n}$$
    pub fn photon_shot_noise_dephasing_khz(&self, mean_photon_number: f64) -> f64 {
        let chi = self.dispersive_shift_chi_mhz();
        let kappa = self.cavity.kappa_total_mhz();
        let rate_mhz = (4.0 * chi.powi(2) / kappa.max(1e-3)) * mean_photon_number;
        rate_mhz * 1.0e3 // kHz
    }

    /// Critical intra-cavity photon number $n_{crit} = \Delta^2 / (4 g^2)$ where dispersive approximation breaks down.
    pub fn critical_photon_number(&self) -> f64 {
        let delta_mhz = self.detuning_ghz().abs() * 1.0e3;
        delta_mhz.powi(2) / (4.0 * self.coupling_g_mhz.powi(2)).max(1e-6)
    }
}
