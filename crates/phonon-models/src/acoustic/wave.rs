//! 3D Acoustic Wave Equations, Spherical Propagation, Doppler Shifts & Pressure Metrics
//!
//! Models 3D spherical acoustic pressure wave propagation $P(\mathbf{r}, t)$,
//! acoustic intensity $I$, Sound Pressure Level (SPL), moving source/receiver Doppler
//! frequency shifts, and strict vacuum isolation.

use super::medium::{AcousticMedium, MediumType, P_REF_AIR};
use crate::em::Vector3D;
use std::f64::consts::PI;

/// Kinematic Doppler frequency shift calculation result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticDopplerResult {
    /// Apparent observed frequency in Hz.
    pub observed_frequency_hz: f64,
    /// Radial velocity of source relative to line of sight (m/s). Positive = moving away.
    pub v_source_radial: f64,
    /// Radial velocity of observer relative to line of sight (m/s). Positive = moving towards source.
    pub v_observer_radial: f64,
    /// Mach number of source relative to local speed of sound ($M = v_s / c_s$).
    pub source_mach: f64,
    /// Flag indicating supersonic flight condition ($M \ge 1.0$).
    pub is_supersonic: bool,
}

/// Point Sound Source with 3D kinematic position, velocity, and emitted sound power.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticSource {
    /// 3D position vector in Cartesian meters.
    pub position: Vector3D,
    /// 3D velocity vector in m/s.
    pub velocity: Vector3D,
    /// Fundamental acoustic frequency in Hz.
    pub frequency_hz: f64,
    /// Total radiated sound power $W$ in Watts.
    pub sound_power_watts: f64,
}

impl AcousticSource {
    /// Creates a new acoustic point source.
    pub fn new(
        position: Vector3D,
        velocity: Vector3D,
        frequency_hz: f64,
        sound_power_watts: f64,
    ) -> Self {
        Self {
            position,
            velocity,
            frequency_hz,
            sound_power_watts: sound_power_watts.max(1e-12),
        }
    }

    /// Evaluates reference acoustic pressure amplitude $P_0$ at 1 meter in free space:
    /// $P_0 = \sqrt{\frac{W \cdot \rho \cdot c_s}{2\pi}}$ (hemispherical) or $\sqrt{\frac{W \cdot \rho \cdot c_s}{4\pi}}$ (spherical).
    pub fn reference_pressure_at_1m(&self, medium: &AcousticMedium) -> f64 {
        if medium.medium_type == MediumType::Vacuum {
            return 0.0;
        }
        let z = medium.acoustic_impedance;
        // Spherical monopole: I = W / (4 * pi * r^2) = P_rms^2 / z => P_0 = sqrt(2 * P_rms^2) = sqrt(W * z / (2 * pi))
        (self.sound_power_watts * z / (2.0 * PI)).sqrt()
    }
}

/// Acoustic Observer / Listener with 3D kinematic state.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticObserver {
    /// 3D position vector in Cartesian meters.
    pub position: Vector3D,
    /// 3D velocity vector in m/s.
    pub velocity: Vector3D,
}

impl AcousticObserver {
    /// Creates a new acoustic observer.
    pub fn new(position: Vector3D, velocity: Vector3D) -> Self {
        Self { position, velocity }
    }
}

