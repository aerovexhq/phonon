//! Relativistic Boris particle integrator with Landau-Lifshitz radiation reaction damping.

use std::f64::consts::PI;

/// Speed of light in vacuum (m/s).
pub const SPEED_OF_LIGHT: f64 = 2.997_924_58e8;
/// Elementary charge (C).
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;
/// Electron rest mass (kg).
pub const ELECTRON_MASS_KG: f64 = 9.109_383_7e-31;
/// Vacuum permittivity (F/m).
pub const VACUUM_PERMITTIVITY: f64 = 8.854_187_812_8e-12;

/// A relativistic charged particle tracked in 3D coordinate and proper velocity space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RelativisticParticle {
    /// 3D position [x, y, z] in meters.
    pub position: [f64; 3],
    /// 3D proper velocity [u_x, u_y, u_z] in m/s, where u = gamma * v.
    pub proper_velocity: [f64; 3],
    /// Electric charge in Coulombs (negative for electrons).
    pub charge: f64,
    /// Invariant rest mass in kilograms.
    pub mass: f64,
}

impl RelativisticParticle {
    /// Creates a generic charged particle.
    pub fn new(position: [f64; 3], proper_velocity: [f64; 3], charge: f64, mass: f64) -> Self {
        Self {
            position,
            proper_velocity,
            charge,
            mass,
        }
    }

    /// Creates an electron with standard charge (-e) and rest mass (m_e).
    pub fn electron(position: [f64; 3], proper_velocity: [f64; 3]) -> Self {
        Self::new(
            position,
            proper_velocity,
            -ELEMENTARY_CHARGE,
            ELECTRON_MASS_KG,
        )
    }

    /// Creates an electron from kinetic energy in MeV directed along the +z axis.
    pub fn electron_with_energy_mev(position: [f64; 3], kinetic_energy_mev: f64) -> Self {
        let rest_energy_mev = 0.510_998_95;
        let gamma = 1.0 + (kinetic_energy_mev / rest_energy_mev).max(0.0);
        let beta = (1.0 - 1.0 / (gamma * gamma)).max(0.0).sqrt();
        let uz = gamma * beta * SPEED_OF_LIGHT;
        Self::electron(position, [0.0, 0.0, uz])
    }

    /// Lorentz factor gamma = sqrt(1 + |u|^2 / c^2).
    #[inline]
    pub fn gamma(&self) -> f64 {
        let u2 = self.proper_velocity[0].powi(2)
            + self.proper_velocity[1].powi(2)
            + self.proper_velocity[2].powi(2);
        (1.0 + u2 / (SPEED_OF_LIGHT * SPEED_OF_LIGHT)).sqrt()
    }

    /// Physical 3D velocity v = u / gamma in m/s.
    #[inline]
    pub fn velocity(&self) -> [f64; 3] {
        let g = self.gamma();
        [
            self.proper_velocity[0] / g,
            self.proper_velocity[1] / g,
            self.proper_velocity[2] / g,
        ]
    }

    /// Kinetic energy in MeV: E_k = (gamma - 1) m c^2 / e * 10^-6.
    pub fn kinetic_energy_mev(&self) -> f64 {
        let g = self.gamma();
        let e_joules = (g - 1.0) * self.mass * SPEED_OF_LIGHT * SPEED_OF_LIGHT;
        e_joules / (ELEMENTARY_CHARGE * 1.0e6)
    }

    /// Total relativistic energy in MeV: E_tot = gamma m c^2 / e * 10^-6.
    pub fn total_energy_mev(&self) -> f64 {
        let g = self.gamma();
        let e_joules = g * self.mass * SPEED_OF_LIGHT * SPEED_OF_LIGHT;
        e_joules / (ELEMENTARY_CHARGE * 1.0e6)
    }
}

/// Relativistic Boris particle pusher with optional radiation reaction damping.
pub struct BorisPusher;

