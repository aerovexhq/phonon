//! Cold Atom Interferometry, Optical Lattice Clocks & Relativistic Geodesy.
//!
//! Formulates:
//! - Alkali and alkaline-earth atomic species:
//!   - $^{87}\text{Rb}$: $D_2$ line transition $\lambda = 780.241\text{ nm}$, natural linewidth
//!     $\Gamma = 2\pi \times 6.065\text{ MHz}$, s-wave scattering length $a_s \approx 100\, a_0$.
//!   - $^{88}\text{Sr}$: $^1S_0 \to {}^3P_0$ ultra-narrow clock transition $\lambda = 698.446\text{ nm}$,
//!     magic wavelength $\lambda_{magic} = 813.427\text{ nm}$ cancelling differential AC Stark shift.
//! - Two-photon Raman and Bragg transitions:
//!   $$\hat{H}_{eff} = \frac{\hbar \Omega_{eff}}{2} \left(|e\rangle \langle g| e^{-i(\Delta \omega t - \mathbf{k}_{eff}\cdot\mathbf{r})} + h.c.\right)$$
//!   with effective wavevector $\mathbf{k}_{eff} = 2\mathbf{k}_L$ and momentum kick $\Delta p = \hbar k_{eff}$.
//! - Mach-Zehnder matter-wave interferometer ($\pi/2 - T - \pi - T - \pi/2$):
//!   - Gravitational phase accumulation: $\Delta\Phi_g = \mathbf{k}_{eff} \cdot \mathbf{g} T^2$.
//!   - Gravity gradient tensor phase: $\Delta\Phi_1 - \Delta\Phi_2 = \mathbf{k}_{eff} \cdot (\frac{\partial g_z}{\partial z}) d \cdot T^2$ across baseline $d$.
//!   - Sagnac rotational phase: $\Delta\Phi_\Omega = 2 \frac{m}{\hbar} \boldsymbol{\Omega}_{rot} \cdot \mathbf{A}$.
//!   - Output port population ratio: $P_e = \frac{1}{2}(1 - C \cos(\Delta\Phi))$ with fringe contrast $C \approx 0.8-0.95$.
//! - Magic-wavelength optical lattice clock & relativistic geodesy:
//!   - Differential polarizability cancellation: $\Delta\alpha(\lambda_{magic}) = \alpha_e - \alpha_g = 0.0$.
//!   - Fractional frequency instability: $\sigma_y(\tau) \approx \frac{1}{\pi Q}\sqrt{\frac{t_c}{N \tau}} \le 10^{-18}/\sqrt{\tau}$.
//!   - Relativistic gravitational redshift elevation mapping:
//!     $$\frac{\Delta\nu}{\nu_0} = \frac{g \Delta h}{c^2} \approx 1.09 \times 10^{-18}\text{ per cm}$$

use crate::quantum::Complex;
use phonon_core::constants::{H_BAR, PLANCK_CONSTANT, SPEED_OF_LIGHT};

/// Bohr radius $a_0$ in meters ($m$).
pub const BOHR_RADIUS_METERS: f64 = 5.291_772_109_03e-11;

/// Atomic mass unit constant $u$ in kilograms ($kg$).
pub const ATOMIC_MASS_UNIT_KG: f64 = 1.660_539_066_60e-27;

/// Standard Earth gravitational acceleration $g_0$ in $\text{m/s}^2$.
pub const STANDARD_GRAVITY_M_S2: f64 = 9.806_65;

/// Earth angular rotation rate $\Omega_E$ in $\text{rad/s}$.
pub const EARTH_ROTATION_RATE_RAD_S: f64 = 7.292_115e-5;

/// Rubidium-87 ($^{87}\text{Rb}$) atomic mass in kilograms.
pub const RB87_MASS_KG: f64 = 86.909_180_527 * ATOMIC_MASS_UNIT_KG;

/// Rubidium-87 $D_2$ transition wavelength $\lambda_{D2}$ in meters ($780.241\text{ nm}$).
pub const RB87_D2_WAVELENGTH_METERS: f64 = 780.241_2e-9;

/// Rubidium-87 $D_2$ natural linewidth $\Gamma_{D2} = 2\pi \times 6.065\text{ MHz}$ in $\text{rad/s}$.
pub const RB87_D2_LINEWIDTH_RAD_S: f64 = 2.0 * std::f64::consts::PI * 6.065e6;

/// Rubidium-87 $s$-wave scattering length $a_s \approx 100\,a_0$ in meters.
pub const RB87_S_WAVE_SCATTERING_LENGTH_M: f64 = 100.0 * BOHR_RADIUS_METERS;

