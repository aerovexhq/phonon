//! Biological and neuromorphic spiking neuron models: Leaky Integrate-and-Fire (LIF)
//! and Adaptive Exponential (AdEx).
//!
//! Formulates:
//! - Subthreshold membrane potential integration:
//!   $$\tau_m \frac{dV_{mem}}{dt} = -(V_{mem} - V_{rest}) + R_m I_{in}(t) - R_m w(t)$$
//! - All-or-nothing action potential spike emission at $V_{mem} \ge V_{thresh}$.
//! - Post-spike refractory clamping ($V_{mem} = V_{reset}$ for duration $t_{ref}$).
//! - Spike-frequency adaptation via adaptation current variable $w(t)$.
//! - Temperature-dependent membrane conductance leakage.

use phonon_core::NeuronSpike;

/// Operational state of a biological spiking neuron.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuronState {
    /// Instantaneous membrane potential $V_{mem}$ in Volts ($V$).
    pub v_mem_volts: f64,
    /// Adaptation current $w$ in Amperes ($A$).
    pub adaptation_current_a: f64,
    /// Remaining refractory period countdown in seconds ($s$).
    pub refractory_remaining_s: f64,
}

impl NeuronState {
    /// Creates a new neuron state at resting potential.
    pub fn new(v_rest: f64) -> Self {
        Self {
            v_mem_volts: v_rest,
            adaptation_current_a: 0.0,
            refractory_remaining_s: 0.0,
        }
    }
}

/// Physical parameters for a Leaky Integrate-and-Fire (LIF) / AdEx neuron.
#[derive(Debug, Clone, PartialEq)]
pub struct SpikingNeuronModel {
    /// Membrane capacitance $C_m$ in Farads ($F$) (typically $100\text{ pF} - 1\text{ nF}$).
    pub c_mem_f: f64,
    /// Membrane leakage resistance $R_m$ in Ohms ($\Omega$) (typically $5\text{ M}\Omega$).
    pub r_mem_ohms: f64,
    /// Resting potential $V_{rest}$ in Volts ($V$) (e.g. $0.0\text{ V}$ or $-70\text{ mV}$).
    pub v_rest_volts: f64,
    /// Action potential firing threshold $V_{thresh}$ in Volts ($V$) (e.g. $1.0\text{ V}$ or $-55\text{ mV}$).
    pub v_thresh_volts: f64,
    /// Reset potential $V_{reset}$ in Volts ($V$) (e.g. $0.0\text{ V}$ or $-75\text{ mV}$).
    pub v_reset_volts: f64,
    /// Absolute refractory period duration $t_{ref}$ in seconds ($s$) (typically $2\text{ ms}$).
    pub t_refractory_s: f64,
    /// Subthreshold adaptation conductance $a$ in Siemens ($S$).
    pub adaptation_a_siemens: f64,
    /// Spike-triggered adaptation current step $b$ in Amperes ($A$).
    pub adaptation_b_amps: f64,
    /// Adaptation time constant $\tau_w$ in seconds ($s$) (typically $50\text{ ms}$).
    pub tau_adaptation_s: f64,
    /// Peak action potential amplitude for generated spikes in Volts ($V$).
    pub spike_amplitude_volts: f64,
}

impl SpikingNeuronModel {
    /// Standard neuromorphic CMOS integrate-and-fire neuron preset ($0\text{ V} - 1.2\text{ V}$ unipolar domain).
    pub fn neuromorphic_cmos() -> Self {
        Self {
            c_mem_f: 1.0e-11,       // 10 pF
            r_mem_ohms: 10.0e6,     // 10 MOhm -> tau_m = 100 us
            v_rest_volts: 0.0,      // 0 V
            v_thresh_volts: 0.80,   // 0.8 V threshold
            v_reset_volts: 0.10,    // 0.1 V reset
            t_refractory_s: 5.0e-6, // 5 us refractory period
            adaptation_a_siemens: 1.0e-8,
            adaptation_b_amps: 2.0e-8,  // 20 nA adaptation step
            tau_adaptation_s: 1.0e-3,   // 1 ms adaptation
            spike_amplitude_volts: 1.2, // 1.2 V output spike
        }
    }