/// Evaluates kinematic acoustic Doppler frequency shift for moving source and moving observer:
///
/// $$f_{obs} = f_0 \frac{c_s - \mathbf{v}_o \cdot \hat{\mathbf{r}}}{c_s - \mathbf{v}_s \cdot \hat{\mathbf{r}}}$$
///
/// where $\hat{\mathbf{r}} = \frac{\mathbf{r}_o - \mathbf{r}_s}{\|\mathbf{r}_o - \mathbf{r}_s\|}$ is the unit vector from source to observer.
pub fn compute_acoustic_doppler(
    source: &AcousticSource,
    observer: &AcousticObserver,
    medium: &AcousticMedium,
) -> AcousticDopplerResult {
    let cs = medium.speed_of_sound;
    if cs <= 1e-6 || medium.medium_type == MediumType::Vacuum {
        return AcousticDopplerResult {
            observed_frequency_hz: 0.0,
            v_source_radial: 0.0,
            v_observer_radial: 0.0,
            source_mach: 0.0,
            is_supersonic: false,
        };
    }

    let diff = observer.position - source.position;
    let dist = diff.norm();
    let r_hat = if dist > 1e-6 {
        diff.normalize()
    } else {
        Vector3D::new(1.0, 0.0, 0.0)
    };

    // Component of velocity along line of sight (from source to observer):
    let vs_r = source.velocity.dot(&r_hat);
    let vo_r = observer.velocity.dot(&r_hat);

    let source_mach = source.velocity.norm() / cs;
    let is_supersonic = source_mach >= 1.0;

    // Denominator: c_s - v_s_r
    let denom = (cs - vs_r).max(1e-3);
    // Numerator: c_s - v_o_r
    let num = (cs - vo_r).max(0.0);

    let observed_f = (source.frequency_hz * (num / denom)).max(0.0);

    AcousticDopplerResult {
        observed_frequency_hz: observed_f,
        v_source_radial: vs_r,
        v_observer_radial: vo_r,
        source_mach,
        is_supersonic,
    }
}

/// Spherical Acoustic Wave evaluation parameters and metrics.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticFieldPoint {
    /// Distance from source to field point (m).
    pub distance_m: f64,
    /// Instantaneous acoustic pressure $p(t)$ in Pascals.
    pub instantaneous_pressure_pa: f64,
    /// Root-mean-square pressure amplitude $P_{rms}$ in Pascals.
    pub rms_pressure_pa: f64,
    /// Sound Pressure Level (SPL) in dB re $20\,\mu\text{Pa}$.
    pub spl_db: f64,
    /// Acoustic sound intensity $I$ in $\text{W/m}^2$.
    pub intensity_w_per_m2: f64,
}

/// Evaluates the acoustic field at an observer location from a sound source in a given medium.
pub fn evaluate_acoustic_field(
    source: &AcousticSource,
    observer: &AcousticObserver,
    medium: &AcousticMedium,
    time_s: f64,
    wall_attenuation_factor: f64,
) -> AcousticFieldPoint {
    if medium.medium_type == MediumType::Vacuum {
        // Hard vacuum isolation: zero sound transmission
        return AcousticFieldPoint {
            distance_m: (observer.position - source.position).norm(),
            instantaneous_pressure_pa: 0.0,
            rms_pressure_pa: 0.0,
            spl_db: -100.0,
            intensity_w_per_m2: 0.0,
        };
    }

    let diff = observer.position - source.position;
    let distance = diff.norm().max(0.05); // Avoid singularity at r=0

    let p0 = source.reference_pressure_at_1m(medium);
    let alpha_db_m = medium.atmospheric_absorption_db_per_m(source.frequency_hz);
    let atm_loss_factor = 10.0f64.powf(-alpha_db_m * distance / 20.0);

    // Doppler shifted frequency and delay
    let doppler = compute_acoustic_doppler(source, observer, medium);
    let effective_f = doppler.observed_frequency_hz;
    let delay_s = distance / medium.speed_of_sound;

    // Pressure amplitude with spherical 1/r spreading, atmospheric absorption, and wall loss:
    let p_amp = (p0 / distance) * atm_loss_factor * wall_attenuation_factor.clamp(0.0, 1.0);
    let phase = 2.0 * PI * effective_f * (time_s - delay_s);
    let instantaneous = p_amp * phase.cos();
    let p_rms = p_amp / std::f64::consts::SQRT_2;

    let spl = if p_rms > 1e-12 {
        20.0 * (p_rms / P_REF_AIR).log10()
    } else {
        -100.0
    };

    let intensity = (p_rms * p_rms) / medium.acoustic_impedance.max(1e-6);

    AcousticFieldPoint {
        distance_m: distance,
        instantaneous_pressure_pa: instantaneous,
        rms_pressure_pa: p_rms,
        spl_db: spl,
        intensity_w_per_m2: intensity,
    }
}
