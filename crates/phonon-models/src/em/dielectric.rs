//! Complex Dielectric Materials, Walls & Obstacle Wave Interactions.
//!
//! Provides complex permittivity, Fresnel reflection/transmission, skin depth,
//! slab attenuation with multiple internal reflections, and knife-edge diffraction.

use super::vector_wave::Vector3D;
use phonon_core::constants::{EPSILON_0, MU_0};
use std::f64::consts::PI;

/// Complex dielectric material specifications.
#[derive(Debug, Clone, PartialEq)]
pub struct RfDielectricMaterial {
    pub name: &'static str,
    /// Real relative permittivity $\epsilon_r^\prime$.
    pub relative_permittivity_real: f64,
    /// Dielectric loss tangent $\tan\delta = \epsilon_r^{\prime\prime} / \epsilon_r^\prime$.
    pub loss_tangent: f64,
    /// DC/AC electrical conductivity in Siemens per meter (S/m).
    pub conductivity_siemens_per_m: f64,
}

impl RfDielectricMaterial {
    pub const fn new(
        name: &'static str,
        relative_permittivity_real: f64,
        loss_tangent: f64,
        conductivity_siemens_per_m: f64,
    ) -> Self {
        Self {
            name,
            relative_permittivity_real,
            loss_tangent,
            conductivity_siemens_per_m,
        }
    }

    /// Air / Free Space ($\epsilon_r = 1.0, \tan\delta = 0$).
    pub const fn air() -> Self {
        Self::new("Air", 1.0, 0.0, 0.0)
    }

    /// Standard Cured Concrete ($\epsilon_r \approx 4.5, \tan\delta \approx 0.05, \sigma \approx 0.01\text{ S/m}$).
    pub const fn concrete() -> Self {
        Self::new("Concrete", 4.5, 0.05, 0.01)
    }

    /// Interior Drywall / Plasterboard ($\epsilon_r \approx 2.8, \tan\delta \approx 0.02$).
    pub const fn drywall() -> Self {
        Self::new("Drywall", 2.8, 0.02, 0.001)
    }

    /// Window Glass ($\epsilon_r \approx 6.0, \tan\delta \approx 0.01$).
    pub const fn glass() -> Self {
        Self::new("Glass", 6.0, 0.01, 1e-12)
    }

    /// Structural Wood ($\epsilon_r \approx 2.0, \tan\delta \approx 0.04$).
    pub const fn wood() -> Self {
        Self::new("Wood", 2.0, 0.04, 1e-4)
    }

    /// Red Brick ($\epsilon_r \approx 4.0, \tan\delta \approx 0.03$).
    pub const fn brick() -> Self {
        Self::new("Brick", 4.0, 0.03, 0.002)
    }

    /// Metal Copper / Perfect Electric Conductor ($\sigma = 5.8 \times 10^7\text{ S/m}$).
    pub const fn metal_copper() -> Self {
        Self::new("Copper", 1.0, 0.0, 5.8e7)
    }

    /// Dry Ground / Soil ($\epsilon_r \approx 3.0, \tan\delta \approx 0.01$).
    pub const fn dry_ground() -> Self {
        Self::new("DryGround", 3.0, 0.01, 1e-4)
    }

    /// Wet Ground / Soil ($\epsilon_r \approx 25.0, \tan\delta \approx 0.2, \sigma \approx 0.02\text{ S/m}$).
    pub const fn wet_ground() -> Self {
        Self::new("WetGround", 25.0, 0.20, 0.02)
    }

    /// Sea Water ($\epsilon_r \approx 81.0, \sigma \approx 4.0\text{ S/m}$).
    pub const fn sea_water() -> Self {
        Self::new("SeaWater", 81.0, 0.05, 4.0)
    }

    /// Evaluates effective complex permittivity $(\epsilon^\prime, \epsilon^{\prime\prime})$ at frequency $f$.
    pub fn complex_permittivity(&self, freq_hz: f64) -> (f64, f64) {
        let omega = 2.0 * PI * freq_hz.max(1.0);
        let eps_real = self.relative_permittivity_real * EPSILON_0;
        let eps_imag_dielectric = eps_real * self.loss_tangent;
        let eps_imag_ohmic = self.conductivity_siemens_per_m / omega;
        (eps_real, eps_imag_dielectric + eps_imag_ohmic)
    }

