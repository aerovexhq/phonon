//! Multi-Tier Synthetic Optical Perception Pipeline
//!
//! Provides headless offscreen synthetic optical image generation across three distinct realism tiers:
//! 1. `Tier0MicroscopicCmos`: Physical ray-casting to 3D scene geometry, evaluating optical irradiance,
//!    incident photon flux per pixel, silicon photodiode quantum efficiency, dark current generation,
//!    Poisson photon shot noise, Johnson read noise, full-well saturation, and ADC quantization.
//! 2. `Tier1PhysicalLensRaster`: Pinhole projection with Brown-Conrady non-linear lens distortion
//!    (k1, k2, k3, p1, p2) and Circle of Confusion (CoC) depth-of-field optical blur.
//! 3. `Tier2AcceleratedPinhole`: High-speed geometric pinhole raycasting with direct linear intensity
//!    mapping for high-throughput multi-agent autonomy simulations.

use phonon_core::constants::{PLANCK_CONSTANT, SPEED_OF_LIGHT};
use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::optics::{silicon_quantum_efficiency, transduce_cmos_pixel, OpticalCamera};
use rayon::prelude::*;

/// Realism and abstraction tier for synthetic optical perception.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpticalRealismTier {
    /// Microscopic CMOS APS physics with silicon photodiode quantum efficiency,
    /// dark current, Poisson photon shot noise, read noise, and ADC quantization.
    Tier0MicroscopicCmos,
    /// Physical lens projection with Brown-Conrady non-linear distortion and CoC blur.
    Tier1PhysicalLensRaster,
    /// Accelerated pinhole raycasting with linear intensity mapping for maximum FPS.
    Tier2AcceleratedPinhole,
}

/// 3D Geometric Sphere primitive for optical scenes.
#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    pub center: Vector3D,
    pub radius: f64,
    pub albedo: f64,
    pub emission: f64,
}

impl Sphere {
    pub fn new(center: Vector3D, radius: f64, albedo: f64, emission: f64) -> Self {
        Self {
            center,
            radius: radius.max(1e-4),
            albedo: albedo.clamp(0.0, 1.0),
            emission: emission.max(0.0),
        }
    }

    /// Tests ray intersection with the sphere.
    pub fn intersect(&self, origin: Vector3D, dir: Vector3D) -> Option<(f64, Vector3D)> {
        let oc = origin - self.center;
        let b = 2.0 * oc.dot(&dir);
        let c = oc.dot(&oc) - self.radius * self.radius;
        let disc = b * b - 4.0 * c;
        if disc < 0.0 {
            return None;
        }
        let sqrt_disc = disc.sqrt();
        let t1 = (-b - sqrt_disc) / 2.0;
        let t2 = (-b + sqrt_disc) / 2.0;

        let t = if t1 > 1e-4 {
            t1
        } else if t2 > 1e-4 {
            t2
        } else {
            return None;
        };

        let hit_point = origin + dir * t;
        let normal = (hit_point - self.center).normalize();
        Some((t, normal))
    }
}

/// Axis-Aligned Bounding Box (AABB) primitive for optical scenes.
#[derive(Debug, Clone, PartialEq)]
pub struct AabbBox {
    pub min: Vector3D,
    pub max: Vector3D,
    pub albedo: f64,
    pub emission: f64,
}

impl AabbBox {
    pub fn new(min: Vector3D, max: Vector3D, albedo: f64, emission: f64) -> Self {
        Self {
            min,
            max,
            albedo: albedo.clamp(0.0, 1.0),
            emission: emission.max(0.0),
        }
    }

