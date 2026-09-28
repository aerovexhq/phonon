//! Transmon superconducting qubit physical models: nonlinear Josephson inductance,
//! charge-phase Hamiltonian, anharmonicity, and charge dispersion suppression.

use std::f64::consts::PI;

/// Physical parameters and operating regime of a superconducting transmon qubit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransmonParams {
    /// Josephson coupling energy $E_J / h$ in GHz.
    pub ej_ghz: f64,
    /// Charging energy $E_C / h$ in GHz ($E_C = e^2 / (2 C_\Sigma)$).
    pub ec_ghz: f64,
    /// Normalized gate offset charge $n_g = C_g V_g / (2e) \in [-0.5, 0.5]$.
    pub offset_charge_ng: f64,
    /// Critical current $I_c = 2\pi E_J / \Phi_0$ in nanoamperes (nA).
    pub critical_current_na: f64,
    /// Total shunt capacitance $C_\Sigma$ in femtofarads (fF).
    pub total_capacitance_ff: f64,
    /// Josephson junction zero-bias inductance $L_J = \Phi_0 / (2\pi I_c)$ in nanohenries (nH).
    pub josephson_inductance_nh: f64,
}

impl TransmonParams {
    /// Constructs a transmon parameter set from $E_J / h$ and $E_C / h$ in GHz.
    pub fn new(ej_ghz: f64, ec_ghz: f64, offset_charge_ng: f64) -> Self {
        let h = 6.626_070_15e-34;
        let e = 1.602_176_634e-19;
        let phi_0 = h / (2.0 * e); // Magnetic flux quantum ~ 2.0678e-15 Wb

        let ej_joules = ej_ghz * 1.0e9 * h;
        let ec_joules = ec_ghz * 1.0e9 * h;

        let critical_current = 2.0 * PI * ej_joules / phi_0;
        let c_sigma = e.powi(2) / (2.0 * ec_joules);
        let l_j = phi_0 / (2.0 * PI * critical_current.max(1e-12));

        Self {
            ej_ghz,
            ec_ghz,
            offset_charge_ng,
            critical_current_na: critical_current * 1.0e9,
            total_capacitance_ff: c_sigma * 1.0e15,
            josephson_inductance_nh: l_j * 1.0e9,
        }
    }

    /// Standard 5 GHz transmon baseline with $E_J / E_C \approx 60$.
    pub fn standard_5ghz() -> Self {
        Self::new(15.0, 0.25, 0.0)
    }

    /// Ratio of Josephson coupling energy to charging energy $E_J / E_C$.
    pub fn ej_over_ec(&self) -> f64 {
        self.ej_ghz / self.ec_ghz.max(1e-6)
    }

    /// Primary qubit transition frequency $\omega_{01} / (2\pi)$ in GHz via semi-classical transmon expansion:
    /// $$\omega_{01} \approx \sqrt{8 E_C E_J} - E_C$$
    pub fn omega_01_ghz(&self) -> f64 {
        (8.0 * self.ec_ghz * self.ej_ghz).sqrt() - self.ec_ghz
    }

    /// Higher transition frequency $\omega_{12} / (2\pi)$ in GHz:
    /// $$\omega_{12} \approx \sqrt{8 E_C E_J} - 2 E_C$$
    pub fn omega_12_ghz(&self) -> f64 {
        (8.0 * self.ec_ghz * self.ej_ghz).sqrt() - 2.0 * self.ec_ghz
    }

    /// Qubit negative anharmonicity $\alpha / (2\pi) = \omega_{12} - \omega_{01}$ in GHz:
    /// $$\alpha \approx -E_C$$
    pub fn anharmonicity_ghz(&self) -> f64 {
        -self.ec_ghz
    }

    /// Peak-to-peak charge dispersion $\epsilon_0$ in kHz:
    /// $$\epsilon_0 \approx (-1)^0 E_C 2^4 \sqrt{\frac{2}{\pi}} \left( \frac{E_J}{2 E_C} \right)^{3/4} \exp\left( -\sqrt{8 E_J / E_C} \right)$$
    pub fn charge_dispersion_epsilon_khz(&self) -> f64 {
        let ratio = self.ej_over_ec();
        let prefactor = self.ec_ghz * 16.0 * (2.0 / PI).sqrt() * (ratio / 2.0).powf(0.75);
        let exponent = (8.0 * ratio).sqrt();
        let eps_ghz = prefactor * (-exponent).exp();
        eps_ghz * 1.0e6 // kHz
    }

    /// Non-linear phase-dependent Josephson inductance $L_J(\phi) = L_{J0} / \cos\phi$ in nanohenries.
    pub fn non_linear_inductance_nh(&self, phase_rad: f64) -> f64 {
        let cos_p = phase_rad.cos().abs().max(1e-4);
        self.josephson_inductance_nh / cos_p
    }
}
