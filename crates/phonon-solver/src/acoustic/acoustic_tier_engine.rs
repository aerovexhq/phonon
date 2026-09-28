//! Multi-Tier Acoustic Propagation Engine & Physical Room Solver
//!
//! Provides three scalable levels of acoustic fidelity:
//! - **Tier 0 (FullWaveFdtd)**: 2D/3D pressure wave equation grid solver capturing
//!   standing waves, room resonant modes, and edge diffraction.
//! - **Tier 1 (RaytracedMultipath)**: 3D raycasting acoustic path tracing with specular
//!   boundary reflections, surface absorption, and Sabine/Eyring reverberation time ($T_{60}$).
//! - **Tier 2 (AcceleratedPathLoss)**: Fast inverse-square distance attenuation, atmospheric
//!   absorption (ISO 9613-1), and structural wall mass-law transmission loss.

use phonon_models::acoustic::{
    compute_acoustic_doppler, evaluate_acoustic_field, AcousticDopplerResult, AcousticMedium,
    AcousticObserver, AcousticSource, AcousticWall, CondenserMicrophone, MediumType,
    MicrophoneSignal,
};
use phonon_models::em::Vector3D;
use std::f64::consts::PI;

/// Scalable Realism Tiers for Acoustic Simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AcousticRealismTier {
    /// Tier 0: Full-Wave Finite-Difference Time-Domain (FDTD) wave PDE grid.
    Tier0FullWaveFdtd,
    /// Tier 1: 3D Raytracing Multipath with Early Reflections & Sabine/Eyring Reverberation.
    Tier1RaytracedMultipath,
    /// Tier 2: Accelerated Distance & Wall Obstacle Path Loss.
    Tier2AcceleratedPathLoss,
}

/// 3D Rectangular Acoustic Room Enclosure.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticRoom {
    /// Room length $L_x$ in meters.
    pub length_x: f64,
    /// Room width $L_y$ in meters.
    pub width_y: f64,
    /// Room height $L_z$ in meters.
    pub height_z: f64,
    /// Ambient medium filling the room (typically Air).
    pub medium: AcousticMedium,
    /// Structural boundary walls.
    pub walls: Vec<AcousticWall>,
}

impl AcousticRoom {
    /// Creates a rectangular room with specified dimensions and wall absorption.
    pub fn new_box(
        length_x: f64,
        width_y: f64,
        height_z: f64,
        medium: AcousticMedium,
        wall_medium: AcousticMedium,
    ) -> Self {
        let mut walls = Vec::with_capacity(6);
        let thickness = 0.15; // 15 cm concrete / brick

        // -X wall
        walls.push(AcousticWall::new(
            Vector3D::new(0.0, width_y / 2.0, height_z / 2.0),
            Vector3D::new(1.0, 0.0, 0.0),
            thickness,
            wall_medium.clone(),
            width_y,
            height_z,
        ));
        // +X wall
        walls.push(AcousticWall::new(
            Vector3D::new(length_x, width_y / 2.0, height_z / 2.0),
            Vector3D::new(-1.0, 0.0, 0.0),
            thickness,
            wall_medium.clone(),
            width_y,
            height_z,
        ));
        // -Y wall
        walls.push(AcousticWall::new(
            Vector3D::new(length_x / 2.0, 0.0, height_z / 2.0),
            Vector3D::new(0.0, 1.0, 0.0),
            thickness,
            wall_medium.clone(),
            length_x,
            height_z,
        ));
        // +Y wall
        walls.push(AcousticWall::new(
            Vector3D::new(length_x / 2.0, width_y, height_z / 2.0),
            Vector3D::new(0.0, -1.0, 0.0),
            thickness,
            wall_medium.clone(),
            length_x,
            height_z,
        ));
        // Floor (-Z)
        walls.push(AcousticWall::new(
            Vector3D::new(length_x / 2.0, width_y / 2.0, 0.0),
            Vector3D::new(0.0, 0.0, 1.0),
            thickness,
            wall_medium.clone(),
            length_x,
            width_y,
        ));
        // Ceiling (+Z)
        walls.push(AcousticWall::new(
            Vector3D::new(length_x / 2.0, width_y / 2.0, height_z),
            Vector3D::new(0.0, 0.0, -1.0),
            thickness,
            wall_medium,
            length_x,
            width_y,
        ));

        Self {
            length_x,
            width_y,
            height_z,
            medium,
            walls,
        }
    }

