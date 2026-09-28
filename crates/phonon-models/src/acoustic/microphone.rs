//! Physical Microphone Transducer Models
//!
//! Synthesizes first-principles physical microphone models:
//! 1. Capacitive Condenser Diaphragms: second-order harmonic oscillator driven by
//!    acoustic pressure $P(t)$, modulating parallel-plate capacitance $C(t)$ to generate
//!    electrical output voltages $v_{out}(t)$.
//! 2. Piezoelectric Transducers: coupled stress-to-voltage conversion with internal
//!    capacitance and source impedance.
//! 3. Directional Polar Patterns: Omnidirectional, Cardioid, Supercardioid, and Figure-8.

use crate::em::Vector3D;
use phonon_core::constants::EPSILON_0;
use std::f64::consts::PI;

/// Physical Directional Polar Directivity Patterns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MicrophonePolarPattern {
    /// Omnidirectional: uniform sensitivity across all angles ($D(\theta) = 1.0$).
    Omnidirectional,
    /// Cardioid: heart-shaped directivity ($D(\theta) = 0.5 + 0.5 \cos\theta$).
    Cardioid,
    /// Supercardioid: tighter front lobe with small rear lobe ($D(\theta) = 0.37 + 0.63 \cos\theta$).
    Supercardioid,
    /// Figure-8 (Bidirectional): equal front and back pickup, zero at 90 degrees ($D(\theta) = |\cos\theta|$).
    Figure8,
}

impl MicrophonePolarPattern {
    /// Evaluates polar directivity factor $D(\theta) \in [0.0, 1.0]$ at angle $\theta$ from main axis.
    pub fn directivity(&self, angle_rad: f64) -> f64 {
        let cos_th = angle_rad.cos();
        match self {
            Self::Omnidirectional => 1.0,
            Self::Cardioid => (0.5 + 0.5 * cos_th).abs().clamp(0.0, 1.0),
            Self::Supercardioid => (0.37 + 0.63 * cos_th).abs().clamp(0.0, 1.0),
            Self::Figure8 => cos_th.abs().clamp(0.0, 1.0),
        }
    }
}

/// Physical Capacitive Condenser Microphone.
///
/// Models a flexible diaphragm with mass $m$, stiffness $k$, damping $c_d$, and area $A_d$
/// spaced distance $d_0$ from a fixed backplate with DC bias voltage $V_{bias}$.
#[derive(Debug, Clone, PartialEq)]
pub struct CondenserMicrophone {
    /// Microphone diaphragm center position in 3D space.
    pub position: Vector3D,
    /// Normal orientation vector pointing along the microphone main pickup axis.
    pub axis: Vector3D,
    /// Diaphragm surface area $A_d$ in $\text{m}^2$ (e.g. $1.0 \times 10^{-4}\,\text{m}^2$ for 1/2-inch capsule).
    pub diaphragm_area: f64,
    /// Nominal plate separation $d_0$ in meters (e.g. $20\,\mu\text{m}$).
    pub plate_spacing_m: f64,
    /// Effective moving diaphragm mass $m$ in kg (e.g. $1.0 \times 10^{-6}\,\text{kg}$).
    pub diaphragm_mass_kg: f64,
    /// Diaphragm suspension mechanical stiffness $k$ in $\text{N/m}$ (e.g. $1500\,\text{N/m}$).
    pub stiffness_n_per_m: f64,
    /// Diaphragm mechanical damping coefficient $c_d$ in $\text{N}\cdot\text{s/m}$.
    pub damping_n_s_per_m: f64,
    /// DC polarization bias voltage $V_{bias}$ in Volts (e.g. $48.0\,\text{V}$).
    pub bias_voltage_v: f64,
    /// Polar directivity pattern.
    pub polar_pattern: MicrophonePolarPattern,
}

impl CondenserMicrophone {
    /// Creates a standard 1/2-inch studio condenser microphone capsule with specified polar pattern.
    pub fn new_studio_capsule(
        position: Vector3D,
        axis: Vector3D,
        polar_pattern: MicrophonePolarPattern,
    ) -> Self {
        Self {
            position,
            axis: axis.normalize(),
            diaphragm_area: 1.2e-4,    // ~1/2 inch circular diameter
            plate_spacing_m: 20.0e-6,  // 20 micrometers
            diaphragm_mass_kg: 1.0e-6, // 1 mg
            stiffness_n_per_m: 12000.0,
            damping_n_s_per_m: 0.04,
            bias_voltage_v: 48.0,
            polar_pattern,
        }
    }

    /// Natural mechanical undamped resonant frequency $f_0 = \frac{1}{2\pi}\sqrt{k/m}$ in Hz.
    pub fn resonant_frequency_hz(&self) -> f64 {
        (self.stiffness_n_per_m / self.diaphragm_mass_kg).sqrt() / (2.0 * PI)
    }

    /// Mechanical damping ratio $\zeta = \frac{c_d}{2\sqrt{m k}}$.
    pub fn damping_ratio(&self) -> f64 {
        self.damping_n_s_per_m / (2.0 * (self.diaphragm_mass_kg * self.stiffness_n_per_m).sqrt())
    }

    /// Nominal rest capacitance $C_0 = \frac{\epsilon_0 A_d}{d_0}$ in Farads.
    pub fn rest_capacitance_f(&self) -> f64 {
        (EPSILON_0 * self.diaphragm_area) / self.plate_spacing_m
    }

