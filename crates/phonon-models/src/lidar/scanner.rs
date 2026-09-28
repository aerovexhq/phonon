//! Configurable LiDAR Scanning Architectures & Ray Generation
//!
//! Supports three primary industry scanning mechanisms:
//! 1. `MechanicalSpinning360`: Multi-beam rotating spindle covering 360 degrees azimuth.
//! 2. `MemsSolidState`: 2D MEMS micro-mirror raster/oscillating beam steering.
//! 3. `FlashLidar`: 2D APD/SPAD focal plane array with simultaneous scene flood illumination.

use crate::em::Vector3D;
use crate::lidar::beam::{LaserPulseConfig, LaserRay};

/// Scanning architecture type for spatial optical beam steering.
#[derive(Debug, Clone, PartialEq)]
pub enum ScanningArchitecture {
    /// 360-degree mechanical rotating LiDAR (e.g. Velodyne, Ouster, Hesai).
    MechanicalSpinning360 {
        /// Number of vertical beam channels / rings (e.g. 16, 32, 64, 128).
        channels: usize,
        /// Vertical field of view bounds (min_deg, max_deg), e.g. (-15.0, 15.0).
        fov_vertical_deg: (f64, f64),
        /// Azimuth angular step resolution in degrees (e.g. 0.2 deg => 1800 points/channel).
        azimuth_resolution_deg: f64,
        /// Spindle rotation speed in revolutions per minute (RPM, e.g. 600 RPM = 10 Hz).
        rotation_rpm: f64,
    },
    /// 2D MEMS micro-mirror solid-state beam steering (e.g. Innoviz, Luminar).
    MemsSolidState {
        /// Horizontal field of view in degrees (e.g. 120.0 deg).
        fov_horizontal_deg: f64,
        /// Vertical field of view in degrees (e.g. 25.0 deg).
        fov_vertical_deg: f64,
        /// Number of horizontal scan lines / columns.
        horizontal_samples: usize,
        /// Number of vertical scan lines / rows.
        vertical_lines: usize,
        /// Scan refresh rate in Hz (e.g. 20 Hz).
        refresh_rate_hz: f64,
    },
    /// Solid-State Flash LiDAR array (e.g. Continental, Sense Photonics).
    FlashLidar {
        /// Horizontal field of view in degrees (e.g. 60.0 deg).
        fov_horizontal_deg: f64,
        /// Vertical field of view in degrees (e.g. 40.0 deg).
        fov_vertical_deg: f64,
        /// Horizontal sensor pixel count (e.g. 128).
        resolution_h: usize,
        /// Vertical sensor pixel count (e.g. 96).
        resolution_v: usize,
        /// Frame rate in Hz (e.g. 30 Hz).
        frame_rate_hz: f64,
    },
}

/// Comprehensive LiDAR Scanner System Configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct LidarScannerConfig {
    /// Laser transmitter and receiver optical parameters.
    pub laser: LaserPulseConfig,
    /// Physical scanning mechanism.
    pub architecture: ScanningArchitecture,
    /// Minimum detection range in meters (dead zone / minimum focus).
    pub min_range_m: f64,
    /// Maximum operational range in meters.
    pub max_range_m: f64,
    /// Scanner 3D position in world Cartesian coordinates.
    pub position: Vector3D,
    /// Scanner forward pointing vector (optical boresight).
    pub forward: Vector3D,
    /// Scanner up vector.
    pub up: Vector3D,
}

impl Default for LidarScannerConfig {
    fn default() -> Self {
        Self::new_automotive_spinning_32ch(Vector3D::new(0.0, 0.0, 0.0))
    }
}

impl LidarScannerConfig {
    /// Creates a 32-channel 360-degree mechanical spinning automotive LiDAR.
    pub fn new_automotive_spinning_32ch(position: Vector3D) -> Self {
        Self {
            laser: LaserPulseConfig::new_automotive_905nm(),
            architecture: ScanningArchitecture::MechanicalSpinning360 {
                channels: 32,
                fov_vertical_deg: (-20.0, 15.0),
                azimuth_resolution_deg: 0.4, // 900 samples per channel
                rotation_rpm: 600.0,
            },
            min_range_m: 0.5,
            max_range_m: 150.0,
            position,
            forward: Vector3D::new(1.0, 0.0, 0.0),
            up: Vector3D::new(0.0, 0.0, 1.0),
        }
    }

    /// Creates a 1550 nm eye-safe MEMS solid-state long-range aerospace LiDAR.
    pub fn new_aerospace_mems_1550nm(position: Vector3D) -> Self {
        Self {
            laser: LaserPulseConfig::new_aerospace_1550nm(),
            architecture: ScanningArchitecture::MemsSolidState {
                fov_horizontal_deg: 120.0,
                fov_vertical_deg: 30.0,
                horizontal_samples: 300,
                vertical_lines: 64,
                refresh_rate_hz: 20.0,
            },
            min_range_m: 1.0,
            max_range_m: 350.0,
            position,
            forward: Vector3D::new(1.0, 0.0, 0.0),
            up: Vector3D::new(0.0, 0.0, 1.0),
        }
    }

    /// Creates a high-speed Flash LiDAR array for proximity navigation and landing.
    pub fn new_flash_lidar(position: Vector3D) -> Self {
        Self {
            laser: LaserPulseConfig::new_aerospace_1550nm(),
            architecture: ScanningArchitecture::FlashLidar {
                fov_horizontal_deg: 60.0,
                fov_vertical_deg: 45.0,
                resolution_h: 64,
                resolution_v: 48,
                frame_rate_hz: 30.0,
            },
            min_range_m: 0.2,
            max_range_m: 80.0,
            position,
            forward: Vector3D::new(1.0, 0.0, 0.0),
            up: Vector3D::new(0.0, 0.0, 1.0),
        }
    }