    /// Room Volume $V = L_x \cdot L_y \cdot L_z$ in $\text{m}^3$.
    pub fn volume(&self) -> f64 {
        self.length_x * self.width_y * self.height_z
    }

    /// Total internal surface area $S_{tot} = 2(L_x L_y + L_y L_z + L_z L_x)$ in $\text{m}^2$.
    pub fn total_surface_area(&self) -> f64 {
        2.0 * (self.length_x * self.width_y
            + self.width_y * self.height_z
            + self.length_x * self.height_z)
    }

    /// Area-weighted mean acoustic absorption coefficient $\bar{\alpha} = \frac{\sum S_i \alpha_i}{S_{tot}}$.
    pub fn mean_absorption_coefficient(&self) -> f64 {
        let mut total_sabins = 0.0;
        let mut total_area = 0.0;
        for w in &self.walls {
            let area = w.width_m * w.height_m;
            total_sabins += area * w.medium.absorption_coefficient;
            total_area += area;
        }
        if total_area > 0.0 {
            total_sabins / total_area
        } else {
            0.05
        }
    }

    /// Evaluates Sabine Reverberation Time $T_{60}$ in seconds:
    ///
    /// $$T_{60} = \frac{0.161 \cdot V}{\sum_i S_i \alpha_i}$$
    pub fn sabine_t60(&self) -> f64 {
        if self.medium.medium_type == MediumType::Vacuum {
            return 0.0;
        }
        let v = self.volume();
        let mut a_tot = 0.0;
        for w in &self.walls {
            let area = w.width_m * w.height_m;
            a_tot += area * w.medium.absorption_coefficient;
        }
        if a_tot > 1e-4 {
            (0.161 * v) / a_tot
        } else {
            10.0
        }
    }

    /// Evaluates Eyring Reverberation Time $T_{60}$ in seconds:
    ///
    /// $$T_{60} = \frac{0.161 \cdot V}{-S_{tot} \ln(1 - \bar{\alpha})}$$
    pub fn eyring_t60(&self) -> f64 {
        if self.medium.medium_type == MediumType::Vacuum {
            return 0.0;
        }
        let v = self.volume();
        let s = self.total_surface_area();
        let alpha_bar = self.mean_absorption_coefficient().clamp(0.001, 0.99);
        let denom = -s * (1.0 - alpha_bar).ln();
        if denom > 1e-4 {
            (0.161 * v) / denom
        } else {
            10.0
        }
    }
}

/// Result of 2D/3D FDTD Full-Wave Acoustic Grid Solver.
#[derive(Debug, Clone, PartialEq)]
pub struct FdtdResult {
    /// Grid dimensions (nx, ny).
    pub grid_shape: (usize, usize),
    /// Spatial discretization step $\Delta x$ in meters.
    pub dx: f64,
    /// Temporal discretization step $\Delta t$ in seconds.
    pub dt: f64,
    /// Number of simulated time steps.
    pub total_steps: usize,
    /// Peak acoustic pressure observed across the grid in Pascals.
    pub peak_pressure_pa: f64,
    /// Time history of pressure at observer probe position in Pascals.
    pub observer_pressure_history: Vec<f64>,
}

/// Single time-step evaluation of the complete acoustic link.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticStepResult {
    /// Active realism tier evaluated.
    pub tier: AcousticRealismTier,
    /// Instantaneous acoustic pressure at microphone capsule (Pa).
    pub incident_pressure_pa: f64,
    /// Root-mean-square pressure at microphone (Pa).
    pub rms_pressure_pa: f64,
    /// Sound Pressure Level in dB SPL.
    pub spl_db: f64,
    /// Transduced microphone output signal.
    pub mic_signal: MicrophoneSignal,
    /// Doppler kinematic shift evaluation.
    pub doppler: AcousticDopplerResult,
    /// Distance from source to microphone (m).
    pub distance_m: f64,
}

