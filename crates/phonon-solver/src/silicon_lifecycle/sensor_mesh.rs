#![deny(unsafe_code)]

//! On-Die Sensor Mesh & Physics Formulation.
//!
//! Models 2D spatial thermal and voltage fields across complex multi-core/heterogeneous
//! floorplans with physical on-die sensors:
//! - Thermal Diodes: Delta V_be junction temperature sensing with ADC quantization and noise.
//! - Ring Oscillators: Multi-stage delay lines measuring local V_dd, temperature, and wear.
//! - Supply Droop Detectors: Sub-nanosecond comparators capturing fast IR-drop events.
//! - Critical Path Monitors: Tunable replica delay paths predicting timing margin degradation.

/// Type and physical sensing modality of an on-die sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorKind {
    /// BJT thermal diode measuring junction temperature in degrees Celsius.
    ThermalDiode,
    /// Ring oscillator measuring local stage delay and clock frequency in MHz.
    RingOscillator,
    /// Fast analog comparator detecting transient supply voltage droops below threshold.
    SupplyDroopDetector,
    /// In-situ replica critical path delay line measuring timing slack in picoseconds.
    CriticalPathMonitor,
}

impl SensorKind {
    /// Returns the standard engineering unit label.
    pub fn unit_str(&self) -> &'static str {
        match self {
            SensorKind::ThermalDiode => "deg C",
            SensorKind::RingOscillator => "MHz",
            SensorKind::SupplyDroopDetector => "V",
            SensorKind::CriticalPathMonitor => "ps",
        }
    }
}

/// Operational and alarm state of a sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStatus {
    /// Reading is within normal operating limits.
    Normal,
    /// Reading has crossed the non-fatal advisory warning threshold.
    Warning,
    /// Reading has crossed the emergency critical threshold requiring throttle/trip.
    Critical,
}

/// Representation of an IP block or functional tile on the silicon die.
#[derive(Debug, Clone)]
pub struct IpBlock {
    pub id: usize,
    pub name: String,
    /// Bounding rectangle in millimeters: [x_min, y_min, x_max, y_max].
    pub rect: [f64; 4],
    /// Nominal power dissipation under 100% active workload in Watts.
    pub nominal_power_w: f64,
    /// Current activity/utilization factor between 0.0 (idle/sleep) and 1.0 (peak load).
    pub activity: f64,
    /// Nominal supply voltage in Volts.
    pub nominal_voltage_v: f64,
    /// Is this block a keep-out zone for sensor insertion (e.g. dense analog PLL or SRAM array).
    pub is_keepout: bool,
}

impl IpBlock {
    /// Computes center coordinate [x, y] in millimeters.
    pub fn center(&self) -> [f64; 2] {
        [
            0.5 * (self.rect[0] + self.rect[2]),
            0.5 * (self.rect[1] + self.rect[3]),
        ]
    }

    /// Computes area in square millimeters.
    pub fn area_mm2(&self) -> f64 {
        let dx = (self.rect[2] - self.rect[0]).abs();
        let dy = (self.rect[3] - self.rect[1]).abs();
        dx * dy
    }

    /// Instantaneous power dissipation in Watts.
    pub fn current_power_w(&self) -> f64 {
        self.nominal_power_w * (0.15 + 0.85 * self.activity.clamp(0.0, 1.2))
    }

    /// Checks if a point [x, y] in mm is inside this block.
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.rect[0] && x <= self.rect[2] && y >= self.rect[1] && y <= self.rect[3]
    }
}

/// An individual physical or virtual on-die sensor instance.
#[derive(Debug, Clone)]
pub struct OnDieSensor {
    pub id: usize,
    pub kind: SensorKind,
    /// Physical location on the die in millimeters: [x, y].
    pub position_mm: [f64; 2],
    /// Optional parent IP block ID enclosing this sensor.
    pub parent_block_id: Option<usize>,
    /// Latest raw reading (quantized engineering value).
    pub value: f64,
    /// True physical unquantized value before ADC conversion and noise.
    pub true_value: f64,
    /// Non-fatal warning threshold.
    pub warning_threshold: f64,
    /// Fatal/emergency critical threshold.
    pub critical_threshold: f64,
    /// Current operational status.
    pub status: SensorStatus,
    /// Flag indicating whether the sensor is intentionally tripped or in stuck-at fault.
    pub is_faulty: bool,
    /// Number of cumulative alarm events recorded.
    pub alarm_count: u32,
}

