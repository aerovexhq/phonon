//! Spin-Torque and Spin-Orbit Torque Nano-Oscillators (STNO / SHNO):
//! Slonczewski spin-transfer torque, Spin Hall effect SOT, non-linear auto-oscillator theory,
//! and supercritical Hopf bifurcation threshold currents.

use crate::stno::giant_spin::HBAR;

/// Physical and geometric parameters of a spin-torque / spin-orbit torque nano-oscillator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StnoParams {
    /// Saturation magnetization M_s in A/m (typically 8e5 A/m for Permalloy or CoFeB).
    pub ms_a_per_m: f64,
    /// Free-layer active volume V in m^3.
    pub volume_m3: f64,
    /// Free-layer magnetic thickness t_F in meters.
    pub thickness_m: f64,
    /// Dimensionless Gilbert damping parameter alpha_G (typically 0.005 - 0.02).
    pub gilbert_damping: f64,
    /// Effective spin polarization efficiency eta (0 < eta <= 1.0).
    pub spin_polarization: f64,
    /// Spin Hall angle theta_SH (for SHNO devices with heavy metal underlayer Pt/W/Ta).
    pub spin_hall_angle: f64,
    /// Gyromagnetic ratio gamma_0 in rad / (s * T) (typically 1.760859644e11 rad/(s*T)).
    pub gyromagnetic_ratio_rad_per_s_t: f64,
    /// Non-linear frequency shift coefficient N in rad/(s * power).
    pub nonlinear_frequency_shift_rad_per_s: f64,
    /// Non-linear damping parameter Q (typically ~1.0 - 3.0).
    pub nonlinear_damping_q: f64,
}

impl StnoParams {
    /// Constructs STNO / SHNO parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ms_a_per_m: f64,
        volume_m3: f64,
        thickness_m: f64,
        gilbert_damping: f64,
        spin_polarization: f64,
        spin_hall_angle: f64,
        nonlinear_frequency_shift_rad_per_s: f64,
        nonlinear_damping_q: f64,
    ) -> Self {
        Self {
            ms_a_per_m,
            volume_m3,
            thickness_m,
            gilbert_damping,
            spin_polarization,
            spin_hall_angle,
            gyromagnetic_ratio_rad_per_s_t: 1.760_859_644e11,
            nonlinear_frequency_shift_rad_per_s,
            nonlinear_damping_q,
        }
    }

    /// Standard CoFeB/MgO/CoFeB magnetic tunnel junction STNO (nanopillar radius 40 nm, thickness 2 nm).
    pub fn standard_cofeb_stno() -> Self {
        let radius: f64 = 40.0e-9;
        let thickness: f64 = 2.0e-9;
        let volume = std::f64::consts::PI * radius.powi(2) * thickness;
        Self::new(
            8.0e5,
            volume,
            thickness,
            0.01,
            0.65,
            0.0,
            2.0 * std::f64::consts::PI * 1.5e9, // 1.5 GHz non-linear shift
            1.5,
        )
    }

    /// Standard Pt/Py Spin Hall Nano-Oscillator (SHNO) with theta_SH = 0.08.
    pub fn standard_pt_py_shno() -> Self {
        let width = 100.0e-9;
        let length = 100.0e-9;
        let thickness = 5.0e-9;
        let volume = width * length * thickness;
        Self::new(
            8.6e5,
            volume,
            thickness,
            0.015,
            0.0,
            0.08,
            2.0 * std::f64::consts::PI * 2.0e9,
            2.0,
        )
    }

    /// Slonczewski spin-transfer torque coefficient sigma in (rad * A^-1 * s^-1):
    /// $$\sigma = \frac{\hbar \eta}{2 e M_s V}$$
    pub fn slonczewski_coefficient(&self) -> f64 {
        let e = 1.602_176_634e-19;
        (HBAR * self.spin_polarization) / (2.0 * e * self.ms_a_per_m * self.volume_m3)
    }

    /// Supercritical Hopf bifurcation threshold current I_th in Amperes:
    /// $$I_{th} = \frac{2 e \alpha_G}{\hbar \eta} M_s V (H_{ext} + 2\pi M_{eff}) \mu_0$$
    pub fn threshold_current_amperes(&self, h_ext_a_per_m: f64) -> f64 {
        let e = 1.602_176_634e-19;
        let mu0 = 4.0 * std::f64::consts::PI * 1.0e-7;
        let total_field = h_ext_a_per_m + 0.5 * self.ms_a_per_m;
        let factor = (2.0 * e * self.gilbert_damping) / (HBAR * self.spin_polarization.max(1e-4));
        factor * self.ms_a_per_m * self.volume_m3 * total_field * mu0
    }

    /// Free oscillation angular frequency omega(p) as a function of normalized precession power p:
    /// $$\omega(p) = \omega_0 + N p$$
    pub fn oscillation_frequency_rad_per_s(&self, omega0_rad_per_s: f64, power_p: f64) -> f64 {
        omega0_rad_per_s + self.nonlinear_frequency_shift_rad_per_s * power_p
    }

    /// Linear FMR frequency in uniform in-plane external magnetic field H_ext (Kittel formula):
    /// $$\omega_0 = \gamma_0 \mu_0 \sqrt{H_{ext} (H_{ext} + M_s)}$$
    pub fn kittel_fmr_frequency_rad_per_s(&self, h_ext_a_per_m: f64) -> f64 {
        let mu0 = 4.0 * std::f64::consts::PI * 1.0e-7;
        let b_ext = mu0 * h_ext_a_per_m;
        let b_eff = mu0 * (h_ext_a_per_m + self.ms_a_per_m);
        self.gyromagnetic_ratio_rad_per_s_t * (b_ext * b_eff).max(0.0).sqrt()
    }
}
