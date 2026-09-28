//! Superconducting Non-linear Asymmetric Inductive Elements (SNAILs):
//! Kerr-free third-order non-linearities, three-wave mixing (3WM),
//! and high saturation power parametric amplifiers.

/// Physical parameters for a SNAIL element loop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SnailElementParams {
    /// Number of identical large Josephson junctions in array arm (typically n = 3).
    pub num_large_junctions: usize,
    /// Critical current ratio alpha_J = I_c,small / I_c,large (typically 0.1 - 0.4).
    pub junction_ratio_alpha: f64,
    /// Josephson coupling energy E_J of large junctions in Joules (typically ~1e-21 - 1e-20 J).
    pub ej_large_joules: f64,
    /// External magnetic flux bias phi_ext = 2*pi*Phi_ext / Phi_0 in radians.
    pub external_flux_rad: f64,
}

impl SnailElementParams {
    /// Constructs SNAIL parameters.
    pub fn new(
        num_large_junctions: usize,
        junction_ratio_alpha: f64,
        ej_large_joules: f64,
        external_flux_rad: f64,
    ) -> Self {
        Self {
            num_large_junctions,
            junction_ratio_alpha,
            ej_large_joules,
            external_flux_rad,
        }
    }

    /// Standard n = 3, alpha = 0.29 SNAIL with optimal Kerr-free flux point.
    pub fn standard_kerr_free_snail() -> Self {
        let phi_ext_optimal = 0.4089 * 2.0 * std::f64::consts::PI; // ~0.4089 Phi_0
        Self::new(3, 0.29, 2.5e-21, phi_ext_optimal)
    }

    /// Equilibrium minimum phase phi_0(phi_ext) satisfying dU/dphi = 0:
    /// $$\sin(\phi_0 / n) = \alpha_J \sin(\phi_{ext} - \phi_0)$$
    pub fn minimum_phase_rad(&self) -> f64 {
        let n = self.num_large_junctions as f64;
        let alpha = self.junction_ratio_alpha;
        let phi_ext = self.external_flux_rad;

        // Newton-Raphson iteration for root of f(phi) = sin(phi/n) - alpha * sin(phi_ext - phi)
        let mut phi = phi_ext / (1.0 + n * alpha);
        for _ in 0..20 {
            let f = (phi / n).sin() - alpha * (phi_ext - phi).sin();
            let df = (1.0 / n) * (phi / n).cos() + alpha * (phi_ext - phi).cos();
            if df.abs() < 1e-12 {
                break;
            }
            let step = f / df;
            phi -= step;
            if step.abs() < 1e-12 {
                break;
            }
        }
        phi
    }

    /// Second-order dimensionless Taylor expansion coefficient c_2 (proportional to linear inductance):
    /// $$c_2 = \frac{1}{n} \cos(\phi_0 / n) + \alpha_J \cos(\phi_{ext} - \phi_0)$$
    pub fn c2_coefficient(&self) -> f64 {
        let n = self.num_large_junctions as f64;
        let alpha = self.junction_ratio_alpha;
        let phi_ext = self.external_flux_rad;
        let phi0 = self.minimum_phase_rad();
        (1.0 / n) * (phi0 / n).cos() + alpha * (phi_ext - phi0).cos()
    }

    /// Third-order dimensionless Taylor expansion coefficient c_3 (Three-Wave Mixing chi^(2) non-linearity):
    /// $$c_3 = -\frac{1}{n^2} \sin(\phi_0 / n) + \alpha_J \sin(\phi_{ext} - \phi_0)$$
    pub fn c3_coefficient(&self) -> f64 {
        let n = self.num_large_junctions as f64;
        let alpha = self.junction_ratio_alpha;
        let phi_ext = self.external_flux_rad;
        let phi0 = self.minimum_phase_rad();
        -(1.0 / (n * n)) * (phi0 / n).sin() + alpha * (phi_ext - phi0).sin()
    }

    /// Fourth-order dimensionless Taylor expansion coefficient c_4 (Kerr chi^(3) non-linearity):
    /// $$c_4 = -\frac{1}{n^3} \cos(\phi_0 / n) - \alpha_J \cos(\phi_{ext} - \phi_0)$$
    pub fn c4_coefficient(&self) -> f64 {
        let n = self.num_large_junctions as f64;
        let alpha = self.junction_ratio_alpha;
        let phi_ext = self.external_flux_rad;
        let phi0 = self.minimum_phase_rad();
        -(1.0 / n.powi(3)) * (phi0 / n).cos() - alpha * (phi_ext - phi0).cos()
    }

    /// Verifies if the SNAIL is biased at the Kerr-free point (|c_4| < 0.05):
    pub fn is_kerr_free(&self) -> bool {
        self.c4_coefficient().abs() < 0.05
    }

    /// 1-dB compression saturation power P_{-1dB} in dBm:
    /// When c_4 -> 0, saturation power increases dramatically above -90 dBm.
    pub fn saturation_power_1db_dbm(&self) -> f64 {
        let c4_abs = self.c4_coefficient().abs().max(1e-4);
        let base_power_dbm = -105.0; // Standard junction JTWPA
                                     // Suppression of c4 scales saturation power as ~ 1 / c4^2
        let bonus_db = 20.0 * (1.0 / (c4_abs * 10.0)).log10().max(0.0);
        (base_power_dbm + bonus_db).min(-75.0)
    }
}
