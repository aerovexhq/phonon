//! WGS-84 Geodetic, Earth-Centered Earth-Fixed (ECEF) & Topocentric Space Coordinates.
//!
//! Provides mathematically rigorous coordinate transformations between Geodetic (Lat/Lon/Alt),
//! ECEF, ECI, and East-North-Up (ENU) topocentric frames, as well as Earth curvature
//! and radio horizon line-of-sight (LOS) clearance checks.

use super::vector_wave::Vector3D;

/// WGS-84 Ellipsoid Semi-Major Axis $a$ (equatorial radius) in meters.
pub const WGS84_A_METERS: f64 = 6_378_137.0;

/// WGS-84 Ellipsoid Semi-Minor Axis $b$ (polar radius) in meters.
pub const WGS84_B_METERS: f64 = 6_356_752.314_245;

/// WGS-84 Ellipsoid Flattening $f = (a - b) / a \approx 1 / 298.257223563$.
pub const WGS84_FLATTENING: f64 = 1.0 / 298.257_223_563;

/// WGS-84 First Eccentricity Squared $e^2 = 2f - f^2 \approx 0.00669437999014$.
pub const WGS84_E_SQ: f64 = 2.0 * WGS84_FLATTENING - WGS84_FLATTENING * WGS84_FLATTENING;

/// WGS-84 Second Eccentricity Squared $e^{\prime 2} = (a^2 - b^2) / b^2 \approx 0.00673949674228$.
pub const WGS84_E_PRIME_SQ: f64 = (WGS84_A_METERS * WGS84_A_METERS
    - WGS84_B_METERS * WGS84_B_METERS)
    / (WGS84_B_METERS * WGS84_B_METERS);

/// Mean Volumetric Earth Radius $R_E$ in meters.
pub const MEAN_EARTH_RADIUS_METERS: f64 = 6_371_000.0;

/// Standard Tropospheric Refraction $k$-factor ($4/3$ effective Earth radius).
pub const STANDARD_K_FACTOR: f64 = 4.0 / 3.0;

/// Geodetic Coordinates on WGS-84 Ellipsoid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeodeticCoord {
    /// Geodetic latitude in degrees ($-90^\circ \le \phi \le +90^\circ$).
    pub lat_deg: f64,
    /// Geodetic longitude in degrees ($-180^\circ \le \lambda \le +180^\circ$).
    pub lon_deg: f64,
    /// Height above WGS-84 reference ellipsoid in meters ($h$).
    pub alt_m: f64,
}

impl GeodeticCoord {
    pub const fn new(lat_deg: f64, lon_deg: f64, alt_m: f64) -> Self {
        Self {
            lat_deg,
            lon_deg,
            alt_m,
        }
    }

    /// Latitude in radians.
    #[inline]
    pub fn lat_rad(&self) -> f64 {
        self.lat_deg.to_radians()
    }

    /// Longitude in radians.
    #[inline]
    pub fn lon_rad(&self) -> f64 {
        self.lon_deg.to_radians()
    }

    /// Converts this geodetic coordinate to Cartesian ECEF.
    pub fn to_ecef(&self) -> EcefCoord {
        let phi = self.lat_rad();
        let lambda = self.lon_rad();

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_lam = lambda.sin();
        let cos_lam = lambda.cos();

        // Prime vertical radius of curvature N(phi)
        let n = WGS84_A_METERS / (1.0 - WGS84_E_SQ * sin_phi * sin_phi).sqrt();

        let x = (n + self.alt_m) * cos_phi * cos_lam;
        let y = (n + self.alt_m) * cos_phi * sin_lam;
        let z = (n * (1.0 - WGS84_E_SQ) + self.alt_m) * sin_phi;

        EcefCoord::new(x, y, z)
    }
}

/// Earth-Centered Earth-Fixed (ECEF) Cartesian Coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EcefCoord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl EcefCoord {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Converts to a standard 3D position vector.
    pub fn to_vector3d(&self) -> Vector3D {
        Vector3D::new(self.x, self.y, self.z)
    }

    /// Converts from a 3D position vector.
    pub fn from_vector3d(v: &Vector3D) -> Self {
        Self::new(v.x, v.y, v.z)
    }

    /// Straight-line slant range (Euclidean distance) in meters between two ECEF positions.
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Converts this ECEF coordinate to Geodetic (Lat/Lon/Alt) using Bowring's closed-form algorithm.
    ///
    /// Accurate to within 1 millimeter across all terrestrial and space altitudes.
    pub fn to_geodetic(&self) -> GeodeticCoord {
        let p = (self.x * self.x + self.y * self.y).sqrt();

        if p < 1e-6 {
            // Near geographic poles
            let lat = if self.z >= 0.0 { 90.0 } else { -90.0 };
            let alt = self.z.abs() - WGS84_B_METERS;
            return GeodeticCoord::new(lat, 0.0, alt);
        }

        let theta = (self.z * WGS84_A_METERS).atan2(p * WGS84_B_METERS);
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        let num = self.z + WGS84_E_PRIME_SQ * WGS84_B_METERS * sin_theta * sin_theta * sin_theta;
        let den = p - WGS84_E_SQ * WGS84_A_METERS * cos_theta * cos_theta * cos_theta;
        let phi = num.atan2(den);

        let lambda = self.y.atan2(self.x);

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let n = WGS84_A_METERS / (1.0 - WGS84_E_SQ * sin_phi * sin_phi).sqrt();

        let alt = p / cos_phi - n;

        GeodeticCoord::new(phi.to_degrees(), lambda.to_degrees(), alt)
    }

    /// Converts target ECEF position to local Topocentric East-North-Up (ENU) coordinates
    /// centered at `ref_geodetic`.
    pub fn to_enu(&self, ref_geodetic: &GeodeticCoord) -> EnuCoord {
        let ref_ecef = ref_geodetic.to_ecef();
        let dx = self.x - ref_ecef.x;
        let dy = self.y - ref_ecef.y;
        let dz = self.z - ref_ecef.z;

        let phi = ref_geodetic.lat_rad();
        let lambda = ref_geodetic.lon_rad();

        let sin_phi = phi.sin();
        let cos_phi = phi.cos();
        let sin_lam = lambda.sin();
        let cos_lam = lambda.cos();

        let east = -sin_lam * dx + cos_lam * dy;
        let north = -sin_phi * cos_lam * dx - sin_phi * sin_lam * dy + cos_phi * dz;
        let up = cos_phi * cos_lam * dx + cos_phi * sin_lam * dy + sin_phi * dz;

        EnuCoord::new(east, north, up)
    }
}

