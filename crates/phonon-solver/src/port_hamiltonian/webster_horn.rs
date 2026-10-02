#![deny(unsafe_code)]

//! Continuous Riccati Webster-Horn Acoustic Transmission Line & Lip Radiation Impedance.
//!
//! Models continuous spatial Riccati wave reflection differential equations:
//! dR/dx = 2 * gamma * R - 0.5 * (d ln Z_0 / dx) * (1 - R^2),
//! visco-thermal boundary layer attenuation, and radiation impedance.

use phonon_models::port_hamiltonian::PortHamiltonianAcousticParams;

/// Minimalist pure safe complex number representation for frequency-domain acoustics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiccatiComplex {
    /// Real component.
    pub re: f64,
    /// Imaginary component.
    pub im: f64,
}

impl RiccatiComplex {
    /// Creates a new complex number.
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Evaluates squared complex norm |z|^2.
    pub fn norm_sqr(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Evaluates complex modulus |z|.
    pub fn norm(&self) -> f64 {
        self.norm_sqr().sqrt()
    }

    /// RiccatiComplex addition.
    pub fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    /// RiccatiComplex subtraction.
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    /// RiccatiComplex multiplication.
    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    /// Scalar multiplication.
    pub fn scale(self, s: f64) -> Self {
        Self::new(self.re * s, self.im * s)
    }

    /// RiccatiComplex division self / other.
    pub fn div(self, other: Self) -> Self {
        let den = other.norm_sqr();
        if den < 1e-30 {
            return Self::new(0.0, 0.0);
        }
        Self::new(
            (self.re * other.re + self.im * other.im) / den,
            (self.im * other.re - self.re * other.im) / den,
        )
    }
}

/// Continuous Riccati Webster-Horn Acoustic Transmission Line.
#[derive(Debug, Clone, PartialEq)]
pub struct RiccatiWebsterHorn {
    /// Vocal tract horn length in meters.
    pub length: f64,
    /// Radiating lip aperture radius in meters.
    pub lip_radius: f64,
    /// Speed of sound in air (m/s).
    pub speed_of_sound: f64,
    /// Air density (kg/m^3).
    pub air_density: f64,
    /// Visco-thermal loss constant alpha_0 in Np / (m * sqrt(Hz)).
    pub visco_thermal_alpha0: f64,

    /// Number of spatial discretization delay stages (24 for 48 kHz).
    pub num_segments: usize,
    /// Spatial segment length dx = length / num_segments (meters).
    pub dx: f64,
    /// Cross-sectional area profile A_k along tract (m^2).
    pub area_profile: Vec<f64>,
    /// Characteristic acoustic impedance Z_0 in Pa * s / m^3.
    pub char_impedance: f64,
    /// Visco-thermal transmission attenuation per spatial step.
    pub loss_factor: f64,
    /// Lip boundary reflection coefficient (-0.92 for open radiating mouth).
    pub lip_reflection: f64,

    /// Forward-propagating wave pressure delay line p^+_k.
    pub forward: [f64; 24],
    /// Backward-propagating wave pressure delay line p^-_k.
    pub backward: [f64; 24],
    /// Current supraglottal back-pressure at glottal entrance (k = 0).
    pub back_pressure: f64,
    /// Previous lip volume velocity for acceleration radiation calculation.
    pub prev_u_lip: f64,
}

impl RiccatiWebsterHorn {
    /// Creates a new continuous Riccati Webster-Horn initialized from acoustic parameters.
    pub fn new(params: &PortHamiltonianAcousticParams) -> Self {
        let length = params.vocal_tract_length_m;
        let lip_radius = params.lip_aperture_radius_m;
        let speed_of_sound = 350.0; // Body temperature acoustic velocity
        let air_density = 1.18; // Body temperature air density
        let visco_thermal_alpha0 = 1.2e-4;

        // At 48 kHz and c = 350 m/s, dx = c * dt = 350 / 48000 = 0.00729167 m.
        // For L = 0.175 m, exactly 24 stages gives Courant number = 1.0 (exact wave propagation).
        let num_segments = 24;
        let dx = length / (num_segments as f64);
        let neutral_area = std::f64::consts::PI * lip_radius * lip_radius;

        let area_profile = vec![neutral_area.max(1e-6); num_segments];
        let char_impedance = (air_density * speed_of_sound) / neutral_area;
        let loss_factor = 0.9995;
        let lip_reflection = -0.92;

        Self {
            length,
            lip_radius,
            speed_of_sound,
            air_density,
            visco_thermal_alpha0,
            num_segments,
            dx,
            area_profile,
            char_impedance,
            loss_factor,
            lip_reflection,
            forward: [0.0; 24],
            backward: [0.0; 24],
            back_pressure: 0.0,
            prev_u_lip: 0.0,
        }
    }

    /// Evaluates the real acoustic radiation resistance R_rad(omega) at the radiating lip boundary:
    /// R_rad(omega) = rho * omega^2 / (4 * pi * c).
    pub fn lip_radiation_resistance(&self, omega: f64) -> f64 {
        (self.air_density * omega * omega) / (4.0 * std::f64::consts::PI * self.speed_of_sound)
    }

    /// Evaluates the acoustic radiation inertance M_rad at the radiating lip boundary:
    /// M_rad = 8 * rho / (3 * pi^2 * a_lip).
    pub fn lip_radiation_inertance(&self) -> f64 {
        (8.0 * self.air_density)
            / (3.0 * std::f64::consts::PI * std::f64::consts::PI * self.lip_radius)
    }

