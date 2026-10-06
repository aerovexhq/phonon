#![deny(unsafe_code)]

//! Comprehensive test suite for Silicon Lifecycle Management (SLM) & On-Die Telemetry Digital Twin.

use phonon_solver::silicon_lifecycle::*;

#[test]
fn test_sensor_mesh_initialization_and_physics() {
    let mesh = SensorMesh::new_heterogeneous_soc();

    // Verify die dimensions and blocks
    assert_eq!(mesh.die_dims_mm, [20.0, 20.0]);
    assert!(mesh.blocks.len() >= 12);
    assert!(!mesh.sensors.is_empty());

    // Temperature at center of high-power GPU block should be higher than ambient + heatsink
    let t_gpu = mesh.temperature_at(5.0, 5.0);
    assert!(t_gpu > 50.0, "GPU temp {} C should be > 50 C", t_gpu);

    // Voltage field should show IR drop below nominal 0.85V
    let v_gpu = mesh.voltage_at(5.0, 5.0);
    assert!(
        v_gpu < mesh.nominal_vdd_v,
        "Voltage {} V should be < nominal 0.85V due to IR drop",
        v_gpu
    );
    assert!(v_gpu >= 0.70, "Voltage {} V should remain reasonable", v_gpu);

    // Ring oscillator frequency should be in reasonable GHz range
    let f_ro = mesh.ring_oscillator_freq_at(5.0, 5.0, 1.0);
    assert!(f_ro > 500.0 && f_ro < 1800.0, "RO freq {} MHz", f_ro);

    // Critical path timing slack should be positive under nominal conditions
    let slack = mesh.critical_path_slack_at(5.0, 5.0);
    assert!(slack > 0.0, "Timing slack {} ps should be positive", slack);
}

#[test]
fn test_sensor_sampling_and_alarm_tripping() {
    let mut mesh = SensorMesh::new_heterogeneous_soc();
    mesh.sample_all_sensors();

    // All sensors should have received valid readings
    for s in &mesh.sensors {
        assert!(s.value > 0.0, "Sensor {} value: {}", s.id, s.value);
    }

    // Force an extreme temperature on an IP block to trigger an alarm
    mesh.blocks[0].nominal_power_w = 120.0;
    mesh.blocks[0].activity = 1.0;
    mesh.sample_all_sensors();

    let (warnings, criticals) = mesh.active_alarm_count();
    assert!(
        warnings + criticals > 0,
        "Thermal alarms should trigger under 120W block power"
    );
}

#[test]
fn test_jtag_tap_controller_state_machine_and_serialization() {
    let mut tap = JtagTapState::TestLogicReset;

    // Reset sequence: TMS=0 transitions to RunTestIdle
    tap = tap.next(false);
    assert_eq!(tap, JtagTapState::RunTestIdle);

    // TMS=1 -> SelectDrScan -> CaptureDr (TMS=0) -> ShiftDr (TMS=0)
    tap = tap.next(true);
    assert_eq!(tap, JtagTapState::SelectDrScan);
    tap = tap.next(false);
    assert_eq!(tap, JtagTapState::CaptureDr);
    tap = tap.next(false);
    assert_eq!(tap, JtagTapState::ShiftDr);

    // Test JTAG packet serialization
    let sensor = OnDieSensor::new_thermal_diode(5, 4.0, 4.0, Some(1));
    let pkt = encode_jtag_packet(1, 10.0, &sensor);

    assert_eq!(pkt.protocol, ProtocolType::Jtag1149);
    assert_eq!(pkt.sensor_id, 5);
    assert_eq!(pkt.size_bits, 32);
    assert!(pkt.hex_dump.starts_with("0x"));
}

