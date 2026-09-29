//! Braiding protocols, gate ramp schedules, topological qubit encoding,
//! and dispersive quantum capacitance parity readout.
//!
//! Formulates:
//! - Alicea 3-step adiabatic exchange choreography in Y-junctions.
//! - Smooth polynomial ($C^2$) vs linear gate voltage schedules.
//! - 4-Majorana topological qubit register with parity conservation.
//! - Non-Abelian braid transformations $B_{12}, B_{23}$ and braid relation verification.
//! - Dispersive quantum capacitance and RF reflectometry parity readout.

use super::hexagonal_array::{ELEMENTARY_CHARGE_C, HBAR_EV_S};
use std::f64::consts::PI;

/// Gate ramp trajectory profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateRampProfile {
    /// Linear interpolation $s(t) = t / \tau$.
    Linear,
    /// Minimum-jerk C2-smooth polynomial $s(t) = 10(t/\tau)^3 - 15(t/\tau)^4 + 6(t/\tau)^5$.
    SmoothPolynomial,
    /// Raised-cosine / sinusoidal ramp $s(t) = \frac{1}{2}[1 - \cos(\pi t / \tau)]$.
    Sinusoidal,
}

impl GateRampProfile {
    /// Evaluates normalized position $s(t) \in [0, 1]$ given normalized time $\tau_{norm} = t / \tau_{step} \in [0, 1]$.
    pub fn evaluate_position(&self, tau_norm: f64) -> f64 {
        let t = tau_norm.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::SmoothPolynomial => t.powi(3) * (10.0 - 15.0 * t + 6.0 * t.powi(2)),
            Self::Sinusoidal => 0.5 * (1.0 - (PI * t).cos()),
        }
    }

    /// Evaluates normalized velocity $\frac{ds}{d(t/\tau)}$ for adiabaticity checks.
    pub fn evaluate_velocity(&self, tau_norm: f64) -> f64 {
        let t = tau_norm.clamp(0.0, 1.0);
        match self {
            Self::Linear => 1.0,
            Self::SmoothPolynomial => 30.0 * t.powi(2) - 60.0 * t.powi(3) + 30.0 * t.powi(4),
            Self::Sinusoidal => 0.5 * PI * (PI * t).sin(),
        }
    }
}

/// A stage in the 3-step Alicea exchange protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliceaBraidStage {
    /// Stage 1: Move $\gamma_1$ from Arm 0 into auxiliary Arm 2 via center vertex.
    MoveGamma1ToAuxiliary,
    /// Stage 2: Move $\gamma_2$ from Arm 1 into Arm 0 via center vertex.
    MoveGamma2ToArm0,
    /// Stage 3: Move $\gamma_1$ from auxiliary Arm 2 into Arm 1 via center vertex.
    MoveGamma1ToArm1,
}

/// Alicea 3-step tri-junction braiding coordinator.
#[derive(Debug, Clone, PartialEq)]
pub struct AliceaTriJunctionBraiding {
    /// Duration of each of the 3 stages in seconds.
    pub stage_duration_s: f64,
    /// Ramp profile for gate modulation.
    pub profile: GateRampProfile,
    /// Physical arm length in nanometers.
    pub arm_length_nm: f64,
}

impl AliceaTriJunctionBraiding {
    pub fn new(stage_duration_s: f64, profile: GateRampProfile, arm_length_nm: f64) -> Self {
        Self {
            stage_duration_s,
            profile,
            arm_length_nm,
        }
    }

    /// Total braid duration $\tau_{total} = 3 \times \tau_{stage}$.
    pub fn total_duration_s(&self) -> f64 {
        3.0 * self.stage_duration_s
    }

