//! Multi-fluid extended magnetohydrodynamics time-domain wave stepper.
//!
//! Propagates shear Alfvén waves, Hall currents, resistive magnetic diffusion,
//! and pressure perturbations across tokamak radial flux surfaces.

use phonon_models::plasma::{
    AlfvenWaveProperties, MhdFluidState, SafetyFactorProfile, TokamakGeometry, VACUUM_PERMEABILITY,
};
use rayon::prelude::*;

/// 1D radial flux-surface discretization node for shear Alfvén wave dynamics.
#[derive(Debug, Clone, PartialEq)]
pub struct AlfvenFluxNode {
    /// Normalized minor radius $\rho = r / a \in [0, 1]$.
    pub rho: f64,
    /// Physical minor radius $r$ (meters).
    pub minor_r: f64,
    /// Local equilibrium magnetic field strength $B_0$ (Tesla).
    pub b0: f64,
    /// Plasma mass density $\rho_m$ ($kg/m^3$).
    pub mass_density: f64,
    /// Safety factor $q(\rho)$.
    pub q: f64,
    /// Magnetic shear $s(\rho)$.
    pub shear: f64,
    /// Local shear Alfvén speed $v_A$ (m/s).
    pub v_a: f64,
    /// Perturbed transverse fluid velocity $\delta v_\perp$ (m/s).
    pub delta_v: f64,
    /// Perturbed transverse magnetic field $\delta B_\perp$ (Tesla).
    pub delta_b: f64,
    /// Plasma resistivity $\eta$ ($\Omega \cdot m$).
    pub resistivity: f64,
}

/// Configuration for radial multi-surface Alfvén wave stepper.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlfvenMhdConfig {
    /// Number of radial flux surfaces.
    pub num_surfaces: usize,
    /// Time step dt in seconds.
    pub dt: f64,
    /// Core plasma mass density in kg/m^3.
    pub core_density: f64,
    /// Core electron/ion temperature in keV.
    pub core_temp_kev: f64,
    /// Poloidal harmonic m.
    pub m: usize,
    /// Toroidal harmonic n.
    pub n: usize,
}

/// Multi-surface extended MHD wave stepper.
#[derive(Debug, Clone, PartialEq)]
pub struct AlfvenMhdStepper {
    /// Radial nodes spanning the tokamak minor radius.
    pub nodes: Vec<AlfvenFluxNode>,
    /// Associated tokamak geometry.
    pub geometry: TokamakGeometry,
    /// Time step $\Delta t$ in seconds.
    pub dt: f64,
    /// Mode poloidal harmonic $m$.
    pub m: usize,
    /// Mode toroidal harmonic $n$.
    pub n: usize,
}

impl AlfvenMhdStepper {
    /// Constructs a radial multi-surface Alfvén wave stepper.
    pub fn new(
        geometry: TokamakGeometry,
        q_profile: &SafetyFactorProfile,
        config: AlfvenMhdConfig,
    ) -> Self {
        assert!(
            config.num_surfaces >= 4,
            "Must have at least 4 radial surfaces"
        );
        let a = geometry.minor_radius_a;
        let r0 = geometry.major_radius_r0;
        let b0 = geometry.toroidal_b0;

        let mut nodes = Vec::with_capacity(config.num_surfaces);
        let dr_norm = 1.0 / ((config.num_surfaces - 1) as f64);

        for i in 0..config.num_surfaces {
            let rho = (i as f64) * dr_norm;
            let minor_r = rho * a;
            let q = q_profile.q_at_rho(rho);
            let shear = q_profile.shear_at_rho(rho);

            // Parabolic density profile n(rho) = n0 * (1 - 0.8 * rho^2)
            let density_factor = (1.0 - 0.8 * rho.powi(2)).max(0.1);
            let mass_density = config.core_density * density_factor;

            // Approximate 1/R variation of toroidal field
            let b_local = b0 * (r0 / (r0 + minor_r));
            let v_a = AlfvenWaveProperties::alfven_speed(b_local, mass_density);

            let temp_kev = config.core_temp_kev * density_factor;
            let fluid_state = MhdFluidState {
                electron_density: mass_density / 3.34e-27,
                ion_density: mass_density / 3.34e-27,
                electron_temp_kev: temp_kev,
                ion_temp_kev: temp_kev,
                velocity: [0.0, 0.0, 0.0],
                magnetic_field: [0.0, b_local, 0.0],
                current_density: [0.0, 0.0, 0.0],
            };
            let resistivity = fluid_state.spitzer_resistivity(1.5);

            nodes.push(AlfvenFluxNode {
                rho,
                minor_r,
                b0: b_local,
                mass_density,
                q,
                shear,
                v_a,
                delta_v: 0.0,
                delta_b: 0.0,
                resistivity,
            });
        }

        Self {
            nodes,
            geometry,
            dt: config.dt,
            m: config.m,
            n: config.n,
        }
    }

