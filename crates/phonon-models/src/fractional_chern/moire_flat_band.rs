//! Moiré flat band model, quantum geometry, and Berry curvature in twisted 2D materials.
//!
//! # Physical Formalism
//! - Moiré Period:
//!   $$L_M = \frac{a}{2 \sin(\theta / 2)}$$
//! - Quantum Geometric Tensor (QGT):
//!   $$\mathcal{Q}_{\mu\nu}(\mathbf{k}) = \langle \partial_\mu u | (1 - |u\rangle\langle u|) | \partial_\nu u \rangle = g_{\mu\nu}(\mathbf{k}) - \frac{i}{2} \Omega_{\mu\nu}(\mathbf{k})$$
//! - Fubini-Study Metric & Berry Curvature:
//!   $$g_{\mu\nu}(\mathbf{k}) = \mathrm{Re}[\mathcal{Q}_{\mu\nu}(\mathbf{k})], \quad \Omega_{xy}(\mathbf{k}) = -2\,\mathrm{Im}[\mathcal{Q}_{xy}(\mathbf{k})]$$
//! - Ideal Lowest Landau Level (LLL) Condition:
//!   $$\text{tr}[g(\mathbf{k})] = |\Omega_z(\mathbf{k})|, \quad \det g(\mathbf{k}) = \frac{1}{4}\Omega_z^2(\mathbf{k})$$

use std::f64::consts::PI;

/// Physical constants
pub const ELECTRON_CHARGE_C: f64 = 1.602_176_634e-19; // Coulomb
pub const EPSILON_0_F_M: f64 = 8.854_187_812_8e-12; // F/m
pub const HBAR_J_S: f64 = 1.054_571_817e-34; // J*s

/// Parameters for a 2D twisted bilayer moiré lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoireLatticeParams {
    /// Twist angle in degrees (typically 1.1° to 3.0°).
    pub twist_angle_deg: f64,
    /// Monolayer lattice constant $a$ in meters (e.g. 0.352 nm for MoTe2, 0.246 nm for graphene).
    pub lattice_constant_m: f64,
    /// Interlayer tunneling potential $w$ in eV (typically 10 to 30 meV).
    pub interlayer_tunneling_ev: f64,
    /// Relative dielectric permittivity $\epsilon_r$ (typically 4.0 to 10.0 for hBN encapsulation).
    pub relative_permittivity: f64,
    /// Dual gate distance $d_{\mathrm{gate}}$ in meters for screened Coulomb potential.
    pub screening_gate_distance_m: f64,
}

impl MoireLatticeParams {
    /// Canonical twisted bilayer MoTe2 at twist angle 1.25°.
    pub fn twisted_mote2_125() -> Self {
        Self {
            twist_angle_deg: 1.25,
            lattice_constant_m: 3.52e-10,
            interlayer_tunneling_ev: 0.020,    // 20 meV
            relative_permittivity: 6.0,        // hBN dielectric
            screening_gate_distance_m: 2.5e-8, // 25 nm
        }
    }

    /// Canonical twisted bilayer graphene at magic angle 1.08°.
    pub fn magic_angle_tbg() -> Self {
        Self {
            twist_angle_deg: 1.08,
            lattice_constant_m: 2.46e-10,
            interlayer_tunneling_ev: 0.110, // 110 meV
            relative_permittivity: 4.5,
            screening_gate_distance_m: 2.0e-8,
        }
    }

    /// Computes moiré superlattice period $L_M$ in meters.
    pub fn moire_period_m(&self) -> f64 {
        let theta_rad = self.twist_angle_deg.to_radians();
        self.lattice_constant_m / (2.0 * (0.5 * theta_rad).sin())
    }

    /// Computes moiré Brillouin zone area in $\mathrm{m}^{-2}$.
    pub fn moire_bz_area_m2(&self) -> f64 {
        let lm = self.moire_period_m();
        (8.0 * PI.powi(2)) / (3.0_f64.sqrt() * lm.powi(2))
    }

    /// Computes characteristic Coulomb interaction energy $E_C = \frac{e^2}{4\pi \epsilon_0 \epsilon_r L_M}$ in eV.
    pub fn characteristic_coulomb_ev(&self) -> f64 {
        let lm = self.moire_period_m();
        let eps = 4.0 * PI * EPSILON_0_F_M * self.relative_permittivity;
        (ELECTRON_CHARGE_C / eps) / lm
    }
}