#[test]
fn test_mipi_i3c_framing_and_in_band_interrupt() {
    let mut normal_sensor = OnDieSensor::new_thermal_diode(12, 10.0, 10.0, None);
    normal_sensor.status = SensorStatus::Normal;
    normal_sensor.value = 65.0;

    let normal_pkt = encode_i3c_packet(1, 15.0, &normal_sensor);
    assert!(!normal_pkt.is_alarm_event);

    let mut crit_sensor = OnDieSensor::new_thermal_diode(12, 10.0, 10.0, None);
    crit_sensor.status = SensorStatus::Critical;
    crit_sensor.value = 112.5;

    let crit_pkt = encode_i3c_packet(2, 25.0, &crit_sensor);
    assert!(crit_pkt.is_alarm_event);
    assert!(crit_pkt.decoded_summary.contains("IBI:YES"));
}

#[test]
fn test_smbus_pmbus_read_and_pec_crc8() {
    let sensor = OnDieSensor::new_thermal_diode(7, 8.0, 8.0, None);
    let pkt = encode_smbus_packet(1, 5.0, &sensor);

    assert_eq!(pkt.protocol, ProtocolType::SmbusPmbus);
    assert_eq!(pkt.size_bits, 48);
    assert!(pkt.hex_dump.contains("PEC:"));
}

#[test]
fn test_pcie_mctp_pldm_encapsulation_and_crc32() {
    let sensor = OnDieSensor::new_thermal_diode(9, 2.0, 2.0, None);
    let pkt = encode_mctp_packet(42, 50.0, &sensor);

    assert_eq!(pkt.protocol, ProtocolType::PcieMctpPldm);
    assert!(pkt.size_bits >= 144);
    assert!(pkt.decoded_summary.contains("MCTP/PLDM"));
    assert!(pkt.decoded_summary.contains("Src:0x14->Dest:0x08"));
}

#[test]
fn test_sensor_placement_optimization_reduces_unobserved_delta() {
    let mesh = SensorMesh::new_heterogeneous_soc();
    let advisor = SensorPlacementAdvisor::new(&mesh);

    let studies = advisor.run_comparative_study(&mesh, 16);
    assert_eq!(studies.len(), 3);

    let uniform = &studies[0];
    let opt = &studies[2];

    assert_eq!(uniform.strategy_name, "Naive Uniform Grid");
    assert_eq!(opt.strategy_name, "Gradient-Optimized Algorithmic");

    // Algorithmic gradient-optimized placement must achieve lower or equal max unobserved delta
    assert!(
        opt.max_unobserved_delta_c <= uniform.max_unobserved_delta_c + 0.1,
        "Optimized delta ({:.2} C) should be <= Uniform delta ({:.2} C)",
        opt.max_unobserved_delta_c,
        uniform.max_unobserved_delta_c
    );
}

#[test]
fn test_digital_twin_spatial_reconstruction_and_anomalies() {
    let mut mesh = SensorMesh::new_heterogeneous_soc();
    let mut twin = DigitalTwinModel::default();

    twin.update(&mesh, 2000.0);
    assert!(twin.reconstructed_peak_temp_c > 45.0);
    assert!(twin.silicon_health_score_pct > 80.0);
    assert!(twin.projected_rul_hours > 10_000.0);

    // Inject sensor fault: sensor 0 value stuck at 0.0 while true value is ~70 C
    mesh.sensors[0].value = 0.0;
    mesh.sensors[0].true_value = 72.0;

    twin.update(&mesh, 2010.0);
    assert!(
        !twin.active_anomalies.is_empty(),
        "Digital twin should detect stuck/deviating sensor anomaly"
    );
}

#[test]
fn test_co_simulator_full_run_and_reporting() {
    let mut sim = SiliconLifecycleCoSimulator::new_fast();
    assert!(sim.latest_report.is_none());

    let report = sim.run_full_analysis();
    assert!(report.total_sensors > 0);
    assert!(report.peak_true_temperature_c > 40.0);
    assert!(report.silicon_health_score_pct > 70.0);
    assert!(sim.latest_report.is_some());

    // Switch protocol and verify telemetry stream updates
    sim.set_protocol(ProtocolType::PcieMctpPldm);
    sim.step_simulation();
    assert_eq!(sim.telemetry.active_protocol, ProtocolType::PcieMctpPldm);
}