impl OnDieSensor {
    /// Creates a new thermal diode with standard 12-bit ADC quantization (0.0625 deg C LSB).
    pub fn new_thermal_diode(id: usize, x: f64, y: f64, block_id: Option<usize>) -> Self {
        Self {
            id,
            kind: SensorKind::ThermalDiode,
            position_mm: [x, y],
            parent_block_id: block_id,
            value: 45.0,
            true_value: 45.0,
            warning_threshold: 85.0,
            critical_threshold: 105.0,
            status: SensorStatus::Normal,
            is_faulty: false,
            alarm_count: 0,
        }
    }

    /// Creates a new ring oscillator process/voltage monitor.
    pub fn new_ring_oscillator(id: usize, x: f64, y: f64, block_id: Option<usize>) -> Self {
        Self {
            id,
            kind: SensorKind::RingOscillator,
            position_mm: [x, y],
            parent_block_id: block_id,
            value: 1200.0,
            true_value: 1200.0,
            warning_threshold: 950.0,
            critical_threshold: 850.0,
            status: SensorStatus::Normal,
            is_faulty: false,
            alarm_count: 0,
        }
    }

    /// Creates a new supply droop detector with 8% warning and 12% critical droop trip points.
    pub fn new_droop_detector(
        id: usize,
        x: f64,
        y: f64,
        block_id: Option<usize>,
        v_nom: f64,
    ) -> Self {
        Self {
            id,
            kind: SensorKind::SupplyDroopDetector,
            position_mm: [x, y],
            parent_block_id: block_id,
            value: v_nom,
            true_value: v_nom,
            warning_threshold: v_nom * 0.92,
            critical_threshold: v_nom * 0.88,
            status: SensorStatus::Normal,
            is_faulty: false,
            alarm_count: 0,
        }
    }

    /// Creates a new critical path delay monitor.
    pub fn new_critical_path_monitor(id: usize, x: f64, y: f64, block_id: Option<usize>) -> Self {
        Self {
            id,
            kind: SensorKind::CriticalPathMonitor,
            position_mm: [x, y],
            parent_block_id: block_id,
            value: 65.0,
            true_value: 65.0,
            warning_threshold: 25.0,
            critical_threshold: 5.0,
            status: SensorStatus::Normal,
            is_faulty: false,
            alarm_count: 0,
        }
    }

    /// Updates sensor reading from true physical value and evaluates threshold statuses.
    pub fn update_reading(&mut self, true_phys_val: f64, noise_seed: u64) {
        if self.is_faulty {
            return;
        }

        self.true_value = true_phys_val;

        // Apply sensor-specific noise and ADC quantization
        match self.kind {
            SensorKind::ThermalDiode => {
                // Pseudo-random Gaussian noise: +/- 0.15 deg C
                let pseudo_rand = ((noise_seed.wrapping_mul(0x5DEECE66D).wrapping_add(0xB)) & 0xFFFF) as f64 / 65536.0;
                let noise = (pseudo_rand - 0.5) * 0.3;
                let noisy_val = true_phys_val + noise;
                // 12-bit ADC quantization: LSB = 0.0625 deg C
                let quantized = (noisy_val / 0.0625).round() * 0.0625;
                self.value = quantized;

                // Threshold check (higher is worse)
                let prev_status = self.status;
                if self.value >= self.critical_threshold {
                    self.status = SensorStatus::Critical;
                } else if self.value >= self.warning_threshold {
                    self.status = SensorStatus::Warning;
                } else {
                    self.status = SensorStatus::Normal;
                }

                if prev_status == SensorStatus::Normal && self.status != SensorStatus::Normal {
                    self.alarm_count = self.alarm_count.saturating_add(1);
                }
            }
            SensorKind::RingOscillator => {
                // Frequency in MHz, quantized to 0.5 MHz
                let quantized = (true_phys_val / 0.5).round() * 0.5;
                self.value = quantized;

                // Lower frequency indicates slowdown from heat, IR droop, or aging
                let prev_status = self.status;
                if self.value <= self.critical_threshold {
                    self.status = SensorStatus::Critical;
                } else if self.value <= self.warning_threshold {
                    self.status = SensorStatus::Warning;
                } else {
                    self.status = SensorStatus::Normal;
                }

                if prev_status == SensorStatus::Normal && self.status != SensorStatus::Normal {
                    self.alarm_count = self.alarm_count.saturating_add(1);
                }
            }
            SensorKind::SupplyDroopDetector => {
                // Supply voltage in Volts, 10-bit ADC or comparator (1 mV LSB)
                let quantized = (true_phys_val * 1000.0).round() / 1000.0;
                self.value = quantized;

                // Lower voltage indicates severe droop
                let prev_status = self.status;
                if self.value <= self.critical_threshold {
                    self.status = SensorStatus::Critical;
                } else if self.value <= self.warning_threshold {
                    self.status = SensorStatus::Warning;
                } else {
                    self.status = SensorStatus::Normal;
                }

                if prev_status == SensorStatus::Normal && self.status != SensorStatus::Normal {
                    self.alarm_count = self.alarm_count.saturating_add(1);
                }
            }
            SensorKind::CriticalPathMonitor => {
                // Timing slack in ps, quantized to 1 ps
                let quantized = true_phys_val.round();
                self.value = quantized;

                // Lower slack indicates impending timing violation
                let prev_status = self.status;
                if self.value <= self.critical_threshold {
                    self.status = SensorStatus::Critical;
                } else if self.value <= self.warning_threshold {
                    self.status = SensorStatus::Warning;
                } else {
                    self.status = SensorStatus::Normal;
                }

                if prev_status == SensorStatus::Normal && self.status != SensorStatus::Normal {
                    self.alarm_count = self.alarm_count.saturating_add(1);
                }
            }
        }
    }
}