    /// Computes propagation constant $\gamma = \alpha + j\beta$ at frequency $f$.
    ///
    /// Returns $(\alpha, \beta)$ where $\alpha$ is attenuation in Np/m and $\beta$ is phase constant in rad/m.
    pub fn propagation_constant(&self, freq_hz: f64) -> (f64, f64) {
        let omega = 2.0 * PI * freq_hz.max(1.0);
        let (eps_real, eps_imag) = self.complex_permittivity(freq_hz);

        let tan_d = if eps_real > 1e-18 {
            eps_imag / eps_real
        } else {
            0.0
        };
        let term = (1.0 + tan_d * tan_d).sqrt();

        let factor = omega * (MU_0 * eps_real * 0.5).sqrt();
        let alpha = factor * (term - 1.0).max(0.0).sqrt();
        let beta = factor * (term + 1.0).sqrt();

        (alpha, beta)
    }

    /// Skin depth $\delta = 1 / \alpha$ in meters.
    pub fn skin_depth(&self, freq_hz: f64) -> f64 {
        let (alpha, _) = self.propagation_constant(freq_hz);
        if alpha > 1e-12 {
            1.0 / alpha
        } else {
            f64::INFINITY
        }
    }

    /// Complex refractive index $n^* = n - j\kappa = \sqrt{\epsilon_r^*}$.
    pub fn complex_refractive_index(&self, freq_hz: f64) -> (f64, f64) {
        let (eps_r_re, eps_r_im) = {
            let (er, ei) = self.complex_permittivity(freq_hz);
            (er / EPSILON_0, ei / EPSILON_0)
        };
        let mag = (eps_r_re * eps_r_re + eps_r_im * eps_r_im).sqrt();
        let n = ((mag + eps_r_re) * 0.5).sqrt();
        let kappa = ((mag - eps_r_re).max(0.0) * 0.5).sqrt();
        (n, kappa)
    }
}

/// Fresnel reflection and transmission coefficients for TE and TM polarizations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FresnelCoefficients {
    /// Complex reflection coefficient for TE (perpendicular) mode: $(r_{re}, r_{im})$.
    pub r_te: (f64, f64),
    /// Complex transmission coefficient for TE mode: $(t_{re}, t_{im})$.
    pub t_te: (f64, f64),
    /// Complex reflection coefficient for TM (parallel) mode: $(r_{re}, r_{im})$.
    pub r_tm: (f64, f64),
    /// Complex transmission coefficient for TM mode: $(t_{re}, t_{im})$.
    pub t_tm: (f64, f64),
    /// Power reflection for TE mode: $|R_{TE}|^2$.
    pub power_reflection_te: f64,
    /// Power transmission for TE mode: $1 - |R_{TE}|^2$.
    pub power_transmission_te: f64,
    /// Power reflection for TM mode: $|R_{TM}|^2$.
    pub power_reflection_tm: f64,
    /// Power transmission for TM mode: $1 - |R_{TM}|^2$.
    pub power_transmission_tm: f64,
}