    /// Tests ray intersection with the AABB using the Kay-Kajiya slab method.
    pub fn intersect(&self, origin: Vector3D, dir: Vector3D) -> Option<(f64, Vector3D)> {
        let inv_dx = if dir.x.abs() > 1e-9 { 1.0 / dir.x } else { 1e9 };
        let inv_dy = if dir.y.abs() > 1e-9 { 1.0 / dir.y } else { 1e9 };
        let inv_dz = if dir.z.abs() > 1e-9 { 1.0 / dir.z } else { 1e9 };

        let tx1 = (self.min.x - origin.x) * inv_dx;
        let tx2 = (self.max.x - origin.x) * inv_dx;
        let (tmin_x, tmax_x, nx) = if tx1 < tx2 {
            (tx1, tx2, -1.0)
        } else {
            (tx2, tx1, 1.0)
        };

        let ty1 = (self.min.y - origin.y) * inv_dy;
        let ty2 = (self.max.y - origin.y) * inv_dy;
        let (tmin_y, tmax_y, ny) = if ty1 < ty2 {
            (ty1, ty2, -1.0)
        } else {
            (ty2, ty1, 1.0)
        };

        let tz1 = (self.min.z - origin.z) * inv_dz;
        let tz2 = (self.max.z - origin.z) * inv_dz;
        let (tmin_z, tmax_z, nz) = if tz1 < tz2 {
            (tz1, tz2, -1.0)
        } else {
            (tz2, tz1, 1.0)
        };

        let t_near = tmin_x.max(tmin_y).max(tmin_z);
        let t_far = tmax_x.min(tmax_y).min(tmax_z);

        if t_near > t_far || t_far < 1e-4 {
            return None;
        }

        let t = if t_near > 1e-4 { t_near } else { t_far };
        let normal = if (t - tmin_x).abs() < 1e-5 {
            Vector3D::new(nx, 0.0, 0.0)
        } else if (t - tmin_y).abs() < 1e-5 {
            Vector3D::new(0.0, ny, 0.0)
        } else {
            Vector3D::new(0.0, 0.0, nz)
        };

        Some((t, normal))
    }
}

/// Planar ground or wall with alternating checkerboard pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckerPlane {
    pub point: Vector3D,
    pub normal: Vector3D,
    pub tile_size: f64,
    pub albedo1: f64,
    pub albedo2: f64,
}

impl CheckerPlane {
    pub fn new(
        point: Vector3D,
        normal: Vector3D,
        tile_size: f64,
        albedo1: f64,
        albedo2: f64,
    ) -> Self {
        Self {
            point,
            normal: normal.normalize(),
            tile_size: tile_size.max(1e-3),
            albedo1: albedo1.clamp(0.0, 1.0),
            albedo2: albedo2.clamp(0.0, 1.0),
        }
    }

    pub fn intersect(&self, origin: Vector3D, dir: Vector3D) -> Option<(f64, Vector3D, f64)> {
        let denom = self.normal.dot(&dir);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = (self.point - origin).dot(&self.normal) / denom;
        if t <= 1e-4 {
            return None;
        }
        let hit = origin + dir * t;
        let s = self.tile_size;
        let ix = (hit.x / s).floor() as i64;
        let iz = (hit.z / s).floor() as i64;
        let albedo = if (ix + iz) % 2 == 0 {
            self.albedo1
        } else {
            self.albedo2
        };
        let normal = if denom < 0.0 {
            self.normal
        } else {
            -self.normal
        };
        Some((t, normal, albedo))
    }
}

/// Geometric Object Primitive in 3D Scene.
#[derive(Debug, Clone, PartialEq)]
pub enum SceneObject {
    Sphere(Sphere),
    Box(AabbBox),
    Plane(CheckerPlane),
}

/// Optical Light Source in 3D Scene.
#[derive(Debug, Clone, PartialEq)]
pub enum LightSource {
    Directional {
        direction: Vector3D,
        illuminance_lux: f64,
        wavelength_nm: f64,
    },
    Point {
        position: Vector3D,
        illuminance_at_1m_lux: f64,
        wavelength_nm: f64,
    },
}

/// Ray Intersection Hit Record.
#[derive(Debug, Clone, PartialEq)]
pub struct RayHit {
    pub distance_t: f64,
    pub hit_point: Vector3D,
    pub normal: Vector3D,
    pub albedo: f64,
    pub emission: f64,
}