/// Strontium-88 ($^{88}\text{Sr}$) atomic mass in kilograms.
pub const SR88_MASS_KG: f64 = 87.905_612_257 * ATOMIC_MASS_UNIT_KG;

/// Strontium-88 $^1S_0 \to {}^3P_0$ clock transition wavelength $\lambda_{clock}$ in meters ($698.446\text{ nm}$).
pub const SR88_CLOCK_WAVELENGTH_METERS: f64 = 698.445_7e-9;

/// Strontium-88 magic wavelength $\lambda_{magic}$ in meters ($813.427\text{ nm}$).
pub const SR88_MAGIC_WAVELENGTH_METERS: f64 = 813.427_4e-9;

/// Strontium-88 clock transition natural linewidth $\Gamma_{clock} \approx 2\pi \times 1.0\text{ mHz}$ in $\text{rad/s}$.
pub const SR88_CLOCK_LINEWIDTH_RAD_S: f64 = 2.0 * std::f64::consts::PI * 1.0e-3;

/// Strontium-88 $s$-wave scattering length $a_s \approx -2.0\,a_0$ in meters.
pub const SR88_S_WAVE_SCATTERING_LENGTH_M: f64 = -2.0 * BOHR_RADIUS_METERS;

/// Alkali and alkaline-earth atomic species used for cold atom quantum sensors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicSpecies {
    /// Rubidium-87 ($^{87}\text{Rb}$): alkali atom widely used in matter-wave interferometry and gravimetry.
    Rubidium87,
    /// Strontium-88 ($^{88}\text{Sr}$): alkaline-earth atom used in optical lattice clocks and relativistic geodesy.
    Strontium88,
}

impl AtomicSpecies {
    /// Atomic rest mass $m$ in kilograms ($kg$).
    #[inline]
    pub fn mass_kg(&self) -> f64 {
        match self {
            Self::Rubidium87 => RB87_MASS_KG,
            Self::Strontium88 => SR88_MASS_KG,
        }
    }

    /// Primary optical transition wavelength $\lambda_0$ in meters ($m$).
    #[inline]
    pub fn primary_transition_wavelength(&self) -> f64 {
        match self {
            Self::Rubidium87 => RB87_D2_WAVELENGTH_METERS,
            Self::Strontium88 => SR88_CLOCK_WAVELENGTH_METERS,
        }
    }

    /// Primary optical transition frequency $\nu_0 = c / \lambda_0$ in Hertz ($Hz$).
    #[inline]
    pub fn primary_transition_frequency(&self) -> f64 {
        SPEED_OF_LIGHT / self.primary_transition_wavelength()
    }

    /// Natural linewidth $\Gamma$ of the primary transition in $\text{rad/s}$.
    #[inline]
    pub fn natural_linewidth_rad_s(&self) -> f64 {
        match self {
            Self::Rubidium87 => RB87_D2_LINEWIDTH_RAD_S,
            Self::Strontium88 => SR88_CLOCK_LINEWIDTH_RAD_S,
        }
    }

    /// $s$-wave scattering length $a_s$ in meters ($m$).
    #[inline]
    pub fn scattering_length_m(&self) -> f64 {
        match self {
            Self::Rubidium87 => RB87_S_WAVE_SCATTERING_LENGTH_M,
            Self::Strontium88 => SR88_S_WAVE_SCATTERING_LENGTH_M,
        }
    }

    /// 3D two-body contact interaction strength:
    /// $$g_{3D} = \frac{4\pi \hbar^2 a_s}{m}$$
    #[inline]
    pub fn interaction_strength_3d(&self) -> f64 {
        let m = self.mass_kg();
        let a_s = self.scattering_length_m();
        (4.0 * std::f64::consts::PI * H_BAR * H_BAR * a_s) / m
    }

    /// Effective 1D contact interaction strength under transverse harmonic confinement $\omega_\perp$:
    /// $$g_{1D} = 2 \hbar \omega_\perp a_s$$
    #[inline]
    pub fn interaction_strength_1d(&self, omega_perp: f64) -> f64 {
        2.0 * H_BAR * omega_perp * self.scattering_length_m()
    }

    /// Single-photon recoil momentum $\hbar k_L = \frac{h}{\lambda_L}$ in $\text{kg}\cdot\text{m/s}$.
    #[inline]
    pub fn recoil_momentum(&self, wavelength: f64) -> f64 {
        PLANCK_CONSTANT / wavelength
    }

