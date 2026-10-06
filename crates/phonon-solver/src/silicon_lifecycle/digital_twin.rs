#![deny(unsafe_code)]

//! Digital Twin Health Monitoring, Anomaly Detection & Sensor Fusion.
//!
//! Provides:
//! - Real-time spatial field reconstruction fusing sparse on-die sensor telemetry.
//! - Statistical residual anomaly detection identifying sensor drift, stuck-at faults, and cooling defects.
//! - Silicon health scoring and Arrhenius-accelerated Remaining Useful Life (RUL) estimation.

use super::sensor_mesh::{OnDieSensor, SensorKind, SensorMesh, SensorStatus};

/// Classified diagnostic anomaly event.
#[derive(Debug, Clone)]
pub struct AnomalyEvent {
    pub sensor_id: usize,
    pub kind: SensorKind,
    pub timestamp_hours: f64,
    pub anomaly_type: &'static str,
    pub severity: &'static str,
    pub description: String,
}

/// Digital Twin state estimation and lifecycle analytics.
#[derive(Debug, Clone)]
pub struct DigitalTwinModel {
    /// Reconstructed peak die temperature in degrees Celsius.
    pub reconstructed_peak_temp_c: f64,
    /// Reconstructed average die temperature in degrees Celsius.
    pub reconstructed_avg_temp_c: f64,
    /// Absolute estimation error at peak hotspot location in deg C.
    pub peak_reconstruction_error_c: f64,
    /// Overall silicon health index from 0.0% (end of life) to 100.0% (pristine).
    pub silicon_health_score_pct: f64,
    /// Cumulative operating hours under current workload profile.
    pub operating_hours: f64,
    /// Arrhenius acceleration factor based on effective thermal exposure.
    pub arrhenius_acceleration_factor: f64,
    /// Projected Remaining Useful Life (RUL) in operating hours.
    pub projected_rul_hours: f64,
    /// Active anomaly alerts identified by the diagnostic engine.
    pub active_anomalies: Vec<AnomalyEvent>,
}

impl Default for DigitalTwinModel {
    fn default() -> Self {
        Self {
            reconstructed_peak_temp_c: 82.5,
            reconstructed_avg_temp_c: 54.0,
            peak_reconstruction_error_c: 1.8,
            silicon_health_score_pct: 97.4,
            operating_hours: 1250.0,
            arrhenius_acceleration_factor: 1.25,
            projected_rul_hours: 74200.0,
            active_anomalies: Vec::new(),
        }
    }
}

impl DigitalTwinModel {
    /// Reconstructs the 2D temperature field from sparse sensor measurements using Inverse Distance Weighting (IDW).
    pub fn reconstruct_temperature_at(&self, x: f64, y: f64, sensors: &[OnDieSensor]) -> f64 {
        let thermal_sensors: Vec<&OnDieSensor> = sensors
            .iter()
            .filter(|s| s.kind == SensorKind::ThermalDiode && !s.is_faulty)
            .collect();

        if thermal_sensors.is_empty() {
            return 45.0; // fallback ambient
        }

        let p = 2.0; // IDW power parameter
        let mut num = 0.0;
        let mut den = 0.0;

        for s in &thermal_sensors {
            let dx = x - s.position_mm[0];
            let dy = y - s.position_mm[1];
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 0.05 {
                return s.value;
            }

            let weight = 1.0 / dist.powf(p);
            num += weight * s.value;
            den += weight;
        }

        if den > 0.0 {
            num / den
        } else {
            45.0
        }
    }

