//! Quantum tunneling models: direct gate dielectric tunneling (WKB), Fowler-Nordheim
//! field emission, and source-to-drain Band-to-Band Tunneling (BTBT).

use phonon_core::{ELEMENTARY_CHARGE, H_BAR};

/// Physical parameters for gate dielectric tunneling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DielectricTunnelingModel {
    /// Physical oxide thickness $t_{ox}$ in meters ($m$).
    pub tox: f64,
    /// Conduction band offset / barrier height $\Phi_B$ in electron-volts ($eV$).
    pub barrier_height_ev: f64,
    /// Effective electron tunneling mass $m_{ox}^*$ relative to $m_0$.
    pub effective_mass: f64,
    /// Relative dielectric permittivity $\kappa$.
    pub relative_permittivity: f64,
}

impl Default for DielectricTunnelingModel {
    fn default() -> Self {
        // Typical SiO2: Phi_B = 3.15 eV, m_ox* = 0.5 m0, tox = 1.2 nm, kappa = 3.9
        Self {
            tox: 1.2e-9,
            barrier_height_ev: 3.15,
            effective_mass: 0.5,
            relative_permittivity: 3.9,
        }
    }
}

impl DielectricTunnelingModel {
    /// Computes the Wentzel-Kramers-Brillouin (WKB) direct tunneling transmission probability
    /// across a trapezoidal barrier under oxide voltage $V_{ox}$:
    /// $$T_{WKB} \approx \exp\left( -2 \int_0^{t_{ox}} \kappa(x) dx \right)$$
    pub fn wkb_direct_transmission(&self, v_ox: f64) -> f64 {
        let m0 = 9.109_383_701_5e-31;
        let m_eff = self.effective_mass * m0;
        let q = ELEMENTARY_CHARGE;
        let phi_j = self.barrier_height_ev * q;

        let v_abs = v_ox.abs();

        if v_abs < 1e-4 {
            // Flat rectangular barrier of height Phi_B
            let k0 = (2.0 * m_eff * phi_j).sqrt() / H_BAR;
            let exponent = -2.0 * k0 * self.tox;
            exponent.clamp(-100.0, 0.0).exp()
        } else if v_abs < self.barrier_height_ev {
            // Trapezoidal barrier: integral = 2/3 * (Phi_B^{3/2} - (Phi_B - q*Vox)^{3/2}) / (q*Vox/tox)
            let phi_right = phi_j - q * v_abs;
            let prefactor = (4.0 / 3.0) * (2.0 * m_eff).sqrt() * self.tox / (H_BAR * q * v_abs);
            let exponent = -prefactor * (phi_j.powf(1.5) - phi_right.powf(1.5));
            exponent.clamp(-100.0, 0.0).exp()
        } else {
            // Triangular barrier (Fowler-Nordheim regime)
            let x_turn = self.tox * (self.barrier_height_ev / v_abs);
            let prefactor = (4.0 / 3.0) * (2.0 * m_eff).sqrt() * x_turn / (H_BAR * phi_j);
            let exponent = -prefactor * phi_j.powf(1.5);
            exponent.clamp(-100.0, 0.0).exp()
        }
    }

    /// Evaluates gate direct tunneling leakage current density $J_{gate}$ in $A / m^2$:
    /// $$J_{gate} = \frac{q^3 V_{ox}^2}{8\pi h \Phi_B t_{ox}^2} T_{WKB}$$
    pub fn gate_leakage_current_density(&self, v_ox: f64) -> f64 {
        if v_ox.abs() < 1e-6 {
            return 0.0;
        }
        let t_wkb = self.wkb_direct_transmission(v_ox);
        let q = ELEMENTARY_CHARGE;
        let v_abs = v_ox.abs();

        // Effective carrier impingement frequency prefactor
        let field = v_abs / self.tox;
        let j_mag = (q * field / (2.0 * std::f64::consts::PI * H_BAR)) * q * field * t_wkb;
        j_mag * v_ox.signum()
    }
}

/// Kane Band-to-Band Tunneling (BTBT) model for reverse-biased junctions and sub-3nm FETs:
/// $$G_{BTBT} = A_{BTBT} \frac{\mathcal{E}^2}{E_g^{1/2}} \exp\left( -B_{BTBT} \frac{E_g^{3/2}}{\mathcal{E}} \right)$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandToBandTunnelingModel {
    /// Pre-exponential factor $A_{BTBT}$ in $A / (V^2 \cdot m)$.
    pub a_btbt: f64,
    /// Exponential decay field factor $B_{BTBT}$ in $V / (m \cdot eV^{3/2})$.
    pub b_btbt: f64,
    /// Semiconductor energy bandgap $E_g$ in $eV$.
    pub bandgap_ev: f64,
}

impl Default for BandToBandTunnelingModel {
    fn default() -> Self {
        // Typical values for Silicon: A ~ 4e14 cm^-1 s^-1 V^-2, B ~ 1.9e7 V/cm
        Self {
            a_btbt: 4.0e19,
            b_btbt: 1.9e9,
            bandgap_ev: 1.12,
        }
    }
}

impl BandToBandTunnelingModel {
    /// Computes generation rate / tunneling current density $J_{BTBT}$ in $A / m^3$:
    pub fn generation_rate(&self, electric_field_v_per_m: f64) -> f64 {
        let field = electric_field_v_per_m.abs();
        if field < 1e6 {
            return 0.0;
        }

        let eg_sqrt = self.bandgap_ev.sqrt();
        let eg_3_2 = self.bandgap_ev * eg_sqrt;

        let exponent = -self.b_btbt * eg_3_2 / field;
        if exponent < -80.0 {
            return 0.0;
        }

        let rate = (self.a_btbt * field * field / eg_sqrt) * exponent.exp();
        rate * electric_field_v_per_m.signum()
    }
}
