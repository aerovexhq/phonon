//! Spin Superfluidity & Long-Range Non-Local Spin Transport Models.
//!
//! Formulates easy-plane magnetic anisotropy, spin superfluid velocity v_s,
//! Landau critical velocity v_c, algebraic (1/L) vs exponential (exp(-L/lambda))
//! spin transmission, and non-local Pt injector/detector voltages.

use crate::magnon_bec::yig_magnon_film::{YigMagnonFilm, GAMMA_E};

/// Physical parameters of the spin superfluid channel.
#[derive(Debug, Clone, PartialEq)]
pub struct SpinSuperfluidChannel {
    /// Base YIG film properties.
    pub film: YigMagnonFilm,
    /// Easy-plane magnetic anisotropy energy density $K_u$ in J/m$^3$ (default 1.2e4 J/m^3).
    pub easy_plane_anisotropy_j_per_m3: f64,
    /// Channel length $L$ in meters (default 20.0 um).
    pub channel_length_m: f64,
    /// Channel width $w$ in meters (default 2.0 um).
    pub channel_width_m: f64,
    /// Diffusive spin diffusion length $\lambda_s$ in meters (default 5.0 um).
    pub diffusive_spin_length_m: f64,
}

impl Default for SpinSuperfluidChannel {
    fn default() -> Self {
        Self {
            film: YigMagnonFilm::default(),
            easy_plane_anisotropy_j_per_m3: 1.2e4,
            channel_length_m: 20.0e-6,
            channel_width_m: 2.0e-6,
            diffusive_spin_length_m: 5.0e-6,
        }
    }
}

/// Parameters for heavy metal (e.g. Platinum) injector and detector electrodes.
#[derive(Debug, Clone, PartialEq)]
pub struct HeavyMetalElectrode {
    /// Spin Hall angle $\theta_{SH}$ (dimensionless, default 0.08 for Pt).
    pub spin_hall_angle: f64,
    /// Electrical resistivity $\rho_{HM}$ in $\Omega\cdot$m (default 2.0e-7 Ohm*m).
    pub resistivity_ohm_m: f64,
    /// Electrode width in meters (default 200 nm).
    pub width_m: f64,
    /// Electrode thickness in meters (default 10 nm).
    pub thickness_m: f64,
    /// Interfacial spin-mixing conductance $g_{\uparrow\downarrow}$ in m$^{-2}$ (default 1.2e19 m^-2).
    pub spin_mixing_conductance_per_m2: f64,
}

impl Default for HeavyMetalElectrode {
    fn default() -> Self {
        Self {
            spin_hall_angle: 0.08,
            resistivity_ohm_m: 2.0e-7,
            width_m: 200.0e-9,
            thickness_m: 10.0e-9,
            spin_mixing_conductance_per_m2: 1.2e19,
        }
    }
}

impl SpinSuperfluidChannel {
    /// Landau critical velocity $v_c = \frac{\gamma \sqrt{2 A_{ex} K_u}}{M_s}$ in m/s.
    /// Exceeding this velocity triggers phase slips (vortex nucleation) destroying superfluidity.
    pub fn landau_critical_velocity_m_per_s(&self) -> f64 {
        let a_ex = self.film.exchange_stiffness_j_per_m;
        let ku = self.easy_plane_anisotropy_j_per_m3;
        let ms = self.film.saturation_magnetization_a_per_m;
        let s_density = ms / GAMMA_E;
        (2.0 * a_ex * ku).sqrt() / s_density
    }

    /// Spin superfluid velocity $v_s = \frac{A_{ex}}{s} \nabla \phi = \frac{\gamma A_{ex}}{M_s} \nabla \phi$ in m/s
    /// given phase gradient $d\phi/dx$.
    #[inline]
    pub fn superfluid_velocity_m_per_s(&self, phase_gradient_rad_per_m: f64) -> f64 {
        let a_ex = self.film.exchange_stiffness_j_per_m;
        let ms = self.film.saturation_magnetization_a_per_m;
        let s_density = ms / GAMMA_E;
        (a_ex / s_density) * phase_gradient_rad_per_m
    }

    /// Dissipationless spin current density $J_s = A_{ex} \nabla \phi$ in J/m$^2$.
    #[inline]
    pub fn spin_current_density_j_per_m2(&self, phase_gradient_rad_per_m: f64) -> f64 {
        self.film.exchange_stiffness_j_per_m * phase_gradient_rad_per_m
    }

    /// Spin transmission factor for normal diffusive magnons: $T_{diff} = \exp(-L / \lambda_s)$.
    #[inline]
    pub fn diffusive_transmission_factor(&self, length_m: f64) -> f64 {
        (-length_m / self.diffusive_spin_length_m).exp()
    }

    /// Spin transmission factor for spin superfluidity: $T_{super} = \frac{1}{1 + L / L_0}$.
    /// Demonstrates long-range algebraic $1/L$ scaling without exponential attenuation.
    #[inline]
    pub fn superfluid_transmission_factor(&self, length_m: f64) -> f64 {
        let l0 = 50.0e-6; // Characteristic hydrodynamic damping length 50 um
        1.0 / (1.0 + length_m / l0)
    }

    /// Transmission ratio $T_{superfluid} / T_{diffusive}$ proving long-range transport advantage.
    pub fn transmission_advantage_ratio(&self, length_m: f64) -> f64 {
        let t_super = self.superfluid_transmission_factor(length_m);
        let t_diff = self.diffusive_transmission_factor(length_m).max(1e-30);
        t_super / t_diff
    }

    /// Non-local ISHE detector voltage in Volts for a given injector current $I_{inj}$ in Amperes.
    pub fn non_local_voltage_volts(
        &self,
        injector_current_a: f64,
        electrode: &HeavyMetalElectrode,
        is_superfluid: bool,
    ) -> f64 {
        let theta_sh = electrode.spin_hall_angle;
        let rho = electrode.resistivity_ohm_m;
        let w = electrode.width_m;
        let t = electrode.thickness_m;

        // Injected spin current via Spin Hall effect
        let spin_current_inj = theta_sh * injector_current_a;

        // Transmission across channel
        let transmission = if is_superfluid {
            self.superfluid_transmission_factor(self.channel_length_m)
        } else {
            self.diffusive_transmission_factor(self.channel_length_m)
        };

        let spin_current_det = spin_current_inj * transmission;

        // Transduced non-local voltage via Inverse Spin Hall Effect:
        // V_nl = theta_SH * rho * (I_s,det / (w * t)) * length_contact
        let contact_len = self.channel_width_m;
        theta_sh * rho * (spin_current_det / (w * t)) * contact_len * 1e-3 // scaling factor
    }
}
