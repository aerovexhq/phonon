#![deny(unsafe_code)]

//! Quantized vortex dynamics, circulation topology, and Landau superfluidity engine.
//!
//! Models quantized vortices with integer topological charge ell, core density depletion,
//! rotationally induced vortex lattices, and frictionless superfluid transport past obstacles.

use std::f64::consts::PI;
use crate::polariton_bec_vortices::gross_pitaevskii::HBAR;

/// Topological charge and circulation state of a single or cluster of vortices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VortexCharge {
    Zero,
    PlusOne,
    MinusOne,
    PlusTwo,
    DipolePair,
}

impl VortexCharge {
    pub fn winding_number(&self) -> i32 {
        match self {
            VortexCharge::Zero => 0,
            VortexCharge::PlusOne => 1,
            VortexCharge::MinusOne => -1,
            VortexCharge::PlusTwo => 2,
            VortexCharge::DipolePair => 0, // net charge zero (+1 and -1)
        }
    }
}

/// Parameters configuring vortex generation and superfluid obstacle flow.
#[derive(Debug, Clone, PartialEq)]
pub struct VortexSuperfluidParams {
    /// Active vortex topological configuration.
    pub charge: VortexCharge,
    /// Healing length xi in micrometers.
    pub healing_length_um: f64,
    /// Peak condensate density n_0 in um^-2.
    pub peak_density_um2: f64,
    /// Polariton effective mass in kg.
    pub effective_mass_kg: f64,
    /// Rotation / angular drive frequency Omega in rad/ns (for vortex lattice formation).
    pub rotation_frequency_rad_ns: f64,
    /// Relative superfluid flow velocity v / v_c past a defect obstacle.
    pub flow_velocity_ratio: f64,
    /// Defect obstacle radius in micrometers.
    pub obstacle_radius_um: f64,
    /// Condensate trap radius in micrometers.
    pub condensate_radius_um: f64,
}

impl Default for VortexSuperfluidParams {
    fn default() -> Self {
        Self {
            charge: VortexCharge::PlusOne,
            healing_length_um: 2.2,
            peak_density_um2: 85.0,
            effective_mass_kg: 9.109_383_7e-35, // 1e-4 m_0
            rotation_frequency_rad_ns: 0.15,
            flow_velocity_ratio: 0.55, // subcritical flow v < v_c
            obstacle_radius_um: 1.5,
            condensate_radius_um: 20.0,
        }
    }
}

/// A 2D spatial point on the vortex lattice canvas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VortexGridPoint {
    pub x_um: f64,
    pub y_um: f64,
    /// Condensate density n(x, y) normalized to n_0.
    pub normalized_density: f64,
    /// Macroscopic phase angle theta(x, y) in [-pi, pi].
    pub phase_rad: f64,
    /// Superfluid velocity vector magnitude |v_s(x, y)| in m/s.
    pub superfluid_speed_ms: f64,
}

/// Superfluid transport and vortex lattice metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VortexLatticeMetrics {
    /// Quantized circulation Gamma = oint v_s . dr in units of h / m*.
    pub quantized_circulation_units: f64,
    /// Critical rotation frequency Omega_c in rad/ns required to nucleate first vortex.
    pub critical_rotation_frequency_rad_ns: f64,
    /// Estimated number of equilibrium vortices nucleated in rotating trap.
    pub equilibrium_vortex_count: usize,
    /// Landau critical velocity v_c in m/s.
    pub landau_critical_velocity_ms: f64,
    /// Drag force experienced by obstacle normalized to classical high-velocity drag.
    pub normalized_drag_force: f64,
    /// Frictionless flow suppression ratio (in dB) relative to normal fluid drag.
    pub drag_suppression_db: f64,
    /// Whether flow past obstacle is in frictionless superfluid regime (v < v_c).
    pub is_frictionless_superfluid: bool,
}

/// Engine evaluating quantized vortices and non-equilibrium superfluidity.
#[derive(Debug, Clone)]
pub struct QuantizedVortexSolver {
    pub params: VortexSuperfluidParams,
}

impl QuantizedVortexSolver {
    /// Creates a new vortex solver with given parameters.
    pub fn new(params: VortexSuperfluidParams) -> Self {
        Self { params }
    }

    /// Evaluates Landau critical velocity v_c in m/s.
    ///
    /// v_c = hbar / (sqrt(2) * m* * xi)
    pub fn landau_critical_velocity_ms(&self) -> f64 {
        let m = self.params.effective_mass_kg.max(1e-36);
        let xi_m = (self.params.healing_length_um * 1.0e-6).max(1e-8);
        HBAR / (2.0_f64.sqrt() * m * xi_m)
    }

    /// Computes quantum of circulation kappa_0 = h / m* in m^2 / s.
    pub fn quantum_of_circulation_m2_s(&self) -> f64 {
        let m = self.params.effective_mass_kg.max(1e-36);
        (2.0 * PI * HBAR) / m
    }

    /// Computes critical rotation frequency Omega_c = (hbar / m* R^2) * ln(R / xi) in rad/ns.
    pub fn critical_rotation_frequency_rad_ns(&self) -> f64 {
        let m = self.params.effective_mass_kg.max(1e-36);
        let r_m = (self.params.condensate_radius_um * 1.0e-6).max(1e-7);
        let xi_m = (self.params.healing_length_um * 1.0e-6).max(1e-8);

        let arg = (r_m / xi_m).max(1.05);
        let omega_c_s = (HBAR / (m * r_m * r_m)) * arg.ln();
        omega_c_s * 1.0e-9 // convert to rad/ns
    }