    /// Evaluates steady-state diaphragm mechanical displacement amplitude $X_0$ (meters)
    /// driven by harmonic acoustic pressure amplitude $P_a$ at frequency $f$:
    ///
    /// $$X_0 = \frac{A_d P_a}{\sqrt{(k - m\omega^2)^2 + (c_d \omega)^2}}$$
    pub fn diaphragm_displacement_amplitude(&self, pressure_pa: f64, frequency_hz: f64) -> f64 {
        let omega = 2.0 * PI * frequency_hz.max(1.0);
        let m = self.diaphragm_mass_kg;
        let k = self.stiffness_n_per_m;
        let cd = self.damping_n_s_per_m;

        let driving_force = self.diaphragm_area * pressure_pa;
        let denom_sq = (k - m * omega * omega).powi(2) + (cd * omega).powi(2);
        driving_force / denom_sq.sqrt().max(1e-12)
    }

    /// Evaluates microphone output analog voltage signal $v_{out}(t)$ given an incident acoustic wave.
    pub fn transduce(
        &self,
        incident_direction: Vector3D,
        instantaneous_pressure_pa: f64,
        frequency_hz: f64,
    ) -> MicrophoneSignal {
        // Incident angle relative to microphone main axis:
        let inc_norm = incident_direction.normalize();
        let cos_angle = self.axis.dot(&inc_norm).clamp(-1.0, 1.0);
        let angle_rad = cos_angle.acos();

        let directivity = self.polar_pattern.directivity(angle_rad);
        let effective_pressure = instantaneous_pressure_pa * directivity;

        // Mechanical displacement:
        let x_amp = self.diaphragm_displacement_amplitude(effective_pressure.abs(), frequency_hz);
        let x_t = if effective_pressure >= 0.0 {
            x_amp
        } else {
            -x_amp
        };

        // Capacitance modulation: C(t) = eps0 * A / (d0 - x(t))
        let c_t = (EPSILON_0 * self.diaphragm_area) / (self.plate_spacing_m - x_t).max(1e-9);

        // Output voltage: v_out = V_bias * (x(t) / d0)
        let v_out = self.bias_voltage_v * (x_t / self.plate_spacing_m);

        MicrophoneSignal {
            voltage_v: v_out,
            diaphragm_displacement_m: x_t,
            instantaneous_capacitance_f: c_t,
            polar_directivity_factor: directivity,
        }
    }
}

/// Physical Piezoelectric Microphone / Contact Transducer.
///
/// Operates via the direct piezoelectric effect: $V(t) = g_{33} \cdot P(t) \cdot t_h$.
#[derive(Debug, Clone, PartialEq)]
pub struct PiezoelectricMicrophone {
    /// Transducer center position in 3D space.
    pub position: Vector3D,
    /// Orientation axis.
    pub axis: Vector3D,
    /// Piezoelectric element thickness $t_h$ in meters (e.g. $1.0\,\text{mm}$).
    pub crystal_thickness_m: f64,
    /// Piezoelectric element surface area $A_c$ in $\text{m}^2$.
    pub crystal_area_m2: f64,
    /// Piezoelectric voltage coefficient $g_{33}$ in $\text{V}\cdot\text{m/N}$ (e.g. $0.025\,\text{V}\cdot\text{m/N}$ for PZT-5A).
    pub g33_coefficient: f64,
    /// Internal element capacitance $C_p$ in Farads.
    pub internal_capacitance_f: f64,
    /// Polar directivity pattern.
    pub polar_pattern: MicrophonePolarPattern,
}

impl PiezoelectricMicrophone {
    /// Creates a PZT-5A piezoelectric acoustic transducer.
    pub fn new_pzt5a(position: Vector3D, axis: Vector3D) -> Self {
        let th = 1.0e-3; // 1 mm
        let area = 1.0e-4; // 1 cm^2
        let eps_r = 1700.0;
        let c_p = (eps_r * EPSILON_0 * area) / th;

        Self {
            position,
            axis: axis.normalize(),
            crystal_thickness_m: th,
            crystal_area_m2: area,
            g33_coefficient: 0.025,
            internal_capacitance_f: c_p,
            polar_pattern: MicrophonePolarPattern::Cardioid,
        }
    }

    /// Evaluates open-circuit output voltage generated by acoustic pressure stress.
    pub fn transduce(&self, incident_direction: Vector3D, instantaneous_pressure_pa: f64) -> f64 {
        let inc_norm = incident_direction.normalize();
        let cos_angle = self.axis.dot(&inc_norm).clamp(-1.0, 1.0);
        let directivity = self.polar_pattern.directivity(cos_angle.acos());

        self.g33_coefficient * instantaneous_pressure_pa * self.crystal_thickness_m * directivity
    }
}

/// Output measurement from physical microphone transduction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicrophoneSignal {
    /// Analog output signal voltage in Volts.
    pub voltage_v: f64,
    /// Diaphragm mechanical displacement from equilibrium in meters.
    pub diaphragm_displacement_m: f64,
    /// Instantaneous parallel-plate capacitance in Farads.
    pub instantaneous_capacitance_f: f64,
    /// Directivity angular weighting factor.
    pub polar_directivity_factor: f64,
}