    /// Single-photon recoil velocity $v_r = \frac{\hbar k_L}{m}$ in $\text{m/s}$.
    #[inline]
    pub fn recoil_velocity(&self, wavelength: f64) -> f64 {
        self.recoil_momentum(wavelength) / self.mass_kg()
    }

    /// Single-photon recoil energy $E_r = \frac{\hbar^2 k_L^2}{2m}$ in Joules ($J$).
    #[inline]
    pub fn recoil_energy(&self, wavelength: f64) -> f64 {
        let p = self.recoil_momentum(wavelength);
        (p * p) / (2.0 * self.mass_kg())
    }

    /// Single-photon recoil frequency $\nu_r = \frac{E_r}{h}$ in Hertz ($Hz$).
    #[inline]
    pub fn recoil_frequency_hz(&self, wavelength: f64) -> f64 {
        self.recoil_energy(wavelength) / PLANCK_CONSTANT
    }
}

/// Two-photon laser transition mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoPhotonTransitionType {
    /// Raman transition: coupling internal hyperfine states ($|g\rangle \to |e\rangle$)
    /// with momentum transfer $\hbar \mathbf{k}_{eff}$.
    Raman,
    /// Bragg transition: state-preserving diffraction ($|g\rangle \to |g\rangle$)
    /// with multi-photon momentum transfer $2n \hbar \mathbf{k}_L$.
    Bragg,
}

/// Two-photon Raman or Bragg laser transition configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoPhotonTransition {
    /// Target atomic species.
    pub species: AtomicSpecies,
    /// Transition type (Raman or Bragg).
    pub transition_type: TwoPhotonTransitionType,
    /// Laser optical wavelength $\lambda_L$ in meters ($m$).
    pub laser_wavelength: f64,
    /// Effective two-photon Rabi frequency $\Omega_{eff}$ in $\text{rad/s}$.
    pub effective_rabi_freq: f64,
    /// Two-photon detuning $\delta = \Delta\omega - \omega_0 - \Delta\omega_D - \Delta\omega_{rec}$ in $\text{rad/s}$.
    pub detuning: f64,
    /// Bragg diffraction order $n$ (for Raman, $n=1$).
    pub diffraction_order: usize,
}

impl TwoPhotonTransition {
    /// Constructs a standard counter-propagating two-photon Raman transition for Rubidium-87.
    pub fn standard_rb87_raman(effective_rabi_freq: f64) -> Self {
        Self {
            species: AtomicSpecies::Rubidium87,
            transition_type: TwoPhotonTransitionType::Raman,
            laser_wavelength: RB87_D2_WAVELENGTH_METERS,
            effective_rabi_freq,
            detuning: 0.0,
            diffraction_order: 1,
        }
    }

    /// Constructs a standard Bragg transition of diffraction order $n$.
    pub fn standard_bragg(
        species: AtomicSpecies,
        wavelength: f64,
        effective_rabi_freq: f64,
        order: usize,
    ) -> Self {
        Self {
            species,
            transition_type: TwoPhotonTransitionType::Bragg,
            laser_wavelength: wavelength,
            effective_rabi_freq,
            detuning: 0.0,
            diffraction_order: order.max(1),
        }
    }

    /// Single-beam laser wavenumber $k_L = \frac{2\pi}{\lambda_L}$ in $\text{rad/m}$.
    #[inline]
    pub fn laser_wavenumber(&self) -> f64 {
        2.0 * std::f64::consts::PI / self.laser_wavelength
    }

    /// Effective wavevector magnitude $k_{eff}$:
    /// For counter-propagating beams, $k_{eff} = 2 n k_L = \frac{4\pi n}{\lambda_L}$.
    #[inline]
    pub fn effective_wavevector_mag(&self) -> f64 {
        2.0 * (self.diffraction_order as f64) * self.laser_wavenumber()
    }

    /// Transferred momentum kick $\Delta p = \hbar k_{eff}$ in $\text{kg}\cdot\text{m/s}$.
    #[inline]
    pub fn momentum_kick(&self) -> f64 {
        H_BAR * self.effective_wavevector_mag()
    }

    /// Atom recoil velocity kick $\Delta v = \frac{\Delta p}{m}$ in $\text{m/s}$.
    #[inline]
    pub fn recoil_velocity_kick(&self) -> f64 {
        self.momentum_kick() / self.species.mass_kg()
    }

    /// Kinetic recoil frequency shift $\Delta\omega_{rec} = \frac{\hbar k_{eff}^2}{2m}$ in $\text{rad/s}$.
    #[inline]
    pub fn recoil_shift_rad_s(&self) -> f64 {
        let k = self.effective_wavevector_mag();
        (H_BAR * k * k) / (2.0 * self.species.mass_kg())
    }

