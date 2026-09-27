//! 3D Finite-Difference Method (FDM) grid for volumetric thermal conduction and vertical heat spreading.

use crate::boundary::ThermalBoundary;
use crate::grid2d::silicon_thermal_conductivity;

/// Structured 3D Finite-Difference thermal grid modeling volumetric heat conduction through silicon and packaging.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalGrid3D {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub temperatures: Vec<f64>,
    pub heat_sources: Vec<f64>,
    pub boundary_z_bottom: ThermalBoundary,
    pub boundary_z_top: ThermalBoundary,
}

impl ThermalGrid3D {
    /// Creates a new 3D thermal grid initialized at $T_{init}$.
    pub fn new(
        nx: usize,
        ny: usize,
        nz: usize,
        width: f64,
        height: f64,
        depth: f64,
        init_temp_k: f64,
    ) -> Self {
        assert!(nx >= 2 && ny >= 2 && nz >= 2);
        assert!(width > 0.0 && height > 0.0 && depth > 0.0);

        Self {
            nx,
            ny,
            nz,
            width,
            height,
            depth,
            temperatures: vec![init_temp_k; nx * ny * nz],
            heat_sources: vec![0.0; nx * ny * nz],
            boundary_z_bottom: ThermalBoundary::Dirichlet {
                temperature_kelvin: init_temp_k,
            },
            boundary_z_top: ThermalBoundary::Robin {
                ambient_kelvin: init_temp_k,
                h_conv: 15.0,
                emissivity: 0.8,
            },
        }
    }

    #[inline]
    pub fn dx(&self) -> f64 {
        self.width / (self.nx as f64)
    }

    #[inline]
    pub fn dy(&self) -> f64 {
        self.height / (self.ny as f64)
    }

    #[inline]
    pub fn dz(&self) -> f64 {
        self.depth / (self.nz as f64)
    }

    #[inline]
    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        (k * self.ny + j) * self.nx + i
    }

    #[inline]
    pub fn get_temperature(&self, i: usize, j: usize, k: usize) -> f64 {
        self.temperatures[self.index(i, j, k)]
    }

    #[inline]
    pub fn set_temperature(&mut self, i: usize, j: usize, k: usize, temp_k: f64) {
        let idx = self.index(i, j, k);
        self.temperatures[idx] = temp_k;
    }

    /// Injects power in Watts at 3D physical coordinates $(x, y, z)$.
    pub fn inject_heat_power(&mut self, x: f64, y: f64, z: f64, power_watts: f64) {
        let i = ((x / self.width) * (self.nx as f64))
            .floor()
            .clamp(0.0, (self.nx - 1) as f64) as usize;
        let j = ((y / self.height) * (self.ny as f64))
            .floor()
            .clamp(0.0, (self.ny - 1) as f64) as usize;
        let k = ((z / self.depth) * (self.nz as f64))
            .floor()
            .clamp(0.0, (self.nz - 1) as f64) as usize;

        let idx = self.index(i, j, k);
        self.heat_sources[idx] += power_watts;
    }

    /// Clears all heat sources.
    pub fn clear_heat_sources(&mut self) {
        self.heat_sources.fill(0.0);
    }

    /// Maximum temperature across the 3D volume.
    pub fn max_temperature(&self) -> f64 {
        self.temperatures
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Advances the 3D temperature volume by a single explicit time step $\Delta t$.
    pub fn step_transient_explicit(&mut self, dt: f64) {
        let dx = self.dx();
        let dy = self.dy();
        let dz = self.dz();
        let cell_vol = dx * dy * dz;
        let c_cell = 1.66e6 * cell_vol;

        let area_x = dy * dz;
        let area_y = dx * dz;
        let area_z = dx * dy;

        let mut next_temps = self.temperatures.clone();

        for k in 0..self.nz {
            for j in 0..self.ny {
                for i in 0..self.nx {
                    let idx = self.index(i, j, k);
                    let t_c = self.temperatures[idx];
                    let kappa_c = silicon_thermal_conductivity(t_c);

                    let mut q_net = self.heat_sources[idx];

                    // X neighbors
                    if i > 0 {
                        let t_w = self.temperatures[self.index(i - 1, j, k)];
                        let g_th = kappa_c * area_x / dx;
                        q_net += g_th * (t_w - t_c);
                    }
                    if i + 1 < self.nx {
                        let t_e = self.temperatures[self.index(i + 1, j, k)];
                        let g_th = kappa_c * area_x / dx;
                        q_net += g_th * (t_e - t_c);
                    }

                    // Y neighbors
                    if j > 0 {
                        let t_s = self.temperatures[self.index(i, j - 1, k)];
                        let g_th = kappa_c * area_y / dy;
                        q_net += g_th * (t_s - t_c);
                    }
                    if j + 1 < self.ny {
                        let t_n = self.temperatures[self.index(i, j + 1, k)];
                        let g_th = kappa_c * area_y / dy;
                        q_net += g_th * (t_n - t_c);
                    }

                    // Z neighbors
                    if k > 0 {
                        let t_b = self.temperatures[self.index(i, j, k - 1)];
                        let g_th = kappa_c * area_z / dz;
                        q_net += g_th * (t_b - t_c);
                    } else if let ThermalBoundary::Dirichlet { temperature_kelvin } =
                        self.boundary_z_bottom
                    {
                        let g_th = kappa_c * area_z / (0.5 * dz);
                        q_net += g_th * (temperature_kelvin - t_c);
                    }

                    if k + 1 < self.nz {
                        let t_t = self.temperatures[self.index(i, j, k + 1)];
                        let g_th = kappa_c * area_z / dz;
                        q_net += g_th * (t_t - t_c);
                    } else if let ThermalBoundary::Robin {
                        ambient_kelvin,
                        h_conv,
                        ..
                    } = self.boundary_z_top
                    {
                        let g_conv = h_conv * area_z;
                        q_net += g_conv * (ambient_kelvin - t_c);
                    }

                    next_temps[idx] = t_c + (dt / c_cell) * q_net;
                }
            }
        }

        self.temperatures = next_temps;
    }
}
