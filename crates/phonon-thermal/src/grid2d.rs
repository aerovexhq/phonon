//! 2D Finite-Difference Method (FDM) grid for spatial thermal conduction in silicon substrates.

use crate::boundary::ThermalBoundary;

/// Reference thermal conductivity of Silicon at 300 K in $\text{W} / (\text{m} \cdot \text{K})$.
pub const KAPPA_SI_300K: f64 = 148.0;
/// Silicon thermal conductivity temperature exponent ($\kappa(T) \propto T^{-\alpha}$).
pub const ALPHA_KAPPA_SI: f64 = 1.33;
/// Silicon volumetric heat capacity $\rho c_p$ at 300 K in $\text{J} / (\text{m}^3 \cdot \text{K})$.
pub const CV_SI_300K: f64 = 1.66e6;

/// Evaluates temperature-dependent thermal conductivity $\kappa(T)$ of Silicon in $\text{W} / (\text{m} \cdot \text{K})$.
#[inline]
pub fn silicon_thermal_conductivity(temp_k: f64) -> f64 {
    let t_ratio = (temp_k / 300.0).max(0.1);
    KAPPA_SI_300K * t_ratio.powf(-ALPHA_KAPPA_SI)
}

/// Structured 2D Finite-Difference thermal grid modeling planar heat spreading and dissipation.
#[derive(Debug, Clone, PartialEq)]
pub struct ThermalGrid2D {
    /// Number of grid divisions in the X direction.
    pub nx: usize,
    /// Number of grid divisions in the Y direction.
    pub ny: usize,
    /// Physical width of the substrate in meters (X dimension).
    pub width: f64,
    /// Physical height of the substrate in meters (Y dimension).
    pub height: f64,
    /// Substrate thickness in meters (Z dimension).
    pub thickness: f64,
    /// Temperature array of size `nx * ny` in Kelvin.
    pub temperatures: Vec<f64>,
    /// Volumetric heat source array of size `nx * ny` in Watts.
    pub heat_sources: Vec<f64>,
    /// Boundary conditions for West (x=0), East (x=W), South (y=0), and North (y=H).
    pub boundary_west: ThermalBoundary,
    pub boundary_east: ThermalBoundary,
    pub boundary_south: ThermalBoundary,
    pub boundary_north: ThermalBoundary,
}

impl ThermalGrid2D {
    /// Creates a new 2D thermal grid initialized with ambient temperature $T_{init}$.
    pub fn new(
        nx: usize,
        ny: usize,
        width: f64,
        height: f64,
        thickness: f64,
        init_temp_k: f64,
    ) -> Self {
        assert!(nx >= 2 && ny >= 2, "Grid must be at least 2x2");
        assert!(width > 0.0 && height > 0.0 && thickness > 0.0);

        Self {
            nx,
            ny,
            width,
            height,
            thickness,
            temperatures: vec![init_temp_k; nx * ny],
            heat_sources: vec![0.0; nx * ny],
            boundary_west: ThermalBoundary::default(),
            boundary_east: ThermalBoundary::default(),
            boundary_south: ThermalBoundary::Dirichlet {
                temperature_kelvin: init_temp_k,
            }, // default bottom heatsink
            boundary_north: ThermalBoundary::Robin {
                ambient_kelvin: init_temp_k,
                h_conv: 10.0,
                emissivity: 0.7,
            },
        }
    }

    /// Cell width $\Delta x = W / N_x$ in meters.
    #[inline]
    pub fn dx(&self) -> f64 {
        self.width / (self.nx as f64)
    }

    /// Cell height $\Delta y = H / N_y$ in meters.
    #[inline]
    pub fn dy(&self) -> f64 {
        self.height / (self.ny as f64)
    }

    /// Converts 2D coordinate $(i, j)$ into 1D linear array index.
    #[inline]
    pub fn index(&self, i: usize, j: usize) -> usize {
        j * self.nx + i
    }

    /// Returns the temperature at grid cell $(i, j)$ in Kelvin.
    #[inline]
    pub fn get_temperature(&self, i: usize, j: usize) -> f64 {
        self.temperatures[self.index(i, j)]
    }

    /// Sets the temperature at grid cell $(i, j)$ in Kelvin.
    #[inline]
    pub fn set_temperature(&mut self, i: usize, j: usize, temp_k: f64) {
        let idx = self.index(i, j);
        self.temperatures[idx] = temp_k;
    }

    /// Clears all external heat sources.
    pub fn clear_heat_sources(&mut self) {
        self.heat_sources.fill(0.0);
    }

    /// Injects localized heat power in Watts at physical coordinates $(x, y)$.
    pub fn inject_heat_power(&mut self, x: f64, y: f64, power_watts: f64) {
        let i = ((x / self.width) * (self.nx as f64))
            .floor()
            .clamp(0.0, (self.nx - 1) as f64) as usize;
        let j = ((y / self.height) * (self.ny as f64))
            .floor()
            .clamp(0.0, (self.ny - 1) as f64) as usize;
        let idx = self.index(i, j);
        self.heat_sources[idx] += power_watts;
    }