    /// Evaluates spatial positions $(d_1, d_2)$ of $\gamma_1$ and $\gamma_2$ along arms at time $t$.
    /// Position is represented as `(arm_index, distance_from_junction_nm)`.
    pub fn instantaneous_positions(&self, t_seconds: f64) -> ((usize, f64), (usize, f64)) {
        let t_total = self.total_duration_s();
        let t_clamped = t_seconds.clamp(0.0, t_total);
        let stage_idx = (t_clamped / self.stage_duration_s).floor() as usize;
        let stage = match stage_idx.min(2) {
            0 => AliceaBraidStage::MoveGamma1ToAuxiliary,
            1 => AliceaBraidStage::MoveGamma2ToArm0,
            _ => AliceaBraidStage::MoveGamma1ToArm1,
        };

        let t_in_stage = t_clamped - (stage_idx.min(2) as f64) * self.stage_duration_s;
        let tau_norm = t_in_stage / self.stage_duration_s;
        let s = self.profile.evaluate_position(tau_norm);

        match stage {
            AliceaBraidStage::MoveGamma1ToAuxiliary => {
                // gamma1 moves from outer tip of Arm 0 to junction, then to outer tip of Arm 2
                let (arm1, pos1) = if s <= 0.5 {
                    let frac = 1.0 - 2.0 * s; // 1.0 -> 0.0 on Arm 0
                    (0, frac * self.arm_length_nm)
                } else {
                    let frac = (s - 0.5) * 2.0; // 0.0 -> 1.0 on Arm 2
                    (2, frac * self.arm_length_nm)
                };
                // gamma2 stays stationary at tip of Arm 1
                let (arm2, pos2) = (1, self.arm_length_nm);
                ((arm1, pos1), (arm2, pos2))
            }
            AliceaBraidStage::MoveGamma2ToArm0 => {
                // gamma1 stays stationary at tip of Arm 2
                let (arm1, pos1) = (2, self.arm_length_nm);
                // gamma2 moves from tip of Arm 1 through junction into tip of Arm 0
                let (arm2, pos2) = if s <= 0.5 {
                    let frac = 1.0 - 2.0 * s; // 1.0 -> 0.0 on Arm 1
                    (1, frac * self.arm_length_nm)
                } else {
                    let frac = (s - 0.5) * 2.0; // 0.0 -> 1.0 on Arm 0
                    (0, frac * self.arm_length_nm)
                };
                ((arm1, pos1), (arm2, pos2))
            }
            AliceaBraidStage::MoveGamma1ToArm1 => {
                // gamma1 moves from tip of Arm 2 through junction into tip of Arm 1
                let (arm1, pos1) = if s <= 0.5 {
                    let frac = 1.0 - 2.0 * s; // 1.0 -> 0.0 on Arm 2
                    (2, frac * self.arm_length_nm)
                } else {
                    let frac = (s - 0.5) * 2.0; // 0.0 -> 1.0 on Arm 1
                    (1, frac * self.arm_length_nm)
                };
                // gamma2 stays stationary at tip of Arm 0
                let (arm2, pos2) = (0, self.arm_length_nm);
                ((arm1, pos1), (arm2, pos2))
            }
        }
    }

    /// Evaluates minimum Euclidean spatial separation $d_{min}(t)$ between $\gamma_1$ and $\gamma_2$ in nm.
    pub fn separation_nm(&self, t_seconds: f64) -> f64 {
        let ((arm1, pos1), (arm2, pos2)) = self.instantaneous_positions(t_seconds);
        if arm1 == arm2 {
            (pos1 - pos2).abs()
        } else {
            // Arms meet at 120 degrees: d^2 = p1^2 + p2^2 - 2 p1 p2 cos(120 deg) = p1^2 + p2^2 + p1 p2
            (pos1.powi(2) + pos2.powi(2) + pos1 * pos2).sqrt()
        }
    }

    /// Diabatic Landau-Zener error probability:
    /// $$\epsilon_{diab} \approx \exp\left( - \frac{\pi \Delta_{top}^2 \tau_{stage}}{2 \hbar |d\mu/dt|} \right)$$
    pub fn diabatic_error(&self, topological_gap_mev: f64) -> f64 {
        let gap_ev = topological_gap_mev * 1e-3;
        let max_vel = match self.profile {
            GateRampProfile::Linear => 1.0,
            GateRampProfile::SmoothPolynomial => 1.875,
            GateRampProfile::Sinusoidal => 0.5 * PI,
        };

        // Suppressed exponentially by smoothness of ramp: C^2 profiles have super-exponential suppression
        let adiabatic_ratio = (gap_ev * self.stage_duration_s) / (HBAR_EV_S * max_vel);
        let factor = match self.profile {
            GateRampProfile::Linear => 2.0,
            GateRampProfile::SmoothPolynomial => 8.0,
            GateRampProfile::Sinusoidal => 4.5,
        };

        let exponent = (factor * adiabatic_ratio).min(100.0);
        (-exponent).exp()
    }
}

/// 4-Majorana topological qubit register $(\gamma_1, \gamma_2, \gamma_3, \gamma_4)$.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalQubitRegister {
    /// Complex state vector in logical basis $[\alpha |0_L\rangle + \beta |1_L\rangle]$.
    pub alpha: (f64, f64),
    pub beta: (f64, f64),
    /// Topological minigap $\Delta_{top}$ in meV.
    pub minigap_mev: f64,
}

impl TopologicalQubitRegister {
    /// Initializes in logical $|0_L\rangle = |0_{12}, 0_{34}\rangle$.
    pub fn new_zero(minigap_mev: f64) -> Self {
        Self {
            alpha: (1.0, 0.0),
            beta: (0.0, 0.0),
            minigap_mev,
        }
    }

    /// Initializes in logical $|1_L\rangle = |1_{12}, 1_{34}\rangle$.
    pub fn new_one(minigap_mev: f64) -> Self {
        Self {
            alpha: (0.0, 0.0),
            beta: (1.0, 0.0),
            minigap_mev,
        }
    }

