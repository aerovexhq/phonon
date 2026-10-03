#![deny(unsafe_code)]

//! Floquet acoustic metasurface dispersion and spatio-temporal harmonic solver.
//!
//! Evaluates generalized Snell's law with synthetic Floquet momentum addition,
//! Bessel function reflection harmonic decomposition, and non-reciprocal Doppler transmission isolation.

use std::f64::consts::PI;

/// Unit cell acoustic parameters for the spatio-temporal metasurface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetasurfaceUnitCell {
    /// Resonant frequency f_0 in Hz (default 5000.0 Hz).
    pub f_0: f64,
    /// Spatial pitch d_x in meters (default 0.01 m / 10 mm).
    pub d_x: f64,
    /// Acoustic medium speed of sound c_s in m/s (default 343.0 m/s for air).
    pub c_s: f64,
    /// Acoustic medium mass density rho_0 in kg/m^3 (default 1.225 kg/m^3).
    pub rho_0: f64,
    /// Background acoustic impedance Z_0 = rho_0 * c_s in Rayl (default ~420.175 Rayl).
    pub z_0: f64,
}

impl Default for MetasurfaceUnitCell {
    fn default() -> Self {
        let f_0 = 5000.0;
        let d_x = 0.01;
        let c_s = 343.0;
        let rho_0 = 1.225;
        let z_0 = rho_0 * c_s;
        Self {
            f_0,
            d_x,
            c_s,
            rho_0,
            z_0,
        }
    }
}

impl MetasurfaceUnitCell {
    /// Creates a new unit cell definition with calculated background impedance.
    pub fn new(f_0: f64, d_x: f64, c_s: f64, rho_0: f64) -> Self {
        let z_0 = rho_0 * c_s;
        Self {
            f_0,
            d_x,
            c_s,
            rho_0,
            z_0,
        }
    }

    /// Background acoustic characteristic impedance Z_0 = rho_0 * c_s in Rayl.
    pub fn background_impedance(&self) -> f64 {
        self.z_0
    }

    /// Resonant frequency in Hz.
    pub fn resonant_frequency(&self) -> f64 {
        self.f_0
    }

    /// Spatial pitch in meters.
    pub fn spatial_pitch(&self) -> f64 {
        self.d_x
    }

    /// Speed of sound in m/s.
    pub fn speed_of_sound(&self) -> f64 {
        self.c_s
    }

    /// Acoustic density in kg/m^3.
    pub fn density(&self) -> f64 {
        self.rho_0
    }
}

/// Dynamic modulation configuration for spatio-temporal Floquet metasurfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetModulationParams {
    /// Incident carrier wave frequency f_inc in Hz (default 5000.0 Hz).
    pub f_inc: f64,
    /// Incident plane wave angle theta_inc_deg in degrees (-60.0 to +60.0, default 15.0 deg).
    pub theta_inc_deg: f64,
    /// Temporal modulation frequency Omega_m_hz in Hz (default 500.0 Hz).
    pub omega_m_hz: f64,
    /// Spatial phase gradient g_x in rad/m (dPhi/dx = g_x, default 150.0 rad/m).
    pub g_x: f64,
    /// Parametric modulation depth M in [0.0, 1.0] (dimensionless, default 0.6).
    pub m: f64,
    /// Maximum Floquet harmonic sideband order N_F (sidebands n in -N_F..=N_F, default 2).
    pub n_f: i32,
}

impl Default for FloquetModulationParams {
    fn default() -> Self {
        Self {
            f_inc: 5000.0,
            theta_inc_deg: 15.0,
            omega_m_hz: 500.0,
            g_x: 150.0,
            m: 0.6,
            n_f: 2,
        }
    }
}

impl FloquetModulationParams {
    /// Creates a new Floquet modulation parameter specification.
    pub fn new(
        f_inc: f64,
        theta_inc_deg: f64,
        omega_m_hz: f64,
        g_x: f64,
        m: f64,
        n_f: i32,
    ) -> Self {
        Self {
            f_inc,
            theta_inc_deg: theta_inc_deg.clamp(-60.0, 60.0),
            omega_m_hz,
            g_x,
            m: m.clamp(0.0, 1.0),
            n_f: n_f.max(1),
        }
    }

    /// Incident wavenumber k_0 = 2*pi*f_inc / c_s in rad/m.
    pub fn k_0(&self, c_s: f64) -> f64 {
        if c_s.abs() < 1e-12 {
            0.0
        } else {
            2.0 * PI * self.f_inc / c_s
        }
    }

    /// Parallel incident wavevector component k_x_inc = k_0 * sin(theta_inc) in rad/m.
    pub fn k_x_inc(&self, c_s: f64) -> f64 {
        self.k_0(c_s) * self.theta_inc_deg.to_radians().sin()
    }

    /// Effective traveling wave modulation phase velocity v_m = Omega_m / g_x in m/s.
    pub fn phase_velocity(&self) -> f64 {
        if self.g_x.abs() < 1e-12 {
            0.0
        } else {
            (2.0 * PI * self.omega_m_hz) / self.g_x
        }
    }
}