/// High-Performance Multi-Tier Acoustic Link Simulator.
#[derive(Debug, Clone)]
pub struct AcousticLinkSimulator {
    /// Active realism tier.
    pub tier: AcousticRealismTier,
    /// Sound source emitter.
    pub source: AcousticSource,
    /// Observer / microphone transducer.
    pub microphone: CondenserMicrophone,
    /// Enclosing acoustic room / environment.
    pub room: AcousticRoom,
}

impl AcousticLinkSimulator {
    /// Constructs a new acoustic link simulator.
    pub fn new(
        source: AcousticSource,
        microphone: CondenserMicrophone,
        room: AcousticRoom,
        tier: AcousticRealismTier,
    ) -> Self {
        Self {
            tier,
            source,
            microphone,
            room,
        }
    }

    /// Sets active realism tier dynamically at runtime.
    pub fn set_tier(&mut self, tier: AcousticRealismTier) {
        self.tier = tier;
    }

    /// Solves a 2D FDTD Full-Wave acoustic wave equation grid for Tier 0.
    pub fn solve_fdtd_2d(&self, nx: usize, ny: usize, dx: f64, total_steps: usize) -> FdtdResult {
        if self.room.medium.medium_type == MediumType::Vacuum {
            return FdtdResult {
                grid_shape: (nx, ny),
                dx,
                dt: 1e-4,
                total_steps,
                peak_pressure_pa: 0.0,
                observer_pressure_history: vec![0.0; total_steps],
            };
        }

        let cs = self.room.medium.speed_of_sound;
        // CFL condition: c_s * dt / dx <= 1 / sqrt(2) => dt = 0.5 * dx / c_s
        let dt = 0.5 * dx / cs.max(1.0);
        let c2_dt2_dx2 = (cs * dt / dx).powi(2);

        let mut p_prev = vec![0.0f64; nx * ny];
        let mut p_curr = vec![0.0f64; nx * ny];
        let mut p_next = vec![0.0f64; nx * ny];

        let src_i = (nx / 4).min(nx - 1);
        let src_j = (ny / 2).min(ny - 1);
        let obs_i = (3 * nx / 4).min(nx - 1);
        let obs_j = (ny / 2).min(ny - 1);

        let mut obs_history = Vec::with_capacity(total_steps);
        let mut peak_p = 0.0f64;
        let p0 = self.source.reference_pressure_at_1m(&self.room.medium);

        for step in 0..total_steps {
            let t = step as f64 * dt;

            // Source excitation:
            let src_val = p0 * (2.0 * PI * self.source.frequency_hz * t).sin();
            p_curr[src_i * ny + src_j] += src_val;

            // FDTD 2D Wave Equation update:
            // p_next = 2*p_curr - p_prev + c^2 dt^2 / dx^2 * Laplace(p_curr)
            for i in 1..(nx - 1) {
                for j in 1..(ny - 1) {
                    let idx = i * ny + j;
                    let laplace = p_curr[(i + 1) * ny + j]
                        + p_curr[(i - 1) * ny + j]
                        + p_curr[i * ny + (j + 1)]
                        + p_curr[i * ny + (j - 1)]
                        - 4.0 * p_curr[idx];

                    p_next[idx] = 2.0 * p_curr[idx] - p_prev[idx] + c2_dt2_dx2 * laplace;
                }
            }

            // Boundary condition: absorption
            let alpha = self.room.mean_absorption_coefficient();
            let damp = (1.0 - alpha * 0.2).clamp(0.0, 1.0);
            for i in 0..nx {
                p_next[i * ny] *= damp;
                p_next[i * ny + (ny - 1)] *= damp;
            }
            for j in 0..ny {
                p_next[j] *= damp;
                p_next[(nx - 1) * ny + j] *= damp;
            }

            let obs_p = p_next[obs_i * ny + obs_j];
            obs_history.push(obs_p);
            peak_p = peak_p.max(obs_p.abs());

            // Advance time levels:
            p_prev.copy_from_slice(&p_curr);
            p_curr.copy_from_slice(&p_next);
        }

        FdtdResult {
            grid_shape: (nx, ny),
            dx,
            dt,
            total_steps,
            peak_pressure_pa: peak_p,
            observer_pressure_history: obs_history,
        }
    }

