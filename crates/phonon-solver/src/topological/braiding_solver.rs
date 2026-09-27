//! Majorana non-Abelian adiabatic braiding solver in T-junction networks.
//!
//! Simulates real-space adiabatic exchange of Majorana Zero Modes in T-junctions
//! via electrostatic gate potential steering. Evaluates non-Abelian holonomy,
//! adiabaticity metrics, and Landau-Zener above-gap excitation probabilities.

use phonon_core::H_BAR;
use phonon_models::topological::{ArmId, TJunctionNanowireNetwork, TopologicalQubit};

/// Result of an adiabatic braiding step.
#[derive(Debug, Clone, PartialEq)]
pub struct BraidingStepResult {
    /// Duration of the braiding operation in seconds.
    pub duration_s: f64,
    /// Minimum bulk gap preserved during the trajectory in meV.
    pub min_bulk_gap_mev: f64,
    /// Maximum adiabaticity parameter $\eta_{ad} = \frac{\hbar |\dot{\mu}|}{\Delta_{top}^2}$.
    pub max_adiabaticity_param: f64,
    /// Landau-Zener transition probability to excited above-gap quasiparticles $\mathcal{P}_{LZ}$.
    pub landau_zener_probability: f64,
    /// Quantum gate fidelity after the braiding operation.
    pub gate_fidelity: f64,
    /// Initial arm locations of the Majoranas: (MZM1, MZM2).
    pub initial_locations: ((ArmId, f64), (ArmId, f64)),
    /// Final arm locations of the Majoranas: (MZM1, MZM2).
    pub final_locations: ((ArmId, f64), (ArmId, f64)),
}

/// Solver simulating adiabatic non-Abelian braiding in T-junction networks.
#[derive(Debug, Clone)]
pub struct MajoranaBraidingSolver {
    pub network: TJunctionNanowireNetwork,
    /// Total duration for a full braiding exchange cycle in seconds.
    /// Typically $\approx 1 - 10\text{ ns}$.
    pub braid_duration_s: f64,
    /// Number of discrete time steps for the adiabatic trajectory.
    pub trajectory_steps: usize,
}

impl Default for MajoranaBraidingSolver {
    fn default() -> Self {
        Self {
            network: TJunctionNanowireNetwork::default(),
            braid_duration_s: 2.0e-9, // 2 ns exchange time
            trajectory_steps: 100,
        }
    }
}

impl MajoranaBraidingSolver {
    /// Creates a solver with a customized network and duration.
    pub fn new(network: TJunctionNanowireNetwork, duration_s: f64) -> Self {
        Self {
            network,
            braid_duration_s: duration_s,
            trajectory_steps: 100,
        }
    }

