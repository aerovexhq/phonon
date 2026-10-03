#![deny(unsafe_code)]

//! Dynamic Floorplan Finite Difference Heat Diffusion Mesh Generator.
//!
//! Provides spatial discretization of 2D semiconductor dies, component footprint mapping,
//! ADI (Alternating Direction Implicit) transient heat conduction solving,
//! peak temperature telemetry, and marching squares isothermal contour extraction.

use crate::grid2d::silicon_thermal_conductivity;
use std::collections::HashMap;

/// Floorplan component descriptor representing a physical heat source on the die.
#[derive(Debug, Clone, PartialEq)]
pub struct FloorplanComponent {
    /// Component identifier matching circuit graph name (e.g. "D1", "R1", "M1").
    pub name: String,
    /// X coordinate of lower-left corner in meters.
    pub x: f64,
    /// Y coordinate of lower-left corner in meters.
    pub y: f64,
    /// Physical footprint width in meters.
    pub width: f64,
    /// Physical footprint height in meters.
    pub height: f64,
    /// Die layer index (0 = surface die).
    pub layer: usize,
    /// Initial component temperature in Kelvin.
    pub initial_temperature: f64,
}

impl FloorplanComponent {
    /// Creates a new floorplan component with default surface layer and 300 K temperature.
    pub fn new(name: impl Into<String>, x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            name: name.into(),
            x,
            y,
            width,
            height,
            layer: 0,
            initial_temperature: 300.0,
        }
    }

    /// Sets the initial temperature in Kelvin.
    pub fn with_initial_temperature(mut self, temp_k: f64) -> Self {
        self.initial_temperature = temp_k;
        self
    }

    /// Sets the die layer index.
    pub fn with_layer(mut self, layer: usize) -> Self {
        self.layer = layer;
        self
    }

    /// Returns the physical center coordinate (x, y) in meters.
    #[inline]
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }

    /// Footprint surface area in square meters.
    #[inline]
    pub fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// Physical semiconductor die dimensions and thermal transport properties.
#[derive(Debug, Clone, PartialEq)]
pub struct DieProperties {
    /// Die width in meters (X dimension).
    pub width: f64,
    /// Die height in meters (Y dimension).
    pub height: f64,
    /// Die thickness in meters (Z dimension).
    pub thickness: f64,
    /// Substrate baseline thermal conductivity k in W / (m * K).
    pub k: f64,
    /// Flag enabling temperature-dependent Silicon thermal conductivity model k(T).
    pub use_silicon_k: bool,
    /// Volumetric heat capacity (rho * cp) in J / (m^3 * K).
    pub volumetric_heat_capacity: f64,
    /// Ambient environment temperature in Kelvin.
    pub ambient_temperature: f64,
    /// Convective heat transfer coefficient h_conv in W / (m^2 * K).
    pub h_conv: f64,
}

impl Default for DieProperties {
    fn default() -> Self {
        Self {
            width: 0.005,                     // 5.0 mm
            height: 0.005,                    // 5.0 mm
            thickness: 0.0003,                // 300 um
            k: 148.0,                         // Silicon at 300 K
            use_silicon_k: true,
            volumetric_heat_capacity: 1.66e6, // Silicon rho * cp
            ambient_temperature: 300.0,
            h_conv: 10.0,                     // Natural air convection
        }
    }
}

impl DieProperties {
    /// Creates a new `DieProperties` with specified physical dimensions.
    pub fn new(width: f64, height: f64, thickness: f64) -> Self {
        Self {
            width,
            height,
            thickness,
            ..Default::default()
        }
    }

    /// Evaluates effective thermal conductivity k at temperature T (Kelvin).
    #[inline]
    pub fn thermal_conductivity(&self, temp_k: f64) -> f64 {
        if self.use_silicon_k {
            silicon_thermal_conductivity(temp_k)
        } else {
            self.k
        }
    }
}

/// Extracted isothermal contour line segment collection for scientific thermography.
#[derive(Debug, Clone, PartialEq)]
pub struct IsothermalContour {
    /// Target isotherm temperature in Kelvin.
    pub target_temperature: f64,
    /// Line segments [(x1, y1), (x2, y2)] in meters.
    pub segments: Vec<((f64, f64), (f64, f64))>,
}