impl FresnelCoefficients {
    /// Computes Fresnel coefficients at dielectric boundary between `mat1` and `mat2`
    /// at incident angle $\theta_i$ (in radians) and frequency $f$.
    pub fn calculate(
        mat1: &RfDielectricMaterial,
        mat2: &RfDielectricMaterial,
        theta_i_rad: f64,
        freq_hz: f64,
    ) -> Self {
        let (n1, k1) = mat1.complex_refractive_index(freq_hz);
        let (n2, k2) = mat2.complex_refractive_index(freq_hz);

        // Approximate Snell angle using real index parts
        let sin_theta_t = (n1 / n2.max(1e-3)) * theta_i_rad.sin();
        let (cos_theta_i, cos_theta_t) = if sin_theta_t.abs() <= 1.0 {
            (theta_i_rad.cos(), (1.0 - sin_theta_t * sin_theta_t).sqrt())
        } else {
            // Total Internal Reflection (evanescent wave)
            (theta_i_rad.cos(), 0.0)
        };

        let n1_eff = (n1, k1);
        let n2_eff = (n2, k2);

        let cmul = |a: (f64, f64), b: f64| (a.0 * b, a.1 * b);
        let cdiv = |a: (f64, f64), b: (f64, f64)| -> (f64, f64) {
            let denom = b.0 * b.0 + b.1 * b.1;
            if denom < 1e-15 {
                (0.0, 0.0)
            } else {
                (
                    (a.0 * b.0 + a.1 * b.1) / denom,
                    (a.1 * b.0 - a.0 * b.1) / denom,
                )
            }
        };

        // TE mode: R_TE = (n1 cos_i - n2 cos_t) / (n1 cos_i + n2 cos_t)
        let n1_cos_i = cmul(n1_eff, cos_theta_i);
        let n2_cos_t = cmul(n2_eff, cos_theta_t);

        let num_te = (n1_cos_i.0 - n2_cos_t.0, n1_cos_i.1 - n2_cos_t.1);
        let den_te = (n1_cos_i.0 + n2_cos_t.0, n1_cos_i.1 + n2_cos_t.1);
        let r_te = cdiv(num_te, den_te);
        let t_te = (1.0 + r_te.0, r_te.1);

        // TM mode: R_TM = (n2 cos_i - n1 cos_t) / (n2 cos_i + n1 cos_t)
        let n2_cos_i = cmul(n2_eff, cos_theta_i);
        let n1_cos_t = cmul(n1_eff, cos_theta_t);

        let num_tm = (n2_cos_i.0 - n1_cos_t.0, n2_cos_i.1 - n1_cos_t.1);
        let den_tm = (n2_cos_i.0 + n1_cos_t.0, n2_cos_i.1 + n1_cos_t.1);
        let r_tm = cdiv(num_tm, den_tm);
        let t_tm = (1.0 + r_tm.0, r_tm.1);

        let p_r_te = (r_te.0 * r_te.0 + r_te.1 * r_te.1).min(1.0);
        let p_t_te = (1.0 - p_r_te).max(0.0);

        let p_r_tm = (r_tm.0 * r_tm.0 + r_tm.1 * r_tm.1).min(1.0);
        let p_t_tm = (1.0 - p_r_tm).max(0.0);

        Self {
            r_te,
            t_te,
            r_tm,
            t_tm,
            power_reflection_te: p_r_te,
            power_transmission_te: p_t_te,
            power_reflection_tm: p_r_tm,
            power_transmission_tm: p_t_tm,
        }
    }
}

/// Planar Dielectric Wall / Obstacle in 3D Space.
#[derive(Debug, Clone, PartialEq)]
pub struct DielectricWall {
    /// Center of the wall slab in meters.
    pub center: Vector3D,
    /// Surface unit normal vector pointing toward incident side.
    pub normal: Vector3D,
    /// Physical wall thickness in meters ($d_w$).
    pub thickness_m: f64,
    /// Wall dimension along horizontal tangent (meters).
    pub width_m: f64,
    /// Wall dimension along vertical tangent (meters).
    pub height_m: f64,
    /// Bulk material of the wall.
    pub material: RfDielectricMaterial,
}

/// Ray-Wall Intersection Result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    /// 3D hit point in meters.
    pub point: Vector3D,
    /// Distance from ray origin to hit point (meters).
    pub distance_m: f64,
    /// Incident angle $\theta_i$ in radians relative to surface normal.
    pub incident_angle_rad: f64,
}

impl DielectricWall {
    pub fn new(
        center: Vector3D,
        normal: Vector3D,
        thickness_m: f64,
        width_m: f64,
        height_m: f64,
        material: RfDielectricMaterial,
    ) -> Self {
        Self {
            center,
            normal: normal.normalize(),
            thickness_m: thickness_m.max(1e-4),
            width_m: width_m.max(0.1),
            height_m: height_m.max(0.1),
            material,
        }
    }

