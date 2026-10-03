#![deny(unsafe_code)]

//! Smith Chart geometry: reflection coefficient Gamma and impedance conversions,
//! constant resistance circles, constant reactance circles and arcs,
//! and Source / Load stability circles.

use super::s_parameters::{Complex64, TwoPortSParameters};
use std::f64::consts::PI;

/// Representation of a circle in the complex reflection coefficient Gamma plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SmithCircle {
    /// Center of circle in the Gamma plane (u + j*v).
    pub center: Complex64,
    /// Radius of circle in the Gamma plane.
    pub radius: f64,
}

impl SmithCircle {
    /// Constructs a new Smith circle.
    pub fn new(center: Complex64, radius: f64) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
        }
    }

    /// Unit circle representing |Gamma| = 1.0 (boundary of passive Smith chart).
    pub fn unit_circle() -> Self {
        Self {
            center: Complex64::ZERO,
            radius: 1.0,
        }
    }

    /// Tests if a complex reflection coefficient point lies inside the circle (within tolerance).
    pub fn contains(&self, p: Complex64, tol: f64) -> bool {
        (p - self.center).abs() <= self.radius + tol
    }

    /// Generates N evenly spaced points along the circle circumference.
    pub fn sample_circumference(&self, points: usize) -> Vec<Complex64> {
        let n = points.max(3);
        (0..n)
            .map(|i| {
                let theta = 2.0 * PI * (i as f64) / (n as f64);
                Complex64::new(
                    self.center.re + self.radius * theta.cos(),
                    self.center.im + self.radius * theta.sin(),
                )
            })
            .collect()
    }
}

/// Converts complex reflection coefficient Gamma to normalized impedance z = r + j*x.
/// Formula: z = (1 + Gamma) / (1 - Gamma)
pub fn gamma_to_normalized_z(gamma: Complex64) -> Complex64 {
    let den = Complex64::ONE - gamma;
    if den.norm_sq() < 1e-28 {
        Complex64::new(1e14, 0.0)
    } else {
        (Complex64::ONE + gamma) / den
    }
}

/// Converts normalized impedance z = r + j*x to complex reflection coefficient Gamma.
/// Formula: Gamma = (z - 1) / (z + 1)
pub fn normalized_z_to_gamma(z: Complex64) -> Complex64 {
    let den = z + Complex64::ONE;
    if den.norm_sq() < 1e-28 {
        Complex64::ONE
    } else {
        (z - Complex64::ONE) / den
    }
}

/// Converts complex reflection coefficient Gamma to actual impedance Z = R + j*X in Ohms,
/// given reference characteristic impedance Z0.
pub fn gamma_to_z(gamma: Complex64, z0: f64) -> Complex64 {
    gamma_to_normalized_z(gamma) * z0
}

/// Converts actual impedance Z = R + j*X in Ohms to reflection coefficient Gamma,
/// given reference characteristic impedance Z0.
pub fn z_to_gamma(z: Complex64, z0: f64) -> Complex64 {
    if z0.abs() < 1e-12 {
        Complex64::ONE
    } else {
        let z_norm = z / z0;
        normalized_z_to_gamma(z_norm)
    }
}

/// Computes the constant resistance circle in the Gamma plane for normalized resistance r >= 0.
/// Center: (r / (1 + r), 0)
/// Radius: 1 / (1 + r)
pub fn constant_resistance_circle(r: f64) -> SmithCircle {
    let r_clamped = r.max(0.0);
    let den = 1.0 + r_clamped;
    let center = Complex64::new(r_clamped / den, 0.0);
    let radius = 1.0 / den;
    SmithCircle::new(center, radius)
}

/// Computes the constant reactance circle in the Gamma plane for normalized reactance x != 0.
/// Center: (1, 1 / x)
/// Radius: 1 / |x|
pub fn constant_reactance_circle(x: f64) -> SmithCircle {
    let x_safe = if x.abs() < 1e-12 {
        if x >= 0.0 { 1e-12 } else { -1e-12 }
    } else {
        x
    };
    let center = Complex64::new(1.0, 1.0 / x_safe);
    let radius = 1.0 / x_safe.abs();
    SmithCircle::new(center, radius)
}

/// Computes sampled points along the constant reactance arc inside the unit circle (|Gamma| <= 1).
/// Normalized impedance z(r) = r + j*x sweeps from r = 0 to r = r_max.
pub fn constant_reactance_arc(x: f64, num_points: usize) -> Vec<Complex64> {
    let n = num_points.max(2);
    // Sweep normalized resistance r from 0 to 50 with quadratic spacing for smooth curvature near Gamma = 1
    (0..n)
        .map(|i| {
            let t = (i as f64) / ((n - 1) as f64);
            let r = 50.0 * t * t;
            let z = Complex64::new(r, x);
            normalized_z_to_gamma(z)
        })
        .collect()
}

/// Computes the Load Stability Circle in the Gamma_L reflection coefficient plane.
/// Center: C_L = (S22 - Delta * S11*)* / (|S22|^2 - |Delta|^2)
/// Radius: R_L = |S12 * S21| / ||S22|^2 - |Delta|^2|
pub fn load_stability_circle(s: &TwoPortSParameters) -> SmithCircle {
    let delta = s.delta();
    let s22_sq = s.s22.norm_sq();
    let delta_sq = delta.norm_sq();
    let denom = s22_sq - delta_sq;

    let term = s.s22 - delta * s.s11.conj();
    let num = term.conj();

    let denom_safe = if denom.abs() < 1e-18 {
        if denom >= 0.0 { 1e-18 } else { -1e-18 }
    } else {
        denom
    };

    let center = num / denom_safe;
    let radius = (s.s12 * s.s21).abs() / denom_safe.abs();

    SmithCircle::new(center, radius)
}

/// Computes the Source Stability Circle in the Gamma_S reflection coefficient plane.
/// Center: C_S = (S11 - Delta * S22*)* / (|S11|^2 - |Delta|^2)
/// Radius: R_S = |S12 * S21| / ||S11|^2 - |Delta|^2|
pub fn source_stability_circle(s: &TwoPortSParameters) -> SmithCircle {
    let delta = s.delta();
    let s11_sq = s.s11.norm_sq();
    let delta_sq = delta.norm_sq();
    let denom = s11_sq - delta_sq;

    let term = s.s11 - delta * s.s22.conj();
    let num = term.conj();

    let denom_safe = if denom.abs() < 1e-18 {
        if denom >= 0.0 { 1e-18 } else { -1e-18 }
    } else {
        denom
    };

    let center = num / denom_safe;
    let radius = (s.s12 * s.s21).abs() / denom_safe.abs();

    SmithCircle::new(center, radius)
}

/// Standard resistance grid values commonly plotted on an RF Smith chart.
pub fn standard_resistance_values() -> Vec<f64> {
    vec![0.0, 0.2, 0.5, 1.0, 2.0, 5.0]
}

/// Standard reactance grid values commonly plotted on an RF Smith chart.
pub fn standard_reactance_values() -> Vec<f64> {
    vec![
        0.2, 0.5, 1.0, 2.0, 5.0,
        -0.2, -0.5, -1.0, -2.0, -5.0,
    ]
}
