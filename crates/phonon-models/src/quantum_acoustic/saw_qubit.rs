#![deny(unsafe_code)]

//! Surface Acoustic Wave (SAW) Qubits, Transmon Coupling & Phonon-Mediated Entanglement.
//!
//! Models quantum acoustic circuit QED (cQAD) where superconducting transmon qubits
//! couple strongly to localized acoustic cavity phonons via piezoelectric potential,
//! executing vacuum Rabi oscillations, quantum memory storage, and virtual phonon
//! mediated remote two-qubit entanglement.

use super::saw_cavity::{ELEMENTARY_CHARGE, HBAR};
use std::f64::consts::PI;

/// Superconducting Transmon Qubit in the Circuit Quantum Acoustodynamics (cQAD) regime.
#[derive(Debug, Clone, Copy)]
pub struct TransmonQubit {
    /// Josephson energy $E_J$ in Joules.
    pub josephson_energy: f64,
    /// Charging energy $E_C$ in Joules.
    pub charging_energy: f64,
    /// Energy relaxation time $T_1$ in seconds.
    pub t1: f64,
    /// Pure dephasing time $T_\phi$ in seconds.
    pub t_phi: f64,
}

impl TransmonQubit {
    /// Creates a new transmon qubit from fundamental circuit energies.
    pub fn from_energies(e_j: f64, e_c: f64, t1: f64, t_phi: f64) -> Self {
        Self {
            josephson_energy: e_j,
            charging_energy: e_c,
            t1,
            t_phi,
        }
    }

    /// Creates a transmon qubit from design frequency $f_q$ in Hz and anharmonicity $\alpha$ in Hz.
    pub fn from_frequency_and_anharmonicity(
        freq_hz: f64,
        anharmonicity_hz: f64,
        t1: f64,
        t_phi: f64,
    ) -> Self {
        // alpha = -E_C / h  =>  E_C = h * |alpha|
        let h = 2.0 * PI * HBAR;
        let e_c = h * anharmonicity_hz.abs();
        // omega_q = (sqrt(8 E_J E_C) - E_C) / hbar
        // sqrt(8 E_J E_C) = h * freq_hz + E_C
        let target = h * freq_hz + e_c;
        let e_j = (target * target) / (8.0 * e_c);
        Self {
            josephson_energy: e_j,
            charging_energy: e_c,
            t1,
            t_phi,
        }
    }

    /// Qubit transition frequency $\omega_q = \frac{\sqrt{8 E_C E_J} - E_C}{\hbar}$ in rad/s.
    pub fn angular_frequency(&self) -> f64 {
        let term = (8.0 * self.charging_energy * self.josephson_energy).sqrt();
        (term - self.charging_energy) / HBAR
    }

    /// Qubit frequency $f_q = \omega_q / 2\pi$ in Hz.
    pub fn frequency_hz(&self) -> f64 {
        self.angular_frequency() / (2.0 * PI)
    }

    /// Negative anharmonicity $\alpha = -E_C / \hbar$ in rad/s.
    pub fn anharmonicity_rad_s(&self) -> f64 {
        -self.charging_energy / HBAR
    }

    /// Anharmonicity $\alpha / 2\pi$ in Hz.
    pub fn anharmonicity_hz(&self) -> f64 {
        self.anharmonicity_rad_s() / (2.0 * PI)
    }

    /// Energy relaxation rate $\gamma_1 = 1 / T_1$ in s^-1.
    pub fn relaxation_rate(&self) -> f64 {
        1.0 / self.t1
    }

    /// Pure dephasing rate $\gamma_\phi = 1 / T_\phi$ in s^-1.
    pub fn pure_dephasing_rate(&self) -> f64 {
        1.0 / self.t_phi
    }

    /// Total decoherence rate $\gamma_2 = \frac{\gamma_1}{2} + \gamma_\phi$ in s^-1.
    pub fn total_dephasing_rate(&self) -> f64 {
        0.5 * self.relaxation_rate() + self.pure_dephasing_rate()
    }
}

