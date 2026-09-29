//! Full non-Abelian Majorana braiding simulator and topological quantum logic engine.
//!
//! Simulates:
//! - Clifford quantum gates ($S, H, Z, X$) implemented via Majorana braiding operations.
//! - Non-Abelian non-commutativity $\|B_{12} B_{23} - B_{23} B_{12}\| > 0$.
//! - Yang-Baxter / Braid group relations: $B_{12} B_{23} B_{12} = B_{23} B_{12} B_{23}$.
//! - Physical decoherence models: quasiparticle poisoning, thermal fluctuations,
//!   finite-length hybridization jitter, and diabatic Landau-Zener leakage.

use phonon_models::hexagonal_majorana::{
    AliceaTriJunctionBraiding, GateRampProfile, InPlaneMagneticField, MajoranaMaterialParams,
    TopologicalQubitRegister,
};

/// Decoherence and noise environment parameters for Majorana qubits.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaNoiseEnvironment {
    /// Quasi-particle poisoning rate $\Gamma_{qp}$ in $\text{s}^{-1}$ (typically 100 - 1000 Hz).
    pub poisoning_rate_hz: f64,
    /// Cryogenic physical temperature $T$ in Kelvin (typically 10 - 50 mK).
    pub temperature_k: f64,
    /// High-frequency gate voltage noise RMS amplitude in meV.
    pub gate_noise_rms_mev: f64,
}

impl MajoranaNoiseEnvironment {
    pub fn standard_cryogenic() -> Self {
        Self {
            poisoning_rate_hz: 500.0,
            temperature_k: 0.020,
            gate_noise_rms_mev: 0.005,
        }
    }
}

/// Simulation result from a non-Abelian braiding operation.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBraidResult {
    /// Initial logical state: $(\alpha, \beta)$.
    pub initial_state: ((f64, f64), (f64, f64)),
    /// Final logical state after braiding: $(\alpha, \beta)$.
    pub final_state: ((f64, f64), (f64, f64)),
    /// Target state expected from exact non-Abelian braiding: $(\alpha, \beta)$.
    pub target_state: ((f64, f64), (f64, f64)),
    /// Quantum state fidelity $F = |\langle \psi_{target} | \psi_{final} \rangle|^2 \in [0, 1]$.
    pub fidelity: f64,
    /// Parity preservation metric $\Delta P = |P_{final} - P_{expected}|$.
    pub parity_error: f64,
    /// Diabatic Landau-Zener excitation probability.
    pub diabatic_leakage: f64,
    /// Quasi-particle poisoning probability during braid duration: $P_{poison} = 1 - e^{-\Gamma_{qp} \tau}$.
    pub poisoning_probability: f64,
}

/// Full topological braiding simulator engine.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBraidSimulator {
    pub materials: MajoranaMaterialParams,
    pub magnetic_field: InPlaneMagneticField,
    pub noise_env: MajoranaNoiseEnvironment,
    pub arm_length_nm: f64,
}

impl MajoranaBraidSimulator {
    pub fn new(
        materials: MajoranaMaterialParams,
        magnetic_field: InPlaneMagneticField,
        noise_env: MajoranaNoiseEnvironment,
        arm_length_nm: f64,
    ) -> Self {
        Self {
            materials,
            magnetic_field,
            noise_env,
            arm_length_nm,
        }
    }

    /// Simulates elementary braid operation $B_{12}$ on input state $(\alpha_0, \beta_0)$.
    pub fn simulate_braid_12(
        &self,
        initial_alpha: (f64, f64),
        initial_beta: (f64, f64),
        braid_duration_s: f64,
        profile: GateRampProfile,
    ) -> MajoranaBraidResult {
        let braiding =
            AliceaTriJunctionBraiding::new(braid_duration_s / 3.0, profile, self.arm_length_nm);
        let ez = self
            .magnetic_field
            .zeeman_energy_mev(self.materials.g_factor);
        let delta = self.materials.induced_gap_mev;
        let minigap = (ez - delta).abs().min(delta);

        let mut qubit = TopologicalQubitRegister {
            alpha: initial_alpha,
            beta: initial_beta,
            minigap_mev: minigap,
        };
        qubit.normalize();
        let norm_init_a = qubit.alpha;
        let norm_init_b = qubit.beta;

        // Target state under exact B12 = S gate: alpha -> alpha, beta -> i * beta
        let tgt_alpha = norm_init_a;
        let tgt_beta = (-norm_init_b.1, norm_init_b.0);

        qubit.apply_braid_12();

        let diabatic_leakage = braiding.diabatic_error(minigap);
        let p_poison = 1.0 - (-self.noise_env.poisoning_rate_hz * braid_duration_s).exp();

        let kb_ev_k = 8.617_333_262e-5;
        let thermal_ev = kb_ev_k * self.noise_env.temperature_k;
        let p_thermal = (-minigap * 1e-3 / thermal_ev.max(1e-6))
            .exp()
            .clamp(0.0, 0.5);

        let ideal_fidelity = qubit.fidelity_with(tgt_alpha, tgt_beta);
        let net_fidelity =
            (ideal_fidelity * (1.0 - diabatic_leakage) * (1.0 - p_poison) * (1.0 - p_thermal))
                .clamp(0.0, 1.0);

        MajoranaBraidResult {
            initial_state: (norm_init_a, norm_init_b),
            final_state: (qubit.alpha, qubit.beta),
            target_state: (tgt_alpha, tgt_beta),
            fidelity: net_fidelity,
            parity_error: p_poison + p_thermal,
            diabatic_leakage,
            poisoning_probability: p_poison,
        }
    }