impl BorisPusher {
    /// Steps a particle forward by timestep dt under electric field E and magnetic field B.
    pub fn step(
        particle: &mut RelativisticParticle,
        e_field: [f64; 3],
        b_field: [f64; 3],
        dt: f64,
        include_radiation_reaction: bool,
    ) {
        let q = particle.charge;
        let m = particle.mass;
        let c = SPEED_OF_LIGHT;

        // 1. Half electric acceleration: u_minus = u_n + (q * dt / (2 * m)) * E
        let half_acc = (q * dt) / (2.0 * m);
        let u_minus = [
            particle.proper_velocity[0] + half_acc * e_field[0],
            particle.proper_velocity[1] + half_acc * e_field[1],
            particle.proper_velocity[2] + half_acc * e_field[2],
        ];

        // 2. Compute gamma_minus
        let u_minus_sq = u_minus[0].powi(2) + u_minus[1].powi(2) + u_minus[2].powi(2);
        let gamma_minus = (1.0 + u_minus_sq / (c * c)).sqrt();

        // 3. Magnetic rotation vector t = (q * dt / (2 * m * gamma_minus)) * B
        let t_factor = (q * dt) / (2.0 * m * gamma_minus);
        let t = [
            t_factor * b_field[0],
            t_factor * b_field[1],
            t_factor * b_field[2],
        ];
        let t_mag_sq = t[0].powi(2) + t[1].powi(2) + t[2].powi(2);
        let s_factor = 2.0 / (1.0 + t_mag_sq);
        let s = [s_factor * t[0], s_factor * t[1], s_factor * t[2]];

        // u_prime = u_minus + u_minus x t
        let u_prime = [
            u_minus[0] + (u_minus[1] * t[2] - u_minus[2] * t[1]),
            u_minus[1] + (u_minus[2] * t[0] - u_minus[0] * t[2]),
            u_minus[2] + (u_minus[0] * t[1] - u_minus[1] * t[0]),
        ];

        // u_plus = u_minus + u_prime x s
        let u_plus = [
            u_minus[0] + (u_prime[1] * s[2] - u_prime[2] * s[1]),
            u_minus[1] + (u_prime[2] * s[0] - u_prime[0] * s[2]),
            u_minus[2] + (u_prime[0] * s[1] - u_prime[1] * s[0]),
        ];

        // 4. Second half electric acceleration: u_{n+1} = u_plus + (q * dt / (2 * m)) * E
        let mut u_next = [
            u_plus[0] + half_acc * e_field[0],
            u_plus[1] + half_acc * e_field[1],
            u_plus[2] + half_acc * e_field[2],
        ];

        // 5. Radiation reaction damping (Landau-Lifshitz / Larmor radiated power)
        if include_radiation_reaction {
            let u_sq = u_next[0].powi(2) + u_next[1].powi(2) + u_next[2].powi(2);
            let gamma_curr = (1.0 + u_sq / (c * c)).sqrt();
            let v = [
                u_next[0] / gamma_curr,
                u_next[1] / gamma_curr,
                u_next[2] / gamma_curr,
            ];

            // Lorentz force: F_L = q * (E + v x B)
            let v_cross_b = [
                v[1] * b_field[2] - v[2] * b_field[1],
                v[2] * b_field[0] - v[0] * b_field[2],
                v[0] * b_field[1] - v[1] * b_field[0],
            ];
            let f_em = [
                q * (e_field[0] + v_cross_b[0]),
                q * (e_field[1] + v_cross_b[1]),
                q * (e_field[2] + v_cross_b[2]),
            ];
            let f_em_sq = f_em[0].powi(2) + f_em[1].powi(2) + f_em[2].powi(2);
            let v_dot_e = v[0] * e_field[0] + v[1] * e_field[1] + v[2] * e_field[2];
            let v_dot_e_sq = (v_dot_e / c).powi(2);

            let factor = (q.powi(2) * gamma_curr.powi(2))
                / (6.0 * PI * VACUUM_PERMITTIVITY * m.powi(2) * c.powi(3));
            let p_rad = (factor * (f_em_sq - q.powi(2) * v_dot_e_sq)).max(0.0);

            let total_energy_joules = gamma_curr * m * c * c;
            let damping = (1.0 - (p_rad * dt) / total_energy_joules).clamp(0.0, 1.0);
            u_next[0] *= damping;
            u_next[1] *= damping;
            u_next[2] *= damping;
        }

        // 6. Update position: x_{n+1} = x_n + (u_{n+1} / gamma_{n+1}) * dt
        let u_final_sq = u_next[0].powi(2) + u_next[1].powi(2) + u_next[2].powi(2);
        let gamma_final = (1.0 + u_final_sq / (c * c)).sqrt();

        particle.proper_velocity = u_next;
        particle.position[0] += (u_next[0] / gamma_final) * dt;
        particle.position[1] += (u_next[1] / gamma_final) * dt;
        particle.position[2] += (u_next[2] / gamma_final) * dt;
    }
}