    /// Evaluates full digital twin state and runs anomaly diagnostics.
    pub fn update(&mut self, mesh: &SensorMesh, operating_hours: f64) {
        self.operating_hours = operating_hours;
        self.active_anomalies.clear();

        // 1. Reconstruct spatial thermal field and find peak
        let mut peak_reconstructed = 0.0f64;
        let mut sum_temp = 0.0f64;
        let steps = 20;
        let dx = mesh.die_dims_mm[0] / steps as f64;
        let dy = mesh.die_dims_mm[1] / steps as f64;
        let count = (steps + 1) * (steps + 1);

        for j in 0..=steps {
            let y = j as f64 * dy;
            for i in 0..=steps {
                let x = i as f64 * dx;
                let t_rec = self.reconstruct_temperature_at(x, y, &mesh.sensors);
                if t_rec > peak_reconstructed {
                    peak_reconstructed = t_rec;
                }
                sum_temp += t_rec;
            }
        }

        self.reconstructed_peak_temp_c = peak_reconstructed;
        self.reconstructed_avg_temp_c = sum_temp / count as f64;

        let true_peak = mesh.peak_die_temperature();
        self.peak_reconstruction_error_c = (true_peak - peak_reconstructed).abs();

        // 2. Anomaly Detection
        for sensor in &mesh.sensors {
            // Check stuck-at fault (true value differs drastically from reading)
            if (sensor.value - sensor.true_value).abs() > 8.0 {
                self.active_anomalies.push(AnomalyEvent {
                    sensor_id: sensor.id,
                    kind: sensor.kind,
                    timestamp_hours: self.operating_hours,
                    anomaly_type: "Sensor Calibration Drift / Stuck",
                    severity: "HIGH",
                    description: format!(
                        "Sensor {} reading ({:.2}) deviates > 8 from physical field ({:.2})",
                        sensor.id, sensor.value, sensor.true_value
                    ),
                });
            }

            // Check localized thermal runaway / emergency trip
            if sensor.kind == SensorKind::ThermalDiode && sensor.status == SensorStatus::Critical {
                self.active_anomalies.push(AnomalyEvent {
                    sensor_id: sensor.id,
                    kind: sensor.kind,
                    timestamp_hours: self.operating_hours,
                    anomaly_type: "Localized Thermal Hotspot Critical",
                    severity: "CRITICAL",
                    description: format!(
                        "Thermal diode {} at [{:.1}, {:.1}] exceeded critical trip threshold ({:.1} C)",
                        sensor.id, sensor.position_mm[0], sensor.position_mm[1], sensor.value
                    ),
                });
            }

            // Check severe voltage droop alert
            if sensor.kind == SensorKind::SupplyDroopDetector && sensor.status == SensorStatus::Critical {
                self.active_anomalies.push(AnomalyEvent {
                    sensor_id: sensor.id,
                    kind: sensor.kind,
                    timestamp_hours: self.operating_hours,
                    anomaly_type: "Transient Supply Droop Violation",
                    severity: "HIGH",
                    description: format!(
                        "Droop detector {} recorded severe rail collapse to {:.3} V",
                        sensor.id, sensor.value
                    ),
                });
            }

            // Check timing margin breach
            if sensor.kind == SensorKind::CriticalPathMonitor && sensor.status == SensorStatus::Critical {
                self.active_anomalies.push(AnomalyEvent {
                    sensor_id: sensor.id,
                    kind: sensor.kind,
                    timestamp_hours: self.operating_hours,
                    anomaly_type: "Zero Timing Slack Hazard",
                    severity: "CRITICAL",
                    description: format!(
                        "Critical path monitor {} slack collapsed to {:.1} ps (imminent setup timing violation)",
                        sensor.id, sensor.value
                    ),
                });
            }
        }

        // 3. Silicon Health Scoring & Arrhenius RUL
        // Arrhenius model: AF = exp( (E_a / k_B) * (1/T_ref - 1/T_eff) )
        // E_a = 0.75 eV, k_B = 8.617e-5 eV/K, T_ref = 55 C = 328.15 K
        let t_eff_k = self.reconstructed_avg_temp_c + 273.15;
        let t_ref_k = 328.15;
        let ea_over_kb = 0.75 / 8.617e-5;
        let exponent = ea_over_kb * (1.0 / t_ref_k - 1.0 / t_eff_k);
        let af = exponent.clamp(-5.0, 5.0).exp();
        self.arrhenius_acceleration_factor = af;

        // Effective aging hours
        let eff_hours = self.operating_hours * af;
        let nominal_lifetime_hours = 87_600.0; // 10 years 24/7

        // Health score decays monotonically with effective thermal wear
        let wear_fraction = (eff_hours / nominal_lifetime_hours).clamp(0.0, 1.0);
        let base_health = 100.0 * (1.0 - 0.25 * wear_fraction - 0.05 * (wear_fraction * wear_fraction));

        // Deduct penalties for active critical alarms
        let alarm_penalty = self.active_anomalies.len() as f64 * 1.5;
        self.silicon_health_score_pct = (base_health - alarm_penalty).clamp(5.0, 100.0);

        let remaining = (nominal_lifetime_hours - eff_hours) / af.max(0.1);
        self.projected_rul_hours = remaining.max(0.0);
    }
}