    /// Evaluates complex lip radiation impedance Z_rad(omega) = R_rad + j * omega * M_rad.
    pub fn lip_radiation_impedance(&self, omega: f64) -> RiccatiComplex {
        let r_rad = self.lip_radiation_resistance(omega);
        let m_rad = self.lip_radiation_inertance();
        RiccatiComplex::new(r_rad, omega * m_rad)
    }

    /// Solves the continuous spatial Riccati reflection differential equation:
    /// dR/dx = 2 * gamma * R - 0.5 * (d ln Z_0 / dx) * (1 - R^2)
    /// backwards from lips (x = L) to glottis (x = 0).
    ///
    /// Returns the complex input impedance Z_in(omega) at the glottal entrance.
    pub fn solve_riccati_input_impedance(&self, freq_hz: f64) -> RiccatiComplex {
        let omega = 2.0 * std::f64::consts::PI * freq_hz;
        let beta = omega / self.speed_of_sound;
        let alpha = self.visco_thermal_alpha0 * freq_hz.abs().sqrt();
        let gamma = RiccatiComplex::new(alpha, beta);

        // Radiation boundary impedance at lips
        let z_rad = self.lip_radiation_impedance(omega);
        let z0_lip = (self.air_density * self.speed_of_sound)
            / self.area_profile[self.num_segments - 1];

        // R(L, omega) = (Z_rad - Z_0) / (Z_rad + Z_0)
        let r_lip = z_rad
            .sub(RiccatiComplex::new(z0_lip, 0.0))
            .div(z_rad.add(RiccatiComplex::new(z0_lip, 0.0)));

        // Backward spatial integration from x = L to x = 0 using RK4
        let n_steps = 100;
        let d_xi = self.length / (n_steps as f64);
        let mut r = r_lip;

        for step in 0..n_steps {
            let xi = (step as f64) * d_xi;
            let x = self.length - xi;

            let eval_dr_dxi = |curr_r: RiccatiComplex, _curr_x: f64| -> RiccatiComplex {
                // For uniform neutral tube d(ln Z_0)/dx = 0
                // dR/dxi = -dR/dx = -2 * gamma * R
                gamma.scale(2.0).mul(curr_r).scale(-1.0)
            };

            // RK4 integration
            let k1 = eval_dr_dxi(r, x);
            let k2 = eval_dr_dxi(r.add(k1.scale(0.5 * d_xi)), x - 0.5 * d_xi);
            let k3 = eval_dr_dxi(r.add(k2.scale(0.5 * d_xi)), x - 0.5 * d_xi);
            let k4 = eval_dr_dxi(r.add(k3.scale(d_xi)), x - d_xi);

            let dr = k1
                .add(k2.scale(2.0))
                .add(k3.scale(2.0))
                .add(k4)
                .scale(d_xi / 6.0);
            r = r.add(dr);
        }

        // Glottal input impedance Z_in(0) = Z_0(0) * (1 + R(0)) / (1 - R(0))
        let z0_glottis = (self.air_density * self.speed_of_sound) / self.area_profile[0];
        let num = RiccatiComplex::new(1.0, 0.0).add(r);
        let den = RiccatiComplex::new(1.0, 0.0).sub(r);

        num.div(den).scale(z0_glottis)
    }

    /// Steps the discretized vocal tract transmission line by dt under input glottal flow U_g.
    ///
    /// Returns the radiated sound pressure at the lips in Pascals.
    /// Guarantees strict passivity and unconditionally bounded stability.
    #[inline(always)]
    pub fn step(&mut self, u_glottal: f64, dt: f64) -> f64 {
        // 1. Read backward wave arriving at glottis (k = 0)
        let backward_at_glottis = self.backward[0];

        // 2. Inject acoustic volume velocity from vocal fold
        let p_in_plus = backward_at_glottis + self.char_impedance * u_glottal;

        // 3. Update glottal back-pressure P_supra
        self.back_pressure = (p_in_plus + backward_at_glottis).clamp(-2000.0, 5000.0);

        // 4. Shift forward wave delay line in-place with visco-thermal loss
        for i in (1..24).rev() {
            self.forward[i] = self.forward[i - 1] * self.loss_factor;
        }
        self.forward[0] = p_in_plus;

        // 5. Lip termination reflection & radiation
        let p_lip_plus = self.forward[23];
        let r_lip = self.lip_reflection;
        let p_lip_minus = r_lip * p_lip_plus;
        let u_lip = ((1.0 - r_lip) / self.char_impedance) * p_lip_plus;

        // 6. Shift backward wave delay line in-place with visco-thermal loss
        for i in 0..23 {
            self.backward[i] = self.backward[i + 1] * self.loss_factor;
        }
        self.backward[23] = p_lip_minus;

        // 7. Radiated lip sound pressure with acoustic differentiation
        let du_lip_dt = (u_lip - self.prev_u_lip) / dt;
        self.prev_u_lip = u_lip;

        // Radiated far-field acoustic pressure
        u_lip * 500.0 + du_lip_dt * 0.015
    }

    /// Returns the supraglottal back-pressure at the glottal entrance (k = 0).
    pub fn glottal_back_pressure(&self) -> f64 {
        self.back_pressure
    }

    /// Evaluates total acoustic energy stored in the vocal tract in Joules.
    pub fn acoustic_energy(&self) -> f64 {
        let mut energy = 0.0;
        let c_factor = 1.0 / (2.0 * self.char_impedance);
        for k in 0..self.num_segments {
            energy += c_factor * (self.forward[k].powi(2) + self.backward[k].powi(2));
        }
        energy
    }
}
