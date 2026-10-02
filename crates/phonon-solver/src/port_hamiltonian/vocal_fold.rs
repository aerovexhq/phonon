#![deny(unsafe_code)]

//! Hirano 3-Layer Vocal Fold Biomechanics with Mucosal Wave Delay & Dynamic Separation.
//!
//! Models stratified cover-body tissue mechanics, bottom-up vertical mucosal traveling
//! wave delay tau_m = T_h / c_m, non-linear Duffing stiffness, Hertzian contact collision,
//! and von Karman-Pohlhausen dynamic boundary layer separation.

use phonon_models::port_hamiltonian::PortHamiltonianAcousticParams;

/// Hirano Stratified Cover-Body Vocal Fold Biomechanical Oscillator.
#[derive(Debug, Clone, PartialEq)]
pub struct HiranoVocalFold {
    /// Effective tissue mass in kilograms.
    pub mass: f64,
    /// Vocal fold anterior-posterior length in meters.
    pub length: f64,
    /// Vocal fold vertical medial surface thickness in meters.
    pub thickness: f64,
    /// Mucosal surface wave propagation velocity in m/s.
    pub mucosal_velocity: f64,
    /// Vertical bottom-up mucosal wave delay in seconds.
    pub mucosal_delay_s: f64,
    /// Neutral rest glottal half-aperture in meters.
    pub rest_gap: f64,
    /// Linear tissue stiffness in N/m.
    pub k1: f64,
    /// Non-linear cubic Duffing hardening stiffness in N/m^3.
    pub k3: f64,
    /// Viscous tissue damping in N*s/m.
    pub damping: f64,
    /// Hertzian contact stiffness in N/m^1.5.
    pub contact_stiffness: f64,
    /// Hertzian contact damping in N*s/m.
    pub contact_damping: f64,

    /// Lower cover margin displacement in meters.
    pub x1: f64,
    /// Lower cover margin velocity in m/s.
    pub v1: f64,
    /// Upper mucosal margin displacement in meters (delayed).
    pub x2: f64,
    /// Current glottal volume velocity in m^3/s.
    pub volume_velocity: f64,
    /// Instantaneous flow detachment coordinate x_s in [0, T_h] (meters).
    pub separation_point: f64,
    /// Air density in kg/m^3.
    pub air_density: f64,
    /// Orifice discharge coefficient.
    pub discharge_coeff: f64,

    /// Circular buffer for mucosal wave traveling delay.
    pub delay_buffer: Vec<f64>,
    /// Write pointer in circular buffer.
    pub buffer_idx: usize,
    /// Total delay in discrete samples.
    pub delay_samples: f64,
    /// Sampling rate in Hz.
    pub sampling_rate_hz: f64,
}

impl HiranoVocalFold {
    /// Creates a new Hirano vocal fold oscillator initialized from acoustic parameters.
    pub fn new(params: &PortHamiltonianAcousticParams) -> Self {
        let mass = params.vocal_fold_mass_kg;
        let length = params.vocal_fold_length_m;
        let thickness = params.vocal_fold_thickness_m;
        let mucosal_velocity = params.mucosal_wave_velocity_m_s;
        let mucosal_delay_s = thickness / mucosal_velocity;
        let rest_gap = 0.0; // Adducted pre-phonatory glottal half-aperture

        // Natural frequency around 140 Hz (loaded phonation pitch ~140-180 Hz)
        let natural_freq_rad = 2.0 * std::f64::consts::PI * 140.0;
        let k1 = mass * natural_freq_rad * natural_freq_rad;
        let k3 = 2.5e7; // Duffing hardening
        let damping_ratio = 0.08;
        let damping = 2.0 * damping_ratio * (mass * k1).sqrt();

        let contact_stiffness = 3.5e4;
        let contact_damping = 1.0 * damping;

        let sampling_rate_hz = params.sampling_rate_hz;
        let delay_samples = mucosal_delay_s * sampling_rate_hz;
        let buffer_size = (delay_samples.ceil() as usize + 8).max(16);
        let delay_buffer = vec![0.0; buffer_size];

        Self {
            mass,
            length,
            thickness,
            mucosal_velocity,
            mucosal_delay_s,
            rest_gap,
            k1,
            k3,
            damping,
            contact_stiffness,
            contact_damping,
            x1: 0.0,
            v1: 0.0,
            x2: 0.0,
            volume_velocity: 0.0,
            separation_point: thickness,
            air_density: 1.2,
            discharge_coeff: 0.85,
            delay_buffer,
            buffer_idx: 0,
            delay_samples,
            sampling_rate_hz,
        }
    }

    /// Evaluates the vertical mucosal traveling wave delay tau_m = T_h / c_m in seconds.
    pub fn mucosal_wave_delay_s(&self) -> f64 {
        self.mucosal_delay_s
    }

    /// Retrieves delayed lower margin displacement x_1(t - tau_m) via fractional circular interpolation.
    pub fn read_delayed_displacement(&self) -> f64 {
        let buf_len = self.delay_buffer.len();
        let exact_read_pos = (self.buffer_idx as f64) - self.delay_samples;
        let norm_pos = if exact_read_pos < 0.0 {
            exact_read_pos + ((exact_read_pos.abs() / (buf_len as f64)).ceil() * (buf_len as f64))
        } else {
            exact_read_pos
        } % (buf_len as f64);

        let idx0 = norm_pos.floor() as usize % buf_len;
        let idx1 = (idx0 + 1) % buf_len;
        let frac = norm_pos - norm_pos.floor();

        (1.0 - frac) * self.delay_buffer[idx0] + frac * self.delay_buffer[idx1]
    }

