//! Laser-plasma wakefield accelerator tracker: beam bunch evolution,
//! self-consistent cavity co-moving frames, slice energy spread, and normalized emittance.

use crate::wakefield::relativistic_boris_pusher::{
    BorisPusher, RelativisticParticle, SPEED_OF_LIGHT,
};
use phonon_models::wakefield::{BetatronRadiation, BubbleRegime};
use rayon::prelude::*;

/// Ensemble of relativistic charged particles forming an accelerated bunch.
#[derive(Debug, Clone, PartialEq)]
pub struct BeamBunch {
    /// Vector of individual particles.
    pub particles: Vec<RelativisticParticle>,
}

impl BeamBunch {
    /// Creates a bunch from a vector of particles.
    pub fn new(particles: Vec<RelativisticParticle>) -> Self {
        Self { particles }
    }

    /// Synthesizes a Gaussian electron bunch with specified spatial and momentum spread.
    pub fn new_gaussian(
        count: usize,
        mean_energy_mev: f64,
        energy_spread_rel: f64,
        sigma_transverse_m: f64,
        sigma_longitudinal_m: f64,
        initial_z_m: f64,
    ) -> Self {
        let mut particles = Vec::with_capacity(count);
        let mut seed = 0x1234_5678_9ABC_DEF0_u64;

        for _ in 0..count {
            let x = sample_gaussian(0.0, sigma_transverse_m, &mut seed);
            let y = sample_gaussian(0.0, sigma_transverse_m, &mut seed);
            let z = sample_gaussian(initial_z_m, sigma_longitudinal_m, &mut seed);

            let e_mev = sample_gaussian(
                mean_energy_mev,
                mean_energy_mev * energy_spread_rel,
                &mut seed,
            )
            .max(1.0);

            let mut p = RelativisticParticle::electron_with_energy_mev([x, y, z], e_mev);
            let div_x = sample_gaussian(0.0, 1.0e-3, &mut seed);
            let div_y = sample_gaussian(0.0, 1.0e-3, &mut seed);
            p.proper_velocity[0] = p.proper_velocity[2] * div_x;
            p.proper_velocity[1] = p.proper_velocity[2] * div_y;

            particles.push(p);
        }

        Self { particles }
    }

    /// Returns the number of particles in the bunch.
    pub fn len(&self) -> usize {
        self.particles.len()
    }

    /// Checks if the bunch is empty.
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    /// Mean position [x_bar, y_bar, z_bar] in meters.
    pub fn mean_position(&self) -> [f64; 3] {
        if self.particles.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let count = self.particles.len() as f64;
        let sum = self.particles.iter().fold([0.0, 0.0, 0.0], |acc, p| {
            [
                acc[0] + p.position[0],
                acc[1] + p.position[1],
                acc[2] + p.position[2],
            ]
        });
        [sum[0] / count, sum[1] / count, sum[2] / count]
    }

    /// Mean electron kinetic energy in MeV.
    pub fn mean_energy_mev(&self) -> f64 {
        if self.particles.is_empty() {
            return 0.0;
        }
        let total: f64 = self.particles.iter().map(|p| p.kinetic_energy_mev()).sum();
        total / self.particles.len() as f64
    }

    /// Relative energy spread sigma_E / E_bar.
    pub fn energy_spread_rel(&self) -> f64 {
        if self.particles.len() < 2 {
            return 0.0;
        }
        let mean = self.mean_energy_mev();
        let var: f64 = self
            .particles
            .iter()
            .map(|p| (p.kinetic_energy_mev() - mean).powi(2))
            .sum::<f64>()
            / (self.particles.len() as f64 - 1.0);
        var.sqrt() / mean.max(1e-12)
    }

    /// RMS bunch length along the longitudinal axis sigma_z in meters.
    pub fn bunch_length_rms_m(&self) -> f64 {
        if self.particles.len() < 2 {
            return 0.0;
        }
        let mean_z = self.mean_position()[2];
        let var: f64 = self
            .particles
            .iter()
            .map(|p| (p.position[2] - mean_z).powi(2))
            .sum::<f64>()
            / (self.particles.len() as f64 - 1.0);
        var.sqrt()
    }

    /// RMS bunch duration tau_b = sigma_z / c in seconds.
    pub fn bunch_duration_s(&self) -> f64 {
        self.bunch_length_rms_m() / SPEED_OF_LIGHT
    }

    /// Normalized transverse emittance epsilon_{n,x} in meter-radians:
    /// epsilon_{n,x} = (1 / c) * sqrt( <(x - x_bar)^2> <(u_x - u_{x,bar})^2> - <(x - x_bar)(u_x - u_{x,bar})>^2 )
    pub fn normalized_emittance_x_m_rad(&self) -> f64 {
        self.calculate_emittance_dim(0)
    }

    /// Normalized transverse emittance epsilon_{n,y} in meter-radians:
    pub fn normalized_emittance_y_m_rad(&self) -> f64 {
        self.calculate_emittance_dim(1)
    }

    fn calculate_emittance_dim(&self, dim: usize) -> f64 {
        if self.particles.len() < 2 {
            return 0.0;
        }
        let count = self.particles.len() as f64;
        let mean_pos = self.particles.iter().map(|p| p.position[dim]).sum::<f64>() / count;
        let mean_u = self
            .particles
            .iter()
            .map(|p| p.proper_velocity[dim])
            .sum::<f64>()
            / count;

        let mut var_pos = 0.0;
        let mut var_u = 0.0;
        let mut cov_pos_u = 0.0;

        for p in &self.particles {
            let dx = p.position[dim] - mean_pos;
            let du = p.proper_velocity[dim] - mean_u;
            var_pos += dx * dx;
            var_u += du * du;
            cov_pos_u += dx * du;
        }

        var_pos /= count;
        var_u /= count;
        cov_pos_u /= count;

        let emittance_sq = (var_pos * var_u - cov_pos_u * cov_pos_u).max(0.0);
        emittance_sq.sqrt() / SPEED_OF_LIGHT
    }
}