impl IsothermalContour {
    /// Creates an empty contour container for the specified target temperature.
    pub fn new(target_temperature: f64) -> Self {
        Self {
            target_temperature,
            segments: Vec::new(),
        }
    }
}

/// Dynamic Floorplan finite difference thermal mesh.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicFloorplanMesh {
    /// Die material and geometrical properties.
    pub die: DieProperties,
    /// Number of grid divisions in the X direction.
    pub nx: usize,
    /// Number of grid divisions in the Y direction.
    pub ny: usize,
    /// Components placed on the die floorplan.
    pub components: Vec<FloorplanComponent>,
    /// Temperature array of size nx * ny in Kelvin.
    pub temperatures: Vec<f64>,
    /// Volumetric heat source array of size nx * ny in Watts / m^3.
    pub heat_sources: Vec<f64>,
}

impl DynamicFloorplanMesh {
    /// Creates a new finite difference floorplan mesh.
    pub fn new(
        die: DieProperties,
        nx: usize,
        ny: usize,
        components: Vec<FloorplanComponent>,
    ) -> Self {
        assert!(nx >= 2 && ny >= 2, "Grid divisions must be at least 2x2");
        assert!(
            die.width > 0.0 && die.height > 0.0 && die.thickness > 0.0,
            "Die dimensions must be positive"
        );

        let mut temperatures = vec![die.ambient_temperature; nx * ny];
        let heat_sources = vec![0.0; nx * ny];

        let dx = die.width / (nx as f64);
        let dy = die.height / (ny as f64);

        // Initialize component footprint cells with their initial temperatures
        for comp in &components {
            let x0 = comp.x.clamp(0.0, die.width);
            let x1 = (comp.x + comp.width).clamp(0.0, die.width);
            let y0 = comp.y.clamp(0.0, die.height);
            let y1 = (comp.y + comp.height).clamp(0.0, die.height);

            let i_start = ((x0 / dx).floor() as usize).min(nx - 1);
            let i_end = ((x1 / dx).ceil() as usize).min(nx);
            let j_start = ((y0 / dy).floor() as usize).min(ny - 1);
            let j_end = ((y1 / dy).ceil() as usize).min(ny);

            for j in j_start..j_end {
                for i in i_start..i_end {
                    let idx = j * nx + i;
                    temperatures[idx] = comp.initial_temperature;
                }
            }
        }

        Self {
            die,
            nx,
            ny,
            components,
            temperatures,
            heat_sources,
        }
    }

    /// Cell width Delta x in meters.
    #[inline]
    pub fn dx(&self) -> f64 {
        self.die.width / (self.nx as f64)
    }

    /// Cell height Delta y in meters.
    #[inline]
    pub fn dy(&self) -> f64 {
        self.die.height / (self.ny as f64)
    }

    /// Linear index from 2D coordinates (i, j).
    #[inline]
    pub fn index(&self, i: usize, j: usize) -> usize {
        j * self.nx + i
    }

    /// Returns physical center coordinate (x, y) of cell (i, j) in meters.
    #[inline]
    pub fn cell_center(&self, i: usize, j: usize) -> (f64, f64) {
        let dx = self.dx();
        let dy = self.dy();
        ((i as f64 + 0.5) * dx, (j as f64 + 0.5) * dy)
    }

    /// Returns cell temperature at (i, j) in Kelvin.
    #[inline]
    pub fn get_temperature(&self, i: usize, j: usize) -> f64 {
        self.temperatures[self.index(i, j)]
    }

    /// Adds a component to the floorplan.
    pub fn add_component(&mut self, comp: FloorplanComponent) {
        self.components.push(comp);
    }

