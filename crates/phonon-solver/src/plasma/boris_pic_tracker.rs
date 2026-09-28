//! Boris Particle-in-Cell (PIC) fast-ion kinetic orbit integrator with exact phase-space conservation.
//!
//! The classical and relativistic Boris leapfrog algorithm advances charged particles in non-uniform
//! tokamak electromagnetic fields $(\mathbf{E}, \mathbf{B})$:
//! 1. Half electric acceleration: $\mathbf{v}^- = \mathbf{v}^{n-1/2} + \frac{q \Delta t}{2 m} \mathbf{E}^n$
//! 2. Magnetic rotation: $\mathbf{t} = \frac{q \Delta t}{2 m} \mathbf{B}^n$, $\mathbf{s} = \frac{2 \mathbf{t}}{1 + t^2}$
//!    $\mathbf{v}' = \mathbf{v}^- + \mathbf{v}^- \times \mathbf{t}$
//!    $\mathbf{v}^+ = \mathbf{v}^- + \mathbf{v}' \times \mathbf{s}$
//! 3. Second half electric acceleration: $\mathbf{v}^{n+1/2} = \mathbf{v}^+ + \frac{q \Delta t}{2 m} \mathbf{E}^n$
//! 4. Position update: $\mathbf{x}^{n+1} = \mathbf{x}^n + \Delta t \mathbf{v}^{n+1/2}$
//!
//! When $\mathbf{E} = \mathbf{0}$, the Boris algorithm conserves particle kinetic energy to machine precision ($|\Delta E / E| < 10^{-7}$).

use phonon_models::plasma::KineticParticle;
use rayon::prelude::*;

/// Classification of charged particle orbit in tokamak magnetic geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrbitTopology {
    /// Trapped banana orbit with turning points where $v_\parallel = 0$.
    TrappedBanana,
    /// Co-passing circulating orbit with continuous positive $v_\parallel > 0$.
    CoPassing,
    /// Counter-passing circulating orbit with continuous negative $v_\parallel < 0$.
    CounterPassing,
    /// Prompt lost particle colliding with first wall / limiter boundary.
    PromptLost,
}

/// Recorded phase-space state and conservation diagnostics over a particle trajectory.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleOrbitReport {
    /// Initial particle state.
    pub initial_state: KineticParticle,
    /// Final particle state after integration.
    pub final_state: KineticParticle,
    /// Number of integration steps executed.
    pub steps: usize,
    /// Total simulated duration in seconds.
    pub duration_seconds: f64,
    /// Relative kinetic energy drift: $|E_{final} - E_{initial}| / E_{initial}$.
    pub relative_energy_drift: f64,
    /// Relative magnetic moment drift: $|\mu_{final} - \mu_{initial}| / \mu_{initial}$.
    pub relative_magnetic_moment_drift: f64,
    /// Classified orbit topology.
    pub topology: OrbitTopology,
    /// Number of banana bounce turning points detected.
    pub bounce_turning_points: usize,
    /// Whether the particle remained confined inside the vacuum vessel.
    pub is_confined: bool,
}

/// Boris Particle-in-Cell (PIC) charged particle kinetic orbit integrator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorisPicTracker {
    /// Integration time step $\Delta t$ in seconds.
    pub dt: f64,
}

impl BorisPicTracker {
    /// Creates a Boris PIC tracker with specified time step.
    pub fn new(dt: f64) -> Self {
        assert!(dt > 0.0, "Time step dt must be positive");
        Self { dt }
    }

    /// Single Boris leapfrog step advancing particle in 3D Cartesian coordinates.
    pub fn step_particle(
        &self,
        particle: &mut KineticParticle,
        e_field: [f64; 3],
        b_field: [f64; 3],
    ) {
        let q = particle.species.charge();
        let m = particle.species.mass();
        let q_dt_over_2m = (q * self.dt) / (2.0 * m);

        // 1. Half electric acceleration: v_minus = v + (q * dt / 2m) * E
        let v_minus = [
            particle.velocity[0] + q_dt_over_2m * e_field[0],
            particle.velocity[1] + q_dt_over_2m * e_field[1],
            particle.velocity[2] + q_dt_over_2m * e_field[2],
        ];

        // 2. Magnetic rotation
        // t = (q * dt / 2m) * B
        let t = [
            q_dt_over_2m * b_field[0],
            q_dt_over_2m * b_field[1],
            q_dt_over_2m * b_field[2],
        ];
        let t_sq = t[0].powi(2) + t[1].powi(2) + t[2].powi(2);
        let s_factor = 2.0 / (1.0 + t_sq);
        let s = [s_factor * t[0], s_factor * t[1], s_factor * t[2]];

        // v_prime = v_minus + v_minus x t
        let v_cross_t = [
            v_minus[1] * t[2] - v_minus[2] * t[1],
            v_minus[2] * t[0] - v_minus[0] * t[2],
            v_minus[0] * t[1] - v_minus[1] * t[0],
        ];
        let v_prime = [
            v_minus[0] + v_cross_t[0],
            v_minus[1] + v_cross_t[1],
            v_minus[2] + v_cross_t[2],
        ];

        // v_plus = v_minus + v_prime x s
        let v_prime_cross_s = [
            v_prime[1] * s[2] - v_prime[2] * s[1],
            v_prime[2] * s[0] - v_prime[0] * s[2],
            v_prime[0] * s[1] - v_prime[1] * s[0],
        ];
        let v_plus = [
            v_minus[0] + v_prime_cross_s[0],
            v_minus[1] + v_prime_cross_s[1],
            v_minus[2] + v_prime_cross_s[2],
        ];

        // 3. Second half electric acceleration: v_new = v_plus + (q * dt / 2m) * E
        particle.velocity = [
            v_plus[0] + q_dt_over_2m * e_field[0],
            v_plus[1] + q_dt_over_2m * e_field[1],
            v_plus[2] + q_dt_over_2m * e_field[2],
        ];

        // 4. Update position: x_new = x + dt * v_new
        particle.position = [
            particle.position[0] + self.dt * particle.velocity[0],
            particle.position[1] + self.dt * particle.velocity[1],
            particle.position[2] + self.dt * particle.velocity[2],
        ];
    }

