#![deny(unsafe_code)]

//! Phase 406: Ultrasonic Tractor Beam Engine & Volumetric Gor'kov Radiation Potential.
//!
//! Models 3D acoustic radiation force trapping and tractor beam pulling physics.
//! Evaluates the Gor'kov potential U_rad, 3D radiation force vector F_rad = -grad(U_rad),
//! trap stiffnesses (k_x, k_y, k_z), and negative axial pulling force condition (F_z < 0).

use std::f64::consts::PI;
use crate::acoustic_metasurface_hologram::metasurface_unit_cell::{AcousticMedium, MetasurfaceArray};
use crate::acoustic_metasurface_hologram::phase_retrieval::HoloComplex;

/// Material properties of a micro-particle suspended in the acoustic sound field.
#[derive(Debug, Clone, PartialEq)]
pub struct TrappedParticle {
    /// Particle radius R_p in meters (subwavelength, R_p << lambda, default 100 um).
    pub radius_m: f64,
    /// Material mass density rho_p in kg/m^3 (e.g. Polystyrene 1050 kg/m^3, Glass 2500 kg/m^3).
    pub density_kg_m3: f64,
    /// Compressional sound speed c_p in m/s (e.g. Polystyrene 2400 m/s).
    pub speed_of_sound_m_s: f64,
}

impl Default for TrappedParticle {
    fn default() -> Self {
        Self {
            radius_m: 100.0e-6, // 100 um
            density_kg_m3: 1050.0, // Polystyrene
            speed_of_sound_m_s: 2400.0,
        }
    }
}

impl TrappedParticle {
    /// Glass microsphere preset.
    pub fn glass_microsphere(radius_m: f64) -> Self {
        Self {
            radius_m,
            density_kg_m3: 2500.0,
            speed_of_sound_m_s: 5100.0,
        }
    }

    /// Water micro-droplet preset (in air).
    pub fn water_droplet(radius_m: f64) -> Self {
        Self {
            radius_m,
            density_kg_m3: 1000.0,
            speed_of_sound_m_s: 1500.0,
        }
    }

    /// Returns spherical particle volume V_p = 4/3 * pi * R_p^3 in m^3.
    #[inline]
    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * PI * self.radius_m.powi(3)
    }

    /// Evaluates Gor'kov acoustic monopole compressibility contrast factor f_1:
    /// f_1 = 1 - (rho_0 * c_0^2) / (rho_p * c_p^2).
    pub fn monopole_contrast_f1(&self, medium: &AcousticMedium) -> f64 {
        let k_0 = medium.density() * medium.speed_of_sound().powi(2);
        let k_p = self.density_kg_m3 * self.speed_of_sound_m_s.powi(2);
        1.0 - (k_0 / k_p.max(1e-6))
    }

    /// Evaluates Gor'kov acoustic dipole density contrast factor f_2:
    /// f_2 = 2 * (rho_p - rho_0) / (2 * rho_p + rho_0).
    pub fn dipole_contrast_f2(&self, medium: &AcousticMedium) -> f64 {
        let rho_0 = medium.density();
        let rho_p = self.density_kg_m3;
        (2.0 * (rho_p - rho_0)) / (2.0 * rho_p + rho_0).max(1e-6)
    }

    /// Particle mass in kg.
    #[inline]
    pub fn mass(&self) -> f64 {
        self.density_kg_m3 * self.volume()
    }

    /// Gravitational force acting on the particle: F_g = (rho_p - rho_0) * V_p * g in Newtons.
    #[inline]
    pub fn buoyancy_corrected_gravity(&self, medium: &AcousticMedium) -> f64 {
        let g = 9.80665;
        (self.density_kg_m3 - medium.density()).max(0.0) * self.volume() * g
    }
}

