#![allow(clippy::needless_range_loop)]
//! Spacecraft Orbital Mechanics, Perturbed Cowell Dynamics & Geopotential Harmonics.
//!
//! Formulates:
//! - Cowell two-body gravitational acceleration with non-spherical Earth geopotential ($J_2, J_3, J_4$ zonal harmonics).
//! - Third-body gravitational perturbations from Moon and Sun.
//! - Upper atmospheric drag with exponential density model and Earth rotation.
//! - Solar Radiation Pressure (SRP) with cylindrical Earth eclipse factor.
//! - Classical Keplerian orbital elements $(a, e, i, \Omega, \omega, \nu)$ conversion to/from Cartesian $(\mathbf{r}, \mathbf{v})$.

use crate::em::Vector3D;
use std::f64::consts::PI;

/// Earth gravitational parameter $\mu_\oplus$ in $\text{m}^3/\text{s}^2$.
pub const MU_EARTH: f64 = 3.986_004_418e14;

/// Earth equatorial radius $R_\oplus$ in meters ($m$).
pub const R_EARTH: f64 = 6_378_137.0;

/// Earth $J_2$ zonal harmonic coefficient (oblateness).
pub const J2_EARTH: f64 = 1.082_626_68e-3;

/// Earth $J_3$ zonal harmonic coefficient (pear shape).
pub const J3_EARTH: f64 = -2.532_7e-6;

/// Earth $J_4$ zonal harmonic coefficient.
pub const J4_EARTH: f64 = -1.619_6e-6;

/// Moon gravitational parameter $\mu_{moon}$ in $\text{m}^3/\text{s}^2$.
pub const MU_MOON: f64 = 4.904_869_5e12;

/// Sun gravitational parameter $\mu_{sun}$ in $\text{m}^3/\text{s}^2$.
pub const MU_SUN: f64 = 1.327_124_400_18e20;

/// 1 Astronomical Unit (AU) in meters.
pub const ASTRONOMICAL_UNIT: f64 = 1.495_978_707e11;

/// Solar Radiation Pressure at 1 AU in $\text{N}/\text{m}^2$.
pub const P_SRP_1AU: f64 = 4.56e-6;

/// Earth rotation angular velocity vector in ECI frame (rad/s).
pub const EARTH_OMEGA_VEC: Vector3D = Vector3D::new(0.0, 0.0, 7.292_115e-5);

/// Classical Keplerian Orbital Elements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeplerianElements {
    /// Semi-major axis $a$ in meters ($m$).
    pub semi_major_axis_m: f64,
    /// Eccentricity $e$ ($0 \le e < 1$ for elliptic orbits).
    pub eccentricity: f64,
    /// Inclination $i$ in radians.
    pub inclination_rad: f64,
    /// Right Ascension of the Ascending Node (RAAN) $\Omega$ in radians.
    pub raan_rad: f64,
    /// Argument of perigee $\omega$ in radians.
    pub arg_perigee_rad: f64,
    /// True anomaly $\nu$ in radians.
    pub true_anomaly_rad: f64,
}

impl KeplerianElements {
    /// Computes orbital period $T = 2\pi \sqrt{a^3 / \mu}$ in seconds ($s$).
    pub fn period_s(&self) -> f64 {
        2.0 * PI * (self.semi_major_axis_m.powi(3) / MU_EARTH).sqrt()
    }

    /// Converts Keplerian orbital elements to Cartesian position and velocity in ECI frame.
    pub fn to_cartesian(&self) -> (Vector3D, Vector3D) {
        let p = self.semi_major_axis_m * (1.0 - self.eccentricity * self.eccentricity);
        let r_mag = p / (1.0 + self.eccentricity * self.true_anomaly_rad.cos());

        // Position and velocity in orbital perifocal plane (PQW frame):
        let p_orb = Vector3D::new(
            r_mag * self.true_anomaly_rad.cos(),
            r_mag * self.true_anomaly_rad.sin(),
            0.0,
        );

        let v_factor = (MU_EARTH / p).sqrt();
        let v_orb = Vector3D::new(
            -v_factor * self.true_anomaly_rad.sin(),
            v_factor * (self.eccentricity + self.true_anomaly_rad.cos()),
            0.0,
        );

        // Rotation matrix from perifocal to ECI frame:
        let o = self.raan_rad;
        let w = self.arg_perigee_rad;
        let i = self.inclination_rad;

        let cos_o = o.cos();
        let sin_o = o.sin();
        let cos_w = w.cos();
        let sin_w = w.sin();
        let cos_i = i.cos();
        let sin_i = i.sin();

        let px = cos_o * cos_w - sin_o * sin_w * cos_i;
        let py = sin_o * cos_w + cos_o * sin_w * cos_i;
        let pz = sin_w * sin_i;

        let qx = -cos_o * sin_w - sin_o * cos_w * cos_i;
        let qy = -sin_o * sin_w + cos_o * cos_w * cos_i;
        let qz = cos_w * sin_i;

        let r_eci = Vector3D::new(
            px * p_orb.x + qx * p_orb.y,
            py * p_orb.x + qy * p_orb.y,
            pz * p_orb.x + qz * p_orb.y,
        );

        let v_eci = Vector3D::new(
            px * v_orb.x + qx * v_orb.y,
            py * v_orb.x + qy * v_orb.y,
            pz * v_orb.x + qz * v_orb.y,
        );

        (r_eci, v_eci)
    }

