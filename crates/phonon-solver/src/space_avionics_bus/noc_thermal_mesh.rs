#![deny(unsafe_code)]

//! On-Die 2D Mesh Network-on-Chip (NoC) & Dynamic Thermal-Deflection Routing Engine.
//!
//! Models multi-core aerospace SoC / space processor on-die interconnects ($4\times 4$ to $8\times 8$ mesh),
//! credit-based wormhole virtual channel flow control, flit-level energy dissipation ($E_{\text{flit}} \approx 1.2\text{ pJ/bit}$),
//! localized silicon thermal conductivity, and compares baseline XY Dimension-Order Routing against
//! Thermal-Deflection Hotspot Avoidance routing under peak sensor streaming.

/// Network-on-Chip routing policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoCRoutingPolicy {
    /// Deterministic Dimension-Order Routing (DOR): routes along X then along Y.
    DimensionOrderXY,
    /// Dynamic Thermal-Deflection Routing: deflecting flits away from overheated routers.
    ThermalDeflection,
}

/// Individual router tile within the 2D on-die mesh.
#[derive(Debug, Clone)]
pub struct NoCRouterTile {
    pub x: usize,
    pub y: usize,
    /// Operating junction temperature in degrees Celsius.
    pub temperature_c: f64,
    /// Cumulative flits routed through this tile.
    pub flits_routed: u64,
    /// Available credit tokens for ingress traffic.
    pub available_credits: u32,
    /// Buffer queue fill level in flits.
    pub queue_fill_flits: usize,
    /// Localized power dissipation in Watts (static leakage + dynamic routing).
    pub power_dissipation_w: f64,
}

impl NoCRouterTile {
    pub fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
            temperature_c: 45.0, // Baseline nominal chip operating temp
            flits_routed: 0,
            available_credits: 16,
            queue_fill_flits: 0,
            power_dissipation_w: 0.15,
        }
    }
}

/// 2D Mesh Network-on-Chip Co-Simulator.
#[derive(Debug, Clone)]
pub struct NoCMeshSimulator {
    /// Mesh dimensions (Nx columns, Ny rows).
    pub grid_cols: usize,
    pub grid_rows: usize,
    /// Grid of router tiles.
    pub tiles: Vec<NoCRouterTile>,
    /// Active routing policy.
    pub policy: NoCRoutingPolicy,
    /// Temperature threshold in deg C above which thermal deflection is triggered.
    pub thermal_threshold_c: f64,
    /// Flit width in bits (e.g. 128 bits).
    pub flit_width_bits: usize,
    /// Clock frequency in Gigahertz (GHz) (e.g. 1.0 GHz).
    pub clock_freq_ghz: f64,
    /// Dynamic switching energy per bit in picojoules (pJ / bit).
    pub energy_per_bit_pj: f64,
}

impl Default for NoCMeshSimulator {
    fn default() -> Self {
        let cols = 4;
        let rows = 4;
        let mut tiles = Vec::with_capacity(cols * rows);
        for y in 0..rows {
            for x in 0..cols {
                tiles.push(NoCRouterTile::new(x, y));
            }
        }

        Self {
            grid_cols: cols,
            grid_rows: rows,
            tiles,
            policy: NoCRoutingPolicy::DimensionOrderXY,
            thermal_threshold_c: 85.0,
            flit_width_bits: 128,
            clock_freq_ghz: 1.2,
            energy_per_bit_pj: 1.2,
        }
    }
}

impl NoCMeshSimulator {
    /// Create a new 2D NoC mesh simulator with specified dimensions.
    pub fn new(cols: usize, rows: usize) -> Self {
        let c = cols.clamp(2, 8);
        let r = rows.clamp(2, 8);
        let mut tiles = Vec::with_capacity(c * r);
        for y in 0..r {
            for x in 0..c {
                tiles.push(NoCRouterTile::new(x, y));
            }
        }

        Self {
            grid_cols: c,
            grid_rows: r,
            tiles,
            ..Default::default()
        }
    }

    /// Tile index in linear vector.
    pub fn tile_index(&self, x: usize, y: usize) -> usize {
        y * self.grid_cols + x
    }

    /// Read router tile at (x, y).
    pub fn tile(&self, x: usize, y: usize) -> Option<&NoCRouterTile> {
        if x < self.grid_cols && y < self.grid_rows {
            Some(&self.tiles[self.tile_index(x, y)])
        } else {
            None
        }
    }

    /// Read mutable router tile at (x, y).
    pub fn tile_mut(&mut self, x: usize, y: usize) -> Option<&mut NoCRouterTile> {
        if x < self.grid_cols && y < self.grid_rows {
            let idx = self.tile_index(x, y);
            Some(&mut self.tiles[idx])
        } else {
            None
        }
    }