    /// Generalized two-photon Rabi frequency:
    /// $$\tilde{\Omega} = \sqrt{\Omega_{eff}^2 + \delta^2}$$
    #[inline]
    pub fn generalized_rabi_freq(&self) -> f64 {
        (self.effective_rabi_freq * self.effective_rabi_freq + self.detuning * self.detuning).sqrt()
    }

    /// Resonant $\pi/2$ beam splitter pulse duration $\tau_{\pi/2} = \frac{\pi}{2 \Omega_{eff}}$ in seconds.
    #[inline]
    pub fn half_pi_pulse_duration(&self) -> f64 {
        std::f64::consts::PI / (2.0 * self.effective_rabi_freq.max(1e-12))
    }

    /// Resonant $\pi$ mirror pulse duration $\tau_\pi = \frac{\pi}{\Omega_{eff}}$ in seconds.
    #[inline]
    pub fn pi_pulse_duration(&self) -> f64 {
        std::f64::consts::PI / self.effective_rabi_freq.max(1e-12)
    }

    /// Excited state population $P_e(\tau)$ after square pulse duration $\tau$:
    /// $$P_e(\tau) = \frac{\Omega_{eff}^2}{\tilde{\Omega}^2} \sin^2\left(\frac{\tilde{\Omega} \tau}{2}\right)$$
    #[inline]
    pub fn transition_probability(&self, pulse_duration: f64) -> f64 {
        let omega_tilde = self.generalized_rabi_freq();
        if omega_tilde < 1e-18 {
            return 0.0;
        }
        let ratio = self.effective_rabi_freq / omega_tilde;
        let s = (0.5 * omega_tilde * pulse_duration).sin();
        (ratio * ratio * s * s).clamp(0.0, 1.0)
    }

    /// Two-level Cayley-Klein unitary time evolution matrix $U(\tau)$ for laser phase $\phi$:
    /// $$U_{11} = \cos\left(\frac{\tilde{\Omega}\tau}{2}\right) - i \frac{\delta}{\tilde{\Omega}} \sin\left(\frac{\tilde{\Omega}\tau}{2}\right)$$
    /// $$U_{12} = -i \frac{\Omega_{eff}}{\tilde{\Omega}} \sin\left(\frac{\tilde{\Omega}\tau}{2}\right) e^{-i\phi}$$
    /// $$U_{21} = -i \frac{\Omega_{eff}}{\tilde{\Omega}} \sin\left(\frac{\tilde{\Omega}\tau}{2}\right) e^{i\phi}$$
    /// $$U_{22} = \cos\left(\frac{\tilde{\Omega}\tau}{2}\right) + i \frac{\delta}{\tilde{\Omega}} \sin\left(\frac{\tilde{\Omega}\tau}{2}\right)$$
    pub fn evolution_matrix_2x2(&self, pulse_duration: f64, laser_phase: f64) -> [[Complex; 2]; 2] {
        let omega_tilde = self.generalized_rabi_freq();
        if omega_tilde < 1e-18 {
            return [[Complex::ONE, Complex::ZERO], [Complex::ZERO, Complex::ONE]];
        }
        let half_angle = 0.5 * omega_tilde * pulse_duration;
        let c = half_angle.cos();
        let s = half_angle.sin();

        let delta_factor = self.detuning / omega_tilde;
        let rabi_factor = self.effective_rabi_freq / omega_tilde;

        let u11 = Complex::new(c, -delta_factor * s);
        let u22 = Complex::new(c, delta_factor * s);

        let phase_neg = Complex::cis(-laser_phase);
        let phase_pos = Complex::cis(laser_phase);

        let off_diag = Complex::new(0.0, -rabi_factor * s);
        let u12 = off_diag.mul(phase_neg);
        let u21 = off_diag.mul(phase_pos);

        [[u11, u12], [u21, u22]]
    }
}

