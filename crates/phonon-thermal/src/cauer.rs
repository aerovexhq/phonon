//! Physically meaningful Cauer ladder thermal network modeling discrete layer stacks
//! (e.g. Junction -> Die -> TIM -> Case -> Heatsink -> Ambient).

/// Physical layer in a Cauer thermal ladder network.
#[derive(Debug, Clone, PartialEq)]
pub struct CauerStage {
    pub name: String,
    /// Thermal resistance to the next stage in $\text{K} / \text{W}$.
    pub r_th: f64,
    /// Thermal capacitance of this layer in $\text{J} / \text{K}$.
    pub c_th: f64,
}

/// A multi-stage Cauer thermal ladder network.
///
/// In a Cauer network, each internal node has a direct physical meaning:
/// Node 0 is the primary semiconductor junction where heat is dissipated.
/// Intermediate nodes represent internal physical layers (silicon, die attach, case).
/// The final stage exhausts heat to the ambient thermal environment.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CauerNetwork {
    pub stages: Vec<CauerStage>,
}

impl CauerNetwork {
    /// Creates an empty Cauer ladder network.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Adds a physical layer stage to the Cauer ladder.
    pub fn add_stage(&mut self, name: &str, r_th: f64, c_th: f64) {
        assert!(r_th > 0.0, "Thermal resistance must be strictly positive");
        assert!(c_th >= 0.0, "Thermal capacitance must be non-negative");
        self.stages.push(CauerStage {
            name: name.to_string(),
            r_th,
            c_th,
        });
    }

    /// Number of physical thermal nodes in the ladder.
    pub fn num_nodes(&self) -> usize {
        self.stages.len()
    }

    /// Total DC thermal resistance from junction (node 0) to ambient:
    /// $R_{th,ja} = \sum_{i=0}^{N-1} R_{th, i}$.
    pub fn total_thermal_resistance(&self) -> f64 {
        self.stages.iter().map(|s| s.r_th).sum()
    }

    /// Evaluates driving-point thermal impedance $Z_{th}(s)$ at complex Laplace frequency $s$:
    /// Uses continued fraction expansion:
    /// $Z_{th}(s) = \frac{1}{s C_0 + \frac{1}{R_0 + \frac{1}{s C_1 + \dots}}}$
    pub fn driving_point_impedance(&self, s: f64) -> f64 {
        if self.stages.is_empty() {
            return 0.0;
        }

        let n = self.stages.len();
        // Start from the ambient side
        let mut z_current = self.stages[n - 1].r_th;

        for i in (0..n).rev() {
            let c_i = self.stages[i].c_th;
            let y_i = if c_i > 0.0 { s * c_i } else { 0.0 } + 1.0 / z_current.max(1e-12);
            z_current = 1.0 / y_i;
            if i > 0 {
                z_current += self.stages[i - 1].r_th;
            }
        }

        z_current
    }

    /// Solves for the steady-state thermal temperature profile given power vector $\mathbf{P}_{diss}$
    /// injected at each node and ambient temperature $T_{amb}$ in Kelvin.
    ///
    /// Returns node temperatures $[T_0, T_1, \dots, T_{N-1}]$.
    pub fn solve_steady_state(&self, p_in: &[f64], ambient_k: f64) -> Vec<f64> {
        let n = self.stages.len();
        if n == 0 {
            return Vec::new();
        }

        let mut p_vec = vec![0.0; n];
        for (i, &p) in p_in.iter().enumerate().take(n) {
            p_vec[i] = p;
        }

        // Backward cumulative sum of heat flux:
        // Heat flowing through resistor i is the sum of power dissipated at all nodes <= i
        let mut heat_flux_through_r = vec![0.0; n];
        let mut cum_p = 0.0;
        for i in 0..n {
            cum_p += p_vec[i];
            heat_flux_through_r[i] = cum_p;
        }

        // Compute temperatures by stepping back from ambient:
        // T_{N-1} = T_amb + heat_flux[N-1] * R_{th, N-1}
        let mut temps = vec![ambient_k; n];
        let mut t_prev = ambient_k;
        for i in (0..n).rev() {
            t_prev += heat_flux_through_r[i] * self.stages[i].r_th;
            temps[i] = t_prev;
        }

        temps
    }

    /// Advances the thermal ladder network by a single time step $\Delta t$
    /// using backward Euler integration:
    ///
    /// $(\mathbf{C}_{th} / \Delta t + \mathbf{G}_{th}) \mathbf{T}^{n+1} = \mathbf{P}_{in} + (\mathbf{C}_{th} / \Delta t) \mathbf{T}^n + \mathbf{G}_{amb} T_{amb}$
    pub fn step_transient_backward_euler(
        &self,
        current_temps: &[f64],
        p_in: &[f64],
        ambient_k: f64,
        dt: f64,
    ) -> Vec<f64> {
        let n = self.stages.len();
        if n == 0 {
            return Vec::new();
        }

        // Tri-diagonal system solver:
        // diag[i] * T_i + lower[i] * T_{i-1} + upper[i] * T_{i+1} = rhs[i]
        let mut diag = vec![0.0; n];
        let mut lower = vec![0.0; n];
        let mut upper = vec![0.0; n];
        let mut rhs = vec![0.0; n];

        for i in 0..n {
            let c_i = self.stages[i].c_th;
            let g_forward = 1.0 / self.stages[i].r_th;
            let g_back = if i > 0 {
                1.0 / self.stages[i - 1].r_th
            } else {
                0.0
            };

            let p_ext = if i < p_in.len() { p_in[i] } else { 0.0 };
            let t_old = if i < current_temps.len() {
                current_temps[i]
            } else {
                ambient_k
            };

            diag[i] = (c_i / dt) + g_forward + g_back;
            rhs[i] = p_ext + (c_i / dt) * t_old;

            if i > 0 {
                lower[i] = -g_back;
            }
            if i + 1 < n {
                upper[i] = -g_forward;
            } else {
                // Connection to ambient at last stage
                rhs[i] += g_forward * ambient_k;
            }
        }

        // Thomas algorithm for tridiagonal systems
        let mut c_prime = vec![0.0; n];
        let mut d_prime = vec![0.0; n];

        c_prime[0] = upper[0] / diag[0];
        d_prime[0] = rhs[0] / diag[0];

        for i in 1..n {
            let denom = diag[i] - lower[i] * c_prime[i - 1];
            if i + 1 < n {
                c_prime[i] = upper[i] / denom;
            }
            d_prime[i] = (rhs[i] - lower[i] * d_prime[i - 1]) / denom;
        }

        let mut next_temps = vec![0.0; n];
        next_temps[n - 1] = d_prime[n - 1];
        for i in (0..n - 1).rev() {
            next_temps[i] = d_prime[i] - c_prime[i] * next_temps[i + 1];
        }

        next_temps
    }
}