    /// Route a stream of flits from source (sx, sy) to destination (dx, dy).
    /// If policy is DimensionOrderXY, routes strictly through minimal XY path.
    /// If policy is ThermalDeflection, deflects around tiles exceeding thermal_threshold_c.
    pub fn route_packet(&mut self, sx: usize, sy: usize, dx: usize, dy: usize, flit_count: usize) {
        let mut cur_x = sx.min(self.grid_cols - 1);
        let mut cur_y = sy.min(self.grid_rows - 1);
        let dest_x = dx.min(self.grid_cols - 1);
        let dest_y = dy.min(self.grid_rows - 1);

        let energy_per_flit_j = (self.flit_width_bits as f64 * self.energy_per_bit_pj) * 1.0e-12;

        let max_hops = self.grid_cols + self.grid_rows + 4;
        let mut hops = 0;

        while (cur_x != dest_x || cur_y != dest_y) && hops < max_hops {
            hops += 1;
            let idx = self.tile_index(cur_x, cur_y);
            self.tiles[idx].flits_routed += flit_count as u64;

            // Compute next candidate hop
            let (next_x, next_y) = match self.policy {
                NoCRoutingPolicy::DimensionOrderXY => {
                    if cur_x != dest_x {
                        let step_x = if dest_x > cur_x { cur_x + 1 } else { cur_x - 1 };
                        (step_x, cur_y)
                    } else {
                        let step_y = if dest_y > cur_y { cur_y + 1 } else { cur_y - 1 };
                        (cur_x, step_y)
                    }
                }
                NoCRoutingPolicy::ThermalDeflection => {
                    // Preferred minimal step
                    let (pref_x, pref_y) = if cur_x != dest_x {
                        let step_x = if dest_x > cur_x { cur_x + 1 } else { cur_x - 1 };
                        (step_x, cur_y)
                    } else {
                        let step_y = if dest_y > cur_y { cur_y + 1 } else { cur_y - 1 };
                        (cur_x, step_y)
                    };

                    // Check if preferred router is overheated
                    let pref_idx = self.tile_index(pref_x, pref_y);
                    if self.tiles[pref_idx].temperature_c > self.thermal_threshold_c {
                        // Deflect orthogonal if possible
                        if cur_x != dest_x && cur_y != dest_y {
                            let alt_y = if dest_y > cur_y { cur_y + 1 } else { cur_y - 1 };
                            (cur_x, alt_y)
                        } else if cur_y < self.grid_rows - 1 {
                            (cur_x, cur_y + 1)
                        } else if cur_y > 0 {
                            (cur_x, cur_y - 1)
                        } else {
                            (pref_x, pref_y)
                        }
                    } else {
                        (pref_x, pref_y)
                    }
                }
            };

            cur_x = next_x.min(self.grid_cols - 1);
            cur_y = next_y.min(self.grid_rows - 1);

            // Dynamic dissipation heat injection
            let hop_idx = self.tile_index(cur_x, cur_y);
            let dynamic_power = (flit_count as f64 * energy_per_flit_j) * (self.clock_freq_ghz * 1.0e9 / 10.0);
            self.tiles[hop_idx].power_dissipation_w = 0.15 + dynamic_power.min(2.5);
            self.tiles[hop_idx].temperature_c += (dynamic_power * 1.5).min(1.2);
        }

        // Apply 2D thermal conduction smoothing across adjacent mesh tiles
        self.apply_thermal_conduction();
    }

    /// Simulate 2D thermal conduction relaxation between neighboring tiles.
    pub fn apply_thermal_conduction(&mut self) {
        let cols = self.grid_cols;
        let rows = self.grid_rows;
        let ambient = 45.0;

        let mut next_temps = Vec::with_capacity(cols * rows);
        for y in 0..rows {
            for x in 0..cols {
                let idx = y * cols + x;
                let cur_t = self.tiles[idx].temperature_c;

                let mut neighbor_sum: f64 = 0.0;
                let mut count: f64 = 0.0;

                if x > 0 { neighbor_sum += self.tiles[y * cols + (x - 1)].temperature_c; count += 1.0; }
                if x < cols - 1 { neighbor_sum += self.tiles[y * cols + (x + 1)].temperature_c; count += 1.0; }
                if y > 0 { neighbor_sum += self.tiles[(y - 1) * cols + x].temperature_c; count += 1.0; }
                if y < rows - 1 { neighbor_sum += self.tiles[(y + 1) * cols + x].temperature_c; count += 1.0; }

                let avg_neighbor = neighbor_sum / count.max(1.0);
                // Heat conduction + substrate cooling dissipation toward ambient
                let new_t = 0.70 * cur_t + 0.20 * avg_neighbor + 0.10 * ambient;
                next_temps.push(new_t);
            }
        }

        for (idx, &t) in next_temps.iter().enumerate() {
            self.tiles[idx].temperature_c = t.clamp(ambient, 130.0);
        }
    }

    /// Find peak maximum router junction temperature in deg C.
    pub fn peak_junction_temperature_c(&self) -> f64 {
        self.tiles
            .iter()
            .map(|t| t.temperature_c)
            .fold(0.0_f64, |a, b| a.max(b))
    }

    /// Find average router temperature in deg C.
    pub fn average_junction_temperature_c(&self) -> f64 {
        let sum: f64 = self.tiles.iter().map(|t| t.temperature_c).sum();
        sum / self.tiles.len().max(1) as f64
    }

    /// Simulate intensive streaming workload comparing XY vs Deflection routing.
    /// Returns (peak_xy_c, peak_deflect_c, hotspot_mitigation_c).
    pub fn evaluate_thermal_mitigation_delta(&self) -> (f64, f64, f64) {
        let mut sim_xy = self.clone();
        sim_xy.policy = NoCRoutingPolicy::DimensionOrderXY;

        let mut sim_deflect = self.clone();
        sim_deflect.policy = NoCRoutingPolicy::ThermalDeflection;

        // Inject heavy streaming cross-traffic focused on center routers
        for _ in 0..20 {
            sim_xy.route_packet(0, 0, 3, 3, 200);
            sim_xy.route_packet(0, 3, 3, 0, 200);
            sim_xy.route_packet(1, 0, 2, 3, 250);

            sim_deflect.route_packet(0, 0, 3, 3, 200);
            sim_deflect.route_packet(0, 3, 3, 0, 200);
            sim_deflect.route_packet(1, 0, 2, 3, 250);
        }

        let peak_xy = sim_xy.peak_junction_temperature_c();
        let peak_deflect = sim_deflect.peak_junction_temperature_c();
        let delta = (peak_xy - peak_deflect).max(0.0);

        (peak_xy, peak_deflect, delta)
    }
}