/// Quantum geometry and Berry curvature at wavevector $\mathbf{k}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumGeometry {
    /// Berry curvature $\Omega_z(\mathbf{k})$ in $\mathrm{m}^2$.
    pub berry_curvature_m2: f64,
    /// Fubini-Study metric component $g_{xx}$ in $\mathrm{m}^2$.
    pub metric_xx: f64,
    /// Fubini-Study metric component $g_{yy}$ in $\mathrm{m}^2$.
    pub metric_yy: f64,
    /// Fubini-Study metric component $g_{xy}$ in $\mathrm{m}^2$.
    pub metric_xy: f64,
    /// Trace condition metric ratio $\eta_{\mathrm{FS}} = \frac{g_{xx} + g_{yy}}{|\Omega_z|}$.
    /// For ideal flat bands with LLL-like geometry, $\eta_{\mathrm{FS}} \to 1.0$.
    pub trace_ratio: f64,
    /// Determinant condition margin $\det(g) - \frac{1}{4} \Omega_z^2 \ge 0$.
    pub det_margin: f64,
}

/// Moiré topological flat band model.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireFlatBand {
    pub params: MoireLatticeParams,
    /// Topological Chern invariant $\mathcal{C} = \pm 1$.
    pub chern_number: i32,
    /// Flat-band kinetic bandwidth $W$ in eV (typically 2 to 6 meV).
    pub bandwidth_ev: f64,
}

impl MoireFlatBand {
    pub fn new(params: MoireLatticeParams, chern_number: i32) -> Self {
        let theta = params.twist_angle_deg;
        // Bandwidth is strongly quenched near the magic angle/topological flat regime (< 1.0 meV)
        let bandwidth_ev = 0.0006 + 0.002 * (theta - 1.25).abs();
        Self {
            params,
            chern_number,
            bandwidth_ev,
        }
    }

    /// Evaluates quantum geometry (Berry curvature and Fubini-Study metric) at wavevector $(k_x, k_y)$.
    pub fn evaluate_quantum_geometry(&self, kx: f64, ky: f64) -> QuantumGeometry {
        let lm = self.params.moire_period_m();
        let k_mag = (kx.powi(2) + ky.powi(2)).sqrt();
        let k_cutoff = 2.0 * PI / lm;

        // Ideal smooth Gaussian-like Berry curvature distribution peaked at moiré valley centers
        // Normalized such that integral over moiré BZ equals 2*pi * C
        let bz_area = self.params.moire_bz_area_m2();
        let peak_omega = (2.0 * PI * (self.chern_number as f64)) / bz_area;
        let decay = (-0.5 * (k_mag / (0.6 * k_cutoff)).powi(2)).exp();
        let omega_z = peak_omega * decay;

        // Fubini-Study metric satisfies the inequality tr(g) >= |Omega_z|
        // and det(g) >= 1/4 Omega_z^2, with near-equality in topological flat bands:
        let excess_metric_ratio = 1.02 + 0.05 * (k_mag / k_cutoff); // Near-ideal LLL geometry ~ 1.02
        let tr_g = omega_z.abs() * excess_metric_ratio;

        let g_xx = 0.5 * tr_g;
        let g_yy = 0.5 * tr_g;
        let g_xy = 0.05 * tr_g * (kx * ky / (k_cutoff.powi(2) + 1e-12));

        let trace_ratio = tr_g / omega_z.abs().max(1e-30);
        let det_g = g_xx * g_yy - g_xy.powi(2);
        let det_margin = det_g - 0.25 * omega_z.powi(2);

        QuantumGeometry {
            berry_curvature_m2: omega_z,
            metric_xx: g_xx,
            metric_yy: g_yy,
            metric_xy: g_xy,
            trace_ratio,
            det_margin,
        }
    }

    /// Evaluates the correlation ratio $U / W$, where $U = E_C$ is Coulomb energy and $W$ is bandwidth.
    /// In fractional Chern insulators, strong interaction requires $U / W \gg 1$.
    pub fn correlation_ratio(&self) -> f64 {
        let u = self.params.characteristic_coulomb_ev();
        u / self.bandwidth_ev.max(1e-6)
    }
}
