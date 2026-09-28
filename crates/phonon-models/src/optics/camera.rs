//! Multi-Tier Optical Camera Models & Non-Linear Lens Distortion
//!
//! Provides geometric optical projection, Brown-Conrady non-linear lens distortion
//! (radial $k_1, k_2, k_3$ and tangential $p_1, p_2$), pinhole ray generation,
//! un-distortion inversion, and optical Circle of Confusion (CoC) depth-of-field blur.

use super::cmos_aps::CmosPixelConfig;
use crate::em::Vector3D;

/// Pinhole Camera Intrinsics with Brown-Conrady Non-Linear Lens Distortion.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraIntrinsics {
    /// Sensor width in pixels.
    pub width: u32,
    /// Sensor height in pixels.
    pub height: u32,
    /// Horizontal focal length in pixels: $f_x$.
    pub fx: f64,
    /// Vertical focal length in pixels: $f_y$.
    pub fy: f64,
    /// Principal point horizontal coordinate in pixels: $c_x$.
    pub cx: f64,
    /// Principal point vertical coordinate in pixels: $c_y$.
    pub cy: f64,
    pub k1: f64,
    pub k2: f64,
    pub k3: f64,
    pub p1: f64,
    pub p2: f64,
    /// Physical pixel pitch in meters.
    pub pixel_pitch_m: f64,
}

impl CameraIntrinsics {
    /// Constructs camera intrinsics from physical lens and sensor dimensions.
    pub fn new(
        width: u32,
        height: u32,
        focal_length_mm: f64,
        sensor_width_mm: f64,
        sensor_height_mm: f64,
    ) -> Self {
        let fx = (focal_length_mm / sensor_width_mm) * (width as f64);
        let fy = (focal_length_mm / sensor_height_mm) * (height as f64);
        let cx = (width as f64) / 2.0;
        let cy = (height as f64) / 2.0;
        let pixel_pitch_m = (sensor_width_mm * 1e-3) / (width as f64);

        Self {
            width,
            height,
            fx,
            fy,
            cx,
            cy,
            k1: 0.0,
            k2: 0.0,
            k3: 0.0,
            p1: 0.0,
            p2: 0.0,
            pixel_pitch_m,
        }
    }

    /// Sets Brown-Conrady radial ($k_1, k_2, k_3$) and tangential ($p_1, p_2$) coefficients.
    pub fn with_distortion(mut self, k1: f64, k2: f64, k3: f64, p1: f64, p2: f64) -> Self {
        self.k1 = k1;
        self.k2 = k2;
        self.k3 = k3;
        self.p1 = p1;
        self.p2 = p2;
        self
    }

    /// Horizontal Field of View in radians: $FOV_x = 2 \arctan(W / (2 f_x))$.
    pub fn horizontal_fov_rad(&self) -> f64 {
        2.0 * ((self.width as f64) / (2.0 * self.fx)).atan()
    }

    /// Vertical Field of View in radians: $FOV_y = 2 \arctan(H / (2 f_y))$.
    pub fn vertical_fov_rad(&self) -> f64 {
        2.0 * ((self.height as f64) / (2.0 * self.fy)).atan()
    }

    /// Projects a 3D point in camera coordinates $(X, Y, Z)$ into distorted pixel coordinates $(u, v)$.
    ///
    /// Returns `Some((u, v))` if in front of the lens ($Z > 0.01\,\text{m}$), or `None` if behind camera.
    pub fn project_to_pixel(&self, p_cam: Vector3D) -> Option<(f64, f64)> {
        if p_cam.z <= 0.01 {
            return None;
        }

        // Normalized image plane coordinates:
        let xn = p_cam.x / p_cam.z;
        let yn = p_cam.y / p_cam.z;

        let r2 = xn * xn + yn * yn;
        let r4 = r2 * r2;
        let r6 = r4 * r2;

        // Radial distortion factor:
        let radial = 1.0 + self.k1 * r2 + self.k2 * r4 + self.k3 * r6;

        // Tangential distortion:
        let dx_tang = 2.0 * self.p1 * xn * yn + self.p2 * (r2 + 2.0 * xn * xn);
        let dy_tang = self.p1 * (r2 + 2.0 * yn * yn) + 2.0 * self.p2 * xn * yn;

        let xd = xn * radial + dx_tang;
        let yd = yn * radial + dy_tang;

        // Pixel coordinates:
        let u = self.fx * xd + self.cx;
        let v = self.fy * yd + self.cy;

        Some((u, v))
    }