    /// Converts Cartesian state vectors $(\mathbf{r}, \mathbf{v})$ in ECI frame to Keplerian elements.
    pub fn from_cartesian(r: Vector3D, v: Vector3D) -> Self {
        let r_mag = r.norm();
        let v_mag = v.norm();

        let h = r.cross(&v);
        let h_mag = h.norm();

        let n = Vector3D::new(-h.y, h.x, 0.0);
        let n_mag = n.norm();

        let e_vec = Vector3D::new(
            (v_mag * v_mag - MU_EARTH / r_mag) * r.x - r.dot(&v) * v.x,
            (v_mag * v_mag - MU_EARTH / r_mag) * r.y - r.dot(&v) * v.y,
            (v_mag * v_mag - MU_EARTH / r_mag) * r.z - r.dot(&v) * v.z,
        );
        let e_vec = Vector3D::new(e_vec.x / MU_EARTH, e_vec.y / MU_EARTH, e_vec.z / MU_EARTH);
        let e = e_vec.norm();

        let specific_energy = 0.5 * v_mag * v_mag - MU_EARTH / r_mag;
        let a = -MU_EARTH / (2.0 * specific_energy);

        let inc = (h.z / h_mag.max(1e-15)).clamp(-1.0, 1.0).acos();

        let raan = if n_mag > 1e-12 {
            let val = (n.x / n_mag).clamp(-1.0, 1.0).acos();
            if n.y >= 0.0 {
                val
            } else {
                2.0 * PI - val
            }
        } else {
            0.0
        };

        let arg_perigee = if n_mag > 1e-12 && e > 1e-12 {
            let val = (n.dot(&e_vec) / (n_mag * e)).clamp(-1.0, 1.0).acos();
            if e_vec.z >= 0.0 {
                val
            } else {
                2.0 * PI - val
            }
        } else {
            0.0
        };

        let true_anomaly = if e > 1e-12 {
            let val = (e_vec.dot(&r) / (e * r_mag)).clamp(-1.0, 1.0).acos();
            if r.dot(&v) >= 0.0 {
                val
            } else {
                2.0 * PI - val
            }
        } else {
            0.0
        };

        Self {
            semi_major_axis_m: a,
            eccentricity: e,
            inclination_rad: inc,
            raan_rad: raan,
            arg_perigee_rad: arg_perigee,
            true_anomaly_rad: true_anomaly,
        }
    }
}

/// Spacecraft physical parameters for environmental force modeling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpacecraftPhysicalProperties {
    /// Spacecraft total mass $m$ in kilograms ($kg$).
    pub mass_kg: f64,
    /// Cross-sectional area for aerodynamic drag $A_{drag}$ in $m^2$.
    pub drag_area_m2: f64,
    /// Aerodynamic drag coefficient $C_D$ (typically $2.0 - 2.5$).
    pub drag_coefficient: f64,
    /// Cross-sectional area for solar radiation pressure $A_{srp}$ in $m^2$.
    pub srp_area_m2: f64,
    /// Radiation pressure reflectivity coefficient $C_R$ ($1.0$ absorbing, $2.0$ specular).
    pub srp_reflectivity: f64,
}

impl Default for SpacecraftPhysicalProperties {
    fn default() -> Self {
        Self {
            mass_kg: 100.0,
            drag_area_m2: 1.0,
            drag_coefficient: 2.2,
            srp_area_m2: 1.5,
            srp_reflectivity: 1.3,
        }
    }
}

/// Spacecraft orbital perturbation solver evaluating Cowell acceleration components.
pub struct OrbitalPerturbationSolver;