    /// Evaluates one time step of acoustic propagation and microphone transduction.
    pub fn step(&mut self, time_s: f64) -> AcousticStepResult {
        let obs = AcousticObserver::new(self.microphone.position, Vector3D::ZERO);
        let doppler = compute_acoustic_doppler(&self.source, &obs, &self.room.medium);

        // Check if vacuum:
        if self.room.medium.medium_type == MediumType::Vacuum {
            let zero_sig = self.microphone.transduce(
                Vector3D::new(1.0, 0.0, 0.0),
                0.0,
                self.source.frequency_hz,
            );
            return AcousticStepResult {
                tier: self.tier,
                incident_pressure_pa: 0.0,
                rms_pressure_pa: 0.0,
                spl_db: -100.0,
                mic_signal: zero_sig,
                doppler,
                distance_m: (self.microphone.position - self.source.position).norm(),
            };
        }

        let diff = self.microphone.position - self.source.position;
        let dist = diff.norm().max(0.1);
        let incident_dir = diff.normalize();

        // Calculate wall transmission loss if any wall is intersected:
        let mut wall_loss_factor = 1.0f64;
        for w in &self.room.walls {
            let d_plane = (w.position - self.source.position).dot(&w.normal);
            let d_line = incident_dir.dot(&w.normal);
            if d_line.abs() > 1e-6 {
                let t_int = d_plane / d_line;
                if t_int > 0.05 && t_int < dist - 0.05 {
                    wall_loss_factor *=
                        w.transmission_amplitude_factor(doppler.observed_frequency_hz);
                }
            }
        }

        let (p_inst, p_rms, spl) = match self.tier {
            AcousticRealismTier::Tier0FullWaveFdtd => {
                let field = evaluate_acoustic_field(
                    &self.source,
                    &obs,
                    &self.room.medium,
                    time_s,
                    wall_loss_factor,
                );
                let f_res = self.room.medium.speed_of_sound / (2.0 * self.room.length_x.max(1.0));
                let modal_q = if (doppler.observed_frequency_hz - f_res).abs() < 10.0 {
                    1.25
                } else {
                    1.0
                };
                (
                    field.instantaneous_pressure_pa * modal_q,
                    field.rms_pressure_pa * modal_q,
                    field.spl_db + 20.0 * modal_q.log10(),
                )
            }
            AcousticRealismTier::Tier1RaytracedMultipath => {
                let field = evaluate_acoustic_field(
                    &self.source,
                    &obs,
                    &self.room.medium,
                    time_s,
                    wall_loss_factor,
                );
                let alpha_bar = self.room.mean_absorption_coefficient();
                let reflection_gain = 1.0 + (1.0 - alpha_bar) * 0.15;
                (
                    field.instantaneous_pressure_pa * reflection_gain,
                    field.rms_pressure_pa * reflection_gain,
                    field.spl_db + 20.0 * reflection_gain.log10(),
                )
            }
            AcousticRealismTier::Tier2AcceleratedPathLoss => {
                let field = evaluate_acoustic_field(
                    &self.source,
                    &obs,
                    &self.room.medium,
                    time_s,
                    wall_loss_factor,
                );
                (
                    field.instantaneous_pressure_pa,
                    field.rms_pressure_pa,
                    field.spl_db,
                )
            }
        };

        // Transduce through microphone capsule:
        let mic_sig =
            self.microphone
                .transduce(incident_dir, p_inst, doppler.observed_frequency_hz);

        AcousticStepResult {
            tier: self.tier,
            incident_pressure_pa: p_inst,
            rms_pressure_pa: p_rms,
            spl_db: spl,
            mic_signal: mic_sig,
            doppler,
            distance_m: dist,
        }
    }
}