    /// Maps component electrical power dissipations (Watts) into grid cell volumetric heat sources (Watts / m^3).
    ///
    /// Distributes power proportionally to overlapping cell footprint areas, conserving total power.
    pub fn map_power_distribution(&mut self, component_powers: &HashMap<String, f64>) {
        self.heat_sources.fill(0.0);
        let dx = self.dx();
        let dy = self.dy();
        let dz = self.die.thickness;
        let cell_vol = dx * dy * dz;

        for comp in &self.components {
            let p_total = component_powers.get(&comp.name).copied().unwrap_or(0.0);
            if p_total <= 0.0 {
                continue;
            }

            let comp_area = (comp.width * comp.height).max(1e-18);

            // Bounding box in physical coordinates clamped to die
            let cx0 = comp.x.clamp(0.0, self.die.width);
            let cx1 = (comp.x + comp.width).clamp(0.0, self.die.width);
            let cy0 = comp.y.clamp(0.0, self.die.height);
            let cy1 = (comp.y + comp.height).clamp(0.0, self.die.height);

            if cx1 <= cx0 || cy1 <= cy0 {
                // If component footprint is outside die or zero, inject at center cell
                let (cen_x, cen_y) = comp.center();
                let i = ((cen_x / self.die.width) * (self.nx as f64))
                    .floor()
                    .clamp(0.0, (self.nx - 1) as f64) as usize;
                let j = ((cen_y / self.die.height) * (self.ny as f64))
                    .floor()
                    .clamp(0.0, (self.ny - 1) as f64) as usize;
                let idx = self.index(i, j);
                self.heat_sources[idx] += p_total / cell_vol;
                continue;
            }

            let i_min = ((cx0 / dx).floor() as usize).min(self.nx - 1);
            let i_max = ((cx1 / dx).ceil() as usize).min(self.nx);
            let j_min = ((cy0 / dy).floor() as usize).min(self.ny - 1);
            let j_max = ((cy1 / dy).ceil() as usize).min(self.ny);

            let mut allocated_power = 0.0;

            for j in j_min..j_max {
                let cell_y0 = (j as f64) * dy;
                let cell_y1 = ((j + 1) as f64) * dy;
                let oy0 = cy0.max(cell_y0);
                let oy1 = cy1.min(cell_y1);
                let h_over = (oy1 - oy0).max(0.0);

                if h_over <= 0.0 {
                    continue;
                }

                for i in i_min..i_max {
                    let cell_x0 = (i as f64) * dx;
                    let cell_x1 = ((i + 1) as f64) * dx;
                    let ox0 = cx0.max(cell_x0);
                    let ox1 = cx1.min(cell_x1);
                    let w_over = (ox1 - ox0).max(0.0);

                    if w_over <= 0.0 {
                        continue;
                    }

                    let overlap_area = w_over * h_over;
                    let frac = overlap_area / comp_area;
                    let cell_p = p_total * frac;
                    allocated_power += cell_p;

                    let idx = self.index(i, j);
                    self.heat_sources[idx] += cell_p / cell_vol;
                }
            }

            // Conserve any remaining edge power due to discretization rounding
            if allocated_power < p_total && (p_total - allocated_power) > 1e-12 {
                let remainder = p_total - allocated_power;
                let (cen_x, cen_y) = comp.center();
                let i = ((cen_x / self.die.width) * (self.nx as f64))
                    .floor()
                    .clamp(0.0, (self.nx - 1) as f64) as usize;
                let j = ((cen_y / self.die.height) * (self.ny as f64))
                    .floor()
                    .clamp(0.0, (self.ny - 1) as f64) as usize;
                let idx = self.index(i, j);
                self.heat_sources[idx] += remainder / cell_vol;
            }
        }
    }