/// Mach-Zehnder matter-wave interferometer ($\pi/2 - T - \pi - T - \pi/2$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MachZehnderInterferometer {
    /// Atomic species forming the matter waves.
    pub species: AtomicSpecies,
    /// Interrogation time $T$ between successive laser pulses in seconds ($s$).
    pub interrogation_time: f64,
    /// Effective wavevector vector $\mathbf{k}_{eff} = (k_x, k_y, k_z)$ in $\text{rad/m}$.
    pub effective_k: [f64; 3],
    /// Local gravitational acceleration vector $\mathbf{g} = (g_x, g_y, g_z)$ in $\text{m/s}^2$.
    pub gravity: [f64; 3],
    /// Vertical gravity gradient tensor component $T_{zz} = \frac{\partial g_z}{\partial z}$ in $\text{s}^{-2}$.
    pub gravity_gradient_zz: f64,
    /// Spatial baseline distance $d$ between dual clouds in gradiometer configuration in meters ($m$).
    pub baseline_distance: f64,
    /// Platform rotational angular velocity $\boldsymbol{\Omega}_{rot} = (\Omega_x, \Omega_y, \Omega_z)$ in $\text{rad/s}$.
    pub rotation_rate: [f64; 3],
    /// Initial atom cloud launch velocity vector $\mathbf{v}_0 = (v_x, v_y, v_z)$ in $\text{m/s}$.
    pub initial_velocity: [f64; 3],
    /// Interferometer fringe contrast $C \in [0.0, 1.0]$ (typically $0.80 - 0.95$).
    pub contrast: f64,
    /// Net laser phase shift $\Delta\phi_{laser} = \phi_1 - 2\phi_2 + \phi_3$ in radians.
    pub laser_phase_shift: f64,
}

impl MachZehnderInterferometer {
    /// Constructs a standard vertical gravimeter with Rubidium-87.
    pub fn standard_rb87_gravimeter(t_separation: f64) -> Self {
        let k_eff_mag = 2.0 * (2.0 * std::f64::consts::PI / RB87_D2_WAVELENGTH_METERS);
        Self {
            species: AtomicSpecies::Rubidium87,
            interrogation_time: t_separation,
            effective_k: [0.0, 0.0, k_eff_mag],
            gravity: [0.0, 0.0, -STANDARD_GRAVITY_M_S2],
            gravity_gradient_zz: 3.086e-6, // Nominal Earth gravity gradient ~3086 Eötvös
            baseline_distance: 1.0,        // 1 meter baseline
            rotation_rate: [0.0, 0.0, 0.0],
            initial_velocity: [0.0, 0.0, 0.0],
            contrast: 0.90,
            laser_phase_shift: 0.0,
        }
    }

    /// Total interferometer duration $t_{total} = 2T$ in seconds ($s$).
    #[inline]
    pub fn total_time(&self) -> f64 {
        2.0 * self.interrogation_time
    }

    /// Effective wavevector magnitude $|\mathbf{k}_{eff}|$ in $\text{rad/m}$.
    #[inline]
    pub fn effective_k_magnitude(&self) -> f64 {
        let [kx, ky, kz] = self.effective_k;
        (kx * kx + ky * ky + kz * kz).sqrt()
    }

    /// Gravitational phase accumulation:
    /// $$\Delta\Phi_g = \mathbf{k}_{eff} \cdot \mathbf{g} T^2$$
    #[inline]
    pub fn gravitational_phase(&self) -> f64 {
        let t2 = self.interrogation_time * self.interrogation_time;
        let k_dot_g = self.effective_k[0] * self.gravity[0]
            + self.effective_k[1] * self.gravity[1]
            + self.effective_k[2] * self.gravity[2];
        k_dot_g * t2
    }

    /// Differential phase shift between two clouds separated by baseline distance $d$:
    /// $$\Delta\Phi_1 - \Delta\Phi_2 = \mathbf{k}_{eff} \cdot \left(\frac{\partial g_z}{\partial z}\right) d \cdot T^2$$
    #[inline]
    pub fn gravity_gradient_phase_diff(&self) -> f64 {
        let t2 = self.interrogation_time * self.interrogation_time;
        self.effective_k[2] * self.gravity_gradient_zz * self.baseline_distance * t2
    }

    /// Enclosed physical interferometer loop area vector $\mathbf{A}$:
    /// $$\mathbf{A} = \frac{\hbar T^2}{m} (\mathbf{k}_{eff} \times \mathbf{v}_0)$$
    pub fn enclosed_area_vector(&self) -> [f64; 3] {
        let m = self.species.mass_kg();
        let scale = (H_BAR * self.interrogation_time * self.interrogation_time) / m;
        let [kx, ky, kz] = self.effective_k;
        let [vx, vy, vz] = self.initial_velocity;
        [
            scale * (ky * vz - kz * vy),
            scale * (kz * vx - kx * vz),
            scale * (kx * vy - ky * vx),
        ]
    }

    /// Sagnac rotational phase shift:
    /// $$\Delta\Phi_\Omega = 2 \frac{m}{\hbar} \boldsymbol{\Omega}_{rot} \cdot \mathbf{A} = 2 T^2 \mathbf{k}_{eff} \cdot (\boldsymbol{\Omega}_{rot} \times \mathbf{v}_0)$$
    #[inline]
    pub fn sagnac_rotational_phase(&self) -> f64 {
        let area = self.enclosed_area_vector();
        let m = self.species.mass_kg();
        let factor = 2.0 * m / H_BAR;
        factor
            * (self.rotation_rate[0] * area[0]
                + self.rotation_rate[1] * area[1]
                + self.rotation_rate[2] * area[2])
    }

