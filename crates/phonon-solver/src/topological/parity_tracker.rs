//! Fermion parity tracking and environmental quasiparticle poisoning solver.
//!
//! Models local pair parities $(P_{12}, P_{34})$, global parity $P_{tot} = P_{12} P_{34}$,
//! and stochastic Quasiparticle Poisoning (QPP) jump processes.
//! Computes topological qubit memory lifetime $T_1^{topo}$ and phase coherence $T_2^{topo}$.

use phonon_models::topological::TopologicalQubit;

/// Report of fermion parity tracking and poisoning events.
#[derive(Debug, Clone, PartialEq)]
pub struct ParityTrackingReport {
    /// Initial total parity $P_{tot}(0)$.
    pub initial_parity: i8,
    /// Final total parity $P_{tot}(t)$.
    pub final_parity: i8,
    /// Number of stochastic quasiparticle poisoning events observed.
    pub poisoning_event_count: u64,
    /// Elapsed physical time in seconds.
    pub elapsed_time_s: f64,
    /// Parity retention fidelity (fraction of time spent in the correct parity subspace).
    pub parity_retention_fidelity: f64,
    /// Quasiparticle poisoning lifetime $T_1^{topo}$ in seconds.
    pub t1_topo_s: f64,
    /// Topological dephasing lifetime $T_2^{topo}$ in seconds.
    pub t2_topo_s: f64,
}

/// Solver simulating fermion parity conservation and quasiparticle poisoning dynamics.
#[derive(Debug, Clone)]
pub struct FermionParitySolver {
    /// Quasiparticle poisoning rate $\Gamma_{qp}$ in $\text{s}^{-1}$ ($\text{Hz}$).
    /// Typically $\approx 1 - 10\text{ Hz}$ at $T \le 20\text{ mK}$ with IR shielding.
    pub poisoning_rate_hz: f64,
    /// Current pair parity $P_{12} \in \{+1, -1\}$.
    pub p12: i8,
    /// Current pair parity $P_{34} \in \{+1, -1\}$.
    pub p34: i8,
    /// Total cumulative poisoning events.
    pub total_poisoning_events: u64,
}

impl Default for FermionParitySolver {
    fn default() -> Self {
        Self {
            poisoning_rate_hz: 5.0, // 5 Hz poisoning rate (200 ms parity lifetime)
            p12: 1,                 // Even
            p34: 1,                 // Even
            total_poisoning_events: 0,
        }
    }
}

impl FermionParitySolver {
    /// Creates a solver with a specified quasiparticle poisoning rate.
    pub fn new(poisoning_rate_hz: f64) -> Self {
        Self {
            poisoning_rate_hz: poisoning_rate_hz.max(1e-6),
            p12: 1,
            p34: 1,
            total_poisoning_events: 0,
        }
    }

    /// Evaluates total parity $P_{tot} = P_{12} P_{34}$.
    #[inline]
    pub fn total_parity(&self) -> i8 {
        self.p12 * self.p34
    }

    /// Characteristic topological $T_1$ relaxation time in seconds.
    #[inline]
    pub fn t1_topo_s(&self) -> f64 {
        1.0 / self.poisoning_rate_hz
    }

    /// Characteristic topological $T_2$ dephasing time in seconds ($T_2 \approx 2 T_1$).
    #[inline]
    pub fn t2_topo_s(&self) -> f64 {
        2.0 * self.t1_topo_s()
    }

    /// Simulates time evolution over interval `dt_s` using a pseudo-random seed.
    /// Updates parities and applies parity flip if a poisoning event occurs.
    pub fn step_evolution(
        &mut self,
        dt_s: f64,
        qubit: &mut TopologicalQubit,
        pseudo_random_draw: f64,
    ) -> bool {
        let p_poison = 1.0 - (-self.poisoning_rate_hz * dt_s).exp();

        if pseudo_random_draw < p_poison {
            // A quasiparticle poisoning event occurs: flips parity of the first pair
            self.p12 = -self.p12;
            self.total_poisoning_events += 1;
            qubit.total_parity = self.total_parity();
            true
        } else {
            false
        }
    }

    /// Simulates parity retention over $N$ consecutive gate cycles of duration `cycle_duration_s`.
    pub fn track_cycles(
        &mut self,
        num_cycles: usize,
        cycle_duration_s: f64,
        qubit: &mut TopologicalQubit,
    ) -> ParityTrackingReport {
        let initial_parity = self.total_parity();
        let total_time = num_cycles as f64 * cycle_duration_s;
        let mut events = 0;

        for i in 0..num_cycles {
            // Deterministic LCG pseudo-random draw for reproducible evaluation
            let seed = (i as u64)
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let draw = (seed >> 33) as f64 / (1u64 << 31) as f64;

            if self.step_evolution(cycle_duration_s, qubit, draw) {
                events += 1;
            }
        }

        let p_retention = if events == 0 {
            1.0
        } else {
            1.0 - (events as f64 / num_cycles as f64)
        };

        ParityTrackingReport {
            initial_parity,
            final_parity: self.total_parity(),
            poisoning_event_count: events,
            elapsed_time_s: total_time,
            parity_retention_fidelity: p_retention.max(0.0),
            t1_topo_s: self.t1_topo_s(),
            t2_topo_s: self.t2_topo_s(),
        }
    }
}