    /// Advances the temperature field by dt seconds using an unconditionally stable
    /// Alternating Direction Implicit (ADI) Peaceman-Rachford 2D time stepping scheme.
    ///
    /// Solves: rho * cp * dT/dt = div(k * grad(T)) + P_d - h_conv * (T - T_amb) / thickness
    pub fn step_transient(
        &mut self,
        dt: f64,
        component_powers: &HashMap<String, f64>,
    ) -> Result<(), String> {
        if dt <= 0.0 {
            return Err("Time step dt must be positive".to_string());
        }
        if !dt.is_finite() {
            return Err("Time step dt must be finite".to_string());
        }

        self.map_power_distribution(component_powers);

        let nx = self.nx;
        let ny = self.ny;
        let dx = self.dx();
        let dy = self.dy();
        let dz = self.die.thickness;
        let cell_vol = dx * dy * dz;
        let c_cell = self.die.volumetric_heat_capacity * cell_vol;
        let dt_half = dt * 0.5;

        let g_conv = self.die.h_conv * (dx * dy);
        let t_amb = self.die.ambient_temperature;

        // Compute thermal conductances between neighboring cells based on temperature-dependent k
        // Pre-compute k for all cells
        let mut k_cells = vec![0.0; nx * ny];
        for (idx, k_cell) in k_cells.iter_mut().enumerate() {
            *k_cell = self.die.thermal_conductivity(self.temperatures[idx]);
        }

        // Horizontal conductances gx: size (nx - 1) * ny between (i, j) and (i+1, j)
        let area_x = dy * dz;
        let mut gx = vec![0.0; (nx - 1) * ny];
        for j in 0..ny {
            for i in 0..nx - 1 {
                let idx_c = j * nx + i;
                let idx_r = j * nx + (i + 1);
                let k_c = k_cells[idx_c];
                let k_r = k_cells[idx_r];
                let k_eff = 2.0 * k_c * k_r / (k_c + k_r).max(1e-6);
                gx[j * (nx - 1) + i] = k_eff * area_x / dx;
            }
        }

        // Vertical conductances gy: size nx * (ny - 1) between (i, j) and (i, j+1)
        let area_y = dx * dz;
        let mut gy = vec![0.0; nx * (ny - 1)];
        for j in 0..ny - 1 {
            for i in 0..nx {
                let idx_c = j * nx + i;
                let idx_t = (j + 1) * nx + i;
                let k_c = k_cells[idx_c];
                let k_t = k_cells[idx_t];
                let k_eff = 2.0 * k_c * k_t / (k_c + k_t).max(1e-6);
                gy[j * nx + i] = k_eff * area_y / dy;
            }
        }

        // --- Half-Step 1: Implicit in X, Explicit in Y ---
        let mut t_star = vec![0.0; nx * ny];

        let mut a_x = vec![0.0; nx];
        let mut b_x = vec![0.0; nx];
        let mut c_x = vec![0.0; nx];
        let mut d_x = vec![0.0; nx];
        let mut sol_row = vec![0.0; nx];

        for j in 0..ny {
            for i in 0..nx {
                let idx = j * nx + i;
                let t_curr = self.temperatures[idx];
                let p_in = self.heat_sources[idx] * cell_vol;

                // Explicit Y conduction flux from previous temperatures
                let mut flux_y = 0.0;
                if j > 0 {
                    let g_down = gy[(j - 1) * nx + i];
                    flux_y += g_down * (self.temperatures[(j - 1) * nx + i] - t_curr);
                }
                if j + 1 < ny {
                    let g_up = gy[j * nx + i];
                    flux_y += g_up * (self.temperatures[(j + 1) * nx + i] - t_curr);
                }

                let g_w = if i > 0 { gx[j * (nx - 1) + (i - 1)] } else { 0.0 };
                let g_e = if i + 1 < nx { gx[j * (nx - 1) + i] } else { 0.0 };

                a_x[i] = -g_w;
                c_x[i] = -g_e;
                b_x[i] = c_cell / dt_half + g_w + g_e + 0.5 * g_conv;

                d_x[i] = (c_cell / dt_half) * t_curr
                    + flux_y
                    - 0.5 * g_conv * (t_curr - t_amb)
                    + 0.5 * g_conv * t_amb
                    + p_in;
            }

            solve_tridiagonal(&a_x, &b_x, &c_x, &d_x, &mut sol_row);

            for i in 0..nx {
                t_star[j * nx + i] = sol_row[i];
            }
        }

        // --- Half-Step 2: Implicit in Y, Explicit in X ---
        let mut t_next = vec![0.0; nx * ny];

        let mut a_y = vec![0.0; ny];
        let mut b_y = vec![0.0; ny];
        let mut c_y = vec![0.0; ny];
        let mut d_y = vec![0.0; ny];
        let mut sol_col = vec![0.0; ny];

        for i in 0..nx {
            for j in 0..ny {
                let idx = j * nx + i;
                let t_s = t_star[idx];
                let p_in = self.heat_sources[idx] * cell_vol;

                // Explicit X conduction flux from half-step temperatures t_star
                let mut flux_x = 0.0;
                if i > 0 {
                    let g_w = gx[j * (nx - 1) + (i - 1)];
                    flux_x += g_w * (t_star[j * nx + (i - 1)] - t_s);
                }
                if i + 1 < nx {
                    let g_e = gx[j * (nx - 1) + i];
                    flux_x += g_e * (t_star[j * nx + (i + 1)] - t_s);
                }

                let g_s = if j > 0 { gy[(j - 1) * nx + i] } else { 0.0 };
                let g_n = if j + 1 < ny { gy[j * nx + i] } else { 0.0 };

                a_y[j] = -g_s;
                c_y[j] = -g_n;
                b_y[j] = c_cell / dt_half + g_s + g_n + 0.5 * g_conv;

                d_y[j] = (c_cell / dt_half) * t_s
                    + flux_x
                    - 0.5 * g_conv * (t_s - t_amb)
                    + 0.5 * g_conv * t_amb
                    + p_in;
            }

            solve_tridiagonal(&a_y, &b_y, &c_y, &d_y, &mut sol_col);

            for j in 0..ny {
                t_next[j * nx + i] = sol_col[j];
            }
        }

        // Sanity check temperatures
        for &t in &t_next {
            if t.is_nan() || t.is_infinite() {
                return Err("Numerical anomaly: temperature became non-finite in transient ADI step".to_string());
            }
        }

        self.temperatures = t_next;
        Ok(())
    }