/// Quantum Acoustic Qubit-Cavity Coupling System.
#[derive(Debug, Clone, Copy)]
pub struct SawQubitCoupling {
    /// Transmon qubit.
    pub qubit: TransmonQubit,
    /// Cavity resonant mode frequency in Hz.
    pub cavity_frequency: f64,
    /// Acoustic cavity decay rate $\kappa$ in rad/s.
    pub cavity_decay_rate: f64,
    /// Capacitive voltage division ratio $\beta \in (0, 1]$ coupling transmon pads to piezoelectric field.
    pub division_ratio: f64,
    /// Acoustic zero-point voltage $V_{zpf}$ in Volts.
    pub v_zpf: f64,
}

impl SawQubitCoupling {
    /// Creates a new SAW qubit coupling system.
    pub fn new(
        qubit: TransmonQubit,
        cavity_frequency: f64,
        cavity_decay_rate: f64,
        division_ratio: f64,
        v_zpf: f64,
    ) -> Self {
        Self {
            qubit,
            cavity_frequency,
            cavity_decay_rate,
            division_ratio,
            v_zpf,
        }
    }

    /// Electro-mechanical vacuum coupling rate $g = \frac{e \beta V_{zpf}}{\hbar}$ in rad/s.
    pub fn coupling_rate_rad_s(&self) -> f64 {
        (ELEMENTARY_CHARGE * self.division_ratio * self.v_zpf) / HBAR
    }

    /// Coupling rate $g / 2\pi$ in Hz.
    pub fn coupling_rate_hz(&self) -> f64 {
        self.coupling_rate_rad_s() / (2.0 * PI)
    }

    /// Vacuum Rabi frequency $\Omega_R = 2 g$ in rad/s.
    pub fn vacuum_rabi_angular_frequency(&self) -> f64 {
        2.0 * self.coupling_rate_rad_s()
    }

    /// Vacuum Rabi splitting $\Delta f_{Rabi} = \frac{2g}{2\pi}$ in Hz.
    pub fn vacuum_rabi_splitting_hz(&self) -> f64 {
        self.coupling_rate_hz() * 2.0
    }

    /// Cavity-qubit detuning $\Delta = \omega_q - \omega_m$ in rad/s.
    pub fn detuning_rad_s(&self) -> f64 {
        self.qubit.angular_frequency() - 2.0 * PI * self.cavity_frequency
    }

    /// Quantum acoustic cooperativity $\mathcal{C} = \frac{4 g^2}{\kappa \gamma_2}$.
    pub fn cooperativity(&self) -> f64 {
        let g = self.coupling_rate_rad_s();
        let kappa = self.cavity_decay_rate;
        let gamma2 = self.qubit.total_dephasing_rate();
        (4.0 * g * g) / (kappa * gamma2)
    }

    /// Checks if the system is in the strong coupling regime ($g > \kappa, \gamma_2$ and $\mathcal{C} > 1$).
    pub fn is_strong_coupling(&self) -> bool {
        let g = self.coupling_rate_rad_s();
        let kappa = self.cavity_decay_rate;
        let gamma2 = self.qubit.total_dephasing_rate();
        g > kappa && g > gamma2 && self.cooperativity() > 1.0
    }

    /// Dispersive shift $\chi = \frac{g^2}{\Delta} \frac{\alpha}{\Delta + \alpha}$ in rad/s.
    pub fn dispersive_shift_rad_s(&self) -> f64 {
        let delta = self.detuning_rad_s();
        let g = self.coupling_rate_rad_s();
        let alpha = self.qubit.anharmonicity_rad_s();
        if delta.abs() < 1e-9 || (delta + alpha).abs() < 1e-9 {
            0.0
        } else {
            (g * g / delta) * (alpha / (delta + alpha))
        }
    }

    /// Dispersive shift $\chi / 2\pi$ in Hz.
    pub fn dispersive_shift_hz(&self) -> f64 {
        self.dispersive_shift_rad_s() / (2.0 * PI)
    }

