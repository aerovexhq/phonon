//! Cavity Magnonics & Magnon-Photon Strong Coupling Models.
//!
//! Formulates Kittel magnon resonance, microwave cavity parameters, coherent
//! magnon-photon hybridization, non-Hermitian polariton Hamiltonian, anti-crossing
//! splitting, and exceptional points.

use phonon_core::constants::MU_0;
use std::f64::consts::PI;

/// Effective gyromagnetic ratio of electron spin in rad/(s·T).
pub const GAMMA_ELECTRON: f64 = 1.760_859_630_23e11;

/// Complex frequency representation for non-Hermitian polariton modes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexFreq {
    /// Real part (oscillation angular frequency in rad/s).
    pub re: f64,
    /// Imaginary part (half decay/growth rate in rad/s).
    pub im: f64,
}

impl ComplexFreq {
    /// Creates a new complex frequency.
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Computes the principal square root of the complex number.
    pub fn sqrt(self) -> Self {
        let r = (self.re * self.re + self.im * self.im).sqrt();
        let u = ((r + self.re) / 2.0).sqrt();
        let v = if self.im >= 0.0 {
            ((r - self.re) / 2.0).sqrt()
        } else {
            -((r - self.re) / 2.0).sqrt()
        };
        Self { re: u, im: v }
    }

    /// Magnitude of the complex frequency.
    pub fn abs(self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }
}

impl std::ops::Add for ComplexFreq {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl std::ops::Sub for ComplexFreq {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }
}

/// Geometry of the Yttrium Iron Garnet (YIG) magnetic sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YigGeometry {
    /// Polished single-crystal YIG sphere with symmetric demagnetization factors ($N_x=N_y=N_z=1/3$).
    Sphere,
    /// Thin film with in-plane magnetization.
    InPlaneFilm,
    /// Thin film with out-of-plane (perpendicular) magnetization.
    PerpendicularFilm,
}

/// Material properties of Yttrium Iron Garnet (YIG, $\text{Y}_3\text{Fe}_5\text{O}_{12}$).
#[derive(Debug, Clone, PartialEq)]
pub struct YigMaterial {
    /// Saturation magnetization $M_s$ in A/m (default ~140 kA/m).
    pub saturation_magnetization: f64,
    /// Gyromagnetic ratio $\gamma$ in rad/(s·T).
    pub gyromagnetic_ratio: f64,
    /// Intrinsic Gilbert damping parameter $\alpha_0$ (dimensionless, ~3e-5).
    pub intrinsic_damping: f64,
    /// Sample geometry.
    pub geometry: YigGeometry,
}

impl Default for YigMaterial {
    fn default() -> Self {
        Self {
            saturation_magnetization: 140_000.0,
            gyromagnetic_ratio: GAMMA_ELECTRON,
            intrinsic_damping: 3.0e-5,
            geometry: YigGeometry::Sphere,
        }
    }
}

impl YigMaterial {
    /// Calculates the Kittel resonance frequency $\omega_m$ in rad/s for an applied DC bias field $B_0$ (in Tesla).
    pub fn kittel_frequency_rad(&self, b0: f64) -> f64 {
        match self.geometry {
            YigGeometry::Sphere => self.gyromagnetic_ratio * b0,
            YigGeometry::InPlaneFilm => {
                let ms_field = MU_0 * self.saturation_magnetization;
                self.gyromagnetic_ratio * (b0 * (b0 + ms_field)).max(0.0).sqrt()
            }
            YigGeometry::PerpendicularFilm => {
                let ms_field = MU_0 * self.saturation_magnetization;
                self.gyromagnetic_ratio * (b0 - ms_field).max(0.0)
            }
        }
    }

    /// Calculates the Kittel resonance frequency $f_m$ in Hz.
    pub fn kittel_frequency_hz(&self, b0: f64) -> f64 {
        self.kittel_frequency_rad(b0) / (2.0 * PI)
    }

    /// Calculates the intrinsic magnon dissipation rate $\gamma_m = 2 \alpha_0 \omega_m$ in rad/s.
    pub fn magnon_dissipation_rate_rad(&self, b0: f64) -> f64 {
        2.0 * self.intrinsic_damping * self.kittel_frequency_rad(b0)
    }

    /// Calculates the intrinsic magnon linewidth (FWHM) in Hz: $\Delta f_m = 2 \alpha_0 f_m$.
    pub fn magnon_linewidth_hz(&self, b0: f64) -> f64 {
        self.magnon_dissipation_rate_rad(b0) / (2.0 * PI)
    }