/// Full silicon die floorplan and multi-sensor mesh.
#[derive(Debug, Clone)]
pub struct SensorMesh {
    /// Die dimensions in millimeters: [width, height].
    pub die_dims_mm: [f64; 2],
    /// Ambient reference temperature in degrees Celsius (e.g. 25.0).
    pub ambient_temp_c: f64,
    /// Base nominal supply voltage in Volts (e.g. 0.85).
    pub nominal_vdd_v: f64,
    /// List of floorplan functional IP blocks.
    pub blocks: Vec<IpBlock>,
    /// List of placed physical and virtual sensors.
    pub sensors: Vec<OnDieSensor>,
    /// Global cycle counter for pseudo-random noise and time evolution.
    pub sample_tick: u64,
}

impl Default for SensorMesh {
    fn default() -> Self {
        Self::new_heterogeneous_soc()
    }
}

impl SensorMesh {
    /// Creates a realistic 20mm x 20mm heterogeneous server SoC floorplan with default sensors.
    pub fn new_heterogeneous_soc() -> Self {
        let die_dims = [20.0, 20.0];
        let v_nom = 0.85;

        // Build floorplan blocks
        let mut blocks = Vec::new();

        // 8 CPU Compute Cores (2x4 cluster) in upper half
        for row in 0..2 {
            for col in 0..4 {
                let id = row * 4 + col;
                let x_min = 1.0 + col as f64 * 3.2;
                let y_min = 11.5 + row as f64 * 3.8;
                blocks.push(IpBlock {
                    id,
                    name: format!("CPU_Core_{id}"),
                    rect: [x_min, y_min, x_min + 2.8, y_min + 3.2],
                    nominal_power_w: 16.0,
                    activity: if id == 2 || id == 3 { 0.95 } else { 0.40 },
                    nominal_voltage_v: v_nom,
                    is_keepout: false,
                });
            }
        }

        // Shared L3 Cache block between CPU cores
        blocks.push(IpBlock {
            id: 8,
            name: "L3_Cache_Array".to_string(),
            rect: [1.0, 9.8, 13.8, 11.2],
            nominal_power_w: 6.0,
            activity: 0.60,
            nominal_voltage_v: v_nom,
            is_keepout: true, // dense SRAM array keepout
        });

        // GPU Compute Complex in lower-left
        blocks.push(IpBlock {
            id: 9,
            name: "GPU_Compute_Complex".to_string(),
            rect: [1.0, 1.0, 9.0, 9.0],
            nominal_power_w: 45.0,
            activity: 0.85,
            nominal_voltage_v: v_nom,
            is_keepout: false,
        });

        // NPU Matrix Accelerator in center-right
        blocks.push(IpBlock {
            id: 10,
            name: "NPU_Tensor_Accelerator".to_string(),
            rect: [9.8, 1.0, 15.0, 9.0],
            nominal_power_w: 32.0,
            activity: 0.70,
            nominal_voltage_v: v_nom,
            is_keepout: false,
        });

        // Memory Controller 0 (DDR5/HBM) at bottom-right
        blocks.push(IpBlock {
            id: 11,
            name: "Mem_Ctrl_0".to_string(),
            rect: [15.5, 1.0, 19.0, 7.0],
            nominal_power_w: 12.0,
            activity: 0.75,
            nominal_voltage_v: v_nom,
            is_keepout: false,
        });

        // PCIe Gen5 / CXL Root Complex at upper-right
        blocks.push(IpBlock {
            id: 12,
            name: "PCIe_CXL_Controller".to_string(),
            rect: [14.5, 8.0, 19.0, 15.0],
            nominal_power_w: 14.0,
            activity: 0.50,
            nominal_voltage_v: v_nom,
            is_keepout: false,
        });

        // PLL and Analog Clock Generator (Keepout zone) at top edge
        blocks.push(IpBlock {
            id: 13,
            name: "Analog_PLL_Clock_Gen".to_string(),
            rect: [14.5, 15.8, 19.0, 19.0],
            nominal_power_w: 4.5,
            activity: 0.90,
            nominal_voltage_v: v_nom,
            is_keepout: true,
        });

        let mut mesh = Self {
            die_dims_mm: die_dims,
            ambient_temp_c: 25.0,
            nominal_vdd_v: v_nom,
            blocks,
            sensors: Vec::new(),
            sample_tick: 0,
        };

        // Populate baseline sensor suite
        mesh.seed_default_sensors();
        mesh.sample_all_sensors();

        mesh
    }