    /// Tests intersection between a ray segment $(\mathbf{p}_{origin} \to \mathbf{p}_{target})$ and this wall.
    pub fn intersects_segment(&self, origin: &Vector3D, target: &Vector3D) -> Option<RayHit> {
        let dir = *target - *origin;
        let max_dist = dir.norm();
        if max_dist < 1e-6 {
            return None;
        }
        let ray_dir = dir.scale(1.0 / max_dist);

        let denom = self.normal.dot(&ray_dir);
        if denom.abs() < 1e-9 {
            return None; // Parallel to wall
        }

        let t = (self.center - *origin).dot(&self.normal) / denom;
        if t <= 1e-4 || t >= max_dist {
            return None; // Outside line segment
        }

        let hit_point = *origin + ray_dir.scale(t);

        // Check if within planar rectangular bounds
        let diff = hit_point - self.center;
        let up = if self.normal.z.abs() < 0.9 {
            Vector3D::new(0.0, 0.0, 1.0)
        } else {
            Vector3D::new(0.0, 1.0, 0.0)
        };
        let tangent = self.normal.cross(&up).normalize();
        let bitangent = self.normal.cross(&tangent).normalize();

        let u = diff.dot(&tangent).abs();
        let v = diff.dot(&bitangent).abs();

        if u <= self.width_m * 0.5 && v <= self.height_m * 0.5 {
            let cos_i = (-denom).abs().clamp(0.0, 1.0);
            Some(RayHit {
                point: hit_point,
                distance_m: t,
                incident_angle_rad: cos_i.acos(),
            })
        } else {
            None
        }
    }

    /// Slab transmission attenuation accounting for internal multiple reflections:
    ///
    /// $$T_{wall} = \frac{(1 - R^2) e^{-\gamma d}}{1 - R^2 e^{-2\gamma d}}$$
    ///
    /// Returns the attenuation factor in decibels ($L_{wall} \ge 0\text{ dB}$).
    pub fn attenuation_db(&self, incident_angle_rad: f64, freq_hz: f64) -> f64 {
        let air = RfDielectricMaterial::air();
        let fresnel =
            FresnelCoefficients::calculate(&air, &self.material, incident_angle_rad, freq_hz);

        let (alpha, _) = self.material.propagation_constant(freq_hz);

        // Effective path length inside dielectric slab accounting for refraction
        let (n_mat, _) = self.material.complex_refractive_index(freq_hz);
        let sin_theta_t = (1.0 / n_mat.max(1.0)) * incident_angle_rad.sin();
        let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).max(0.0).sqrt();
        let d_eff = self.thickness_m / cos_theta_t.max(0.1);

        // Dielectric bulk attenuation
        let one_way_bulk_att = (-alpha * d_eff).exp();

        // Average power reflection coefficient across TE and TM
        let r_sq = (fresnel.power_reflection_te + fresnel.power_reflection_tm) * 0.5;

        // Multiple bounce transmission amplitude
        let num = (1.0 - r_sq) * one_way_bulk_att;
        let den = 1.0 - r_sq * one_way_bulk_att * one_way_bulk_att;
        let trans_factor = if den > 1e-12 {
            (num / den).min(1.0)
        } else {
            1e-10
        };

        let power_trans = (trans_factor * trans_factor).max(1e-15);
        -10.0 * power_trans.log10()
    }
}

/// Knife-Edge Obstacle Diffraction based on ITU-R P.526.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnifeEdgeObstacle;

impl KnifeEdgeObstacle {
    /// Computes Fresnel-Kirchhoff diffraction parameter $\nu$:
    ///
    /// $$\nu = h_{obs} \sqrt{\frac{2(d_1 + d_2)}{\lambda d_1 d_2}}$$
    pub fn diffraction_parameter(
        obstacle_height_above_los_m: f64,
        dist_tx_to_obs_m: f64,
        dist_obs_to_rx_m: f64,
        wavelength_m: f64,
    ) -> f64 {
        let d1 = dist_tx_to_obs_m.max(1.0);
        let d2 = dist_obs_to_rx_m.max(1.0);
        let lam = wavelength_m.max(1e-4);

        obstacle_height_above_los_m * (2.0 * (d1 + d2) / (lam * d1 * d2)).sqrt()
    }

    /// Evaluates knife-edge diffraction loss $J(\nu)$ in decibels according to ITU-R P.526 approximation:
    ///
    /// $$J(\nu) = 6.9 + 20\log_{10}\left(\sqrt{(\nu - 0.1)^2 + 1} + \nu - 0.1\right)\text{ dB}$$
    pub fn diffraction_loss_db(nu: f64) -> f64 {
        if nu <= -0.7 {
            0.0 // Line-of-sight clear, negligible diffraction loss
        } else {
            let term = ((nu - 0.1).powi(2) + 1.0).sqrt() + nu - 0.1;
            if term > 1e-6 {
                (6.9 + 20.0 * term.log10()).max(0.0)
            } else {
                0.0
            }
        }
    }
}