    /// Calculates the resonant DC bias field $B_0$ (in Tesla) corresponding to a target frequency in Hz.
    pub fn resonant_field(&self, target_freq_hz: f64) -> f64 {
        let target_omega = target_freq_hz * 2.0 * PI;
        match self.geometry {
            YigGeometry::Sphere => target_omega / self.gyromagnetic_ratio,
            YigGeometry::InPlaneFilm => {
                let ms_field = MU_0 * self.saturation_magnetization;
                let target_b = target_omega / self.gyromagnetic_ratio;
                0.5 * (-ms_field + (ms_field * ms_field + 4.0 * target_b * target_b).sqrt())
            }
            YigGeometry::PerpendicularFilm => {
                let ms_field = MU_0 * self.saturation_magnetization;
                target_omega / self.gyromagnetic_ratio + ms_field
            }
        }
    }
}

/// Parameters of the 3D microwave cavity resonator.
#[derive(Debug, Clone, PartialEq)]
pub struct MicrowaveCavityParams {
    /// Resonant frequency $f_c$ in Hz (e.g. 10 GHz).
    pub resonant_frequency_hz: f64,
    /// Loaded quality factor $Q_c$.
    pub quality_factor: f64,
    /// External coupling ratio $\eta_c = \kappa_{ext} / \kappa_c$ (0.5 for critical coupling).
    pub external_coupling_ratio: f64,
}

impl Default for MicrowaveCavityParams {
    fn default() -> Self {
        Self {
            resonant_frequency_hz: 10.0e9,
            quality_factor: 5000.0,
            external_coupling_ratio: 0.5,
        }
    }
}

impl MicrowaveCavityParams {
    /// Resonant angular frequency $\omega_c = 2\pi f_c$ in rad/s.
    pub fn omega_c(&self) -> f64 {
        2.0 * PI * self.resonant_frequency_hz
    }

    /// Total cavity photon dissipation rate $\kappa_c = \omega_c / Q_c$ in rad/s.
    pub fn total_decay_rate_rad(&self) -> f64 {
        self.omega_c() / self.quality_factor
    }

    /// Total cavity linewidth in Hz.
    pub fn total_decay_rate_hz(&self) -> f64 {
        self.resonant_frequency_hz / self.quality_factor
    }

    /// External decay rate $\kappa_{ext}$ in rad/s.
    pub fn external_decay_rate_rad(&self) -> f64 {
        self.total_decay_rate_rad() * self.external_coupling_ratio
    }

    /// Intrinsic cavity internal loss rate $\kappa_{int}$ in rad/s.
    pub fn intrinsic_decay_rate_rad(&self) -> f64 {
        self.total_decay_rate_rad() * (1.0 - self.external_coupling_ratio)
    }
}

/// Magnon-photon strong coupling system in a 3D microwave cavity.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnonCavityCoupling {
    /// Coupling rate $g_{mp} / (2\pi)$ in Hz (e.g. 40 MHz).
    pub coupling_rate_hz: f64,
    /// Cavity parameters.
    pub cavity: MicrowaveCavityParams,
    /// YIG material properties.
    pub magnon: YigMaterial,
    /// External DC bias magnetic field $B_0$ in Tesla.
    pub bias_field: f64,
}

impl Default for MagnonCavityCoupling {
    fn default() -> Self {
        let magnon = YigMaterial::default();
        let cavity = MicrowaveCavityParams::default();
        let b0_res = magnon.resonant_field(cavity.resonant_frequency_hz);
        Self {
            coupling_rate_hz: 40.0e6,
            cavity,
            magnon,
            bias_field: b0_res,
        }
    }
}

impl MagnonCavityCoupling {
    /// Coupling rate $g_{mp} = 2\pi \times g_{mp,hz}$ in rad/s.
    pub fn coupling_rate_rad(&self) -> f64 {
        2.0 * PI * self.coupling_rate_hz
    }

    /// Angular detuning $\Delta\omega = \omega_c - \omega_m(B_0)$ in rad/s.
    pub fn detuning_rad(&self) -> f64 {
        self.cavity.omega_c() - self.magnon.kittel_frequency_rad(self.bias_field)
    }