    /// Seeds standard set of on-die sensors across functional blocks.
    pub fn seed_default_sensors(&mut self) {
        self.sensors.clear();
        let mut s_id = 0;

        // Place thermal diode and ring oscillator in each non-keepout block
        for block in &self.blocks {
            if block.is_keepout {
                continue;
            }
            let center = block.center();

            // Thermal diode near center
            self.sensors.push(OnDieSensor::new_thermal_diode(
                s_id,
                center[0],
                center[1],
                Some(block.id),
            ));
            s_id += 1;

            // Ring oscillator offset slightly
            self.sensors.push(OnDieSensor::new_ring_oscillator(
                s_id,
                center[0] + 0.4,
                center[1] - 0.4,
                Some(block.id),
            ));
            s_id += 1;

            // Droop detector
            self.sensors.push(OnDieSensor::new_droop_detector(
                s_id,
                center[0] - 0.4,
                center[1] + 0.4,
                Some(block.id),
                self.nominal_vdd_v,
            ));
            s_id += 1;

            // Critical path monitor on high performance blocks
            if block.nominal_power_w > 15.0 {
                self.sensors.push(OnDieSensor::new_critical_path_monitor(
                    s_id,
                    center[0] + 0.6,
                    center[1] + 0.6,
                    Some(block.id),
                ));
                s_id += 1;
            }
        }

        // Additional perimeter sensors for thermal gradient tracking
        let perimeter_points = [
            [2.0, 18.5],
            [10.0, 18.5],
            [18.0, 18.5],
            [18.0, 10.0],
            [18.0, 2.0],
            [10.0, 0.5],
            [2.0, 0.5],
            [0.5, 10.0],
        ];

        for pt in &perimeter_points {
            self.sensors.push(OnDieSensor::new_thermal_diode(
                s_id,
                pt[0],
                pt[1],
                None,
            ));
            s_id += 1;
        }
    }

    /// Evaluates continuous physical 2D temperature field T(x, y) in degrees Celsius.
    pub fn temperature_at(&self, x: f64, y: f64) -> f64 {
        let mut temp = self.ambient_temp_c + 18.0; // base heatsink offset ~43C

        // Analytical Green's function thermal diffusion summation
        for block in &self.blocks {
            let p_curr = block.current_power_w();
            let center = block.center();
            let dx = x - center[0];
            let dy = y - center[1];
            let dist_sq = dx * dx + dy * dy;

            // Spreading radius based on block area
            let r_spread = (block.area_mm2() / std::f64::consts::PI).sqrt().max(1.0);
            let sigma_sq = 2.0 * r_spread * r_spread;

            // Thermal impedance scaling factor: ~1.15 C/W effective
            let delta_t_peak = p_curr * 1.15;
            let contrib = delta_t_peak * (-dist_sq / sigma_sq).exp();
            temp += contrib;
        }

        temp
    }