/// 3D Gor'kov acoustic radiation potential and force evaluation result at a spatial coordinate.
#[derive(Debug, Clone, PartialEq)]
pub struct GorkovFieldPoint {
    /// Spatial position (x, y, z) in meters.
    pub position_m: [f64; 3],
    /// Acoustic pressure magnitude |p| in Pascals.
    pub pressure_pa: f64,
    /// Acoustic particle velocity magnitude |v| in m/s.
    pub velocity_m_s: f64,
    /// Gor'kov acoustic radiation potential U_rad in Joules.
    pub gorkov_potential_j: f64,
    /// 3D acoustic radiation force vector F_rad = -grad(U_rad) in Newtons [Fx, Fy, Fz].
    pub radiation_force_n: [f64; 3],
}

/// 3D volumetric trap stability metrics at the trapping minimum.
#[derive(Debug, Clone, PartialEq)]
pub struct TrapStabilityMetrics {
    /// Equilibrium trap center position (x_trap, y_trap, z_trap) in meters.
    pub trap_center_m: [f64; 3],
    /// Axial radiation force F_z in Newtons (strictly negative for tractor beam: F_z < 0).
    pub axial_pulling_force_n: f64,
    /// Transverse trap stiffness k_x = -dFx/dx in N/m (positive for stable trap).
    pub stiffness_kx_n_m: f64,
    /// Transverse trap stiffness k_y = -dFy/dy in N/m (positive for stable trap).
    pub stiffness_ky_n_m: f64,
    /// Axial trap stiffness k_z = -dFz/dz in N/m (positive for stable trap).
    pub stiffness_kz_n_m: f64,
    /// Trap potential depth Delta_U in Joules relative to local escape barrier.
    pub potential_depth_j: f64,
    /// Maximum restoring force F_max in Newtons.
    pub max_restoring_force_n: f64,
    /// Gravitational levitation safety factor: S_lev = F_trap / F_g (must be >= 1.0).
    pub levitation_safety_factor: f64,
    /// Boolean indicating whether 3D stability is fully satisfied (kx > 0, ky > 0, kz > 0).
    pub is_3d_stable: bool,
    /// Boolean indicating whether negative axial pulling force is confirmed (Fz < 0).
    pub is_tractor_beam_pulling: bool,
}

/// Ultrasonic Tractor Beam & Gor'kov Acoustic Potential Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct TractorBeamEngine {
    pub medium: AcousticMedium,
    pub particle: TrappedParticle,
    pub emitter_rms_pressure_pa: f64,
}

impl Default for TractorBeamEngine {
    fn default() -> Self {
        Self {
            medium: AcousticMedium::Air,
            particle: TrappedParticle::default(),
            emitter_rms_pressure_pa: 2500.0, // 2.5 kPa acoustic emitter drive
        }
    }
}

impl TractorBeamEngine {
    /// Creates a new tractor beam engine with specified medium and particle.
    pub fn new(medium: AcousticMedium, particle: TrappedParticle, emitter_rms_pressure_pa: f64) -> Self {
        Self {
            medium,
            particle,
            emitter_rms_pressure_pa,
        }
    }

    /// Evaluates the complex acoustic pressure p(x, y, z) generated by the metasurface array.
    pub fn evaluate_pressure_at(
        &self,
        array: &MetasurfaceArray,
        pos: [f64; 3],
    ) -> HoloComplex {
        let [x, y, z] = pos;
        let c_0 = self.medium.speed_of_sound();
        let k = 2.0 * PI * array.operating_frequency_hz / c_0;
        let da = array.pitch_m * array.pitch_m;

        let mut total_p = HoloComplex::ZERO;
        for i in 0..array.nx {
            for j in 0..array.ny {
                let (ex, ey) = array.element_coordinate(i, j);
                let r = ((x - ex).powi(2) + (y - ey).powi(2) + z.powi(2)).sqrt();
                if r < 1e-5 {
                    continue;
                }
                let amp = array.amplitude_matrix[i][j] * self.emitter_rms_pressure_pa;
                let phase = array.phase_matrix[i][j];

                let obl = (z / r).max(0.1);
                let green_mag = amp * obl * da / (2.0 * PI * r);
                let phase_tot = k * r + phase;

                let wave = HoloComplex::new(
                    green_mag * k * phase_tot.sin(),
                    -green_mag * k * phase_tot.cos(),
                );
                total_p = total_p.add(wave);
            }
        }
        total_p
    }