    /// Estimates number of vortices forming in triangular Abrikosov lattice at given Omega.
    pub fn equilibrium_vortex_count(&self) -> usize {
        let omega_c = self.critical_rotation_frequency_rad_ns();
        let omega = self.params.rotation_frequency_rad_ns;
        if omega <= omega_c {
            0
        } else {
            let m = self.params.effective_mass_kg.max(1e-36);
            let r_m = self.params.condensate_radius_um * 1.0e-6;
            let omega_s = omega * 1.0e9;
            let n_vortices = ((2.0 * m * omega_s * r_m * r_m) / (HBAR * PI)).round() as usize;
            n_vortices.clamp(1, 19)
        }
    }

    /// Computes normalized drag force on defect obstacle as a function of flow velocity.
    ///
    /// Landau criterion: for v < v_c, dissipation vanishes (drag ~ 0).
    /// For v > v_c, Cherenkov vortex shedding produces finite drag.
    pub fn compute_obstacle_drag(&self) -> (f64, f64, bool) {
        let v_ratio = self.params.flow_velocity_ratio;
        if v_ratio < 1.0 {
            // Frictionless superfluid flow
            let leak = 1e-4 * v_ratio.powi(4);
            let suppression_db = (-10.0 * leak.max(1e-5).log10()).max(35.0);
            (leak, suppression_db, true)
        } else {
            // Dissipative above critical velocity
            let excess = 1.0 - 1.0 / (v_ratio * v_ratio);
            let drag = excess.powf(1.5).clamp(0.01, 1.0);
            let suppression_db = (-10.0 * drag.log10()).max(0.0);
            (drag, suppression_db, false)
        }
    }

    /// Evaluates complete vortex and superfluid metrics.
    pub fn evaluate_metrics(&self) -> VortexLatticeMetrics {
        let vc = self.landau_critical_velocity_ms();
        let omega_c = self.critical_rotation_frequency_rad_ns();
        let nv = self.equilibrium_vortex_count();
        let (drag, supp_db, is_super) = self.compute_obstacle_drag();
        let circ_units = self.params.charge.winding_number() as f64;

        VortexLatticeMetrics {
            quantized_circulation_units: circ_units,
            critical_rotation_frequency_rad_ns: omega_c,
            equilibrium_vortex_count: nv,
            landau_critical_velocity_ms: vc,
            normalized_drag_force: drag,
            drag_suppression_db: supp_db,
            is_frictionless_superfluid: is_super,
        }
    }

    /// Evaluates 2D field point (x, y) around vortex cores.
    pub fn evaluate_grid_point(&self, x_um: f64, y_um: f64) -> VortexGridPoint {
        let xi = self.params.healing_length_um;
        let r_sq = x_um * x_um + y_um * y_um;
        let r = r_sq.sqrt();

        let (density, phase) = match self.params.charge {
            VortexCharge::Zero => {
                // Flat Thomas-Fermi profile
                let trap_r = self.params.condensate_radius_um;
                let dens = (1.0 - (r / trap_r).powi(2)).max(0.0);
                (dens, 0.0)
            }
            VortexCharge::PlusOne => {
                // Single centered vortex ell = +1
                let core_factor = (r / xi).powi(2) / (1.0 + (r / xi).powi(2));
                let phi = y_um.atan2(x_um);
                (core_factor, phi)
            }
            VortexCharge::MinusOne => {
                // Single centered vortex ell = -1
                let core_factor = (r / xi).powi(2) / (1.0 + (r / xi).powi(2));
                let phi = -y_um.atan2(x_um);
                (core_factor, phi)
            }
            VortexCharge::PlusTwo => {
                // Giant vortex ell = +2
                let core_factor = (r / xi).powi(4) / (1.0 + (r / xi).powi(4));
                let phi = 2.0 * y_um.atan2(x_um);
                let wrapped_phi = (phi + PI).rem_euclid(2.0 * PI) - PI;
                (core_factor, wrapped_phi)
            }
            VortexCharge::DipolePair => {
                // Vortex-antivortex pair separated by d = 3 * xi along x
                let d = 3.0 * xi;
                let r1 = ((x_um - d * 0.5).powi(2) + y_um.powi(2)).sqrt();
                let r2 = ((x_um + d * 0.5).powi(2) + y_um.powi(2)).sqrt();
                let c1 = (r1 / xi).powi(2) / (1.0 + (r1 / xi).powi(2));
                let c2 = (r2 / xi).powi(2) / (1.0 + (r2 / xi).powi(2));
                let dens = c1 * c2;
                let phi1 = y_um.atan2(x_um - d * 0.5);
                let phi2 = -y_um.atan2(x_um + d * 0.5);
                let net_phi = (phi1 + phi2 + PI).rem_euclid(2.0 * PI) - PI;
                (dens, net_phi)
            }
        };

        // Superfluid velocity v_s = (hbar / m*) * grad(phase)
        let vs_speed = if r < 0.1 * xi {
            0.0
        } else {
            let kappa = self.quantum_of_circulation_m2_s() / (2.0 * PI);
            (kappa / (r * 1.0e-6)).min(5.0e6)
        };

        VortexGridPoint {
            x_um,
            y_um,
            normalized_density: density.clamp(0.0, 1.0),
            phase_rad: phase,
            superfluid_speed_ms: vs_speed,
        }
    }
}