/// Wakefield accelerator simulator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WakefieldAccelerator {
    /// Relativistic blowout bubble regime parameters.
    pub bubble: BubbleRegime,
    /// Synchrotron betatron radiation dynamics.
    pub betatron: BetatronRadiation,
    /// Flag indicating whether radiation reaction damping is modeled.
    pub radiation_reaction_enabled: bool,
}

/// Summary of a tracking run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackingSummary {
    /// Initial bunch mean energy in MeV.
    pub initial_energy_mev: f64,
    /// Final bunch mean energy in MeV.
    pub final_energy_mev: f64,
    /// Initial relative energy spread.
    pub initial_energy_spread: f64,
    /// Final relative energy spread.
    pub final_energy_spread: f64,
    /// Initial normalized emittance in meter-radians.
    pub initial_emittance_x: f64,
    /// Final normalized emittance in meter-radians.
    pub final_emittance_x: f64,
    /// Initial bunch duration in seconds.
    pub initial_duration_s: f64,
    /// Final bunch duration in seconds.
    pub final_duration_s: f64,
    /// Critical synchrotron photon energy at final mean energy in keV.
    pub critical_synchrotron_photon_energy_kev: f64,
    /// Total acceleration distance in meters.
    pub distance_traveled_m: f64,
    /// Number of time integration steps executed.
    pub steps_taken: usize,
}

impl WakefieldAccelerator {
    /// Creates a wakefield accelerator simulator.
    pub fn new(
        bubble: BubbleRegime,
        betatron: BetatronRadiation,
        radiation_reaction_enabled: bool,
    ) -> Self {
        Self {
            bubble,
            betatron,
            radiation_reaction_enabled,
        }
    }

    /// Computes self-consistent (E, B) electromagnetic field at coordinate and time.
    /// Laser co-moving coordinate xi = z - c * t.
    pub fn field_at(&self, position: [f64; 3], time: f64) -> ([f64; 3], [f64; 3]) {
        let c = SPEED_OF_LIGHT;
        let xi = position[2] - c * time;
        let rb = self.bubble.bubble_radius_m();

        // Check if inside wakefield cavitation bubble: xi in [-rb, 0]
        if xi <= 0.0 && xi >= -rb {
            let r = (position[0].powi(2) + position[1].powi(2))
                .sqrt()
                .max(1e-12);

            // Longitudinal accelerating field: E_z < 0 accelerates electrons (q < 0) along +z
            let ez = self.bubble.longitudinal_field_v_per_m(xi);

            // Transverse bare ion restoring field:
            // Bare ion background has positive charge density, producing outward E_r > 0
            // which exerts restoring force F_r = -e E_r < 0 on electrons.
            let er_magnitude = self.bubble.transverse_focusing_field_v_per_m(r).abs();
            let ex = er_magnitude * (position[0] / r);
            let ey = er_magnitude * (position[1] / r);

            ([ex, ey, ez], [0.0, 0.0, 0.0])
        } else {
            ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0])
        }
    }

    /// Steps an entire beam bunch forward by dt using parallel Rayon iteration.
    pub fn step_bunch(&self, bunch: &mut BeamBunch, time: f64, dt: f64) {
        let rad_reaction = self.radiation_reaction_enabled;
        bunch.particles.par_iter_mut().for_each(|particle| {
            let (e, b) = self.field_at(particle.position, time);
            BorisPusher::step(particle, e, b, dt, rad_reaction);
        });
    }

    /// Tracks a bunch through the accelerator for specified target distance.
    pub fn track(&self, bunch: &mut BeamBunch, target_distance_m: f64, dt: f64) -> TrackingSummary {
        let initial_energy = bunch.mean_energy_mev();
        let initial_spread = bunch.energy_spread_rel();
        let initial_emit = bunch.normalized_emittance_x_m_rad();
        let initial_duration = bunch.bunch_duration_s();

        let initial_z = bunch.mean_position()[2];
        let mut time = 0.0;
        let mut steps = 0;

        while (bunch.mean_position()[2] - initial_z) < target_distance_m && steps < 100_000 {
            self.step_bunch(bunch, time, dt);
            time += dt;
            steps += 1;
        }

        let final_energy = bunch.mean_energy_mev();
        let final_gamma = if bunch.is_empty() {
            1.0
        } else {
            bunch.particles[0].gamma()
        };
        let final_emit = bunch.normalized_emittance_x_m_rad();
        let crit_photon_kev = self
            .betatron
            .critical_photon_energy_kev(final_gamma, 1.0e-6);

        TrackingSummary {
            initial_energy_mev: initial_energy,
            final_energy_mev: final_energy,
            initial_energy_spread: initial_spread,
            final_energy_spread: bunch.energy_spread_rel(),
            initial_emittance_x: initial_emit,
            final_emittance_x: final_emit,
            initial_duration_s: initial_duration,
            final_duration_s: bunch.bunch_duration_s(),
            critical_synchrotron_photon_energy_kev: crit_photon_kev,
            distance_traveled_m: bunch.mean_position()[2] - initial_z,
            steps_taken: steps,
        }
    }
}

fn sample_gaussian(mean: f64, std_dev: f64, state: &mut u64) -> f64 {
    let u1 = next_pseudo_uniform(state).max(1e-12);
    let u2 = next_pseudo_uniform(state);
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + z0 * std_dev
}

fn next_pseudo_uniform(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state as f64) / (u64::MAX as f64)
}