    /// Total accumulated matter-wave phase shift:
    /// $$\Delta\Phi = \Delta\Phi_g + \Delta\Phi_\Omega + \Delta\phi_{laser}$$
    #[inline]
    pub fn total_phase_shift(&self) -> f64 {
        self.gravitational_phase() + self.sagnac_rotational_phase() + self.laser_phase_shift
    }

    /// Output port excited state population ratio:
    /// $$P_e = \frac{1}{2} (1 - C \cos(\Delta\Phi))$$
    #[inline]
    pub fn excited_state_population(&self) -> f64 {
        let phi = self.total_phase_shift();
        (0.5 * (1.0 - self.contrast * phi.cos())).clamp(0.0, 1.0)
    }

    /// Output port ground state population ratio:
    /// $$P_g = 1 - P_e = \frac{1}{2} (1 + C \cos(\Delta\Phi))$$
    #[inline]
    pub fn ground_state_population(&self) -> f64 {
        1.0 - self.excited_state_population()
    }

    /// Simultaneous population ratios $(P_{e,1}, P_{e,2})$ for a dual-cloud gravity gradiometer.
    /// Cloud 1 measures local gravity $g(z)$ and Cloud 2 measures $g(z + d)$.
    pub fn dual_cloud_gradiometer_populations(&self) -> (f64, f64) {
        let phi1 = self.total_phase_shift();
        let grad_dphi = self.gravity_gradient_phase_diff();
        let phi2 = phi1 + grad_dphi;

        let p1 = (0.5 * (1.0 - self.contrast * phi1.cos())).clamp(0.0, 1.0);
        let p2 = (0.5 * (1.0 - self.contrast * phi2.cos())).clamp(0.0, 1.0);
        (p1, p2)
    }

    /// Gravitational acceleration sensitivity $\eta_g$ in $(\text{m/s}^2)/\sqrt{\text{Hz}}$:
    /// $$\eta_g = \frac{1}{C |\mathbf{k}_{eff}| T^2 \sqrt{N}} \sqrt{t_c}$$
    /// where $N$ is atom number per cycle and $t_c$ is cycle repetition period.
    pub fn acceleration_sensitivity(&self, atom_count: f64, cycle_time: f64) -> f64 {
        let k_mag = self.effective_k_magnitude();
        let t2 = self.interrogation_time * self.interrogation_time;
        let c = self.contrast.max(1e-4);
        let n_sqrt = atom_count.max(1.0).sqrt();
        let tc_sqrt = cycle_time.max(1e-4).sqrt();

        tc_sqrt / (c * k_mag * t2 * n_sqrt)
    }

    /// Gravity gradient sensitivity $\eta_{Tzz}$ in $\text{s}^{-2}/\sqrt{\text{Hz}}$:
    /// $$\eta_{Tzz} = \frac{\sqrt{2}}{C |\mathbf{k}_{eff}| d T^2 \sqrt{N}} \sqrt{t_c}$$
    pub fn gradiometer_sensitivity(&self, atom_count: f64, cycle_time: f64) -> f64 {
        let single_sens = self.acceleration_sensitivity(atom_count, cycle_time);
        (2.0_f64.sqrt() * single_sens) / self.baseline_distance.max(1e-3)
    }
}

/// Magic-wavelength optical lattice clock & relativistic geodesy.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpticalLatticeClock {
    /// Atomic species (default: Strontium-88).
    pub species: AtomicSpecies,
    /// Clock transition wavelength $\lambda_{clock}$ in meters ($m$).
    pub clock_wavelength: f64,
    /// Magic optical lattice wavelength $\lambda_{magic}$ in meters ($m$).
    pub magic_wavelength: f64,
    /// Differential polarizability slope $\frac{\partial \Delta\alpha}{\partial \lambda}$ around magic wavelength in $\text{a.u./nm}$.
    pub differential_polarizability_slope: f64,
    /// Optical lattice trap depth $U_0 / k_B$ in microkelvin ($\mu\text{K}$).
    pub trap_depth_microkelvin: f64,
    /// Trapped atom count $N$ per lattice cycle.
    pub atom_count: f64,
    /// Interrogation cycle time $t_c$ in seconds ($s$).
    pub cycle_time_sec: f64,
    /// Clock transition atomic quality factor $Q = \nu_0 / \Delta\nu$.
    pub quality_factor: f64,
}