    /// Evaluates continuous temperature at physical coordinates (x, y) via bilinear interpolation.
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

    /// Returns average junction temperature across component footprint in Kelvin.
    pub fn get_component_temperature(&self, name: &str) -> Option<f64> {
        let comp = self.components.iter().find(|c| c.name == name)?;

        let dx = self.dx();
        let dy = self.dy();

        let cx0 = comp.x.clamp(0.0, self.die.width);
        let cx1 = (comp.x + comp.width).clamp(0.0, self.die.width);
        let cy0 = comp.y.clamp(0.0, self.die.height);
        let cy1 = (comp.y + comp.height).clamp(0.0, self.die.height);

        let (cen_x, cen_y) = comp.center();

        if cx1 <= cx0 || cy1 <= cy0 {
            return Some(self.sample_temperature(cen_x, cen_y));
        }

        let i_min = ((cx0 / dx).floor() as usize).min(self.nx - 1);
        let i_max = ((cx1 / dx).ceil() as usize).min(self.nx);
        let j_min = ((cy0 / dy).floor() as usize).min(self.ny - 1);
        let j_max = ((cy1 / dy).ceil() as usize).min(self.ny);

        let mut total_area = 0.0;
        let mut weighted_temp = 0.0;

        for j in j_min..j_max {
            let cell_y0 = (j as f64) * dy;
            let cell_y1 = ((j + 1) as f64) * dy;
            let oy0 = cy0.max(cell_y0);
            let oy1 = cy1.min(cell_y1);
            let h_over = (oy1 - oy0).max(0.0);
            if h_over <= 0.0 {
                continue;
            }

            for i in i_min..i_max {
                let cell_x0 = (i as f64) * dx;
                let cell_x1 = ((i + 1) as f64) * dx;
                let ox0 = cx0.max(cell_x0);
                let ox1 = cx1.min(cell_x1);
                let w_over = (ox1 - ox0).max(0.0);
                if w_over <= 0.0 {
                    continue;
                }

                let area = w_over * h_over;
                total_area += area;
                weighted_temp += area * self.get_temperature(i, j);
            }
        }

        if total_area > 0.0 {
            Some(weighted_temp / total_area)
        } else {
            Some(self.sample_temperature(cen_x, cen_y))
        }
    }

    /// Peak hotspot telemetry: returns (temp_k, x_m, y_m).
    pub fn peak_temperature(&self) -> (f64, f64, f64) {
        let mut max_t = f64::NEG_INFINITY;
        let mut max_i = 0;
        let mut max_j = 0;

        for j in 0..self.ny {
            for i in 0..self.nx {
                let t = self.get_temperature(i, j);
                if t > max_t {
                    max_t = t;
                    max_i = i;
                    max_j = j;
                }
            }
        }

        let (x, y) = self.cell_center(max_i, max_j);
        (max_t, x, y)
    }