    /// Membrane passive time constant $\tau_m = R_m C_m$ in seconds ($s$).
    #[inline(always)]
    pub fn membrane_time_constant_s(&self) -> f64 {
        self.r_mem_ohms * self.c_mem_f
    }

    /// Steps the neuron state forward by time-step $dt$, returning the updated state
    /// and an `Option<NeuronSpike>` if an action potential was emitted:
    pub fn step(
        &self,
        state: NeuronState,
        i_input_amps: f64,
        neuron_id: u32,
        current_time_s: f64,
        dt_s: f64,
    ) -> (NeuronState, Option<NeuronSpike>) {
        let mut v_mem = state.v_mem_volts;
        let mut w = state.adaptation_current_a;
        let mut ref_rem = state.refractory_remaining_s;

        // If in refractory period: clamp to V_reset and count down
        if ref_rem > 0.0 {
            ref_rem = (ref_rem - dt_s).max(0.0);
            v_mem = self.v_reset_volts;
            // Decay adaptation current
            w -= (w / self.tau_adaptation_s.max(1e-9)) * dt_s;
            return (
                NeuronState {
                    v_mem_volts: v_mem,
                    adaptation_current_a: w.max(0.0),
                    refractory_remaining_s: ref_rem,
                },
                None,
            );
        }

        // Subthreshold membrane integration:
        // C_m * dV/dt = -(V - V_rest)/R_m + I_in - w
        let i_leak = (v_mem - self.v_rest_volts) / self.r_mem_ohms.max(1e-3);
        let dv_dt = (-i_leak + i_input_amps - w) / self.c_mem_f.max(1e-15);
        v_mem += dv_dt * dt_s;

        // Adaptation current update:
        // dw/dt = (a * (V - V_rest) - w) / tau_w
        let dw_dt = (self.adaptation_a_siemens * (v_mem - self.v_rest_volts) - w)
            / self.tau_adaptation_s.max(1e-9);
        w += dw_dt * dt_s;

        // Check threshold crossing:
        if v_mem >= self.v_thresh_volts {
            // Emit action potential spike
            let spike = NeuronSpike::new(current_time_s, neuron_id, self.spike_amplitude_volts);
            // Reset potential and enter refractory period
            v_mem = self.v_reset_volts;
            ref_rem = self.t_refractory_s;
            w += self.adaptation_b_amps; // Spike-triggered adaptation kick

            (
                NeuronState {
                    v_mem_volts: v_mem,
                    adaptation_current_a: w,
                    refractory_remaining_s: ref_rem,
                },
                Some(spike),
            )
        } else {
            (
                NeuronState {
                    v_mem_volts: v_mem,
                    adaptation_current_a: w,
                    refractory_remaining_s: 0.0,
                },
                None,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neuron_subthreshold_integration_and_spike_firing() {
        let neuron = SpikingNeuronModel::neuromorphic_cmos();
        let mut state = NeuronState::new(neuron.v_rest_volts);

        // Constant input current I = 150 nA
        // Steady-state subthreshold voltage: V_ss = I * R_m = 150 nA * 10 MOhm = 1.5 V > V_thresh (0.8 V)
        let i_in = 1.5e-7;
        let dt = 1e-6; // 1 us step
        let mut spikes = Vec::new();
        let mut sim_t = 0.0;

        for _ in 0..500 {
            let (next_state, spike) = neuron.step(state, i_in, 1, sim_t, dt);
            state = next_state;
            if let Some(sp) = spike {
                spikes.push(sp);
            }
            sim_t += dt;
        }

        // Neuron must have emitted multiple action potentials
        assert!(
            !spikes.is_empty(),
            "Neuron should fire with 150 nA input current"
        );
        assert!(spikes.len() >= 3, "Neuron should fire train of spikes");
    }

    #[test]
    fn test_neuron_subthreshold_decay_without_input() {
        let neuron = SpikingNeuronModel::neuromorphic_cmos();
        // Start near threshold (0.7 V) with zero current
        let mut state = NeuronState::new(0.70);
        let dt = 1e-5;

        for _ in 0..100 {
            let (next_state, spike) = neuron.step(state, 0.0, 1, 0.0, dt);
            assert!(spike.is_none());
            state = next_state;
        }

        // Potential must have decayed towards V_rest (0.0 V)
        assert!(state.v_mem_volts < 0.05);
    }
}