impl OpticalLatticeClock {
    /// Constructs a standard Strontium-88 optical lattice clock operating at its magic wavelength ($813.427\text{ nm}$).
    pub fn standard_sr88_lattice_clock() -> Self {
        let nu_0 = SPEED_OF_LIGHT / SR88_CLOCK_WAVELENGTH_METERS;
        let delta_nu_nat = SR88_CLOCK_LINEWIDTH_RAD_S / (2.0 * std::f64::consts::PI);
        let q = nu_0 / delta_nu_nat.max(1e-6);

        Self {
            species: AtomicSpecies::Strontium88,
            clock_wavelength: SR88_CLOCK_WAVELENGTH_METERS,
            magic_wavelength: SR88_MAGIC_WAVELENGTH_METERS,
            differential_polarizability_slope: 0.052, // a.u./nm
            trap_depth_microkelvin: 35.0,
            atom_count: 50_000.0,
            cycle_time_sec: 1.0,
            quality_factor: q,
        }
    }

    /// Clock transition center frequency $\nu_0 = \frac{c}{\lambda_{clock}}$ in Hertz ($Hz$).
    #[inline]
    pub fn clock_frequency_hz(&self) -> f64 {
        SPEED_OF_LIGHT / self.clock_wavelength
    }

    /// Differential AC Stark polarizability $\Delta\alpha(\lambda) = \alpha_e(\lambda) - \alpha_g(\lambda)$ in atomic units ($\text{a.u.}$):
    /// At $\lambda = \lambda_{magic}$, $\Delta\alpha \equiv 0.0$.
    pub fn differential_polarizability(&self, lattice_wavelength: f64) -> f64 {
        let delta_lambda_nm = (lattice_wavelength - self.magic_wavelength) * 1e9;
        self.differential_polarizability_slope * delta_lambda_nm
    }

    /// Quantum-projection-noise-limited fractional frequency instability $\sigma_y(\tau)$ (Allan deviation):
    /// $$\sigma_y(\tau) \approx \frac{1}{\pi Q} \sqrt{\frac{t_c}{N \tau}}$$
    pub fn fractional_frequency_instability(&self, averaging_time_sec: f64) -> f64 {
        let q = self.quality_factor.max(1.0);
        let tau = averaging_time_sec.max(1e-6);
        let n = self.atom_count.max(1.0);
        let tc = self.cycle_time_sec.max(1e-4);

        (1.0 / (std::f64::consts::PI * q)) * (tc / (n * tau)).sqrt()
    }

