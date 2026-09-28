//! Multi-fluid extended magnetohydrodynamics, Spitzer resistivity, Hall effect,
//! neoclassical bootstrap currents, ICRF heating, and thermonuclear D-T fusion reactivity.

use super::constants::{
    DT_ALPHA_ENERGY_JOULES, DT_TOTAL_ENERGY_JOULES, ELECTRON_MASS, ELEMENTARY_CHARGE, KEV_TO_JOULES,
};

/// Multi-fluid plasma macroscopic physical state at a spatial continuum point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MhdFluidState {
    /// Electron number density $n_e$ ($m^{-3}$).
    pub electron_density: f64,
    /// Ion number density $n_i$ ($m^{-3}$).
    pub ion_density: f64,
    /// Electron temperature $T_e$ in keV.
    pub electron_temp_kev: f64,
    /// Ion temperature $T_i$ in keV.
    pub ion_temp_kev: f64,
    /// Plasma bulk fluid velocity $[u_R, u_\phi, u_Z]$ in m/s.
    pub velocity: [f64; 3],
    /// Magnetic field vector $[B_R, B_\phi, B_Z]$ in Tesla.
    pub magnetic_field: [f64; 3],
    /// Electric current density vector $[J_R, J_\phi, J_Z]$ in $A/m^2$.
    pub current_density: [f64; 3],
}

impl MhdFluidState {
    /// Total scalar kinetic plasma pressure $p = p_e + p_i = (n_e T_e + n_i T_i)$ in Pascals ($N/m^2$).
    pub fn total_pressure(&self) -> f64 {
        let p_e = self.electron_density * self.electron_temp_kev * KEV_TO_JOULES;
        let p_i = self.ion_density * self.ion_temp_kev * KEV_TO_JOULES;
        p_e + p_i
    }

    /// Mass density $\rho = n_e m_e + n_i m_i$ in $kg/m^3$.
    pub fn mass_density(&self, ion_mass: f64) -> f64 {
        self.electron_density * ELECTRON_MASS + self.ion_density * ion_mass
    }

    /// Spitzer parallel resistivity $\eta = 5.2 \times 10^{-5} \frac{Z_{eff} \ln\Lambda}{T_e^{3/2}}$ ($\Omega\cdot m$).
    pub fn spitzer_resistivity(&self, z_eff: f64) -> f64 {
        let t_ev = (self.electron_temp_kev * 1000.0).max(1.0);
        let coulomb_log = 17.0; // typical tokamak core value
        5.2e-5 * (z_eff * coulomb_log) / t_ev.powf(1.5)
    }

    /// Magnetic Lorentz force density $\mathbf{F}_L = \mathbf{J} \times \mathbf{B}$ in $N/m^3$.
    pub fn lorentz_force(&self) -> [f64; 3] {
        let j = self.current_density;
        let b = self.magnetic_field;
        [
            j[1] * b[2] - j[2] * b[1],
            j[2] * b[0] - j[0] * b[2],
            j[0] * b[1] - j[1] * b[0],
        ]
    }

    /// Hall electric field $\mathbf{E}_{Hall} = \frac{1}{n_e e} (\mathbf{J} \times \mathbf{B})$ in $V/m$.
    pub fn hall_electric_field(&self) -> [f64; 3] {
        let f_l = self.lorentz_force();
        let denom = (self.electron_density * ELEMENTARY_CHARGE).max(1e-12);
        [f_l[0] / denom, f_l[1] / denom, f_l[2] / denom]
    }

    /// Neoclassical bootstrap current density estimate:
    /// $$J_{boot} \approx - \frac{\sqrt{\epsilon}}{B_p} \frac{dp}{dr}$$
    pub fn bootstrap_current(dp_dr: f64, bp: f64, epsilon: f64) -> f64 {
        let safe_bp = bp.max(1e-5);
        let safe_eps = epsilon.clamp(0.0, 1.0);
        -(safe_eps.sqrt() / safe_bp) * dp_dr
    }
}

/// Deuterium-Tritium (D-T) thermonuclear fusion reactivity, power densities, and Lawson criterion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermonuclearFusion;

impl ThermonuclearFusion {
    /// Evaluates Maxwellian-averaged D-T fusion reactivity $\langle \sigma v \rangle_{DT}$ in $m^3/s$.
    ///
    /// Uses the parameterized Gamow-peak formula accurate for $1\text{ keV} \le T_i \le 100\text{ keV}$:
    /// $$\langle \sigma v \rangle_{DT} \approx 3.68 \times 10^{-18} T_i^{-2/3} \exp(-19.94 / T_i^{1/3})$$
    pub fn dt_reactivity(ti_kev: f64) -> f64 {
        let t = ti_kev.clamp(0.5, 150.0);
        let t13 = t.powf(1.0 / 3.0);
        let t23 = t13.powi(2);
        3.68e-18 * (1.0 / t23) * (-19.94 / t13).exp()
    }