    /// SWAP gate duration $t_{swap} = \frac{\pi}{2 g}$ for swapping a qubit excitation into a single phonon.
    pub fn swap_duration_seconds(&self) -> f64 {
        let g = self.coupling_rate_rad_s();
        if g <= 0.0 {
            0.0
        } else {
            PI / (2.0 * g)
        }
    }
}

/// Remote Two-Qubit Coupling mediated by a virtual SAW phonon bus.
#[derive(Debug, Clone, Copy)]
pub struct VirtualPhononBus {
    /// Qubit 1 coupling.
    pub coupling1: SawQubitCoupling,
    /// Qubit 2 coupling.
    pub coupling2: SawQubitCoupling,
}

impl VirtualPhononBus {
    /// Creates a virtual phonon bus connecting two remote qubits.
    pub fn new(coupling1: SawQubitCoupling, coupling2: SawQubitCoupling) -> Self {
        Self {
            coupling1,
            coupling2,
        }
    }

    /// Effective exchange interaction $J_{eff} = \frac{g_1 g_2}{2} \left( \frac{1}{\Delta_1} + \frac{1}{\Delta_2} \right)$ in rad/s.
    pub fn effective_exchange_coupling_rad_s(&self) -> f64 {
        let g1 = self.coupling1.coupling_rate_rad_s();
        let g2 = self.coupling2.coupling_rate_rad_s();
        let d1 = self.coupling1.detuning_rad_s();
        let d2 = self.coupling2.detuning_rad_s();

        if d1.abs() < 1e-9 || d2.abs() < 1e-9 {
            0.0
        } else {
            0.5 * g1 * g2 * (1.0 / d1 + 1.0 / d2)
        }
    }

    /// Effective exchange coupling $J_{eff} / 2\pi$ in Hz.
    pub fn effective_exchange_coupling_hz(&self) -> f64 {
        self.effective_exchange_coupling_rad_s() / (2.0 * PI)
    }

    /// Entanglement gate duration $t_{bell} = \frac{\pi}{4 |J_{eff}|}$ for generating Bell state $\frac{|eg\rangle + |ge\rangle}{\sqrt{2}}$.
    pub fn bell_state_duration_seconds(&self) -> f64 {
        let j = self.effective_exchange_coupling_rad_s().abs();
        if j <= 0.0 {
            0.0
        } else {
            PI / (4.0 * j)
        }
    }
}

/// SAW Directional Coupler / Acoustic Beam Splitter.
#[derive(Debug, Clone, Copy)]
pub struct SawBeamSplitter {
    /// Coupling length $L_{bs}$ in meters.
    pub coupling_length: f64,
    /// Distributed spatial acoustic coupling coefficient $C_{bs}$ in m^-1.
    pub coupling_coefficient: f64,
}

impl SawBeamSplitter {
    /// Creates a new SAW acoustic beam splitter.
    pub fn new(coupling_length: f64, coupling_coefficient: f64) -> Self {
        Self {
            coupling_length,
            coupling_coefficient,
        }
    }

    /// Power transmission coefficient $T = \cos^2(C_{bs} L_{bs})$.
    pub fn power_transmission(&self) -> f64 {
        let theta = self.coupling_coefficient * self.coupling_length;
        let c = theta.cos();
        c * c
    }

    /// Power reflection / coupled port coefficient $R = \sin^2(C_{bs} L_{bs})$.
    pub fn power_reflection(&self) -> f64 {
        let theta = self.coupling_coefficient * self.coupling_length;
        let s = theta.sin();
        s * s
    }

    /// Hong-Ou-Mandel phononic bunching dip visibility $V_{HOM} = 1 - 2 |T - R|^2 / (T + R)^2$.
    /// For a 50:50 beam splitter ($T = R = 0.5$), $V_{HOM} = 1.0$ (100% two-phonon bunching).
    pub fn hong_ou_mandel_visibility(&self) -> f64 {
        let t = self.power_transmission();
        let r = self.power_reflection();
        let sum = t + r;
        if sum <= 0.0 {
            0.0
        } else {
            let diff = t - r;
            1.0 - (diff * diff) / (sum * sum)
        }
    }
}