    /// Total number of scan rays produced in one full scanning cycle or frame.
    pub fn total_points_per_frame(&self) -> usize {
        match &self.architecture {
            ScanningArchitecture::MechanicalSpinning360 {
                channels,
                azimuth_resolution_deg,
                ..
            } => {
                let az_samples = (360.0 / azimuth_resolution_deg.max(0.01)).round() as usize;
                channels * az_samples
            }
            ScanningArchitecture::MemsSolidState {
                horizontal_samples,
                vertical_lines,
                ..
            } => horizontal_samples * vertical_lines,
            ScanningArchitecture::FlashLidar {
                resolution_h,
                resolution_v,
                ..
            } => resolution_h * resolution_v,
        }
    }

    /// Generates the complete set of optical laser rays for one full scan sweep.
    ///
    /// Returns a vector of tuples: `(channel_or_row, sample_or_col, LaserRay)`.
    pub fn generate_scan_rays(&self) -> Vec<(usize, usize, LaserRay)> {
        let fwd = self.forward.normalize();
        let up = self.up.normalize();
        let right = fwd.cross(&up).normalize();
        let true_up = right.cross(&fwd).normalize();

        match &self.architecture {
            ScanningArchitecture::MechanicalSpinning360 {
                channels,
                fov_vertical_deg,
                azimuth_resolution_deg,
                ..
            } => {
                let n_ch = *channels;
                let az_step = azimuth_resolution_deg.max(0.01);
                let n_az = (360.0 / az_step).round() as usize;
                let mut rays = Vec::with_capacity(n_ch * n_az);

                let (v_min, v_max) = *fov_vertical_deg;
                let v_range = v_max - v_min;

                for ch in 0..n_ch {
                    let el_deg = if n_ch > 1 {
                        v_min + (ch as f64 / (n_ch - 1) as f64) * v_range
                    } else {
                        (v_min + v_max) / 2.0
                    };
                    let el_rad = el_deg.to_radians();
                    let cos_el = el_rad.cos();
                    let sin_el = el_rad.sin();

                    for az_idx in 0..n_az {
                        let az_deg = (az_idx as f64) * az_step;
                        let az_rad = az_deg.to_radians();

                        // Direction in scanner local coordinates (X fwd, Y left, Z up):
                        let dir_local =
                            Vector3D::new(cos_el * az_rad.cos(), cos_el * az_rad.sin(), sin_el);

                        // Transform to world frame:
                        let dir_world =
                            fwd * dir_local.x + right * (-dir_local.y) + true_up * dir_local.z;
                        let ray = LaserRay::new(self.position, dir_world);
                        rays.push((ch, az_idx, ray));
                    }
                }
                rays
            }
            ScanningArchitecture::MemsSolidState {
                fov_horizontal_deg,
                fov_vertical_deg,
                horizontal_samples,
                vertical_lines,
                ..
            } => {
                let n_h = *horizontal_samples;
                let n_v = *vertical_lines;
                let mut rays = Vec::with_capacity(n_h * n_v);

                let h_half = (fov_horizontal_deg / 2.0).to_radians();
                let v_half = (fov_vertical_deg / 2.0).to_radians();

                for row in 0..n_v {
                    let v_frac = if n_v > 1 {
                        (row as f64 / (n_v - 1) as f64) * 2.0 - 1.0
                    } else {
                        0.0
                    };
                    let el_rad = v_frac * v_half;

                    for col in 0..n_h {
                        let h_frac = if n_h > 1 {
                            (col as f64 / (n_h - 1) as f64) * 2.0 - 1.0
                        } else {
                            0.0
                        };
                        let az_rad = h_frac * h_half;

                        let dir_local = Vector3D::new(
                            el_rad.cos() * az_rad.cos(),
                            el_rad.cos() * az_rad.sin(),
                            el_rad.sin(),
                        );
                        let dir_world =
                            fwd * dir_local.x + right * dir_local.y + true_up * dir_local.z;
                        rays.push((row, col, LaserRay::new(self.position, dir_world)));
                    }
                }
                rays
            }
            ScanningArchitecture::FlashLidar {
                fov_horizontal_deg,
                fov_vertical_deg,
                resolution_h,
                resolution_v,
                ..
            } => {
                let n_h = *resolution_h;
                let n_v = *resolution_v;
                let mut rays = Vec::with_capacity(n_h * n_v);

                let h_half = (fov_horizontal_deg / 2.0).to_radians().tan();
                let v_half = (fov_vertical_deg / 2.0).to_radians().tan();

                for row in 0..n_v {
                    let v_norm = if n_v > 1 {
                        1.0 - (row as f64 / (n_v - 1) as f64) * 2.0
                    } else {
                        0.0
                    };
                    let y_offset = v_norm * v_half;

                    for col in 0..n_h {
                        let h_norm = if n_h > 1 {
                            (col as f64 / (n_h - 1) as f64) * 2.0 - 1.0
                        } else {
                            0.0
                        };
                        let x_offset = h_norm * h_half;

                        let dir_cam = Vector3D::new(1.0, x_offset, y_offset).normalize();
                        let dir_world = fwd * dir_cam.x + right * dir_cam.y + true_up * dir_cam.z;
                        rays.push((row, col, LaserRay::new(self.position, dir_world)));
                    }
                }
                rays
            }
        }
    }
}
