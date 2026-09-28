//! Relativistic laser envelope parameters: normalized vector potential a0,
//! ponderomotive potentials, and Gaussian focal spot profiles.

use std::f64::consts::PI;

/// Physical parameters of an ultra-short, ultra-intense laser pulse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaserPulseParams {
    /// Central wavelength $\lambda_0$ in meters (typically 0.8 um for Ti:Sapphire).
    pub wavelength_m: f64,
    /// Pulse duration (Full-Width at Half-Maximum intensity) $\tau_{fwhm}$ in seconds (typically 20-30 fs).
    pub pulse_duration_fwhm_s: f64,
    /// Focal spot waist radius $w_0$ in meters (typically 10-30 um).
    pub waist_radius_m: f64,
    /// Dimensionless normalized vector potential $a_0 = e A_0 / (m_e c)$.
    pub a0: f64,
}

impl LaserPulseParams {
    /// Constructs laser pulse parameters from wavelength, duration, waist, and peak $a_0$.
    pub fn new(
        wavelength_m: f64,
        pulse_duration_fwhm_s: f64,
        waist_radius_m: f64,
        a0: f64,
    ) -> Self {
        Self {
            wavelength_m,
            pulse_duration_fwhm_s,
            waist_radius_m,
            a0,
        }
    }

    /// Standard Ti:Sapphire laser baseline ($0.8\,\mu\text{m}$, 25 fs, $w_0 = 15\,\mu\text{m}$, $a_0 = 3.0$).
    pub fn standard_tisapphire() -> Self {
        Self::new(0.8e-6, 25.0e-15, 15.0e-6, 3.0)
    }

    /// Optical angular frequency $\omega_0 = 2\pi c / \lambda_0$ in rad/s.
    pub fn angular_frequency_rad_per_s(&self) -> f64 {
        let c = 2.997_924_58e8;
        2.0 * PI * c / self.wavelength_m.max(1e-12)
    }

    /// Peak electric field $E_0 = a_0 m_e \omega_0 c / e$ in V/m.
    pub fn peak_electric_field_v_per_m(&self) -> f64 {
        let c = 2.997_924_58e8;
        let me = 9.109_383_7e-31;
        let e = 1.602_176_634e-19;
        let omega0 = self.angular_frequency_rad_per_s();
        self.a0 * me * omega0 * c / e
    }

    /// Peak laser intensity $I_0 = \frac{1}{2} \epsilon_0 c E_0^2$ in $\text{W/cm}^2$.
    pub fn peak_intensity_w_per_cm2(&self) -> f64 {
        let eps0 = 8.854_187_812_8e-12;
        let c = 2.997_924_58e8;
        let e0 = self.peak_electric_field_v_per_m();
        let i0_w_m2 = 0.5 * eps0 * c * e0.powi(2);
        i0_w_m2 * 1.0e-4 // W/cm^2
    }

    /// Total pulse energy $\mathcal{E}_L$ in Joules:
    /// $$\mathcal{E}_L \approx I_0 \cdot \frac{\pi w_0^2}{2} \cdot \sqrt{\frac{\pi}{2\ln 2}} \tau_{fwhm}$$
    pub fn pulse_energy_joules(&self) -> f64 {
        let eps0 = 8.854_187_812_8e-12;
        let c = 2.997_924_58e8;
        let e0 = self.peak_electric_field_v_per_m();
        let i0 = 0.5 * eps0 * c * e0.powi(2); // W/m^2
        let area = 0.5 * PI * self.waist_radius_m.powi(2);
        let tau_eff = (PI / (2.0 * 2.0f64.ln())).sqrt() * self.pulse_duration_fwhm_s;
        i0 * area * tau_eff
    }

    /// Local normalized envelope amplitude $a(r, \xi)$ at radial position $r$ and co-moving coordinate $\xi = z - ct$:
    /// $$a(r, \xi) = a_0 \exp\left( -\frac{r^2}{w_0^2} \right) \exp\left( -\frac{2\ln 2 \cdot \xi^2}{c^2 \tau_{fwhm}^2} \right)$$
    pub fn local_envelope(&self, radius_m: f64, xi_m: f64) -> f64 {
        let c = 2.997_924_58e8;
        let r_term = -(radius_m / self.waist_radius_m.max(1e-12)).powi(2);
        let z_sigma = c * self.pulse_duration_fwhm_s / (2.0 * (2.0f64.ln()).sqrt());
        let z_term = -0.5 * (xi_m / z_sigma.max(1e-12)).powi(2);
        self.a0 * (r_term + z_term).exp()
    }

    /// Normalized ponderomotive potential $\Phi_p = \sqrt{1 + a^2 / 2} - 1$.
    pub fn ponderomotive_potential(&self, radius_m: f64, xi_m: f64) -> f64 {
        let a = self.local_envelope(radius_m, xi_m);
        (1.0 + 0.5 * a.powi(2)).sqrt() - 1.0
    }
}