    /// Normalizes state vector.
    pub fn normalize(&mut self) {
        let norm_sq =
            self.alpha.0.powi(2) + self.alpha.1.powi(2) + self.beta.0.powi(2) + self.beta.1.powi(2);
        let norm = norm_sq.sqrt().max(1e-14);
        self.alpha.0 /= norm;
        self.alpha.1 /= norm;
        self.beta.0 /= norm;
        self.beta.1 /= norm;
    }

    /// Applies braid operator $B_{12} = \exp(\frac{\pi}{4}\gamma_1 \gamma_2) = \frac{1}{\sqrt{2}}(1 + \gamma_1 \gamma_2)$.
    /// Corresponds to Phase gate $S = \text{diag}(1, i)$ up to global phase $e^{-i\pi/4}$.
    pub fn apply_braid_12(&mut self) {
        // |0_L> acquires phase e^{-i*pi/4}, |1_L> acquires phase e^{i*pi/4} -> relative phase +pi/2 (i)
        let new_beta_re = -self.beta.1;
        let new_beta_im = self.beta.0;
        self.beta = (new_beta_re, new_beta_im);
        self.normalize();
    }

    /// Applies braid operator $B_{23} = \exp(\frac{\pi}{4}\gamma_2 \gamma_3) = \frac{1}{\sqrt{2}}(1 + \gamma_2 \gamma_3)$.
    /// Corresponds to Hadamard-like rotation mixing parities:
    /// $$\alpha' = \frac{1}{\sqrt{2}}(\alpha - i \beta), \quad \beta' = \frac{1}{\sqrt{2}}(-i \alpha + \beta)$$
    pub fn apply_braid_23(&mut self) {
        let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
        let a_re = inv_sqrt2 * (self.alpha.0 + self.beta.1);
        let a_im = inv_sqrt2 * (self.alpha.1 - self.beta.0);
        let b_re = inv_sqrt2 * (self.beta.0 + self.alpha.1);
        let b_im = inv_sqrt2 * (self.beta.1 - self.alpha.0);

        self.alpha = (a_re, a_im);
        self.beta = (b_re, b_im);
        self.normalize();
    }

    /// Applies logical Pauli $Z_L = B_{12}^2$:
    pub fn apply_pauli_z(&mut self) {
        self.apply_braid_12();
        self.apply_braid_12();
    }

    /// Applies topological Hadamard gate $H_L = B_{12} B_{23} B_{12}$:
    pub fn apply_hadamard(&mut self) {
        self.apply_braid_12();
        self.apply_braid_23();
        self.apply_braid_12();
    }

    /// Evaluates quantum state fidelity $F = |\langle \psi_{target} | \psi \rangle|^2$:
    pub fn fidelity_with(&self, target_alpha: (f64, f64), target_beta: (f64, f64)) -> f64 {
        let inner_re = self.alpha.0 * target_alpha.0
            + self.alpha.1 * target_alpha.1
            + self.beta.0 * target_beta.0
            + self.beta.1 * target_beta.1;
        let inner_im = self.alpha.0 * (-target_alpha.1)
            + self.alpha.1 * target_alpha.0
            + self.beta.0 * (-target_beta.1)
            + self.beta.1 * target_beta.0;
        inner_re.powi(2) + inner_im.powi(2)
    }

    /// Dispersive quantum capacitance parity readout $C_q$ in Farads:
    /// $$C_q = \pm \frac{e^2}{2 \delta E_{hyb}} \tanh\left(\frac{\delta E_{hyb}}{2 k_B T}\right)$$
    pub fn dispersive_quantum_capacitance_f(
        &self,
        hybridization_mev: f64,
        temperature_kelvin: f64,
        parity_even: bool,
    ) -> f64 {
        let kb_ev_k = 8.617_333_262e-5;
        let e_hyb_ev = hybridization_mev.max(1e-6) * 1e-3;
        let thermal_ev = kb_ev_k * temperature_kelvin.max(1e-3);
        let arg = (e_hyb_ev / (2.0 * thermal_ev)).clamp(-30.0, 30.0);
        let thermal_factor = arg.tanh();

        let e_c = ELEMENTARY_CHARGE_C;
        let e_hyb_joules = e_hyb_ev * e_c;
        let c_q_magnitude = (e_c.powi(2) / (2.0 * e_hyb_joules)) * thermal_factor;

        if parity_even {
            c_q_magnitude
        } else {
            -c_q_magnitude
        }
    }

    /// Parity measurement dispersive cavity frequency shift $\chi_{rf}$ in Hz:
    /// $$\chi_{rf} = \pm \frac{g_{res}^2}{\omega_{res} - \delta E_{hyb}/\hbar}$$
    pub fn dispersive_frequency_shift_hz(
        &self,
        coupling_hz: f64,
        detuning_hz: f64,
        parity_even: bool,
    ) -> f64 {
        let chi = (coupling_hz.powi(2) / detuning_hz.max(1e3)).abs();
        if parity_even {
            chi
        } else {
            -chi
        }
    }
}