    /// Evaluates continuous physical 2D voltage field V(x, y) in Volts.
    pub fn voltage_at(&self, x: f64, y: f64) -> f64 {
        let mut droop = 0.0;
        let r_grid_eff = 0.0018; // Effective on-die grid sheet resistance

        for block in &self.blocks {
            let i_curr = block.current_power_w() / block.nominal_voltage_v;
            let center = block.center();
            let dist_sq = (x - center[0]).powi(2) + (y - center[1]).powi(2);
            let sigma_sq = 4.0; // IR droop localized spreading

            let block_droop = i_curr * r_grid_eff * (-dist_sq / sigma_sq).exp();
            droop += block_droop;
        }

        (self.nominal_vdd_v - droop).max(0.60)
    }

    /// Evaluates local ring oscillator frequency in MHz given local voltage and temperature.
    pub fn ring_oscillator_freq_at(&self, x: f64, y: f64, aging_derate: f64) -> f64 {
        let v = self.voltage_at(x, y);
        let t_c = self.temperature_at(x, y);

        // Alpha-power law inverter delay model:
        // f_ro proportional to (V - V_th(T))^1.3 / V
        let v_th_t = 0.32 - 0.0008 * (t_c - 25.0); // Threshold voltage roll-off with temp
        let v_overdrive = (v - v_th_t).max(0.1);
        let f_nominal = 1200.0;
        let v_nom_overdrive = 0.85 - 0.32;

        let scale_v = (v_overdrive / v_nom_overdrive).powf(1.3) * (0.85 / v);
        // High temperature mobility reduction slowdown: ~ -0.15% per deg C
        let scale_t = 1.0 - 0.0015 * (t_c - 25.0);

        (f_nominal * scale_v * scale_t * aging_derate).max(200.0)
    }

    /// Evaluates critical path timing slack in picoseconds.
    pub fn critical_path_slack_at(&self, x: f64, y: f64) -> f64 {
        let v = self.voltage_at(x, y);
        let t_c = self.temperature_at(x, y);

        // Target clock period: 500 ps (2.0 GHz)
        let t_clk = 500.0;
        // Path delay increases as voltage drops and temperature rises
        let base_delay = 340.0; // Nominal 160 ps slack at 0.85V, 25C
        let delay = base_delay * (0.85 / v).powf(1.4) * (1.0 + 0.0018 * (t_c - 25.0));

        t_clk - delay
    }

    /// Samples all sensors across the mesh, updating their readings and alarm states.
    pub fn sample_all_sensors(&mut self) {
        self.sample_tick = self.sample_tick.wrapping_add(1);
        let tick = self.sample_tick;

        let n = self.sensors.len();
        let mut updates = Vec::with_capacity(n);
        for sensor in &self.sensors {
            let [x, y] = sensor.position_mm;
            let seed = tick.wrapping_mul(1009).wrapping_add(sensor.id as u64);
            let val = match sensor.kind {
                SensorKind::ThermalDiode => self.temperature_at(x, y),
                SensorKind::RingOscillator => self.ring_oscillator_freq_at(x, y, 0.98),
                SensorKind::SupplyDroopDetector => self.voltage_at(x, y),
                SensorKind::CriticalPathMonitor => self.critical_path_slack_at(x, y),
            };
            updates.push((val, seed));
        }

        for (sensor, (val, seed)) in self.sensors.iter_mut().zip(updates) {
            sensor.update_reading(val, seed);
        }
    }

    /// Returns the maximum true junction temperature across the entire die floorplan.
    pub fn peak_die_temperature(&self) -> f64 {
        let mut max_t = 0.0;
        // Sample across fine grid
        let steps = 25;
        let dx = self.die_dims_mm[0] / steps as f64;
        let dy = self.die_dims_mm[1] / steps as f64;

        for i in 0..=steps {
            for j in 0..=steps {
                let t = self.temperature_at(i as f64 * dx, j as f64 * dy);
                if t > max_t {
                    max_t = t;
                }
            }
        }
        max_t
    }

    /// Returns the maximum temperature measured by currently deployed thermal diodes.
    pub fn peak_measured_temperature(&self) -> f64 {
        self.sensors
            .iter()
            .filter(|s| s.kind == SensorKind::ThermalDiode)
            .map(|s| s.value)
            .fold(0.0, f64::max)
    }

    /// Returns the count of sensors currently in Warning or Critical state.
    pub fn active_alarm_count(&self) -> (usize, usize) {
        let mut warnings = 0;
        let mut criticals = 0;
        for s in &self.sensors {
            match s.status {
                SensorStatus::Normal => {}
                SensorStatus::Warning => warnings += 1,
                SensorStatus::Critical => criticals += 1,
            }
        }
        (warnings, criticals)
    }
}