    /// Simulates elementary braid operation $B_{23}$ on input state $(\alpha_0, \beta_0)$.
    pub fn simulate_braid_23(
        &self,
        initial_alpha: (f64, f64),
        initial_beta: (f64, f64),
        braid_duration_s: f64,
        profile: GateRampProfile,
    ) -> MajoranaBraidResult {
        let braiding =
            AliceaTriJunctionBraiding::new(braid_duration_s / 3.0, profile, self.arm_length_nm);
        let ez = self
            .magnetic_field
            .zeeman_energy_mev(self.materials.g_factor);
        let delta = self.materials.induced_gap_mev;
        let minigap = (ez - delta).abs().min(delta);

        let mut qubit = TopologicalQubitRegister {
            alpha: initial_alpha,
            beta: initial_beta,
            minigap_mev: minigap,
        };
        qubit.normalize();
        let norm_init_a = qubit.alpha;
        let norm_init_b = qubit.beta;

        let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
        let tgt_alpha = (
            inv_sqrt2 * (norm_init_a.0 + norm_init_b.1),
            inv_sqrt2 * (norm_init_a.1 - norm_init_b.0),
        );
        let tgt_beta = (
            inv_sqrt2 * (norm_init_b.0 + norm_init_a.1),
            inv_sqrt2 * (norm_init_b.1 - norm_init_a.0),
        );

        qubit.apply_braid_23();

        let diabatic_leakage = braiding.diabatic_error(minigap);
        let p_poison = 1.0 - (-self.noise_env.poisoning_rate_hz * braid_duration_s).exp();
        let kb_ev_k = 8.617_333_262e-5;
        let thermal_ev = kb_ev_k * self.noise_env.temperature_k;
        let p_thermal = (-minigap * 1e-3 / thermal_ev.max(1e-6))
            .exp()
            .clamp(0.0, 0.5);

        let ideal_fidelity = qubit.fidelity_with(tgt_alpha, tgt_beta);
        let net_fidelity =
            (ideal_fidelity * (1.0 - diabatic_leakage) * (1.0 - p_poison) * (1.0 - p_thermal))
                .clamp(0.0, 1.0);

        MajoranaBraidResult {
            initial_state: (norm_init_a, norm_init_b),
            final_state: (qubit.alpha, qubit.beta),
            target_state: (tgt_alpha, tgt_beta),
            fidelity: net_fidelity,
            parity_error: p_poison + p_thermal,
            diabatic_leakage,
            poisoning_probability: p_poison,
        }
    }

    /// Verifies the Yang-Baxter braid relation up to global projective phase:
    /// $$B_{12} B_{23} B_{12} \cong B_{23} B_{12} B_{23}$$
    /// Returns the quantum state infidelity $1 - |\langle \psi_A | \psi_B \rangle|^2$.
    pub fn verify_yang_baxter_relation(&self) -> f64 {
        let minigap = 0.20;

        let mut q_a = TopologicalQubitRegister::new_zero(minigap);
        q_a.apply_braid_12();
        q_a.apply_braid_23();
        q_a.apply_braid_12();

        let mut q_b = TopologicalQubitRegister::new_zero(minigap);
        q_b.apply_braid_23();
        q_b.apply_braid_12();
        q_b.apply_braid_23();

        let fid = q_a.fidelity_with(q_b.alpha, q_b.beta);
        (1.0 - fid).abs()
    }

    /// Demonstrates non-Abelian statistics:
    /// Shows that $B_{12} B_{23} \neq B_{23} B_{12}$ on state $|0\rangle$.
    pub fn verify_non_abelian_commutator(&self) -> f64 {
        let minigap = 0.20;

        let mut q1 = TopologicalQubitRegister::new_zero(minigap);
        q1.apply_braid_12();
        q1.apply_braid_23();

        let mut q2 = TopologicalQubitRegister::new_zero(minigap);
        q2.apply_braid_23();
        q2.apply_braid_12();

        let d_alpha = (q1.alpha.0 - q2.alpha.0).powi(2) + (q1.alpha.1 - q2.alpha.1).powi(2);
        let d_beta = (q1.beta.0 - q2.beta.0).powi(2) + (q1.beta.1 - q2.beta.1).powi(2);
        (d_alpha + d_beta).sqrt()
    }
}
