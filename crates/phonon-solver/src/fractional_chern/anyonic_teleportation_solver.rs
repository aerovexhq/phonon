//! Autonomous solver for anyonic quantum state teleportation.
//!
//! # Protocol Steps
//! 1. Alice prepares unknown anyonic qudit state $|\psi\rangle$ in the $d = q$ degenerate ground space.
//! 2. Alice and Bob share topologically entangled anyon pair $|\Phi^+\rangle = \frac{1}{\sqrt{d}} \sum |j, j\rangle$.
//! 3. Alice performs anyonic Bell-state measurement (BSM) via topological anyon braiding/fusion.
//! 4. Alice sends classical measurement outcome $(m, n) \in \mathbb{Z}_d \times \mathbb{Z}_d$ to Bob.
//! 5. Bob applies topological feedforward correction $U_{m, n} = X^{-m} Z^{-n}$ to recover $|\psi\rangle$.
//! 6. Evaluates teleportation fidelity $\mathcal{F} \ge 99\%$.

use phonon_models::fractional_chern::{
    AnyonQudit, AnyonicTeleportationChannel, FractionalChernState, FractionalFilling,
    MoireFlatBand, MoireLatticeParams,
};

/// Result of anyonic state teleportation execution.
#[derive(Debug, Clone, PartialEq)]
pub struct AnyonicTeleportationResult {
    /// Dimension $d = q$ of anyonic qudit.
    pub qudit_dimension: usize,
    /// BSM outcome index $m \in \{0, \dots, d-1\}$.
    pub bsm_m: usize,
    /// BSM outcome index $n \in \{0, \dots, d-1\}$.
    pub bsm_n: usize,
    /// Teleportation state fidelity $\mathcal{F} \in [0, 1]$.
    pub state_fidelity: f64,
    /// Operating dilution refrigerator temperature in Kelvin.
    pub temperature_k: f64,
    /// Topological protection gap in eV.
    pub protection_gap_ev: f64,
    /// Channel length in nanometers.
    pub channel_length_nm: f64,
}

/// Anyonic quantum state teleportation solver.
#[derive(Debug, Clone, PartialEq)]
pub struct AnyonicTeleportationSolver {
    pub channel: AnyonicTeleportationChannel,
}

impl AnyonicTeleportationSolver {
    pub fn new(
        params: MoireLatticeParams,
        filling: FractionalFilling,
        channel_length_nm: f64,
        temperature_k: f64,
    ) -> Self {
        let flat_band = MoireFlatBand::new(params, 1);
        let fci_state = FractionalChernState::new(flat_band, filling);
        let mut channel = AnyonicTeleportationChannel::new(fci_state, channel_length_nm);
        channel.temperature_k = temperature_k;
        Self { channel }
    }

    /// Executes anyonic teleportation of an input qudit state across the moiré channel.
    pub fn execute_teleportation(
        &self,
        input_state: &AnyonQudit,
        bsm_outcome_m: usize,
        bsm_outcome_n: usize,
    ) -> AnyonicTeleportationResult {
        let (_teleported_qudit, fidelity) =
            self.channel
                .teleport(input_state, bsm_outcome_m, bsm_outcome_n);

        AnyonicTeleportationResult {
            qudit_dimension: self.channel.fci_state.ground_state_degeneracy,
            bsm_m: bsm_outcome_m,
            bsm_n: bsm_outcome_n,
            state_fidelity: fidelity,
            temperature_k: self.channel.temperature_k,
            protection_gap_ev: self.channel.fci_state.spectral_gap_ev,
            channel_length_nm: self.channel.channel_length_nm,
        }
    }
}