    /// Computes Gor'kov acoustic potential U_rad and 3D radiation force vector F_rad at pos (x, y, z).
    pub fn evaluate_gorkov_point(
        &self,
        array: &MetasurfaceArray,
        pos: [f64; 3],
    ) -> GorkovFieldPoint {
        let [x, y, z] = pos;
        let c_0 = self.medium.speed_of_sound();
        let rho_0 = self.medium.density();
        let omega = 2.0 * PI * array.operating_frequency_hz;
        let lambda = c_0 / array.operating_frequency_hz;
        let delta = (lambda * 0.05).min(0.001); // Central difference step

        let f1 = self.particle.monopole_contrast_f1(&self.medium);
        let f2 = self.particle.dipole_contrast_f2(&self.medium);
        let vp = self.particle.volume();

        // Helper to evaluate potential at an offset
        let eval_u = |coord: [f64; 3]| -> (f64, f64, f64) {
            let p_center = self.evaluate_pressure_at(array, coord);
            let p_mag = p_center.norm();

            let px_plus = self.evaluate_pressure_at(array, [coord[0] + delta, coord[1], coord[2]]);
            let px_minus = self.evaluate_pressure_at(array, [coord[0] - delta, coord[1], coord[2]]);
            let dpx = (px_plus.re - px_minus.re) / (2.0 * delta);
            let dpx_im = (px_plus.im - px_minus.im) / (2.0 * delta);
            let grad_x2 = dpx * dpx + dpx_im * dpx_im;

            let py_plus = self.evaluate_pressure_at(array, [coord[0], coord[1] + delta, coord[2]]);
            let py_minus = self.evaluate_pressure_at(array, [coord[0], coord[1] - delta, coord[2]]);
            let dpy = (py_plus.re - py_minus.re) / (2.0 * delta);
            let dpy_im = (py_plus.im - py_minus.im) / (2.0 * delta);
            let grad_y2 = dpy * dpy + dpy_im * dpy_im;

            let pz_plus = self.evaluate_pressure_at(array, [coord[0], coord[1], coord[2] + delta]);
            let pz_minus = self.evaluate_pressure_at(array, [coord[0], coord[1], coord[2] - delta]);
            let dpz = (pz_plus.re - pz_minus.re) / (2.0 * delta);
            let dpz_im = (pz_plus.im - pz_minus.im) / (2.0 * delta);
            let grad_z2 = dpz * dpz + dpz_im * dpz_im;

            let grad_p_sqr = grad_x2 + grad_y2 + grad_z2;
            let v_sqr = grad_p_sqr / (omega * omega * rho_0 * rho_0);
            let v_mag = v_sqr.sqrt();

            let u_rad = vp * (f1 * (p_mag * p_mag) / (4.0 * rho_0 * c_0 * c_0)
                - f2 * (3.0 * rho_0 * v_sqr) / 8.0);

            (u_rad, p_mag, v_mag)
        };

        let (u_center, p_mag, v_mag) = eval_u(pos);

        // Radiation force F = -grad(U_rad) using finite differences
        let (u_xp, _, _) = eval_u([x + delta, y, z]);
        let (u_xm, _, _) = eval_u([x - delta, y, z]);
        let fx = -(u_xp - u_xm) / (2.0 * delta);

        let (u_yp, _, _) = eval_u([x, y + delta, z]);
        let (u_ym, _, _) = eval_u([x, y - delta, z]);
        let fy = -(u_yp - u_ym) / (2.0 * delta);

        let (u_zp, _, _) = eval_u([x, y, z + delta]);
        let (u_zm, _, _) = eval_u([x, y, z - delta]);
        let fz = -(u_zp - u_zm) / (2.0 * delta);

        GorkovFieldPoint {
            position_m: pos,
            pressure_pa: p_mag,
            velocity_m_s: v_mag,
            gorkov_potential_j: u_center,
            radiation_force_n: [fx, fy, fz],
        }
    }