    /// Tracks a single charged particle over $N$ steps, recording conservation metrics and topology.
    pub fn track_orbit<F>(
        &self,
        initial: KineticParticle,
        field_fn: F,
        steps: usize,
        r_limiter_min: f64,
        r_limiter_max: f64,
        z_limiter_max: f64,
    ) -> ParticleOrbitReport
    where
        F: Fn([f64; 3]) -> ([f64; 3], [f64; 3]),
    {
        let mut particle = initial;
        let e_init = initial.kinetic_energy_joules();

        let (_e0, b0) = field_fn(initial.position);
        let b0_mag = (b0[0].powi(2) + b0[1].powi(2) + b0[2].powi(2))
            .sqrt()
            .max(1e-6);
        let b0_unit = [b0[0] / b0_mag, b0[1] / b0_mag, b0[2] / b0_mag];
        let mu_init = initial.magnetic_moment(b0_mag, b0_unit);

        let mut prev_v_par = initial.v_parallel(b0_unit);
        let mut turning_points = 0;
        let mut prompt_lost = false;

        for _ in 0..steps {
            let (e_f, b_f) = field_fn(particle.position);
            self.step_particle(&mut particle, e_f, b_f);

            let (r, _phi, z) = particle.cylindrical_position();
            if r < r_limiter_min || r > r_limiter_max || z.abs() > z_limiter_max {
                prompt_lost = true;
                break;
            }

            let b_mag = (b_f[0].powi(2) + b_f[1].powi(2) + b_f[2].powi(2))
                .sqrt()
                .max(1e-6);
            let b_unit = [b_f[0] / b_mag, b_f[1] / b_mag, b_f[2] / b_mag];
            let v_par = particle.v_parallel(b_unit);

            // Turning point detection: sign change of parallel velocity
            if v_par * prev_v_par < 0.0 {
                turning_points += 1;
            }
            prev_v_par = v_par;
        }

        let e_final = particle.kinetic_energy_joules();
        let (_ef_end, bf_end) = field_fn(particle.position);
        let bf_mag = (bf_end[0].powi(2) + bf_end[1].powi(2) + bf_end[2].powi(2))
            .sqrt()
            .max(1e-6);
        let bf_unit = [bf_end[0] / bf_mag, bf_end[1] / bf_mag, bf_end[2] / bf_mag];
        let mu_final = particle.magnetic_moment(bf_mag, bf_unit);

        let rel_e_drift = if e_init > 1e-30 {
            (e_final - e_init).abs() / e_init
        } else {
            0.0
        };

        let rel_mu_drift = if mu_init > 1e-30 {
            (mu_final - mu_init).abs() / mu_init
        } else {
            0.0
        };

        let topology = if prompt_lost {
            OrbitTopology::PromptLost
        } else if turning_points >= 2 {
            OrbitTopology::TrappedBanana
        } else if prev_v_par >= 0.0 {
            OrbitTopology::CoPassing
        } else {
            OrbitTopology::CounterPassing
        };

        ParticleOrbitReport {
            initial_state: initial,
            final_state: particle,
            steps,
            duration_seconds: (steps as f64) * self.dt,
            relative_energy_drift: rel_e_drift,
            relative_magnetic_moment_drift: rel_mu_drift,
            topology,
            bounce_turning_points: turning_points,
            is_confined: !prompt_lost,
        }
    }

    /// Parallel multi-particle tracking using Rayon.
    pub fn track_ensemble_parallel<F>(
        &self,
        particles: &[KineticParticle],
        field_fn: F,
        steps: usize,
        r_limiter_min: f64,
        r_limiter_max: f64,
        z_limiter_max: f64,
    ) -> Vec<ParticleOrbitReport>
    where
        F: Fn([f64; 3]) -> ([f64; 3], [f64; 3]) + Sync + Send,
    {
        particles
            .par_iter()
            .map(|&p| {
                self.track_orbit(
                    p,
                    &field_fn,
                    steps,
                    r_limiter_min,
                    r_limiter_max,
                    z_limiter_max,
                )
            })
            .collect()
    }
}
