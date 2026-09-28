//! Plasma channel physics: plasma density, frequency, skin depth,
//! critical density, and parabolic optical guiding profiles.

use std::f64::consts::PI;

/// Physical parameters of an underdense plasma channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlasmaChannelParams {
    /// On-axis electron plasma density $n_0$ in $\text{m}^{-3}$ (typically $10^{24} - 10^{25}\,\text{m}^{-3}$).
    pub on_axis_density_m3: f64,
    /// Channel matched radius $r_{ch}$ in meters (typically matched to laser waist $w_0$).
    pub channel_radius_m: f64,
}

impl PlasmaChannelParams {
    /// Constructs a plasma channel parameter set.
    pub fn new(on_axis_density_m3: f64, channel_radius_m: f64) -> Self {
        Self {
            on_axis_density_m3,
            channel_radius_m,
        }
    }

    /// Standard baseline with $n_0 = 1.5 \times 10^{18}\,\text{cm}^{-3} = 1.5 \times 10^{24}\,\text{m}^{-3}$.
    pub fn standard_underdense() -> Self {
        Self::new(1.5e24, 15.0e-6)
    }

    /// Electron plasma frequency $\omega_p = \sqrt{n_0 e^2 / (\epsilon_0 m_e)}$ in rad/s.
    pub fn plasma_frequency_rad_per_s(&self) -> f64 {
        let eps0: f64 = 8.854_187_812_8e-12;
        let me: f64 = 9.109_383_7e-31;
        let e: f64 = 1.602_176_634e-19;
        (self.on_axis_density_m3 * e.powi(2) / (eps0 * me)).sqrt()
    }

    /// Plasma wavelength $\lambda_p = 2\pi c / \omega_p$ in meters.
    pub fn plasma_wavelength_m(&self) -> f64 {
        let c = 2.997_924_58e8;
        2.0 * PI * c / self.plasma_frequency_rad_per_s().max(1.0)
    }

    /// Plasma wavenumber $k_p = \omega_p / c = 2\pi / \lambda_p$ in $\text{m}^{-1}$.
    pub fn plasma_wavenumber_m1(&self) -> f64 {
        let c = 2.997_924_58e8;
        self.plasma_frequency_rad_per_s() / c
    }

    /// Plasma skin depth $c / \omega_p$ in meters.
    pub fn plasma_skin_depth_m(&self) -> f64 {
        1.0 / self.plasma_wavenumber_m1().max(1e-6)
    }

    /// Wavebreaking cold electric field limit $E_{wb} = m_e c \omega_p / e$ in V/m:
    /// $$E_{wb} \approx 96 \sqrt{n_0 [\text{cm}^{-3}]}\;\text{V/m}$$
    pub fn wavebreaking_field_v_per_m(&self) -> f64 {
        let c = 2.997_924_58e8;
        let me = 9.109_383_7e-31;
        let e = 1.602_176_634e-19;
        me * c * self.plasma_frequency_rad_per_s() / e
    }

    /// Critical plasma density $n_c = \epsilon_0 m_e \omega_0^2 / e^2$ in $\text{m}^{-3}$ for laser wavelength $\lambda_0$.
    pub fn critical_density_m3(&self, laser_wavelength_m: f64) -> f64 {
        let eps0: f64 = 8.854_187_812_8e-12;
        let c: f64 = 2.997_924_58e8;
        let me: f64 = 9.109_383_7e-31;
        let e: f64 = 1.602_176_634e-19;
        let omega0 = 2.0 * PI * c / laser_wavelength_m.max(1e-12);
        eps0 * me * omega0.powi(2) / e.powi(2)
    }

    /// Critical channel depth $\Delta n_c = 1 / (\pi r_e w_0^2)$ for matched optical guiding:
    pub fn matched_guiding_depth_m3(&self) -> f64 {
        let re = 2.817_940_322_7e-15; // Classical electron radius
        1.0 / (PI * re * self.channel_radius_m.powi(2)).max(1e-30)
    }

    /// Radial parabolic plasma density profile $n(r) = n_0 + \Delta n (r / w_0)^2$:
    pub fn local_density(&self, radius_m: f64) -> f64 {
        let delta_n = self.matched_guiding_depth_m3();
        let r_norm = radius_m / self.channel_radius_m.max(1e-12);
        self.on_axis_density_m3 + delta_n * r_norm.powi(2)
    }
}