    /// Un-projects a distorted pixel $(u, v)$ to an ideal normalized 3D ray direction in camera frame.
    ///
    /// Uses 5 iterations of fixed-point contraction mapping to invert the non-linear distortion.
    pub fn unproject_to_ray(&self, u: f64, v: f64) -> Vector3D {
        let xd = (u - self.cx) / self.fx;
        let yd = (v - self.cy) / self.fy;

        let mut xn = xd;
        let mut yn = yd;

        // Contraction mapping inversion:
        for _ in 0..5 {
            let r2 = xn * xn + yn * yn;
            let r4 = r2 * r2;
            let r6 = r4 * r2;
            let radial = 1.0 + self.k1 * r2 + self.k2 * r4 + self.k3 * r6;
            let dx_tang = 2.0 * self.p1 * xn * yn + self.p2 * (r2 + 2.0 * xn * xn);
            let dy_tang = self.p1 * (r2 + 2.0 * yn * yn) + 2.0 * self.p2 * xn * yn;

            if radial > 1e-6 {
                xn = (xd - dx_tang) / radial;
                yn = (yd - dy_tang) / radial;
            }
        }

        Vector3D::new(xn, yn, 1.0).normalize()
    }

    /// Evaluates Circle of Confusion ($CoC$) in meters for an object at depth $Z$
    /// with aperture f-number $N_f$, lens focal length $f$, and focus distance $d_0$:
    ///
    /// $$CoC = \frac{A \cdot |Z - d_0|}{Z} \cdot \frac{f}{d_0 - f}$$
    pub fn circle_of_confusion_m(
        &self,
        depth_z: f64,
        focus_dist_m: f64,
        f_number: f64,
        focal_len_m: f64,
    ) -> f64 {
        if depth_z <= 0.01 || focus_dist_m <= focal_len_m {
            return 0.0;
        }

        let aperture_d = focal_len_m / f_number.max(0.5);
        let num = aperture_d * (depth_z - focus_dist_m).abs() * focal_len_m;
        let denom = depth_z * (focus_dist_m - focal_len_m);
        if denom > 1e-9 {
            num / denom
        } else {
            0.0
        }
    }
}

/// Optical Camera System combining optics, CMOS pixel array, and kinematic pose.
#[derive(Debug, Clone, PartialEq)]
pub struct OpticalCamera {
    /// Lens intrinsics and resolution.
    pub intrinsics: CameraIntrinsics,
    /// 3D position in world Cartesian space.
    pub position: Vector3D,
    /// 3D forward optical axis pointing direction.
    pub forward: Vector3D,
    /// 3D up vector.
    pub up: Vector3D,
    /// Aperture f-number (e.g. 2.8).
    pub f_number: f64,
    /// Focal length in meters (e.g. 0.035 m for 35 mm lens).
    pub focal_length_m: f64,
    /// Distance of optical focus plane in meters (e.g. 5.0 m).
    pub focus_distance_m: f64,
    /// Sensor integration exposure time in seconds (e.g. 0.01667 s).
    pub exposure_time_s: f64,
    /// ISO analog amplification gain factor (e.g. 1.0 for ISO 100, 4.0 for ISO 400).
    pub iso_gain: f64,
    /// Underlying CMOS pixel sensor architecture configuration.
    pub sensor_config: CmosPixelConfig,
}

impl OpticalCamera {
    /// Creates a 1080p full-frame 35mm optical camera.
    pub fn new_standard_1080p(position: Vector3D, forward: Vector3D, up: Vector3D) -> Self {
        let intrinsics = CameraIntrinsics::new(1920, 1080, 35.0, 36.0, 20.25);
        let sensor_config = CmosPixelConfig::new_standard_industrial();

        Self {
            intrinsics,
            position,
            forward: forward.normalize(),
            up: up.normalize(),
            f_number: 2.8,
            focal_length_m: 0.035,
            focus_distance_m: 5.0,
            exposure_time_s: 0.01667, // 1/60s
            iso_gain: 1.0,
            sensor_config,
        }
    }

    /// Evaluates row readout time delay for rolling shutter mode:
    /// $\Delta t_{row} = \frac{t_{frame}}{H}$.
    pub fn rolling_shutter_row_delay_s(&self) -> f64 {
        self.exposure_time_s / (self.intrinsics.height as f64).max(1.0)
    }

    /// Transforms a 3D world coordinate $\mathbf{p}_{world}$ into the camera local reference frame.
    pub fn world_to_camera(&self, p_world: Vector3D) -> Vector3D {
        let diff = p_world - self.position;

        // Camera frame basis:
        // Z_cam = forward
        // X_cam = up x forward (right)
        // Y_cam = forward x right (down in image coordinates)
        let z_cam = self.forward;
        let x_cam = self.up.cross(&z_cam).normalize();
        let y_cam = z_cam.cross(&x_cam).normalize();

        Vector3D::new(diff.dot(&x_cam), diff.dot(&y_cam), diff.dot(&z_cam))
    }
}