impl OrbitalPerturbationSolver {
    /// Evaluates central two-body gravitational acceleration $\mathbf{a}_0 = -\frac{\mu}{r^3} \mathbf{r}$.
    pub fn two_body_accel(r: Vector3D) -> Vector3D {
        let r_mag = r.norm();
        let factor = -MU_EARTH / (r_mag * r_mag * r_mag).max(1e-6);
        Vector3D::new(factor * r.x, factor * r.y, factor * r.z)
    }

    /// Evaluates geopotential perturbation acceleration from $J_2, J_3, J_4$ zonal harmonics:
    pub fn geopotential_j2_j4_accel(r: Vector3D) -> Vector3D {
        let r_mag = r.norm();
        if r_mag < 1e-3 {
            return Vector3D::ZERO;
        }

        let re = R_EARTH;
        let x = r.x;
        let y = r.y;
        let z = r.z;

        let u_x = x / r_mag;
        let u_y = y / r_mag;
        let u_z = z / r_mag;
        let u_z2 = u_z * u_z;
        let u_z3 = u_z2 * u_z;
        let u_z4 = u_z2 * u_z2;

        // J2 contribution:
        let factor_j2 = 1.5 * J2_EARTH * MU_EARTH * (re * re) / r_mag.powi(4);
        let a_j2_x = -factor_j2 * u_x * (1.0 - 5.0 * u_z2);
        let a_j2_y = -factor_j2 * u_y * (1.0 - 5.0 * u_z2);
        let a_j2_z = -factor_j2 * u_z * (3.0 - 5.0 * u_z2);

        // J3 contribution (pear-shape asymmetry):
        let factor_j3 = 0.5 * J3_EARTH * MU_EARTH * re.powi(3) / r_mag.powi(5);
        let a_j3_x = 5.0 * factor_j3 * u_x * (7.0 * u_z3 - 3.0 * u_z);
        let a_j3_y = 5.0 * factor_j3 * u_y * (7.0 * u_z3 - 3.0 * u_z);
        let a_j3_z = factor_j3 * (35.0 * u_z4 - 30.0 * u_z2 + 3.0);

        // J4 contribution:
        let factor_j4 = 0.625 * J4_EARTH * MU_EARTH * re.powi(4) / r_mag.powi(6);
        let a_j4_x = factor_j4 * u_x * (3.0 - 42.0 * u_z2 + 63.0 * u_z4);
        let a_j4_y = factor_j4 * u_y * (3.0 - 42.0 * u_z2 + 63.0 * u_z4);
        let a_j4_z = factor_j4 * u_z * (15.0 - 70.0 * u_z2 + 63.0 * u_z4);

        Vector3D::new(
            a_j2_x + a_j3_x + a_j4_x,
            a_j2_y + a_j3_y + a_j4_y,
            a_j2_z + a_j3_z + a_j4_z,
        )
    }

    /// Evaluates third-body gravitational perturbation from point mass (Moon or Sun):
    /// $$\mathbf{a}_{3rd} = \mu_{3rd} \left( \frac{\mathbf{r}_{3rd} - \mathbf{r}}{\|\mathbf{r}_{3rd} - \mathbf{r}\|^3} - \frac{\mathbf{r}_{3rd}}{\|\mathbf{r}_{3rd}\|^3} \right)$$
    pub fn third_body_accel(r_sc: Vector3D, r_body: Vector3D, mu_body: f64) -> Vector3D {
        let d = Vector3D::new(r_body.x - r_sc.x, r_body.y - r_sc.y, r_body.z - r_sc.z);
        let d_mag = d.norm();
        let r_body_mag = r_body.norm();

        let term1 = mu_body / (d_mag * d_mag * d_mag).max(1e-6);
        let term2 = mu_body / (r_body_mag * r_body_mag * r_body_mag).max(1e-6);

        Vector3D::new(
            term1 * d.x - term2 * r_body.x,
            term1 * d.y - term2 * r_body.y,
            term1 * d.z - term2 * r_body.z,
        )
    }

