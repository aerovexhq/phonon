//! Strongly correlated Mott insulator states and Hartree-Fock Coulomb interactions in moiré flat bands.
//!
//! Evaluates on-site Hubbard repulsion U = e^2 / (4\u{03c0} \u{03b5} L_M), filling-dependent correlated Mott gaps,
//! and Arrhenius thermal activation transport.

/// Model for strongly correlated electron states in moiré superlattices.
#[derive(Debug, Clone, PartialEq)]
pub struct CorrelatedInsulatorModel {
    /// Moiré period L_M (nm).
    pub moire_period_nm: f64,
    /// Relative dielectric constant \u{03b5}_r (typically ~4-6 for hBN encapsulation).
    pub epsilon_r: f64,
    /// On-site Coulomb interaction energy U (eV).
    pub coulomb_u_ev: f64,
    /// Flat-band kinetic bandwidth W (eV).
    pub bandwidth_w_ev: f64,
    /// Correlation strength ratio U / W.
    pub correlation_ratio: f64,
}

impl CorrelatedInsulatorModel {
    /// Creates a new correlated insulator model given moiré period (nm), \u{03b5}_r, and bandwidth W (eV).
    pub fn new(moire_period_nm: f64, epsilon_r: f64, bandwidth_w_ev: f64) -> Self {
        // U = e^2 / (4 * \u{03c0} * \u{03b5}_0 * \u{03b5}_r * L_M)
        // With e^2 / (4 \u{03c0} \u{03b5}_0) \u{2248} 1.43996 eV*nm
        let l_m = moire_period_nm.max(1.0);
        let eps = epsilon_r.max(1.0);
        let coulomb_u_ev = 1.439_964_547_8 / (eps * l_m);
        let correlation_ratio = coulomb_u_ev / bandwidth_w_ev.max(1e-6);

        Self {
            moire_period_nm,
            epsilon_r,
            coulomb_u_ev,
            bandwidth_w_ev,
            correlation_ratio,
        }
    }

    /// Evaluates the correlated Mott insulating gap \u{0394}_corr(\u{03bd}) at integer or fractional filling \u{03bd} \u{2208} [-4, 4].
    /// Mott states are most pronounced at half-filling \u{03bd} = \u{00b1}2, and quarter/three-quarter \u{03bd} = \u{00b1}1, \u{00b1}3.
    pub fn correlated_gap_ev(&self, filling: f64) -> f64 {
        let abs_nu = filling.abs();
        if abs_nu >= 3.99 {
            // Full band insulator at \u{03bd} = \u{00b1}4 (dominated by remote bandgap)
            return 0.025; // 25 meV
        }
        if abs_nu < 0.05 {
            // Charge neutrality point (Dirac semimetal, zero Mott gap)
            return 0.0;
        }

        // Modulation by commensurate filling factor:
        let filling_weight = if (abs_nu - 2.0).abs() < 0.25 {
            1.0 // Half-filling \u{03bd} = \u{00b1}2 (strongest correlated Mott state)
        } else if (abs_nu - 1.0).abs() < 0.25 || (abs_nu - 3.0).abs() < 0.25 {
            0.65 // Quarter-filling \u{03bd} = \u{00b1}1 or \u{00b1}3
        } else {
            0.0 // Metallic between integer fillings
        };

        if filling_weight <= 0.0 {
            return 0.0;
        }

        // Mott-Hubbard gap formula: \u{0394} = 0.25 * U * \u{221a}(1 - (W / U)^2)
        if self.correlation_ratio > 1.0 {
            let factor = (1.0 - (1.0 / self.correlation_ratio.powi(2)))
                .max(0.0)
                .sqrt();
            0.25 * self.coulomb_u_ev * factor * filling_weight
        } else {
            0.0
        }
    }

    /// Evaluates the temperature-dependent electrical resistance R(T) across the correlated gap:
    /// R(T) = R_0 * exp(\u{0394}_corr / (2 k_B T)) for T < T_c, and metallic R_0 * (1 + \u{03b1} T) above T_c.
    pub fn evaluate_resistance_ohms(&self, filling: f64, temperature_k: f64, r0_ohms: f64) -> f64 {
        let gap_ev = self.correlated_gap_ev(filling);
        let kb_ev_k = 8.617_333_262e-5; // eV/K
        let t = temperature_k.max(0.01);

        if gap_ev <= 1e-6 {
            // Metallic behavior:
            return r0_ohms * (1.0 + 0.02 * t);
        }

        // Correlated Mott transition temperature T_Mott \u{2248} \u{0394} / (4 k_B)
        let t_mott = gap_ev / (4.0 * kb_ev_k);

        if t < t_mott {
            // Arrhenius thermal activation:
            let exponent = (gap_ev / (2.0 * kb_ev_k * t)).min(30.0);
            r0_ohms * exponent.exp()
        } else {
            // Metallic above Mott melting temperature:
            r0_ohms * (1.0 + 0.015 * (t - t_mott))
        }
    }

    /// Evaluates the Hartree-Fock exchange self-energy \u{03a3}_HF \u{2248} U * \u{03bd} / 4.
    pub fn hartree_fock_self_energy_ev(&self, filling: f64) -> f64 {
        self.coulomb_u_ev * (filling / 4.0)
    }
}