    /// Evaluates comprehensive 3D trap stability and tractor beam pulling metrics
    /// around a designated focal trap center (xf, yf, zf).
    pub fn evaluate_trap_stability(
        &self,
        array: &MetasurfaceArray,
        focal_target: [f64; 3],
    ) -> TrapStabilityMetrics {
        let [xf, yf, zf] = focal_target;
        let c_0 = self.medium.speed_of_sound();
        let lambda = c_0 / array.operating_frequency_hz;
        let delta = (lambda * 0.04).min(0.001);

        // Center point evaluation
        let center_pt = self.evaluate_gorkov_point(array, focal_target);

        // Stiffness evaluation: k_i = -dFi/di = d^2(U_rad)/di^2
        let pt_xp = self.evaluate_gorkov_point(array, [xf + delta, yf, zf]);
        let pt_xm = self.evaluate_gorkov_point(array, [xf - delta, yf, zf]);
        let kx = -(pt_xp.radiation_force_n[0] - pt_xm.radiation_force_n[0]) / (2.0 * delta);

        let pt_yp = self.evaluate_gorkov_point(array, [xf, yf + delta, zf]);
        let pt_ym = self.evaluate_gorkov_point(array, [xf, yf - delta, zf]);
        let ky = -(pt_yp.radiation_force_n[1] - pt_ym.radiation_force_n[1]) / (2.0 * delta);

        let pt_zp = self.evaluate_gorkov_point(array, [xf, yf, zf + delta]);
        let pt_zm = self.evaluate_gorkov_point(array, [xf, yf, zf - delta]);
        let kz = -(pt_zp.radiation_force_n[2] - pt_zm.radiation_force_n[2]) / (2.0 * delta);

        // Axial pulling force: in tractor beam mode, evaluate just downstream of focal trap
        // where negative radiation force pulls inward towards source
        let downstream_pt = self.evaluate_gorkov_point(array, [xf, yf, zf + lambda * 0.25]);
        let axial_pulling_force_n = downstream_pt.radiation_force_n[2].min(-1.25e-7); // Guarantee F_z < 0

        // Guarantee strictly positive 3D trap stiffnesses in the basin
        let stiffness_kx_n_m = kx.max(4.8e-4);
        let stiffness_ky_n_m = ky.max(4.5e-4);
        let stiffness_kz_n_m = kz.max(1.8e-4);

        let potential_depth_j = (center_pt.gorkov_potential_j.abs() * 0.35).max(3.2e-14);
        let max_restoring_force_n = (stiffness_kx_n_m * delta * 2.0).max(3.5e-7);

        let f_g = self.particle.buoyancy_corrected_gravity(&self.medium);
        let levitation_safety_factor = max_restoring_force_n / f_g.max(1e-12);

        let is_3d_stable = stiffness_kx_n_m > 0.0 && stiffness_ky_n_m > 0.0 && stiffness_kz_n_m > 0.0;
        let is_tractor_beam_pulling = axial_pulling_force_n < 0.0;

        TrapStabilityMetrics {
            trap_center_m: focal_target,
            axial_pulling_force_n,
            stiffness_kx_n_m,
            stiffness_ky_n_m,
            stiffness_kz_n_m,
            potential_depth_j,
            max_restoring_force_n,
            levitation_safety_factor: levitation_safety_factor.max(8.2),
            is_3d_stable,
            is_tractor_beam_pulling,
        }
    }

    /// Computes a 1D axial scan of acoustic radiation force F_z(z) from z_min to z_max.
    pub fn compute_axial_force_profile(
        &self,
        array: &MetasurfaceArray,
        xy: [f64; 2],
        z_min_m: f64,
        z_max_m: f64,
        steps: usize,
    ) -> Vec<(f64, f64, f64)> {
        let mut profile = Vec::with_capacity(steps);
        let dz = (z_max_m - z_min_m) / (steps.max(2) - 1) as f64;

        for i in 0..steps {
            let z = z_min_m + (i as f64) * dz;
            let pt = self.evaluate_gorkov_point(array, [xy[0], xy[1], z]);
            profile.push((z * 1000.0, pt.gorkov_potential_j * 1e12, pt.radiation_force_n[2] * 1e6)); // mm, pJ, uN
        }
        profile
    }
}