    /// Atmospheric density at altitude $h$ above WGS-84 reference ellipsoid ($kg/m^3$):
    pub fn atmospheric_density(altitude_m: f64) -> f64 {
        if altitude_m < 0.0 {
            return 1.225;
        }
        if altitude_m > 1_000_000.0 {
            return 0.0;
        }
        // Multi-scale atmospheric density model (100 km to 1000 km):
        let (h0, rho0, scale_h) = if altitude_m < 200_000.0 {
            (100_000.0, 5.297e-7, 5_877.0)
        } else if altitude_m < 400_000.0 {
            (200_000.0, 2.789e-10, 37_105.0)
        } else if altitude_m < 600_000.0 {
            (400_000.0, 2.803e-12, 58_200.0)
        } else {
            (600_000.0, 8.637e-14, 71_835.0)
        };

        rho0 * (-(altitude_m - h0) / scale_h).exp()
    }

    /// Evaluates aerodynamic drag acceleration $\mathbf{a}_{drag} = -\frac{1}{2} \rho \frac{C_D A}{m} v_{rel} \mathbf{v}_{rel}$:
    pub fn atmospheric_drag_accel(
        r: Vector3D,
        v: Vector3D,
        props: &SpacecraftPhysicalProperties,
    ) -> Vector3D {
        let r_mag = r.norm();
        let alt = r_mag - R_EARTH;
        let rho = Self::atmospheric_density(alt);
        if rho <= 1e-25 {
            return Vector3D::ZERO;
        }

        // Relative velocity with respect to rotating Earth atmosphere:
        let v_atm = EARTH_OMEGA_VEC.cross(&r);
        let v_rel = Vector3D::new(v.x - v_atm.x, v.y - v_atm.y, v.z - v_atm.z);
        let v_rel_mag = v_rel.norm();

        let drag_factor =
            -0.5 * rho * props.drag_coefficient * props.drag_area_m2 / props.mass_kg * v_rel_mag;
        Vector3D::new(
            drag_factor * v_rel.x,
            drag_factor * v_rel.y,
            drag_factor * v_rel.z,
        )
    }

    /// Evaluates cylindrical Earth eclipse shadow factor $\nu \in [0.0, 1.0]$:
    pub fn shadow_factor(r_sc: Vector3D, r_sun: Vector3D) -> f64 {
        let s = r_sun.normalize();
        let r_proj = r_sc.dot(&s);

        if r_proj > 0.0 {
            // Spacecraft is on dayside of Earth
            1.0
        } else {
            // Spacecraft is on nightside; check distance to Earth shadow axis:
            let perp = Vector3D::new(
                r_sc.x - r_proj * s.x,
                r_sc.y - r_proj * s.y,
                r_sc.z - r_proj * s.z,
            );
            if perp.norm() < R_EARTH {
                0.0 // Full umbra eclipse
            } else {
                1.0 // Sunlit
            }
        }
    }

    /// Evaluates Solar Radiation Pressure (SRP) acceleration:
    pub fn solar_radiation_accel(
        r_sc: Vector3D,
        r_sun: Vector3D,
        props: &SpacecraftPhysicalProperties,
    ) -> Vector3D {
        let nu = Self::shadow_factor(r_sc, r_sun);
        if nu <= 0.0 {
            return Vector3D::ZERO;
        }

        let sun_vec = Vector3D::new(r_sun.x - r_sc.x, r_sun.y - r_sc.y, r_sun.z - r_sc.z);
        let sun_unit = sun_vec.normalize();

        let srp_factor =
            -P_SRP_1AU * props.srp_reflectivity * props.srp_area_m2 / props.mass_kg * nu;
        Vector3D::new(
            srp_factor * sun_unit.x,
            srp_factor * sun_unit.y,
            srp_factor * sun_unit.z,
        )
    }

    /// Total Cowell perturbed orbital acceleration $\ddot{\mathbf{r}}$:
    pub fn total_orbital_accel(
        r: Vector3D,
        v: Vector3D,
        r_sun: Vector3D,
        r_moon: Vector3D,
        props: &SpacecraftPhysicalProperties,
    ) -> Vector3D {
        let a0 = Self::two_body_accel(r);
        let a_geo = Self::geopotential_j2_j4_accel(r);
        let a_drag = Self::atmospheric_drag_accel(r, v, props);
        let a_srp = Self::solar_radiation_accel(r, r_sun, props);
        let a_sun = Self::third_body_accel(r, r_sun, MU_SUN);
        let a_moon = Self::third_body_accel(r, r_moon, MU_MOON);

        Vector3D::new(
            a0.x + a_geo.x + a_drag.x + a_srp.x + a_sun.x + a_moon.x,
            a0.y + a_geo.y + a_drag.y + a_srp.y + a_sun.y + a_moon.y,
            a0.z + a_geo.z + a_drag.z + a_srp.z + a_sun.z + a_moon.z,
        )
    }
}