    /// Initializes a Gaussian shear Alfvén wavepacket perturbation centered at radial index `center_idx`.
    pub fn initialize_wavepacket(&mut self, center_idx: usize, width_nodes: f64, amplitude_b: f64) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let diff = (i as f64) - (center_idx as f64);
            let envelope = (-0.5 * (diff / width_nodes).powi(2)).exp();
            node.delta_b = amplitude_b * envelope;
            // Equipartition of energy: delta_v = delta_b / sqrt(mu_0 * rho)
            node.delta_v =
                amplitude_b / (VACUUM_PERMEABILITY * node.mass_density).sqrt() * envelope;
        }
    }

    /// Advances the wave system by a single time step $\Delta t$ using parallel Rayon iteration.
    pub fn step(&mut self) {
        let r0 = self.geometry.major_radius_r0;
        let m = self.m;
        let n = self.n;
        let dt = self.dt;
        let num_nodes = self.nodes.len();

        // 1. Compute updates for delta_v: dv/dt = (B0 / (mu_0 * rho)) * k_par * delta_b - nu * delta_v
        let dv_updates: Vec<f64> = self
            .nodes
            .par_iter()
            .map(|node| {
                let k_par = AlfvenWaveProperties::k_parallel(m, n, node.q, r0);
                let accel =
                    (node.b0 / (VACUUM_PERMEABILITY * node.mass_density)) * k_par * node.delta_b;
                // Numerical damping
                let damping = 1e-4 * node.delta_v;
                node.delta_v + dt * (accel - damping)
            })
            .collect();

        // Apply updated velocities
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.delta_v = dv_updates[i];
        }

        // 2. Compute updates for delta_b: dB/dt = B0 * k_par * delta_v + (eta / mu_0) * d2B/dr2
        let db_updates: Vec<f64> = (0..num_nodes)
            .into_par_iter()
            .map(|i| {
                let node = &self.nodes[i];
                let k_par = AlfvenWaveProperties::k_parallel(m, n, node.q, r0);
                let wave_source = node.b0 * k_par * node.delta_v;

                // Radial diffusion term (eta / mu_0) * d2B/dr2
                let diff_term = if i > 0 && i < num_nodes - 1 {
                    let dr = (self.nodes[i + 1].minor_r - self.nodes[i - 1].minor_r) * 0.5;
                    let d2b = (self.nodes[i + 1].delta_b - 2.0 * node.delta_b
                        + self.nodes[i - 1].delta_b)
                        / dr.powi(2).max(1e-8);
                    (node.resistivity / VACUUM_PERMEABILITY) * d2b
                } else {
                    0.0
                };

                node.delta_b + dt * (wave_source + diff_term)
            })
            .collect();

        // Apply updated magnetic perturbations (clamping zero at metal boundaries)
        for (i, node) in self.nodes.iter_mut().enumerate() {
            if i == 0 || i == num_nodes - 1 {
                node.delta_b = 0.0;
            } else {
                node.delta_b = db_updates[i];
            }
        }
    }

    /// Evaluates total wave energy density (kinetic + magnetic) integrated over the volume.
    pub fn total_wave_energy(&self) -> f64 {
        let r0 = self.geometry.major_radius_r0;
        let mut total_energy = 0.0;

        for i in 0..(self.nodes.len() - 1) {
            let n0 = &self.nodes[i];
            let n1 = &self.nodes[i + 1];
            let dr = n1.minor_r - n0.minor_r;
            let r_avg = 0.5 * (n0.minor_r + n1.minor_r);

            let e_k0 = 0.5 * n0.mass_density * n0.delta_v.powi(2);
            let e_m0 = 0.5 * n0.delta_b.powi(2) / VACUUM_PERMEABILITY;

            let e_k1 = 0.5 * n1.mass_density * n1.delta_v.powi(2);
            let e_m1 = 0.5 * n1.delta_b.powi(2) / VACUUM_PERMEABILITY;

            let d_vol = 4.0 * std::f64::consts::PI.powi(2) * r0 * r_avg * dr;
            let energy_dens_avg = 0.5 * ((e_k0 + e_m0) + (e_k1 + e_m1));
            total_energy += energy_dens_avg * d_vol;
        }

        total_energy
    }

    /// Total integrated poloidal magnetic flux variation $\int \delta B_\perp dS$.
    pub fn total_flux_perturbation(&self) -> f64 {
        let mut flux = 0.0;
        for i in 0..(self.nodes.len() - 1) {
            let dr = self.nodes[i + 1].minor_r - self.nodes[i].minor_r;
            let b_avg = 0.5 * (self.nodes[i].delta_b + self.nodes[i + 1].delta_b);
            flux += b_avg * dr;
        }
        flux
    }
}