    /// Evaluates the temperature at continuous physical coordinates $(x, y)$
    /// using bilinear interpolation between neighboring grid centers.
    pub fn sample_temperature(&self, x: f64, y: f64) -> f64 {
        let dx = self.dx();
        let dy = self.dy();

        let gx = (x / dx - 0.5).clamp(0.0, (self.nx - 1) as f64);
        let gy = (y / dy - 0.5).clamp(0.0, (self.ny - 1) as f64);

        let i0 = (gx.floor() as usize).min(self.nx - 1);
        let j0 = (gy.floor() as usize).min(self.ny - 1);
        let i1 = (i0 + 1).min(self.nx - 1);
        let j1 = (j0 + 1).min(self.ny - 1);

        let fx = gx - (i0 as f64);
        let fy = gy - (j0 as f64);

        let t00 = self.get_temperature(i0, j0);
        let t10 = self.get_temperature(i1, j0);
        let t01 = self.get_temperature(i0, j1);
        let t11 = self.get_temperature(i1, j1);

        (1.0 - fx) * (1.0 - fy) * t00
            + fx * (1.0 - fy) * t10
            + (1.0 - fx) * fy * t01
            + fx * fy * t11
    }

    /// Maximum temperature across the entire 2D grid.
    pub fn max_temperature(&self) -> f64 {
        self.temperatures
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Advances the temperature field by a single time step $\Delta t$ using
    /// an explicit forward Euler diffusion step with physical conduction conductances.
    pub fn step_transient_explicit(&mut self, dt: f64) {
        let dx = self.dx();
        let dy = self.dy();
        let dz = self.thickness;
        let cell_vol = dx * dy * dz;
        let c_cell = CV_SI_300K * cell_vol;

        let area_x = dy * dz;
        let area_y = dx * dz;

        let mut next_temps = self.temperatures.clone();

        for j in 0..self.ny {
            for i in 0..self.nx {
                let idx = self.index(i, j);
                let t_c = self.temperatures[idx];
                let kappa_c = silicon_thermal_conductivity(t_c);

                let mut q_net_in = self.heat_sources[idx];

                // 1. Conduction in X direction: West (i-1)
                if i > 0 {
                    let t_w = self.temperatures[self.index(i - 1, j)];
                    let kappa_w = silicon_thermal_conductivity(t_w);
                    let kappa_avg = 2.0 * kappa_c * kappa_w / (kappa_c + kappa_w);
                    let g_th = kappa_avg * area_x / dx;
                    q_net_in += g_th * (t_w - t_c);
                } else {
                    // West boundary
                    match self.boundary_west {
                        ThermalBoundary::Dirichlet { temperature_kelvin } => {
                            let g_th = kappa_c * area_x / (0.5 * dx);
                            q_net_in += g_th * (temperature_kelvin - t_c);
                        }
                        ThermalBoundary::Neumann { heat_flux_w_m2 } => {
                            q_net_in += heat_flux_w_m2 * area_x;
                        }
                        ThermalBoundary::Robin { .. } => {
                            let (flux, _) = self.boundary_west.evaluate_flux(t_c);
                            q_net_in -= flux * area_x;
                        }
                    }
                }

                // 2. Conduction in X direction: East (i+1)
                if i + 1 < self.nx {
                    let t_e = self.temperatures[self.index(i + 1, j)];
                    let kappa_e = silicon_thermal_conductivity(t_e);
                    let kappa_avg = 2.0 * kappa_c * kappa_e / (kappa_c + kappa_e);
                    let g_th = kappa_avg * area_x / dx;
                    q_net_in += g_th * (t_e - t_c);
                } else {
                    // East boundary
                    match self.boundary_east {
                        ThermalBoundary::Dirichlet { temperature_kelvin } => {
                            let g_th = kappa_c * area_x / (0.5 * dx);
                            q_net_in += g_th * (temperature_kelvin - t_c);
                        }
                        ThermalBoundary::Neumann { heat_flux_w_m2 } => {
                            q_net_in += heat_flux_w_m2 * area_x;
                        }
                        ThermalBoundary::Robin { .. } => {
                            let (flux, _) = self.boundary_east.evaluate_flux(t_c);
                            q_net_in -= flux * area_x;
                        }
                    }
                }

                // 3. Conduction in Y direction: South (j-1)
                if j > 0 {
                    let t_s = self.temperatures[self.index(i, j - 1)];
                    let kappa_s = silicon_thermal_conductivity(t_s);
                    let kappa_avg = 2.0 * kappa_c * kappa_s / (kappa_c + kappa_s);
                    let g_th = kappa_avg * area_y / dy;
                    q_net_in += g_th * (t_s - t_c);
                } else {
                    // South boundary
                    match self.boundary_south {
                        ThermalBoundary::Dirichlet { temperature_kelvin } => {
                            let g_th = kappa_c * area_y / (0.5 * dy);
                            q_net_in += g_th * (temperature_kelvin - t_c);
                        }
                        ThermalBoundary::Neumann { heat_flux_w_m2 } => {
                            q_net_in += heat_flux_w_m2 * area_y;
                        }
                        ThermalBoundary::Robin { .. } => {
                            let (flux, _) = self.boundary_south.evaluate_flux(t_c);
                            q_net_in -= flux * area_y;
                        }
                    }
                }

                // 4. Conduction in Y direction: North (j+1)
                if j + 1 < self.ny {
                    let t_n = self.temperatures[self.index(i, j + 1)];
                    let kappa_n = silicon_thermal_conductivity(t_n);
                    let kappa_avg = 2.0 * kappa_c * kappa_n / (kappa_c + kappa_n);
                    let g_th = kappa_avg * area_y / dy;
                    q_net_in += g_th * (t_n - t_c);
                } else {
                    // North boundary
                    match self.boundary_north {
                        ThermalBoundary::Dirichlet { temperature_kelvin } => {
                            let g_th = kappa_c * area_y / (0.5 * dy);
                            q_net_in += g_th * (temperature_kelvin - t_c);
                        }
                        ThermalBoundary::Neumann { heat_flux_w_m2 } => {
                            q_net_in += heat_flux_w_m2 * area_y;
                        }
                        ThermalBoundary::Robin { .. } => {
                            let (flux, _) = self.boundary_north.evaluate_flux(t_c);
                            q_net_in -= flux * area_y;
                        }
                    }
                }

                next_temps[idx] = t_c + (dt / c_cell) * q_net_in;
            }
        }

        self.temperatures = next_temps;
    }

    /// Relaxes the grid to its steady-state solution using successive iterations
    /// until maximum temperature change $\Delta T_{max} < \text{tolerance}$.
    pub fn solve_steady_state(
        &mut self,
        tolerance: f64,
        max_iters: usize,
    ) -> Result<usize, String> {
        let dx = self.dx();
        let dy = self.dy();
        let dz = self.thickness;
        let area_x = dy * dz;
        let area_y = dx * dz;

        let omega = 1.3;

        for iter in 0..max_iters {
            let mut max_diff = 0.0f64;

            for j in 0..self.ny {
                for i in 0..self.nx {
                    let idx = self.index(i, j);
                    let t_c = self.temperatures[idx];
                    let kappa_c = silicon_thermal_conductivity(t_c);

                    let mut g_sum = 0.0;
                    let mut rhs = self.heat_sources[idx];

                    // West
                    if i > 0 {
                        let t_w = self.temperatures[self.index(i - 1, j)];
                        let g_th = kappa_c * area_x / dx;
                        g_sum += g_th;
                        rhs += g_th * t_w;
                    } else if let ThermalBoundary::Dirichlet { temperature_kelvin } =
                        self.boundary_west
                    {
                        let g_th = kappa_c * area_x / (0.5 * dx);
                        g_sum += g_th;
                        rhs += g_th * temperature_kelvin;
                    }

                    // East
                    if i + 1 < self.nx {
                        let t_e = self.temperatures[self.index(i + 1, j)];
                        let g_th = kappa_c * area_x / dx;
                        g_sum += g_th;
                        rhs += g_th * t_e;
                    } else if let ThermalBoundary::Dirichlet { temperature_kelvin } =
                        self.boundary_east
                    {
                        let g_th = kappa_c * area_x / (0.5 * dx);
                        g_sum += g_th;
                        rhs += g_th * temperature_kelvin;
                    }

                    // South
                    if j > 0 {
                        let t_s = self.temperatures[self.index(i, j - 1)];
                        let g_th = kappa_c * area_y / dy;
                        g_sum += g_th;
                        rhs += g_th * t_s;
                    } else if let ThermalBoundary::Dirichlet { temperature_kelvin } =
                        self.boundary_south
                    {
                        let g_th = kappa_c * area_y / (0.5 * dy);
                        g_sum += g_th;
                        rhs += g_th * temperature_kelvin;
                    }

                    // North
                    if j + 1 < self.ny {
                        let t_n = self.temperatures[self.index(i, j + 1)];
                        let g_th = kappa_c * area_y / dy;
                        g_sum += g_th;
                        rhs += g_th * t_n;
                    } else {
                        match self.boundary_north {
                            ThermalBoundary::Dirichlet { temperature_kelvin } => {
                                let g_th = kappa_c * area_y / (0.5 * dy);
                                g_sum += g_th;
                                rhs += g_th * temperature_kelvin;
                            }
                            ThermalBoundary::Robin {
                                ambient_kelvin,
                                h_conv,
                                ..
                            } => {
                                let g_conv = h_conv * area_y;
                                g_sum += g_conv;
                                rhs += g_conv * ambient_kelvin;
                            }
                            _ => {}
                        }
                    }

                    if g_sum > 0.0 {
                        let t_target = rhs / g_sum;
                        let t_new = (1.0 - omega) * t_c + omega * t_target;
                        let diff = (t_new - t_c).abs();
                        if diff > max_diff {
                            max_diff = diff;
                        }
                        self.temperatures[idx] = t_new;
                    }
                }
            }

            if max_diff < tolerance {
                return Ok(iter + 1);
            }
        }

        Err(format!("Steady-state thermal relaxation failed to reach tolerance after {max_iters} iterations"))
    }
}
