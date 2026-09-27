//! Differential tunneling conductance and Zero-Bias Conductance Peak (ZBCP) spectroscopy.
//!
//! Models normal-lead to topological-superconductor tunneling interfaces
//! using resonant Andreev reflection and Blonder-Tinkham-Klapwijk (BTK) formalisms.
//! Verifies the quantized zero-bias conductance peak $G(0) = \frac{2e^2}{h} \approx 77.48\,\mu\text{S}$.

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, PLANCK_CONSTANT};

/// Quantum of conductance $G_0 = \frac{2e^2}{h} \approx 77.480917\,\mu\text{S}$.
pub const QUANTUM_CONDUCTANCE: f64 =
    (2.0 * ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / PLANCK_CONSTANT;

/// Tunneling conductance model for Majorana nanowire endpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct TunnelingConductanceModel {
    /// Normal metal lead tunnel coupling rate $\Gamma$ in meV.
    pub tunnel_coupling_mev: f64,
    /// Inelastic scattering / dissipation rate $\Gamma_{in}$ in meV.
    pub dissipation_mev: f64,
    /// Majorana mode energy offset $E_M$ in meV (0 for uncoupled Majoranas).
    pub majorana_energy_mev: f64,
    /// Temperature $T$ in Kelvin.
    pub temperature_k: f64,
    /// Normal-state background conductance $G_N$ in units of $2e^2/h$.
    pub background_gn: f64,
}

impl Default for TunnelingConductanceModel {
    fn default() -> Self {
        Self {
            tunnel_coupling_mev: 0.05, // 50 ueV coupling
            dissipation_mev: 0.001,    // 1 ueV dissipation
            majorana_energy_mev: 0.0,  // Exact zero-energy mode
            temperature_k: 0.02,       // 20 mK (dilution fridge mixing chamber)
            background_gn: 0.05,
        }
    }
}

impl TunnelingConductanceModel {
    /// Resonant Andreev reflection coefficient $R_A(E)$ for incident electron energy $E$ (meV).
    pub fn andreev_reflection(&self, energy_mev: f64) -> f64 {
        let gamma = self.tunnel_coupling_mev;
        let gamma_tot = gamma + self.dissipation_mev;
        let delta_e = energy_mev - self.majorana_energy_mev;

        (gamma * gamma) / (delta_e * delta_e + gamma_tot * gamma_tot)
    }

    /// Normal reflection coefficient $R_N(E) = 1 - R_A(E)$.
    pub fn normal_reflection(&self, energy_mev: f64) -> f64 {
        let ra = self.andreev_reflection(energy_mev);
        (1.0 - ra).max(0.0)
    }

    /// Differential conductance $G(V) = \frac{dI}{dV}$ in Siemens (S) at bias voltage $V$ (Volts).
    pub fn differential_conductance(&self, bias_voltage_v: f64) -> f64 {
        let ev_bias_mev = (bias_voltage_v * ELEMENTARY_CHARGE) / (ELEMENTARY_CHARGE * 1e-3);
        let kb_t_mev = (BOLTZMANN_CONSTANT * self.temperature_k / ELEMENTARY_CHARGE) * 1e3;

        // If T is extremely low (T < 1 mK), use the zero-temperature analytical limit
        if kb_t_mev < 1e-4 {
            let ra = self.andreev_reflection(ev_bias_mev);
            return QUANTUM_CONDUCTANCE * (ra + self.background_gn);
        }

        // Numerical integration of Fermi-Dirac derivative convolution:
        // G(V) = G0 * \int dE (-df/dE) [R_A(E) + G_N]
        let num_points = 201;
        let window = 6.0 * kb_t_mev;
        let de = (2.0 * window) / ((num_points - 1) as f64);
        let mut g_sum = 0.0;

        for i in 0..num_points {
            let e = (ev_bias_mev - window) + (i as f64) * de;
            let arg = (e - ev_bias_mev) / kb_t_mev;
            // -df/dE = 1 / (4 k_B T cosh^2(arg / 2))
            let cosh_half = (arg / 2.0).cosh();
            let thermal_kernel = 1.0 / (4.0 * kb_t_mev * cosh_half * cosh_half);

            let ra = self.andreev_reflection(e);
            g_sum += thermal_kernel * (ra + self.background_gn) * de;
        }

        QUANTUM_CONDUCTANCE * g_sum
    }

    /// Zero-bias conductance $G(V=0)$ in Siemens.
    #[inline]
    pub fn zero_bias_conductance(&self) -> f64 {
        self.differential_conductance(0.0)
    }

    /// Evaluates conductance spectrum across a voltage sweep range.
    /// Returns vector of (bias_voltage_v, conductance_siemens).
    pub fn sweep_bias(&self, v_min: f64, v_max: f64, steps: usize) -> Vec<(f64, f64)> {
        assert!(steps >= 2, "Steps must be at least 2");
        let dv = (v_max - v_min) / ((steps - 1) as f64);
        (0..steps)
            .map(|i| {
                let v = v_min + (i as f64) * dv;
                (v, self.differential_conductance(v))
            })
            .collect()
    }
}