    /// Evaluates and updates dynamic boundary layer separation coordinate x_s in [0, T_h].
    pub fn update_separation_point(&mut self) -> f64 {
        let h1 = self.rest_gap + self.x1;
        let h2 = self.rest_gap + self.x2;

        if h1 >= h2 {
            // Convergent duct: favorable pressure gradient, attached to exit
            self.separation_point = self.thickness;
        } else {
            // Divergent duct: adverse pressure gradient detachment
            let expansion_ratio = if h2 > 1e-9 {
                (h1 / h2).clamp(0.05, 1.0)
            } else {
                0.25
            };
            self.separation_point = (self.thickness * expansion_ratio * 0.25).min(self.thickness * 0.9);
        }
        self.separation_point
    }

    /// Evaluates the intraglottal aerodynamic pressure P(z) at vertical position z in [0, T_h].
    pub fn pressure_at_depth(&self, z: f64, p_sub: f64, p_supra: f64) -> f64 {
        let z_clamped = z.clamp(0.0, self.thickness);
        let h1 = self.rest_gap + self.x1;
        let h2 = self.rest_gap + self.x2;
        let h_min = h1.min(h2);

        if h_min <= 0.0 {
            // Closed glottis: subglottal pressure acts on inferior surface
            return p_sub;
        }

        // Downstream of separation: pressure recovers to supraglottal exit pressure
        if z_clamped >= self.separation_point {
            return p_supra;
        }

        // Upstream of separation: attached Bernoulli flow
        let frac = if self.thickness > 0.0 {
            z_clamped / self.thickness
        } else {
            0.0
        };
        let h_z = (h1 + frac * (h2 - h1)).max(1e-6);
        let area_z = (2.0 * self.length * h_z).max(1e-8);
        let dynamic_p = 0.5 * self.air_density * (self.volume_velocity / area_z).powi(2);

        (p_sub - dynamic_p).max(p_supra)
    }

    /// Steps the vocal fold biomechanical oscillator by time dt under given pressures.
    ///
    /// Returns the glottal volume flow U_g (m^3/s).
    #[inline(always)]
    pub fn step(&mut self, dt: f64, p_sub: f64, p_supra: f64) -> f64 {
        // 1. Read mucosal delayed upper margin displacement
        self.x2 = self.read_delayed_displacement();

        let h1 = self.rest_gap + self.x1;
        let h2 = self.rest_gap + self.x2;
        let h_min = h1.min(h2);

        let f_aero: f64;

        if h_min <= 0.0 {
            // Glottis closed
            self.volume_velocity = 0.0;
            self.separation_point = 0.0;
            f_aero = p_sub * self.length * self.thickness;
        } else {
            // Glottis open
            let glottal_area = 2.0 * self.length * h_min;
            let transglottal_dp = (p_sub - p_supra).max(0.0);
            let flow_vel = (2.0 * transglottal_dp / self.air_density).sqrt();
            self.volume_velocity = self.discharge_coeff * glottal_area * flow_vel;

            // Determine dynamic boundary layer detachment point x_s
            self.update_separation_point();

            let xs = self.separation_point;
            let dynamic_p = 0.5 * self.air_density * flow_vel * flow_vel * 0.4;
            let p_attached = (p_sub - dynamic_p).max(p_supra);
            let force_attached = xs * p_attached;
            let force_separated = (self.thickness - xs) * p_supra;
            f_aero = (force_attached + force_separated) * self.length;
        }

        // Laryngeal adductory muscle pre-stress balancing mean respiratory drive
        let f_adduct = 0.22 * p_sub * self.length * self.thickness;

        // 2. Elastic restoring force with Duffing cubic hardening
        let f_elastic = -(self.k1 * self.x1 + self.k3 * self.x1.powi(3));

        // 3. Viscous damping force
        let f_damping = -self.damping * self.v1;

        // 4. Contact collision mechanics: Hertzian contact with compressive damping
        let mut f_contact = 0.0;
        let penetration = -self.rest_gap - self.x1;
        if penetration > 0.0 {
            let p_contact = penetration.powf(1.5);
            let inward_vel = (-self.v1).max(0.0);
            f_contact = self.contact_stiffness * p_contact + self.contact_damping * inward_vel;
        }

        // 5. Total force & acceleration
        let f_total = f_elastic + f_damping + f_contact + f_aero - f_adduct;
        let a1 = f_total / self.mass;

        // 6. Symplectic time-stepping update
        self.v1 += a1 * dt;
        self.x1 += self.v1 * dt;

        // 7. Record current lower displacement into delay ring buffer
        self.delay_buffer[self.buffer_idx] = self.x1;
        self.buffer_idx = (self.buffer_idx + 1) % self.delay_buffer.len();

        self.volume_velocity
    }

    /// Evaluates mechanical stored energy (kinetic + Duffing potential + contact potential).
    pub fn mechanical_energy(&self) -> f64 {
        let kinetic = 0.5 * self.mass * self.v1.powi(2);
        let v_elastic = 0.5 * self.k1 * self.x1.powi(2) + 0.25 * self.k3 * self.x1.powi(4);
        let penetration = (-self.rest_gap - self.x1).max(0.0);
        let v_contact = if penetration > 0.0 {
            0.4 * self.contact_stiffness * penetration.powf(2.5)
        } else {
            0.0
        };
        kinetic + v_elastic + v_contact
    }
}
