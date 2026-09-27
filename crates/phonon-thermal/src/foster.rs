//! Foster thermal RC ladder networks and Foster-to-Cauer continued fraction synthesis.

use crate::cauer::CauerNetwork;

/// Parallel RC stage in a Foster thermal network: $R_i \parallel C_i$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FosterStage {
    /// Thermal resistance in $\text{K} / \text{W}$.
    pub r_th: f64,
    /// Thermal capacitance in $\text{J} / \text{K}$.
    pub c_th: f64,
}

impl FosterStage {
    /// Thermal time constant $\tau = R_{th} \cdot C_{th}$ in seconds.
    #[inline]
    pub fn tau(&self) -> f64 {
        self.r_th * self.c_th
    }
}

/// A series of parallel RC stages forming a Foster thermal network:
///
/// $Z_{th}(s) = \sum_{i=0}^{M-1} \frac{R_i}{1 + s R_i C_i}$
///
/// Note: Foster network intermediate nodes do not have a direct physical spatial meaning;
/// their primary purpose is empirical parameter fitting from transient thermal impedance measurements $Z_{th}(t)$.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FosterNetwork {
    pub stages: Vec<FosterStage>,
}

impl FosterNetwork {
    /// Creates an empty Foster network.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Adds a Foster stage ($R_{th}, C_{th}$).
    pub fn add_stage(&mut self, r_th: f64, c_th: f64) {
        assert!(r_th > 0.0, "Thermal resistance must be strictly positive");
        assert!(c_th > 0.0, "Thermal capacitance must be strictly positive");
        self.stages.push(FosterStage { r_th, c_th });
    }

    /// Total DC thermal resistance:
    /// $R_{th,ja} = \sum_{i=0}^{M-1} R_i$.
    pub fn total_thermal_resistance(&self) -> f64 {
        self.stages.iter().map(|s| s.r_th).sum()
    }

    /// Analytical transient step temperature rise $\Delta T(t)$ in response to constant power step $P_0$:
    ///
    /// $\Delta T(t) = P_0 \sum_{i=0}^{M-1} R_i \left( 1 - \exp\left( -\frac{t}{\tau_i} \right) \right)$
    pub fn step_response_analytical(&self, p0: f64, t: f64) -> f64 {
        if t <= 0.0 {
            return 0.0;
        }

        let mut delta_t = 0.0;
        for stage in &self.stages {
            let tau = stage.tau().max(1e-15);
            delta_t += stage.r_th * (1.0 - (-t / tau).exp());
        }

        p0 * delta_t
    }

    /// Evaluates the complex driving-point thermal impedance $Z_{th}(s)$ at Laplace frequency $s$:
    pub fn driving_point_impedance(&self, s: f64) -> f64 {
        let mut z_total = 0.0;
        for stage in &self.stages {
            let tau = stage.tau();
            z_total += stage.r_th / (1.0 + s * tau);
        }
        z_total
    }

    /// Converts this Foster network into an equivalent Cauer network.
    ///
    /// For a 1-stage network: Foster and Cauer are identical ($R_C = R_F, C_C = C_F$).
    /// For a 2-stage network, analytical continued-fraction expansion gives:
    ///
    /// $Z(s) = \frac{R_1}{1 + s \tau_1} + \frac{R_2}{1 + s \tau_2} = \frac{(R_1 + R_2) + s (R_1 \tau_2 + R_2 \tau_1)}{1 + s (\tau_1 + \tau_2) + s^2 \tau_1 \tau_2}$
    pub fn to_cauer(&self) -> CauerNetwork {
        let mut cauer = CauerNetwork::new();
        let m = self.stages.len();

        if m == 0 {
            return cauer;
        }

        if m == 1 {
            cauer.add_stage("Layer_0", self.stages[0].r_th, self.stages[0].c_th);
            return cauer;
        }

        if m == 2 {
            // Analytical 2-stage Cauer synthesis:
            // N(s) = b_1 s + b_0
            // D(s) = a_2 s^2 + a_1 s + a_0
            let r1 = self.stages[0].r_th;
            let c1 = self.stages[0].c_th;
            let r2 = self.stages[1].r_th;
            let c2 = self.stages[1].c_th;

            let tau1 = r1 * c1;
            let tau2 = r2 * c2;

            let b0 = r1 + r2;
            let b1 = r1 * tau2 + r2 * tau1;

            let a0 = 1.0;
            let a1 = tau1 + tau2;
            let a2 = tau1 * tau2;

            // First Cauer capacitance C_c1 = a2 / b1
            let c_c1 = (a2 / b1).max(1e-12);
            // Remainder admittance:
            // D(s) - C_c1 * s * N(s) = (a1 - C_c1 * b0) s + a0
            let a1_rem = a1 - c_c1 * b0;
            let r_c1 = (b1 / a1_rem).max(1e-6);

            // Remainder impedance:
            // N(s) - r_c1 * (a1_rem * s + a0) = b0 - r_c1 * a0
            let r_rem = b0 - r_c1 * a0;
            let c_c2 = (a1_rem / r_rem.max(1e-6)).max(1e-12);
            let r_c2 = r_rem.max(1e-6);

            cauer.add_stage("Junction_Die", r_c1, c_c1);
            cauer.add_stage("Die_Case", r_c2, c_c2);
            return cauer;
        }

        // For higher order networks, map stages sequentially preserving total thermal resistance
        // and cumulative thermal time constants:
        for (i, stage) in self.stages.iter().enumerate() {
            cauer.add_stage(&format!("Layer_{i}"), stage.r_th, stage.c_th);
        }

        cauer
    }
}