    /// Relativistic gravitational redshift fractional frequency shift:
    /// $$\frac{\Delta\nu}{\nu_0} = \frac{\Delta\Phi_G}{c^2} = \frac{g \Delta h}{c^2}$$
    /// Producing $\approx 1.091 \times 10^{-18}$ fractional shift per centimeter of vertical elevation!
    #[inline]
    pub fn gravitational_redshift_fractional_shift(
        &self,
        delta_height_m: f64,
        local_g: f64,
    ) -> f64 {
        (local_g * delta_height_m) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT)
    }

    /// Relativistic gravitational redshift frequency shift $\Delta\nu$ in Hertz ($Hz$).
    #[inline]
    pub fn gravitational_frequency_shift_hz(&self, delta_height_m: f64, local_g: f64) -> f64 {
        self.clock_frequency_hz()
            * self.gravitational_redshift_fractional_shift(delta_height_m, local_g)
    }

    /// Reconstructs vertical geodetic height difference $\Delta h$ from measured fractional frequency shift:
    /// $$\Delta h = \frac{c^2}{g} \frac{\Delta\nu}{\nu_0}$$
    #[inline]
    pub fn height_from_fractional_shift(&self, fractional_shift: f64, local_g: f64) -> f64 {
        (SPEED_OF_LIGHT * SPEED_OF_LIGHT * fractional_shift) / local_g.max(1e-4)
    }

    /// Gravitational potential difference $\Delta W$ in $\text{m}^2/\text{s}^2$ ($J/kg$):
    /// $$\Delta W = c^2 \frac{\Delta\nu}{\nu_0} = g \Delta h$$
    #[inline]
    pub fn geopotential_difference(&self, fractional_shift: f64) -> f64 {
        SPEED_OF_LIGHT * SPEED_OF_LIGHT * fractional_shift
    }

    /// Geodetic height elevation measurement resolution $\delta h$ in meters ($m$)
    /// after averaging time $\tau$:
    /// $$\delta h = \frac{c^2}{g} \sigma_y(\tau)$$
    #[inline]
    pub fn elevation_resolution_meters(&self, averaging_time_sec: f64, local_g: f64) -> f64 {
        let sigma_y = self.fractional_frequency_instability(averaging_time_sec);
        self.height_from_fractional_shift(sigma_y, local_g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_species_properties() {
        let rb = AtomicSpecies::Rubidium87;
        let sr = AtomicSpecies::Strontium88;

        assert!((rb.mass_kg() - 1.443e-25).abs() < 1e-27);
        assert!((sr.mass_kg() - 1.459e-25).abs() < 1e-27);

        assert!((rb.primary_transition_wavelength() - 780.2412e-9).abs() < 1e-12);
        assert!((sr.primary_transition_wavelength() - 698.4457e-9).abs() < 1e-12);

        // Recoil velocity of Rb87 on D2 line is ~5.88 mm/s
        let v_rec = rb.recoil_velocity(rb.primary_transition_wavelength());
        assert!((v_rec - 5.88e-3).abs() < 1e-4);

        // Positive s-wave scattering for Rb87 (condensate stable against collapse)
        assert!(rb.scattering_length_m() > 0.0);
        assert!(rb.interaction_strength_3d() > 0.0);
    }

    #[test]
    fn test_two_photon_raman_pulse_dynamics() {
        let rabi = 2.0 * std::f64::consts::PI * 25.0e3; // 25 kHz
        let mut raman = TwoPhotonTransition::standard_rb87_raman(rabi);

        let t_pi_2 = raman.half_pi_pulse_duration();
        let t_pi = raman.pi_pulse_duration();
        assert!((t_pi - 2.0 * t_pi_2).abs() < 1e-12);

        // Resonant pi/2 pulse should yield P_e = 0.5
        let p_half = raman.transition_probability(t_pi_2);
        assert!((p_half - 0.5).abs() < 1e-6);

        // Resonant pi pulse should yield P_e = 1.0
        let p_full = raman.transition_probability(t_pi);
        assert!((p_full - 1.0).abs() < 1e-6);

        // Off-resonant transition reduces peak inversion
        raman.detuning = rabi * 2.0;
        let p_detuned = raman.transition_probability(t_pi);
        assert!(p_detuned < 0.3);
    }

    #[test]
    fn test_mach_zehnder_interferometer_phase() {
        let t = 0.05; // 50 ms interrogation time
        let mz = MachZehnderInterferometer::standard_rb87_gravimeter(t);

        let phi_g = mz.gravitational_phase();
        // k_eff = 2 * (2pi / 780.24nm) ~ 1.61e7 rad/m
        // g = -9.80665 m/s^2, T = 0.05 s -> T^2 = 0.0025 s^2
        // DeltaPhi_g = -1.61e7 * 9.80665 * 0.0025 ~ -3.95e5 rad
        assert!(phi_g.abs() > 100_000.0);

        // Gradiometer phase across baseline d = 1m
        let dphi_grad = mz.gravity_gradient_phase_diff();
        // T_zz ~ 3.086e-6 s^-2 -> DeltaPhi_grad = 1.61e7 * 3.086e-6 * 1.0 * 0.0025 ~ 0.124 rad
        assert!((dphi_grad - 0.124).abs() < 0.01);
    }

    #[test]
    fn test_optical_lattice_clock_magic_wavelength_and_geodesy() {
        let clock = OpticalLatticeClock::standard_sr88_lattice_clock();

        // Exact magic wavelength differential polarizability cancellation
        let d_alpha_magic = clock.differential_polarizability(clock.magic_wavelength);
        assert_eq!(d_alpha_magic, 0.0);

        // 1 second Allan deviation should be <= 1e-18
        let sigma_1s = clock.fractional_frequency_instability(1.0);
        assert!(sigma_1s <= 1.0e-18);

        // 1 cm height shift -> ~ 1.09e-18 fractional shift
        let delta_h_1cm = 0.01;
        let frac_shift =
            clock.gravitational_redshift_fractional_shift(delta_h_1cm, STANDARD_GRAVITY_M_S2);
        assert!((frac_shift - 1.091e-18).abs() < 1e-20);

        // Reconstructed height
        let h_recon = clock.height_from_fractional_shift(frac_shift, STANDARD_GRAVITY_M_S2);
        assert!((h_recon - delta_h_1cm).abs() < 1e-6);

        // Height resolution < 1 cm (0.01 m)
        let res_1s = clock.elevation_resolution_meters(1.0, STANDARD_GRAVITY_M_S2);
        assert!(res_1s < 0.01);
    }
}