/// Synthetic 3D Perception Environment Scene.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntheticScene {
    pub objects: Vec<SceneObject>,
    pub lights: Vec<LightSource>,
    pub ambient_illuminance_lux: f64,
    pub default_wavelength_nm: f64,
}

impl Default for SyntheticScene {
    fn default() -> Self {
        Self::new_default_environment(500.0) // 500 lux office lighting
    }
}

impl SyntheticScene {
    pub fn new(ambient_lux: f64) -> Self {
        Self {
            objects: Vec::new(),
            lights: Vec::new(),
            ambient_illuminance_lux: ambient_lux.max(0.0),
            default_wavelength_nm: 550.0, // 550 nm (green optical peak)
        }
    }

    /// Creates a standard robotics validation scene with a ground plane, obstacles, and directional sun.
    pub fn new_default_environment(ambient_lux: f64) -> Self {
        let mut scene = Self::new(ambient_lux);

        // Ground checkerboard plane at y = -1.0:
        scene.objects.push(SceneObject::Plane(CheckerPlane::new(
            Vector3D::new(0.0, -1.0, 0.0),
            Vector3D::new(0.0, 1.0, 0.0),
            1.0,
            0.8,
            0.2,
        )));

        // Center target sphere at (0.0, 0.0, 5.0):
        scene.objects.push(SceneObject::Sphere(Sphere::new(
            Vector3D::new(0.0, 0.0, 5.0),
            1.0,
            0.75,
            0.0,
        )));

        // Obstacle box at (2.5, 0.0, 6.0):
        scene.objects.push(SceneObject::Box(AabbBox::new(
            Vector3D::new(1.8, -0.8, 5.2),
            Vector3D::new(3.2, 0.8, 6.8),
            0.6,
            0.0,
        )));

        // Directional sunlight coming from top-left:
        scene.lights.push(LightSource::Directional {
            direction: Vector3D::new(-0.5, -0.8, 0.3).normalize(),
            illuminance_lux: 10_000.0,
            wavelength_nm: 550.0,
        });

        scene
    }

    /// Adds an object to the scene.
    pub fn add_object(&mut self, object: SceneObject) {
        self.objects.push(object);
    }

    /// Adds a light to the scene.
    pub fn add_light(&mut self, light: LightSource) {
        self.lights.push(light);
    }

    /// Intersects a 3D ray with all scene objects, returning the nearest hit.
    pub fn intersect_ray(&self, origin: Vector3D, dir: Vector3D) -> Option<RayHit> {
        let mut closest_t = f64::INFINITY;
        let mut closest_hit: Option<RayHit> = None;

        for obj in &self.objects {
            match obj {
                SceneObject::Sphere(s) => {
                    if let Some((t, normal)) = s.intersect(origin, dir) {
                        if t < closest_t {
                            closest_t = t;
                            closest_hit = Some(RayHit {
                                distance_t: t,
                                hit_point: origin + dir * t,
                                normal,
                                albedo: s.albedo,
                                emission: s.emission,
                            });
                        }
                    }
                }
                SceneObject::Box(b) => {
                    if let Some((t, normal)) = b.intersect(origin, dir) {
                        if t < closest_t {
                            closest_t = t;
                            closest_hit = Some(RayHit {
                                distance_t: t,
                                hit_point: origin + dir * t,
                                normal,
                                albedo: b.albedo,
                                emission: b.emission,
                            });
                        }
                    }
                }
                SceneObject::Plane(p) => {
                    if let Some((t, normal, albedo)) = p.intersect(origin, dir) {
                        if t < closest_t {
                            closest_t = t;
                            closest_hit = Some(RayHit {
                                distance_t: t,
                                hit_point: origin + dir * t,
                                normal,
                                albedo,
                                emission: 0.0,
                            });
                        }
                    }
                }
            }
        }

        closest_hit
    }