    /// Alpha particle birth power density $P_\alpha = n_D n_T \langle \sigma v \rangle E_\alpha$ ($W/m^3$).
    pub fn alpha_power_density(n_deuterium: f64, n_tritium: f64, ti_kev: f64) -> f64 {
        let sigv = Self::dt_reactivity(ti_kev);
        n_deuterium * n_tritium * sigv * DT_ALPHA_ENERGY_JOULES
    }

    /// Total fusion power density $P_{fus} = n_D n_T \langle \sigma v \rangle Q_{DT}$ ($W/m^3$).
    pub fn total_fusion_power_density(n_deuterium: f64, n_tritium: f64, ti_kev: f64) -> f64 {
        let sigv = Self::dt_reactivity(ti_kev);
        n_deuterium * n_tritium * sigv * DT_TOTAL_ENERGY_JOULES
    }

    /// Bremsstrahlung radiation power loss density:
    /// $$P_{brem} \approx 5.35 \times 10^{-37} Z_{eff} n_e^2 \sqrt{T_e (\text{keV})} \quad (W/m^3)$$
    pub fn bremsstrahlung_power_loss(ne: f64, te_kev: f64, z_eff: f64) -> f64 {
        let t_kev = te_kev.max(0.001);
        5.35e-37 * z_eff * ne.powi(2) * t_kev.sqrt()
    }

    /// Lawson triple product $\Pi_L = n_e \cdot T_i \cdot \tau_E$ in units of $m^{-3} \cdot \text{keV} \cdot s$.
    pub fn lawson_triple_product(ne: f64, ti_kev: f64, tau_e_sec: f64) -> f64 {
        ne * ti_kev * tau_e_sec
    }

    /// Evaluates whether the plasma meets the ignition condition (Lawson criterion):
    /// $\Pi_L \ge 3.0 \times 10^{21}\text{ m}^{-3}\text{keV}\cdot\text{s}$.
    pub fn is_ignited(ne: f64, ti_kev: f64, tau_e_sec: f64) -> bool {
        Self::lawson_triple_product(ne, ti_kev, tau_e_sec) >= 3.0e21
    }

    /// Fusion gain factor $Q = \frac{P_{fus}}{P_{aux}}$ ($Q = 1$ is breakeven, $Q = \infty$ is ignition).
    pub fn fusion_energy_gain_q(p_fusion_watts: f64, p_aux_in_watts: f64) -> f64 {
        if p_aux_in_watts <= 1e-6 {
            f64::INFINITY
        } else {
            p_fusion_watts / p_aux_in_watts
        }
    }
}

/// External Ion Cyclotron Resonance Frequency (ICRF) heating deposition profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IcrfHeatingSource {
    /// Injected RF wave frequency in Hertz (typically $40 - 60\text{ MHz}$).
    pub frequency_hz: f64,
    /// Total coupled RF power in Watts.
    pub total_power_watts: f64,
    /// Major radius of fundamental cyclotron resonance layer $R_{res}$ (meters).
    pub resonance_major_radius: f64,
    /// Radial Gaussian deposition width $w$ (meters).
    pub deposition_width: f64,
}

impl IcrfHeatingSource {
    /// Creates a new ICRF heating source.
    pub fn new(
        frequency_hz: f64,
        total_power_watts: f64,
        resonance_major_radius: f64,
        deposition_width: f64,
    ) -> Self {
        Self {
            frequency_hz,
            total_power_watts,
            resonance_major_radius,
            deposition_width,
        }
    }

    /// Local power deposition density $p_{ICRF}(R)$ in $W/m^3$ assuming toroidal symmetry.
    pub fn power_density_at(&self, r: f64, plasma_cross_section_area: f64) -> f64 {
        let dr = r - self.resonance_major_radius;
        let w = self.deposition_width.max(0.01);
        let gaussian = (-0.5 * (dr / w).powi(2)).exp();
        let norm = (2.0 * std::f64::consts::PI).sqrt() * w * plasma_cross_section_area;
        let volume_approx = 2.0 * std::f64::consts::PI * self.resonance_major_radius * norm;
        (self.total_power_watts / volume_approx.max(0.1)) * gaussian
    }
}