/// East-North-Up (ENU) Local Tangent Plane Coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnuCoord {
    pub east_m: f64,
    pub north_m: f64,
    pub up_m: f64,
}

impl EnuCoord {
    pub const fn new(east_m: f64, north_m: f64, up_m: f64) -> Self {
        Self {
            east_m,
            north_m,
            up_m,
        }
    }

    /// Horizontal ground distance in meters.
    pub fn ground_range(&self) -> f64 {
        (self.east_m * self.east_m + self.north_m * self.north_m).sqrt()
    }

    /// 3D slant range in meters.
    pub fn slant_range(&self) -> f64 {
        (self.east_m * self.east_m + self.north_m * self.north_m + self.up_m * self.up_m).sqrt()
    }

    /// Azimuth angle in degrees ($0^\circ \le \text{Az} < 360^\circ$ clockwise from True North).
    pub fn azimuth_deg(&self) -> f64 {
        let az = self.east_m.atan2(self.north_m).to_degrees();
        if az < 0.0 {
            az + 360.0
        } else {
            az
        }
    }

    /// Elevation angle in degrees above local horizontal ($-90^\circ \le \text{El} \le +90^\circ$).
    pub fn elevation_deg(&self) -> f64 {
        let ground = self.ground_range();
        self.up_m.atan2(ground).to_degrees()
    }

    /// Converts ENU to (Azimuth in deg, Elevation in deg, Slant Range in meters).
    pub fn to_az_el_range(&self) -> (f64, f64, f64) {
        (self.azimuth_deg(), self.elevation_deg(), self.slant_range())
    }
}

/// Radio horizon and Earth curvature line-of-sight calculations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EarthHorizon;

impl EarthHorizon {
    /// Computes the optical/radio horizon distance in meters from an antenna at height $h$ (meters)
    /// accounting for atmospheric refraction via effective Earth radius factor $k$ (default $4/3$):
    ///
    /// $$d_h = \sqrt{2 k R_E h}$$
    pub fn radio_horizon_distance(antenna_height_m: f64, k_factor: f64) -> f64 {
        let r_eff = k_factor * MEAN_EARTH_RADIUS_METERS;
        (2.0 * r_eff * antenna_height_m.max(0.0)).sqrt()
    }

    /// Computes the maximum geometric line-of-sight distance in meters between a transmitter
    /// at height $h_{tx}$ and receiver at height $h_{rx}$ over curved Earth:
    ///
    /// $$d_{max\_los} = \sqrt{2 k R_E h_{tx}} + \sqrt{2 k R_E h_{rx}}$$
    pub fn max_line_of_sight_distance(tx_height_m: f64, rx_height_m: f64, k_factor: f64) -> f64 {
        Self::radio_horizon_distance(tx_height_m, k_factor)
            + Self::radio_horizon_distance(rx_height_m, k_factor)
    }

    /// Verifies whether the direct line-of-sight ray between two ECEF coordinates clears
    /// the Earth ellipsoid (i.e. is not occluded by Earth curvature).
    ///
    /// Computes the point of closest approach along the line segment to Earth's center.
    pub fn has_ellipsoid_line_of_sight(tx: &EcefCoord, rx: &EcefCoord, k_factor: f64) -> bool {
        let p1 = tx.to_vector3d();
        let p2 = rx.to_vector3d();

        let d = p2 - p1;
        let d_norm_sq = d.norm_squared();
        if d_norm_sq < 1.0 {
            return true;
        }

        // Parameter t of point on line segment closest to origin (0,0,0)
        let t = (-p1.dot(&d) / d_norm_sq).clamp(0.0, 1.0);
        let closest_point = p1 + d.scale(t);

        // Convert closest approach point to geodetic to get exact altitude above ellipsoid
        let closest_ecef = EcefCoord::from_vector3d(&closest_point);
        let closest_geo = closest_ecef.to_geodetic();

        // Atmospheric refraction effectively lifts the ray at the midpoint of distance s
        let s = d_norm_sq.sqrt();
        let k = k_factor.max(0.1);
        let refr_lift_m = if k > 1.0 {
            (1.0 - 1.0 / k) * (s * s) / (8.0 * MEAN_EARTH_RADIUS_METERS)
        } else {
            0.0
        };

        (closest_geo.alt_m + refr_lift_m) >= 0.0
    }
}