    /// Computes incident illuminance in Lux at the given hit point and normal.
    pub fn evaluate_surface_illuminance(&self, hit: &RayHit) -> (f64, f64) {
        let mut total_lux = self.ambient_illuminance_lux * hit.albedo + hit.emission;
        let mut dominant_wavelength = self.default_wavelength_nm;

        for light in &self.lights {
            match light {
                LightSource::Directional {
                    direction,
                    illuminance_lux,
                    wavelength_nm,
                } => {
                    let l_dir = -(*direction).normalize();
                    let cos_theta = hit.normal.dot(&l_dir).max(0.0);
                    total_lux += illuminance_lux * cos_theta * hit.albedo;
                    dominant_wavelength = *wavelength_nm;
                }
                LightSource::Point {
                    position,
                    illuminance_at_1m_lux,
                    wavelength_nm,
                } => {
                    let to_light = *position - hit.hit_point;
                    let dist = to_light.norm().max(0.05);
                    let l_dir = to_light / dist;
                    let cos_theta = hit.normal.dot(&l_dir).max(0.0);
                    let atten = 1.0 / (dist * dist);
                    total_lux += illuminance_at_1m_lux * atten * cos_theta * hit.albedo;
                    dominant_wavelength = *wavelength_nm;
                }
            }
        }

        (total_lux.max(0.0), dominant_wavelength)
    }
}

/// Synthesized Optical Image Perception Frame.
#[derive(Debug, Clone, PartialEq)]
pub struct PerceptionFrame {
    pub width: u32,
    pub height: u32,
    /// Raw integrated photoelectrons per pixel.
    pub raw_electrons: Vec<f64>,
    /// 8-bit quantized grayscale pixel buffer (0..255).
    pub image_u8: Vec<u8>,
    /// 16-bit quantized digital pixel buffer (0..65535 or 0..4095).
    pub image_u16: Vec<u16>,
    /// Mean Signal-to-Noise Ratio (SNR) in dB across the frame.
    pub mean_snr_db: f64,
    /// Number of saturated pixels that exceeded full-well capacity.
    pub saturated_pixel_count: usize,
    /// Ratio of saturated pixels to total pixels [0.0, 1.0].
    pub saturation_ratio: f64,
}

/// Headless Offscreen Synthetic Perception Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct OffscreenPerceptionEngine {
    pub camera: OpticalCamera,
    pub tier: OpticalRealismTier,
    pub scene: SyntheticScene,
    pub sensor_temp_kelvin: f64,
}

impl OffscreenPerceptionEngine {
    /// Creates a new perception engine with camera, realism tier, and scene.
    pub fn new(camera: OpticalCamera, tier: OpticalRealismTier, scene: SyntheticScene) -> Self {
        Self {
            camera,
            tier,
            scene,
            sensor_temp_kelvin: 293.15, // 20 C
        }
    }

    /// Sets the sensor operating temperature in Kelvin.
    pub fn with_temperature(mut self, temp_kelvin: f64) -> Self {
        self.sensor_temp_kelvin = temp_kelvin;
        self
    }

    /// Computes incident photon count on a pixel given surface illuminance.
    pub fn photons_per_pixel(
        surface_illuminance_lux: f64,
        f_number: f64,
        pixel_pitch_m: f64,
        exposure_time_s: f64,
        wavelength_nm: f64,
    ) -> f64 {
        // Image plane illuminance: E_sensor = E_surface / (4 * N_f^2)
        let n_f = f_number.max(0.5);
        let e_sensor = surface_illuminance_lux / (4.0 * n_f * n_f);

        // Pixel area:
        let a_pixel = pixel_pitch_m * pixel_pitch_m;

        // Luminous flux per pixel (lumens = lux * m^2):
        let phi_v = e_sensor * a_pixel;

        // Radiant flux (Watts) at 555 nm peak (683 lm/W):
        let p_e = phi_v / 683.0;

        // Photon energy E_ph = h * c / lambda:
        let lambda_m = wavelength_nm * 1e-9;
        let e_ph = (PLANCK_CONSTANT * SPEED_OF_LIGHT) / lambda_m.max(100e-9);

        // Photon count integrated during exposure:
        (p_e / e_ph) * exposure_time_s
    }

