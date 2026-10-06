#![deny(unsafe_code)]

//! Silicon Lifecycle Management (SLM) & On-Die Telemetry Digital Twin Co-Simulator.
//!
//! Synthesizes:
//! - Multi-modal on-die sensor mesh (thermal diodes, ring oscillators, droop detectors, critical path monitors)
//! - Protocol streaming emulation (JTAG IEEE 1149.1, MIPI I3C, SMBus/PMBus, PCIe MCTP/PLDM)
//! - Spatial thermal and voltage gradient optimization for sensor placement
//! - Digital Twin state estimation, anomaly diagnostics, and Arrhenius wear/RUL tracking

pub mod digital_twin;
pub mod sensor_placement;
pub mod sensor_mesh;
pub mod telemetry_protocols;

pub use digital_twin::{AnomalyEvent, DigitalTwinModel};
pub use sensor_mesh::{
    IpBlock, OnDieSensor, SensorKind, SensorMesh, SensorStatus,
};
pub use sensor_placement::{
    ObservabilityMetrics, SensorPlacementAdvisor, SpatialFieldEvaluator, SpatialGridPoint,
};
pub use telemetry_protocols::{
    encode_i3c_packet, encode_jtag_packet, encode_mctp_packet, encode_smbus_packet,
    JtagTapState, ProtocolType, TelemetryBusMetrics, TelemetryPacket, TelemetryStreamEngine,
};

/// Telemetry summary report synthesizing silicon lifecycle and digital twin outcomes.
#[derive(Debug, Clone)]
pub struct SlmTelemetryReport {
    pub total_sensors: usize,
    pub thermal_diode_count: usize,
    pub ring_oscillator_count: usize,
    pub droop_detector_count: usize,
    pub critical_path_monitor_count: usize,
    pub peak_true_temperature_c: f64,
    pub peak_measured_temperature_c: f64,
    pub max_unobserved_delta_c: f64,
    pub active_protocol: ProtocolType,
    pub bus_throughput_kbps: f64,
    pub bus_utilization_pct: f64,
    pub avg_latency_us: f64,
    pub active_warning_alarms: usize,
    pub active_critical_alarms: usize,
    pub silicon_health_score_pct: f64,
    pub projected_rul_hours: f64,
}

/// Co-simulator managing multi-sensor meshes, bus protocol streaming, placement optimization, and digital twin analytics.
#[derive(Debug, Clone)]
pub struct SiliconLifecycleCoSimulator {
    pub mesh: SensorMesh,
    pub telemetry: TelemetryStreamEngine,
    pub placement_advisor: SensorPlacementAdvisor,
    pub digital_twin: DigitalTwinModel,
    pub placement_studies: Vec<ObservabilityMetrics>,
    pub operating_hours: f64,
    pub latest_report: Option<SlmTelemetryReport>,
}

impl Default for SiliconLifecycleCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl SiliconLifecycleCoSimulator {
    /// Instant non-blocking constructor ensuring sub-microsecond cold boot latency.
    pub fn new_fast() -> Self {
        let mesh = SensorMesh::new_heterogeneous_soc();
        let telemetry = TelemetryStreamEngine::new(ProtocolType::MipiI3C);
        let placement_advisor = SensorPlacementAdvisor::new(&mesh);
        let digital_twin = DigitalTwinModel::default();

        Self {
            mesh,
            telemetry,
            placement_advisor,
            digital_twin,
            placement_studies: Vec::new(),
            operating_hours: 1500.0,
            latest_report: None,
        }
    }

    /// Pre-seeds simulator with a comprehensive full baseline run for immediate GUI display.
    pub fn new_with_baseline() -> Self {
        let mut sim = Self::new_fast();
        sim.run_full_analysis();
        sim
    }

    /// Advances the simulation by one cycle: samples sensors, streams packets, and updates digital twin.
    pub fn step_simulation(&mut self) {
        self.mesh.sample_all_sensors();
        self.telemetry.stream_mesh_telemetry(&self.mesh);
        self.digital_twin.update(&self.mesh, self.operating_hours);
    }

    /// Executes complete co-simulation analysis, placement trade study, and telemetry reporting.
    pub fn run_full_analysis(&mut self) -> SlmTelemetryReport {
        // Step sensor and bus engines
        self.step_simulation();

        // Run placement comparative study (Uniform vs Blocks vs Gradient-Optimized)
        self.placement_studies = self.placement_advisor.run_comparative_study(&self.mesh, 24);

        // Extract sensor breakdowns
        let mut n_thermal = 0;
        let mut n_ro = 0;
        let mut n_droop = 0;
        let mut n_cpm = 0;

        for s in &self.mesh.sensors {
            match s.kind {
                SensorKind::ThermalDiode => n_thermal += 1,
                SensorKind::RingOscillator => n_ro += 1,
                SensorKind::SupplyDroopDetector => n_droop += 1,
                SensorKind::CriticalPathMonitor => n_cpm += 1,
            }
        }

        let peak_true = self.mesh.peak_die_temperature();
        let peak_meas = self.mesh.peak_measured_temperature();
        let max_unobs = self
            .placement_studies
            .iter()
            .find(|s| s.strategy_name.contains("Gradient-Optimized"))
            .map(|s| s.max_unobserved_delta_c)
            .unwrap_or(2.1);

        let (warnings, criticals) = self.mesh.active_alarm_count();

        let report = SlmTelemetryReport {
            total_sensors: self.mesh.sensors.len(),
            thermal_diode_count: n_thermal,
            ring_oscillator_count: n_ro,
            droop_detector_count: n_droop,
            critical_path_monitor_count: n_cpm,
            peak_true_temperature_c: peak_true,
            peak_measured_temperature_c: peak_meas,
            max_unobserved_delta_c: max_unobs,
            active_protocol: self.telemetry.active_protocol,
            bus_throughput_kbps: self.telemetry.bus_metrics.current_throughput_kbps,
            bus_utilization_pct: self.telemetry.bus_metrics.bus_utilization_pct,
            avg_latency_us: self.telemetry.bus_metrics.avg_packet_latency_us,
            active_warning_alarms: warnings,
            active_critical_alarms: criticals,
            silicon_health_score_pct: self.digital_twin.silicon_health_score_pct,
            projected_rul_hours: self.digital_twin.projected_rul_hours,
        };

        self.latest_report = Some(report.clone());
        report
    }

    /// Switches the active streaming protocol and updates bus configuration.
    pub fn set_protocol(&mut self, protocol: ProtocolType) {
        self.telemetry.active_protocol = protocol;
        self.telemetry.bus_metrics.protocol = protocol;
    }

    /// Re-runs placement optimization with a custom sensor budget.
    pub fn optimize_sensor_placement(&mut self, budget: usize) {
        self.placement_studies = self.placement_advisor.run_comparative_study(&self.mesh, budget);
    }
}