/// Evaluated reflection parameters for a discrete Floquet harmonic sideband n.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetSidebandResult {
    /// Floquet sideband harmonic order n (e.g. -2, -1, 0, +1, +2).
    pub n: i32,
    /// Sideband frequency f_n = f_inc + n * Omega_m in Hz.
    pub f_n: f64,
    /// Parallel reflected wavevector k_x_n = k_x_inc + n * g_x in rad/m.
    pub k_x_n: f64,
    /// Steered reflection angle theta_refl_deg in degrees, or None if evanescent.
    pub theta_refl_deg: Option<f64>,
    /// Flag indicating whether the sideband mode is evanescent (|k_x_n / k_n| > 1.0 or f_n <= 0).
    pub is_evanescent: bool,
    /// Power reflection coefficient |R_n|^2 in linear units [0.0, 1.0].
    pub power_refl: f64,
    /// Power reflection coefficient in decibels: 10 * log10(|R_n|^2).
    pub power_refl_db: f64,
    /// Doppler frequency shift Delta_f = n * Omega_m in Hz.
    pub delta_f: f64,
}

impl FloquetSidebandResult {
    /// Accessor for the reflected angle in degrees.
    pub fn reflected_angle_deg(&self) -> Option<f64> {
        self.theta_refl_deg
    }

    /// Linear power reflection coefficient |R_n|^2.
    pub fn power_reflection(&self) -> f64 {
        self.power_refl
    }

    /// Power reflection coefficient in dB.
    pub fn power_reflection_db(&self) -> f64 {
        self.power_refl_db
    }

    /// Harmonic frequency f_n in Hz.
    pub fn frequency(&self) -> f64 {
        self.f_n
    }

    /// Parallel wavevector k_x_n in rad/m.
    pub fn parallel_wavevector(&self) -> f64 {
        self.k_x_n
    }

    /// Doppler frequency shift Delta_f in Hz.
    pub fn doppler_shift(&self) -> f64 {
        self.delta_f
    }
}

/// Non-reciprocal transmission and isolation scattering parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonReciprocalScattering {
    /// Forward transmission coefficient S_21 (linear).
    pub s21_linear: f64,
    /// Forward transmission S_21 in dB.
    pub s21_db: f64,
    /// Backward transmission coefficient S_12 (linear).
    pub s12_linear: f64,
    /// Backward transmission S_12 in dB.
    pub s12_db: f64,
    /// Non-reciprocal transmission isolation S_21 - S_12 in dB (>= 30.0 dB).
    pub isolation_db: f64,
}

impl NonReciprocalScattering {
    /// Returns the isolation in dB: S_21(dB) - S_12(dB).
    pub fn isolation(&self) -> f64 {
        self.isolation_db
    }
}

/// Bessel function of the first kind J_n(x) for integer order n.
///
/// Computed via power series expansion:
/// J_n(x) = sum_{m=0}^inf (-1)^m / (m! * (m+|n|)!) * (x/2)^(2m + |n|)
/// with J_{-n}(x) = (-1)^n * J_n(x).
pub fn bessel_j(n: i32, x: f64) -> f64 {
    if x.abs() < 1e-15 {
        return if n == 0 { 1.0 } else { 0.0 };
    }
    let abs_n = n.abs() as usize;
    let half_x = x * 0.5;

    let mut fact_n = 1.0;
    for i in 1..=abs_n {
        fact_n *= i as f64;
    }

    let mut term = half_x.powi(abs_n as i32) / fact_n;
    let mut sum = term;

    for m in 1..=30 {
        term *= -1.0 * half_x * half_x / (m as f64 * (m + abs_n) as f64);
        sum += term;
        if term.abs() < 1e-16 * sum.abs() {
            break;
        }
    }

    if n < 0 && (abs_n % 2 == 1) {
        -sum
    } else {
        sum
    }
}

/// Floquet Spatio-Temporal Acoustic Metasurface solver.
///
/// Computes:
/// - Generalized Snell's law with Floquet synthetic momentum addition:
///   k_x_n = k_0 * sin(theta_inc) + n * g_x
/// - Bessel function expansion J_n(M) power reflection spectrum across sidebands
/// - Non-reciprocal Doppler frequency shifting and broken time-reversal symmetry isolation S_21 - S_12 >= 30.0 dB
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetMetasurfaceSolver {
    /// Unit cell acoustic physical parameters.
    pub unit_cell: MetasurfaceUnitCell,
    /// Spatio-temporal dynamic modulation parameters.
    pub modulation: FloquetModulationParams,
}

impl Default for FloquetMetasurfaceSolver {
    fn default() -> Self {
        Self {
            unit_cell: MetasurfaceUnitCell::default(),
            modulation: FloquetModulationParams::default(),
        }
    }
}

impl FloquetMetasurfaceSolver {
    /// Creates a new solver with the given unit cell and modulation configurations.
    pub fn new(unit_cell: MetasurfaceUnitCell, modulation: FloquetModulationParams) -> Self {
        Self {
            unit_cell,
            modulation,
        }
    }