    /// Minimum temperature across the die in Kelvin.
    pub fn min_temperature(&self) -> f64 {
        self.temperatures
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min)
    }

    /// Maximum temperature across the die in Kelvin.
    pub fn max_temperature(&self) -> f64 {
        self.temperatures
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Computes isothermal contour line segments using the Marching Squares algorithm.
    pub fn compute_isothermal_contours(&self, contour_temps_k: &[f64]) -> Vec<IsothermalContour> {
        let mut contours = Vec::with_capacity(contour_temps_k.len());
        let dx = self.dx();
        let dy = self.dy();

        for &t_iso in contour_temps_k {
            let mut contour = IsothermalContour::new(t_iso);

            // Iterate over quad cells between cell centers
            for j in 0..self.ny - 1 {
                for i in 0..self.nx - 1 {
                    let p0 = ((i as f64 + 0.5) * dx, (j as f64 + 0.5) * dy);
                    let p1 = (((i + 1) as f64 + 0.5) * dx, (j as f64 + 0.5) * dy);
                    let p2 = (((i + 1) as f64 + 0.5) * dx, ((j + 1) as f64 + 0.5) * dy);
                    let p3 = ((i as f64 + 0.5) * dx, ((j + 1) as f64 + 0.5) * dy);

                    let v0 = self.get_temperature(i, j);
                    let v1 = self.get_temperature(i + 1, j);
                    let v2 = self.get_temperature(i + 1, j + 1);
                    let v3 = self.get_temperature(i, j + 1);

                    let min_v = v0.min(v1).min(v2).min(v3);
                    let max_v = v0.max(v1).max(v2).max(v3);

                    if t_iso < min_v || t_iso > max_v {
                        continue;
                    }

                    let b0 = if v0 >= t_iso { 1 } else { 0 };
                    let b1 = if v1 >= t_iso { 2 } else { 0 };
                    let b2 = if v2 >= t_iso { 4 } else { 0 };
                    let b3 = if v3 >= t_iso { 8 } else { 0 };

                    let case_idx = b0 | b1 | b2 | b3;
                    if case_idx == 0 || case_idx == 15 {
                        continue;
                    }

                    let interp = |pa: (f64, f64), pb: (f64, f64), va: f64, vb: f64| -> (f64, f64) {
                        let span = vb - va;
                        let t = if span.abs() < 1e-12 {
                            0.5
                        } else {
                            ((t_iso - va) / span).clamp(0.0, 1.0)
                        };
                        (pa.0 + t * (pb.0 - pa.0), pa.1 + t * (pb.1 - pa.1))
                    };

                    let e0 = interp(p0, p1, v0, v1); // bottom edge
                    let e1 = interp(p1, p2, v1, v2); // right edge
                    let e2 = interp(p3, p2, v3, v2); // top edge
                    let e3 = interp(p0, p3, v0, v3); // left edge

                    match case_idx {
                        1 | 14 => contour.segments.push((e3, e0)),
                        2 | 13 => contour.segments.push((e0, e1)),
                        3 | 12 => contour.segments.push((e3, e1)),
                        4 | 11 => contour.segments.push((e1, e2)),
                        5 => {
                            let avg = 0.25 * (v0 + v1 + v2 + v3);
                            if avg >= t_iso {
                                contour.segments.push((e3, e2));
                                contour.segments.push((e0, e1));
                            } else {
                                contour.segments.push((e3, e0));
                                contour.segments.push((e1, e2));
                            }
                        }
                        6 | 9 => contour.segments.push((e0, e2)),
                        7 | 8 => contour.segments.push((e3, e2)),
                        10 => {
                            let avg = 0.25 * (v0 + v1 + v2 + v3);
                            if avg >= t_iso {
                                contour.segments.push((e0, e3));
                                contour.segments.push((e2, e1));
                            } else {
                                contour.segments.push((e0, e1));
                                contour.segments.push((e2, e3));
                            }
                        }
                        _ => {}
                    }
                }
            }

            contours.push(contour);
        }

        contours
    }
}

/// Solves tridiagonal linear equation system: a[i]*x[i-1] + b[i]*x[i] + c[i]*x[i+1] = d[i]
/// using Thomas algorithm in O(N) operations.
fn solve_tridiagonal(a: &[f64], b: &[f64], c: &[f64], d: &[f64], x: &mut [f64]) {
    let n = b.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        x[0] = d[0] / b[0];
        return;
    }

    let mut cp = vec![0.0; n];
    let mut dp = vec![0.0; n];

    cp[0] = c[0] / b[0];
    dp[0] = d[0] / b[0];

    for i in 1..n {
        let m = b[i] - a[i] * cp[i - 1];
        if m.abs() < 1e-30 {
            cp[i] = 0.0;
            dp[i] = 0.0;
        } else {
            cp[i] = c[i] / m;
            dp[i] = (d[i] - a[i] * dp[i - 1]) / m;
        }
    }

    x[n - 1] = dp[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = dp[i] - cp[i] * x[i + 1];
    }
}