    /// Renders a single pixel at coordinate (u, v) given RNG.
    ///
    /// Returns `(electrons, dn16, snr_db, is_saturated)`.
    pub fn render_pixel(&self, u: u32, v: u32, rng: &mut ChannelRng) -> (f64, u16, f64, bool) {
        let (ray_origin, ray_dir) = match self.tier {
            OpticalRealismTier::Tier0MicroscopicCmos
            | OpticalRealismTier::Tier1PhysicalLensRaster => {
                // Brown-Conrady unprojection:
                let cam_ray = self.camera.intrinsics.unproject_to_ray(u as f64, v as f64);
                // Transform ray from camera frame to world frame:
                let z_cam = self.camera.forward;
                let x_cam = self.camera.up.cross(&z_cam).normalize();
                let y_cam = z_cam.cross(&x_cam).normalize();
                let world_dir = x_cam * cam_ray.x + y_cam * cam_ray.y + z_cam * cam_ray.z;
                (self.camera.position, world_dir.normalize())
            }
            OpticalRealismTier::Tier2AcceleratedPinhole => {
                // Ideal pinhole without distortion iteration:
                let xn = (u as f64 - self.camera.intrinsics.cx) / self.camera.intrinsics.fx;
                let yn = (v as f64 - self.camera.intrinsics.cy) / self.camera.intrinsics.fy;
                let cam_ray = Vector3D::new(xn, yn, 1.0).normalize();
                let z_cam = self.camera.forward;
                let x_cam = self.camera.up.cross(&z_cam).normalize();
                let y_cam = z_cam.cross(&x_cam).normalize();
                let world_dir = x_cam * cam_ray.x + y_cam * cam_ray.y + z_cam * cam_ray.z;
                (self.camera.position, world_dir.normalize())
            }
        };

        // Trace ray into scene:
        let (surface_lux, wavelength_nm, hit_dist) =
            if let Some(hit) = self.scene.intersect_ray(ray_origin, ray_dir) {
                let (lux, wl) = self.scene.evaluate_surface_illuminance(&hit);
                (lux, wl, hit.distance_t)
            } else {
                (
                    self.scene.ambient_illuminance_lux * 0.2,
                    self.scene.default_wavelength_nm,
                    100.0,
                )
            };

        match self.tier {
            OpticalRealismTier::Tier0MicroscopicCmos => {
                // Rolling shutter delay adjustment if configured:
                let _dt_row = if self.camera.sensor_config.shutter_type
                    == phonon_models::optics::ShutterType::RollingShutter
                {
                    v as f64 * self.camera.rolling_shutter_row_delay_s()
                } else {
                    0.0
                };
                let effective_exposure = self.camera.exposure_time_s;

                let incident_photons = Self::photons_per_pixel(
                    surface_lux,
                    self.camera.f_number,
                    self.camera.sensor_config.pixel_pitch_m,
                    effective_exposure,
                    wavelength_nm,
                );

                let pixel_out = transduce_cmos_pixel(
                    &self.camera.sensor_config,
                    incident_photons,
                    wavelength_nm,
                    effective_exposure,
                    self.sensor_temp_kelvin,
                    rng,
                );

                (
                    pixel_out.collected_electrons,
                    pixel_out.digital_number as u16,
                    pixel_out.snr_db,
                    pixel_out.is_saturated,
                )
            }
            OpticalRealismTier::Tier1PhysicalLensRaster => {
                // Circle of confusion blur factor:
                let coc = self.camera.intrinsics.circle_of_confusion_m(
                    hit_dist,
                    self.camera.focus_distance_m,
                    self.camera.f_number,
                    self.camera.focal_length_m,
                );
                let blur_factor = (1.0 + coc / self.camera.sensor_config.pixel_pitch_m).recip();

                let incident_photons = Self::photons_per_pixel(
                    surface_lux * blur_factor,
                    self.camera.f_number,
                    self.camera.sensor_config.pixel_pitch_m,
                    self.camera.exposure_time_s,
                    wavelength_nm,
                );

                let qe = silicon_quantum_efficiency(wavelength_nm);
                let electrons_mean = incident_photons * qe;
                let shot_std = electrons_mean.sqrt();
                let (g, _) = rng.next_gaussian();
                let electrons_noisy = (electrons_mean + g * shot_std).max(0.0);

                let is_sat = electrons_noisy >= self.camera.sensor_config.full_well_capacity;
                let final_e = electrons_noisy.min(self.camera.sensor_config.full_well_capacity);

                let dn = ((final_e / self.camera.sensor_config.full_well_capacity) * 4095.0).round()
                    as u16;
                let snr = if electrons_mean > 1e-6 {
                    20.0 * (electrons_mean / shot_std.max(1.0)).log10()
                } else {
                    0.0
                };

                (final_e, dn, snr, is_sat)
            }
            OpticalRealismTier::Tier2AcceleratedPinhole => {
                // Fast direct linear mapping:
                let max_lux = 10_000.0;
                let norm = (surface_lux / max_lux).clamp(0.0, 1.0);
                let dn = (norm * 255.0).round() as u16;
                let electrons = norm * self.camera.sensor_config.full_well_capacity;
                let is_sat = norm >= 1.0;
                (electrons, dn, 40.0, is_sat)
            }
        }
    }