    /// Frequency detuning $\Delta f = f_c - f_m(B_0)$ in Hz.
    pub fn detuning_hz(&self) -> f64 {
        self.cavity.resonant_frequency_hz - self.magnon.kittel_frequency_hz(self.bias_field)
    }

    /// Magnon-photon cooperativity:
    /// $$C_{mp} = \frac{g_{mp}^2}{\kappa_c \gamma_m}$$
    pub fn cooperativity(&self) -> f64 {
        let g = self.coupling_rate_rad();
        let kappa = self.cavity.total_decay_rate_rad();
        let gamma = self.magnon.magnon_dissipation_rate_rad(self.bias_field);
        (g * g) / (kappa * gamma)
    }

    /// Checks if the system is in the strong coupling regime:
    /// $$g_{mp} > \frac{\kappa_c + \gamma_m}{4}$$
    pub fn is_strong_coupling(&self) -> bool {
        let g = self.coupling_rate_rad();
        let kappa = self.cavity.total_decay_rate_rad();
        let gamma = self.magnon.magnon_dissipation_rate_rad(self.bias_field);
        g > (kappa + gamma) / 4.0
    }

    /// Computes the complex eigenvalues $\omega_\pm$ of the non-Hermitian polariton Hamiltonian:
    /// $$\mathbf{H}_{eff} = \begin{pmatrix} \omega_c - i\kappa_c/2 & g_{mp} \\ g_{mp} & \omega_m - i\gamma_m/2 \end{pmatrix}$$
    pub fn polariton_eigenvalues(&self) -> (ComplexFreq, ComplexFreq) {
        let wc = self.cavity.omega_c();
        let wm = self.magnon.kittel_frequency_rad(self.bias_field);
        let kc = self.cavity.total_decay_rate_rad();
        let gm = self.magnon.magnon_dissipation_rate_rad(self.bias_field);
        let g = self.coupling_rate_rad();

        // Trace / 2
        let avg = ComplexFreq::new((wc + wm) / 2.0, -(kc + gm) / 4.0);

        // Half difference (wc - wm)/2 - i (kc - gm)/4
        let delta = ComplexFreq::new((wc - wm) / 2.0, -(kc - gm) / 4.0);

        // delta^2 + g^2
        let delta_sq = ComplexFreq::new(
            delta.re * delta.re - delta.im * delta.im + g * g,
            2.0 * delta.re * delta.im,
        );

        let root = delta_sq.sqrt();
        (avg + root, avg - root)
    }

    /// Polariton resonance frequencies in Hz: $(f_+, f_-)$.
    pub fn polariton_frequencies_hz(&self) -> (f64, f64) {
        let (e_plus, e_minus) = self.polariton_eigenvalues();
        (e_plus.re / (2.0 * PI), e_minus.re / (2.0 * PI))
    }

    /// Polariton dissipation linewidths in Hz: $(\Delta f_+, \Delta f_-)$.
    pub fn polariton_linewidths_hz(&self) -> (f64, f64) {
        let (e_plus, e_minus) = self.polariton_eigenvalues();
        (
            -2.0 * e_plus.im / (2.0 * PI),
            -2.0 * e_minus.im / (2.0 * PI),
        )
    }

    /// Anti-crossing Rabi frequency gap $\Omega_{Rabi} \approx 2 g_{mp}$ in Hz at zero detuning.
    pub fn anticrossing_gap_hz(&self) -> f64 {
        let g = self.coupling_rate_rad();
        let kc = self.cavity.total_decay_rate_rad();
        let gm = self.magnon.magnon_dissipation_rate_rad(self.bias_field);
        let diff = (kc - gm) / 4.0;
        let disc = g * g - diff * diff;
        if disc > 0.0 {
            2.0 * disc.sqrt() / (2.0 * PI)
        } else {
            0.0
        }
    }

    /// Checks if the system is at an Exceptional Point (EP2) within a given frequency tolerance.
    pub fn is_exceptional_point(&self, tolerance_hz: f64) -> bool {
        let det_hz = self.detuning_hz().abs();
        let g_hz = self.coupling_rate_hz;
        let kc_hz = self.cavity.total_decay_rate_hz();
        let gm_hz = self.magnon.magnon_linewidth_hz(self.bias_field);
        let ep_coupling_hz = (kc_hz - gm_hz).abs() / 4.0;

        det_hz <= tolerance_hz && (g_hz - ep_coupling_hz).abs() <= tolerance_hz
    }
}
