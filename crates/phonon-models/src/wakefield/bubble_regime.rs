//! Relativistic blowout / bubble regime: spherical electron cavitation,
//! multi-gigavolt accelerating gradients, dephasing lengths, and maximum energy gain.

use crate::wakefield::{LaserPulseParams, PlasmaChannelParams};

/// Relativistic bubble / blowout regime wakefield model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BubbleRegime {
    /// Laser pulse parameters.
    pub laser: LaserPulseParams,
    /// Plasma channel parameters.
    pub plasma: PlasmaChannelParams,
}

impl BubbleRegime {
    /// Constructs a bubble regime accelerator model.
    pub fn new(laser: LaserPulseParams, plasma: PlasmaChannelParams) -> Self {
        Self { laser, plasma }
    }

    /// Verifies if laser parameters drive the non-linear bubble/blowout regime ($a_0 > 2.0$).
    pub fn is_bubble_regime(&self) -> bool {
        self.laser.a0 >= 2.0
    }

    /// Cavitation bubble radius $R_b \approx 2 \sqrt{a_0} k_p^{-1}$ in meters:
    pub fn bubble_radius_m(&self) -> f64 {
        let kp = self.plasma.plasma_wavenumber_m1();
        2.0 * self.laser.a0.sqrt() / kp.max(1e-6)
    }

    /// Peak longitudinal accelerating field $E_{z,peak} \approx \frac{1}{2} E_{wb} k_p R_b$ in V/m (GV/m):
    pub fn peak_accelerating_field_v_per_m(&self) -> f64 {
        let e_wb = self.plasma.wavebreaking_field_v_per_m();
        let kp = self.plasma.plasma_wavenumber_m1();
        let rb = self.bubble_radius_m();
        0.5 * e_wb * kp * rb
    }

    /// Longitudinal accelerating electric field $E_z(\xi)$ at co-moving distance $\xi = z - c t$ inside bubble:
    /// $$E_z(\xi) = E_{z,peak} \cdot \frac{\xi}{R_b}$$
    pub fn longitudinal_field_v_per_m(&self, xi_m: f64) -> f64 {
        let rb = self.bubble_radius_m();
        let e_peak = self.peak_accelerating_field_v_per_m();
        let xi_clamped = xi_m.clamp(-rb, rb);
        e_peak * (xi_clamped / rb.max(1e-12))
    }

    /// Transverse focusing field $E_r - c B_\theta$ in V/m at radial offset $r$:
    /// $$F_r / e = E_r - c B_\theta = -\frac{m_e \omega_p^2}{2 e} r = -\frac{1}{2} E_{wb} k_p r$$
    pub fn transverse_focusing_field_v_per_m(&self, radius_m: f64) -> f64 {
        let e_wb = self.plasma.wavebreaking_field_v_per_m();
        let kp = self.plasma.plasma_wavenumber_m1();
        -0.5 * e_wb * kp * radius_m
    }

    /// Electron dephasing length $L_d$ in meters:
    /// $$L_d \approx \frac{4}{3} \frac{\omega_0^2}{\omega_p^2} \frac{\sqrt{a_0}}{k_p}$$
    pub fn dephasing_length_m(&self) -> f64 {
        let omega0 = self.laser.angular_frequency_rad_per_s();
        let omegap = self.plasma.plasma_frequency_rad_per_s();
        let kp = self.plasma.plasma_wavenumber_m1();
        let density_ratio = (omega0 / omegap.max(1.0)).powi(2);
        (4.0 / 3.0) * density_ratio * self.laser.a0.sqrt() / kp.max(1e-6)
    }

    /// Laser pump depletion length $L_{dep}$ in meters:
    /// $$L_{dep} \approx \frac{\omega_0^2}{\omega_p^2} c \tau_{laser}$$
    pub fn depletion_length_m(&self) -> f64 {
        let c = 2.997_924_58e8;
        let omega0 = self.laser.angular_frequency_rad_per_s();
        let omegap = self.plasma.plasma_frequency_rad_per_s();
        let density_ratio = (omega0 / omegap.max(1.0)).powi(2);
        density_ratio * c * self.laser.pulse_duration_fwhm_s
    }

    /// Maximum attainable electron beam energy gain $\Delta W_{max} = e E_z L_d$ in MeV:
    /// $$\Delta W_{max} \approx \frac{2}{3} m_e c^2 \left( \frac{\omega_0}{\omega_p} \right)^2 a_0$$
    pub fn max_energy_gain_mev(&self) -> f64 {
        let me_c2_mev = 0.510_998_95; // MeV
        let omega0 = self.laser.angular_frequency_rad_per_s();
        let omegap = self.plasma.plasma_frequency_rad_per_s();
        let ratio = (omega0 / omegap.max(1.0)).powi(2);
        (2.0 / 3.0) * me_c2_mev * ratio * self.laser.a0
    }
}