    /// Renders a complete perception frame using parallel Rayon worker threads.
    pub fn render_frame(&self, frame_seed: u64) -> PerceptionFrame {
        let width = self.camera.intrinsics.width;
        let height = self.camera.intrinsics.height;
        let num_pixels = (width * height) as usize;

        // Render rows in parallel:
        let row_results: Vec<Vec<(f64, u16, f64, bool)>> = (0..height)
            .into_par_iter()
            .map(|v| {
                let mut row = Vec::with_capacity(width as usize);
                let row_seed = frame_seed.wrapping_add((v as u64) << 16);
                let mut rng = ChannelRng::new(row_seed);

                for u in 0..width {
                    row.push(self.render_pixel(u, v, &mut rng));
                }
                row
            })
            .collect();

        let mut raw_electrons = Vec::with_capacity(num_pixels);
        let mut image_u8 = Vec::with_capacity(num_pixels);
        let mut image_u16 = Vec::with_capacity(num_pixels);
        let mut total_snr = 0.0;
        let mut saturated_count = 0;

        for row in row_results {
            for (e, dn, snr, sat) in row {
                raw_electrons.push(e);
                image_u16.push(dn);
                let val_u8 = match self.tier {
                    OpticalRealismTier::Tier0MicroscopicCmos => (dn >> 4).min(255) as u8, // 12-bit to 8-bit
                    OpticalRealismTier::Tier1PhysicalLensRaster => (dn >> 4).min(255) as u8,
                    OpticalRealismTier::Tier2AcceleratedPinhole => dn.min(255) as u8,
                };
                image_u8.push(val_u8);
                total_snr += snr;
                if sat {
                    saturated_count += 1;
                }
            }
        }

        let mean_snr_db = if num_pixels > 0 {
            total_snr / (num_pixels as f64)
        } else {
            0.0
        };
        let saturation_ratio = if num_pixels > 0 {
            saturated_count as f64 / (num_pixels as f64)
        } else {
            0.0
        };

        PerceptionFrame {
            width,
            height,
            raw_electrons,
            image_u8,
            image_u16,
            mean_snr_db,
            saturated_pixel_count: saturated_count,
            saturation_ratio,
        }
    }
}