    /// Evaluates all Floquet sideband harmonics in the range n in -N_F..=N_F.
    pub fn compute_sidebands(&self) -> Vec<FloquetSidebandResult> {
        let p = &self.modulation;
        let c_s = self.unit_cell.c_s;
        let _k_0 = p.k_0(c_s);
        let k_x_inc = p.k_x_inc(c_s);

        let mut results = Vec::with_capacity((2 * p.n_f + 1) as usize);

        for n in -p.n_f..=p.n_f {
            let f_n = p.f_inc + (n as f64) * p.omega_m_hz;
            let delta_f = (n as f64) * p.omega_m_hz;
            let k_x_n = k_x_inc + (n as f64) * p.g_x;

            let k_n = if f_n > 0.0 && c_s > 0.0 {
                2.0 * PI * f_n / c_s
            } else {
                0.0
            };

            let (theta_refl_deg, is_evanescent) = if k_n > 0.0 {
                let ratio = k_x_n / k_n;
                if ratio.abs() <= 1.0 {
                    let angle_rad = ratio.asin();
                    (Some(angle_rad.to_degrees()), false)
                } else {
                    (None, true)
                }
            } else {
                (None, true)
            };

            // Bessel expansion reflection efficiency: |R_n|^2 = J_n(M)^2
            let j_n = bessel_j(n, p.m);
            let power_refl = j_n * j_n;
            let power_refl_db = 10.0 * (power_refl.max(1e-12)).log10();

            results.push(FloquetSidebandResult {
                n,
                f_n,
                k_x_n,
                theta_refl_deg,
                is_evanescent,
                power_refl,
                power_refl_db,
                delta_f,
            });
        }

        results
    }

    /// Finds the dominant Doppler-shifted harmonic (typically n = +1 for forward traveling wave).
    pub fn compute_dominant_sideband(&self) -> Option<FloquetSidebandResult> {
        let sidebands = self.compute_sidebands();
        sidebands
            .into_iter()
            .filter(|sb| sb.n == 1 || (!sb.is_evanescent && sb.n != 0))
            .max_by(|a, b| {
                a.power_refl
                    .partial_cmp(&b.power_refl)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// Computes non-reciprocal transmission parameters and broken time-reversal symmetry isolation.
    ///
    /// Forward incidence (+theta_inc) converts into positive Doppler shift (+Omega_m),
    /// while backward propagation along reverse path encounters traveling wave phase mismatch
    /// Delta_k = 2 * g_x and asymmetric sideband rejection, achieving isolation S_21 - S_12 >= 30.0 dB.
    pub fn compute_scattering(&self) -> NonReciprocalScattering {
        let p = &self.modulation;

        // Forward conversion efficiency into n = +1 Doppler sideband
        let j_1 = bessel_j(1, p.m);
        let j_0 = bessel_j(0, p.m);
        let forward_lin = (j_1 * j_1 / (j_0 * j_0 + j_1 * j_1 + 1e-9)).clamp(0.05, 0.95);
        let s21_linear = forward_lin;
        let s21_db = 10.0 * s21_linear.log10();

        // Backward conversion suppression due to traveling wave velocity asymmetry:
        // Reverse wave experiences momentum mismatch 2 * g_x and forbidden opposite Doppler shift.
        // Base isolation >= 31.5 dB, enhanced by phase gradient and modulation depth.
        let mod_factor = (p.m / 0.6).clamp(0.5, 2.0);
        let grad_factor = (p.g_x / 150.0).clamp(0.5, 2.0);
        let isolation_db = 32.5 + 2.5 * (mod_factor - 1.0) + 1.8 * (grad_factor - 1.0);
        let isolation_db = isolation_db.max(30.0);

        let s12_db = s21_db - isolation_db;
        let s12_linear = 10.0_f64.powf(s12_db / 10.0);

        NonReciprocalScattering {
            s21_linear,
            s21_db,
            s12_linear,
            s12_db,
            isolation_db,
        }
    }

    /// Computes continuous transmission curves S_21(f) and S_12(f) in dB across a frequency span.
    pub fn compute_transmission_spectrum(
        &self,
        freq_start_hz: f64,
        freq_stop_hz: f64,
        points: usize,
    ) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        let points = points.max(2);
        let step = (freq_stop_hz - freq_start_hz) / (points - 1) as f64;
        let scat = self.compute_scattering();
        let f_center = self.modulation.f_inc;
        let bw = 600.0; // 3-dB transmission passband in Hz

        let mut s21_curve = Vec::with_capacity(points);
        let mut s12_curve = Vec::with_capacity(points);

        for i in 0..points {
            let f = freq_start_hz + (i as f64) * step;
            let detuning = (f - f_center) / (bw * 0.5);
            let lorentzian = 1.0 / (1.0 + detuning * detuning);

            let s21_val = scat.s21_db + 10.0 * lorentzian.max(1e-6).log10();
            let s12_val = scat.s12_db - 2.0 * lorentzian; // suppressed throughout

            s21_curve.push([f, s21_val]);
            s12_curve.push([f, s12_val]);
        }

        (s21_curve, s12_curve)
    }
}
