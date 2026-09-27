//! Distributed lossless transmission line modeled via Branin's Method of Characteristics (MoC).

/// Lossless transmission line parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LosslessTransmissionLine {
    /// Characteristic impedance in Ohms ($Z_0$).
    pub z0: f64,
    /// Propagation delay in seconds ($\tau$).
    pub tau: f64,
}

impl LosslessTransmissionLine {
    pub fn new(z0: f64, tau: f64) -> Self {
        Self { z0, tau }
    }

    /// Creates a transmission line from physical line parameters:
    /// inductance per unit length $L'$ (H/m), capacitance per unit length $C'$ (F/m), and length $\ell$ (m).
    pub fn from_distributed(l_per_m: f64, c_per_m: f64, length_m: f64) -> Self {
        let z0 = (l_per_m / c_per_m).sqrt();
        let tau = length_m * (l_per_m * c_per_m).sqrt();
        Self { z0, tau }
    }

    /// Evaluates the voltage reflection coefficient $\Gamma_L$ at a load termination $R_L$:
    /// $$\Gamma_L = \frac{R_L - Z_0}{R_L + Z_0}$$
    #[inline]
    pub fn reflection_coefficient(&self, r_load: f64) -> f64 {
        (r_load - self.z0) / (r_load + self.z0)
    }

    /// Evaluates the voltage transmission coefficient $T_L$ at a load termination $R_L$:
    /// $$T_L = 1 + \Gamma_L = \frac{2 R_L}{R_L + Z_0}$$
    #[inline]
    pub fn transmission_coefficient(&self, r_load: f64) -> f64 {
        (2.0 * r_load) / (r_load + self.z0)
    }
}

/// Historical wave tracking buffer for Branin's Method of Characteristics.
/// Maintains wave variables $W_1(t)$ and $W_2(t)$ to resolve delayed boundary waves
/// without artificial spatial discretization or numerical dispersion.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BraninWaveHistory {
    /// Ordered samples: `(time, w1, w2)`
    samples: Vec<(f64, f64, f64)>,
}

impl BraninWaveHistory {
    pub fn new() -> Self {
        Self {
            samples: Vec::with_capacity(256),
        }
    }

    /// Clears the wave history buffer.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Evaluates delayed waves $(W_2(t - \tau), W_1(t - \tau))$ at simulation time $t$:
    pub fn evaluate_delayed_waves(&self, time: f64, tau: f64) -> (f64, f64) {
        if self.samples.is_empty() {
            return (0.0, 0.0);
        }

        let t_target = time - tau;
        if t_target <= self.samples[0].0 {
            return (self.samples[0].2, self.samples[0].1);
        }

        let last_idx = self.samples.len() - 1;
        if t_target >= self.samples[last_idx].0 {
            return (self.samples[last_idx].2, self.samples[last_idx].1);
        }

        // Binary search for interval [t_i, t_{i+1}] containing t_target
        let mut low = 0;
        let mut high = last_idx;
        while low + 1 < high {
            let mid = (low + high) / 2;
            if self.samples[mid].0 <= t_target {
                low = mid;
            } else {
                high = mid;
            }
        }

        let (t0, w1_0, w2_0) = self.samples[low];
        let (t1, w1_1, w2_1) = self.samples[high];

        let dt = (t1 - t0).max(1e-18);
        let alpha = (t_target - t0) / dt;

        let w1_interp = w1_0 + alpha * (w1_1 - w1_0);
        let w2_interp = w2_0 + alpha * (w2_1 - w2_0);

        // Port 1 receives W2(t - tau), Port 2 receives W1(t - tau)
        (w2_interp, w1_interp)
    }

    /// Records solved port voltages $v_1(t)$ and $v_2(t)$ at the current time step,
    /// updating wave variables $W_1(t)$ and $W_2(t)$:
    /// $$W_1(t) = 2 v_1(t) - W_2(t - \tau)$$
    /// $$W_2(t) = 2 v_2(t) - W_1(t - \tau)$$
    pub fn record_step(&mut self, time: f64, v1: f64, v2: f64, tau: f64) {
        let (w2_delayed, w1_delayed) = self.evaluate_delayed_waves(time, tau);
        let w1_current = 2.0 * v1 - w2_delayed;
        let w2_current = 2.0 * v2 - w1_delayed;

        // Maintain monotonic time sequence
        if let Some(last) = self.samples.last_mut() {
            if (last.0 - time).abs() < 1e-15 {
                *last = (time, w1_current, w2_current);
                return;
            }
        }

        self.samples.push((time, w1_current, w2_current));

        // Memory prune: keep samples within 3*tau of the current time
        let prune_threshold = time - 3.0 * tau;
        if prune_threshold > 0.0 && self.samples.len() > 1000 {
            let prune_count = self
                .samples
                .iter()
                .take_while(|s| s.0 < prune_threshold)
                .count();
            if prune_count > 100 {
                self.samples.drain(0..prune_count.saturating_sub(2));
            }
        }
    }

    /// Returns the equivalent Norton companion parameters for Port 1 and Port 2 at time $t$:
    /// Port 1: Conductance $G_0 = 1/Z_0$, current source $I_{1,eq} = \frac{W_2(t - \tau)}{Z_0}$
    /// Port 2: Conductance $G_0 = 1/Z_0$, current source $I_{2,eq} = \frac{W_1(t - \tau)}{Z_0}$
    pub fn companion_sources(&self, time: f64, z0: f64, tau: f64) -> (f64, f64) {
        let (w2_delayed, w1_delayed) = self.evaluate_delayed_waves(time, tau);
        let g0 = 1.0 / z0;
        let i1_eq = w2_delayed * g0;
        let i2_eq = w1_delayed * g0;
        (i1_eq, i2_eq)
    }
}