    /// Executes the 3-stage adiabatic exchange protocol exchanging MZM1 and MZM2.
    ///
    /// The 3 stages:
    /// 1. MZM2 (initially in Right arm) is moved into the Stem arm:
    ///    - Right arm depleted to trivial, Stem arm rendered topological.
    /// 2. MZM1 (initially in Left arm) is moved past the intersection into the Right arm:
    ///    - Left arm depleted to trivial, Right arm rendered topological.
    /// 3. MZM2 (in Stem arm) is moved up into the Left arm:
    ///    - Stem arm depleted to trivial, Left arm rendered topological.
    ///
    /// After this cycle, MZM1 and MZM2 have exchanged places in real space,
    /// applying the transformation $\gamma_1 \to \gamma_2, \gamma_2 \to -\gamma_1$.
    pub fn execute_braid_exchange(&mut self, qubit: &mut TopologicalQubit) -> BraidingStepResult {
        let initial_locs = (self.network.mzm1_location, self.network.mzm2_location);

        let _dt = self.braid_duration_s / (3.0 * self.trajectory_steps as f64);
        let mut min_gap = f64::MAX;
        let mut max_eta = 0.0;

        let delta_top_mev = self.network.base_params.pairing_delta_mev;
        let delta_joules = delta_top_mev * 1e-3 * phonon_core::ELEMENTARY_CHARGE;

        // Stage 1: Move MZM2 from Right (0.0) to Stem (0.0)
        for s in 0..=self.trajectory_steps {
            let frac = s as f64 / self.trajectory_steps as f64;
            // Ramp Right from 0 to 3 meV, Ramp Stem from 3 to 0 meV
            let mu_r = 3.0 * frac;
            let mu_s = 3.0 * (1.0 - frac);
            self.network.set_gates(0.0, mu_r, mu_s);

            let gap = self.network.network_minigap_mev();
            if gap < min_gap {
                min_gap = gap;
            }

            // d(mu)/dt estimate in meV / s -> Joules / s
            let dmu_dt_joules =
                (3.0 * 1e-3 * phonon_core::ELEMENTARY_CHARGE) / (self.braid_duration_s / 3.0);
            let eta = (H_BAR * dmu_dt_joules) / (delta_joules * delta_joules);
            if eta > max_eta {
                max_eta = eta;
            }
        }
        self.network
            .set_mzm_locations(self.network.mzm1_location, (ArmId::Stem, 0.0));

        // Stage 2: Move MZM1 from Left (0.0) to Right (0.0)
        for s in 0..=self.trajectory_steps {
            let frac = s as f64 / self.trajectory_steps as f64;
            // Ramp Left from 0 to 3 meV, Ramp Right from 3 to 0 meV
            let mu_l = 3.0 * frac;
            let mu_r = 3.0 * (1.0 - frac);
            self.network.set_gates(mu_l, mu_r, 0.0);

            let gap = self.network.network_minigap_mev();
            if gap < min_gap {
                min_gap = gap;
            }
        }
        self.network
            .set_mzm_locations((ArmId::Right, 0.0), self.network.mzm2_location);

        // Stage 3: Move MZM2 from Stem (0.0) to Left (0.0)
        for s in 0..=self.trajectory_steps {
            let frac = s as f64 / self.trajectory_steps as f64;
            // Ramp Stem from 0 to 3 meV, Ramp Left from 3 to 0 meV
            let mu_s = 3.0 * frac;
            let mu_l = 3.0 * (1.0 - frac);
            self.network.set_gates(mu_l, 0.0, mu_s);

            let gap = self.network.network_minigap_mev();
            if gap < min_gap {
                min_gap = gap;
            }
        }
        self.network
            .set_mzm_locations(self.network.mzm1_location, (ArmId::Left, 0.0));

        // Swap tracked identity: MZM1 is now in Right, MZM2 is in Left
        let final_locs = (self.network.mzm1_location, self.network.mzm2_location);

        // Landau-Zener excitation probability: P_LZ = exp(-2*pi * Delta^2 / (hbar * |dE/dt|))
        let exponent = (2.0 * std::f64::consts::PI) / max_eta.max(1e-12);
        let p_lz = (-exponent).exp().min(1.0);

        // Apply topological braid gate transformation to the qubit state
        qubit.apply_braid_12();

        // Gate fidelity is limited only by Landau-Zener adiabatic leakage
        let fidelity = (1.0 - p_lz).clamp(0.0, 1.0);

        BraidingStepResult {
            duration_s: self.braid_duration_s,
            min_bulk_gap_mev: min_gap,
            max_adiabaticity_param: max_eta,
            landau_zener_probability: p_lz,
            gate_fidelity: fidelity,
            initial_locations: initial_locs,
            final_locations: final_locs,
        }
    }

    /// Executes a Clifford Phase Gate $S = \text{diag}(1, i)$ via braid $B_{12}$.
    pub fn execute_phase_gate(&mut self, qubit: &mut TopologicalQubit) -> BraidingStepResult {
        self.execute_braid_exchange(qubit)
    }

    /// Executes a Clifford Hadamard Gate $H$ on the topological qubit.
    pub fn execute_hadamard_gate(&mut self, qubit: &mut TopologicalQubit) -> BraidingStepResult {
        // H = B_12 * B_23 * B_12
        let res1 = self.execute_braid_exchange(qubit);
        qubit.apply_braid_23();
        qubit.apply_braid_12();

        BraidingStepResult {
            duration_s: res1.duration_s * 3.0,
            min_bulk_gap_mev: res1.min_bulk_gap_mev,
            max_adiabaticity_param: res1.max_adiabaticity_param,
            landau_zener_probability: res1.landau_zener_probability * 3.0,
            gate_fidelity: (1.0 - res1.landau_zener_probability * 3.0).clamp(0.0, 1.0),
            initial_locations: res1.initial_locations,
            final_locations: res1.final_locations,
        }
    }
}
