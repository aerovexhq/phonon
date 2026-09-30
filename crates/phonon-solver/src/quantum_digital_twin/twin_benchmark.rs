#![deny(unsafe_code)]

//! Parallel parameter sweep benchmark suite for the Phonon Universal Multi-Scale
//! Visual Studio Quantum Digital Twin Micro-Architecture Simulator & Sub-System Co-Emulation Fabric
//! across multi-threaded Rayon workers.

use crate::quantum_digital_twin::QuantumDigitalTwinSolver;
use phonon_models::quantum_digital_twin::{
    QuantumDigitalTwinMetrics, QuantumDigitalTwinParams,
};
use rayon::prelude::*;
use std::time::Instant;

/// Benchmark outcome summary for quantum digital twin micro-architecture simulator and co-emulation sweeps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumDigitalTwinBenchmarkResult {
    pub total_cycles: usize,
    pub elapsed_seconds: f64,
    pub throughput_sweeps_per_sec: f64,
    pub mean_coemulation_fidelity: f64,
    pub min_coemulation_fidelity: f64,
    pub max_coemulation_fidelity: f64,
    pub mean_quantum_bus_state_retention_fraction: f64,
    pub min_quantum_bus_state_retention_fraction: f64,
    pub max_quantum_bus_state_retention_fraction: f64,
    pub mean_topological_protection_gap_mhz: f64,
    pub min_topological_protection_gap_mhz: f64,
    pub max_topological_protection_gap_mhz: f64,
    pub mean_inter_core_crosstalk_isolation_db: f64,
    pub min_inter_core_crosstalk_isolation_db: f64,
    pub max_inter_core_crosstalk_isolation_db: f64,
    pub mean_topological_mode_dephasing_rate_hz: f64,
    pub min_topological_mode_dephasing_rate_hz: f64,
    pub max_topological_mode_dephasing_rate_hz: f64,
    pub physical_compliance_fraction: f64,
}

/// Benchmark runner evaluating large-scale parallel parameter sweeps via Rayon.
pub struct QuantumDigitalTwinBenchmarkRunner;

impl QuantumDigitalTwinBenchmarkRunner {
    /// Executes a parallel parameter sweep across the specified number of cycles.
    pub fn run_benchmark(cycles: usize) -> QuantumDigitalTwinBenchmarkResult {
        let sweep_params: Vec<QuantumDigitalTwinParams> = (0..cycles)
            .map(|i| {
                let coemulation_coupling_mev =
                    1.0 + 34.0 * (((i * 7) % 50) as f64 / 50.0);
                let topological_emulation_gap_mev =
                    2.0 + 43.0 * (((i * 13) % 45) as f64 / 45.0);
                let acoustic_drive_frequency_ghz =
                    1.0 + 11.0 * (((i * 23) % 35) as f64 / 35.0);
                let instruction_dispatch_speed_m_per_s =
                    200.0 + 2800.0 * (((i * 17) % 50) as f64 / 50.0);
                let cryogenic_temperature_mk =
                    1.0 + 49.0 * (((i * 29) % 45) as f64 / 45.0);
                let microwave_probe_power_uw =
                    0.5 + 29.5 * (((i * 31) % 40) as f64 / 40.0);
                let synthetic_coemulation_cores_factor =
                    1.0 + 7.0 * (((i * 11) % 40) as f64 / 40.0);
                let bus_interconnect_pitch_um =
                    0.5 + 19.5 * (((i * 19) % 45) as f64 / 45.0);

                QuantumDigitalTwinParams::new(
                    coemulation_coupling_mev,
                    topological_emulation_gap_mev,
                    acoustic_drive_frequency_ghz,
                    instruction_dispatch_speed_m_per_s,
                    cryogenic_temperature_mk,
                    microwave_probe_power_uw,
                    synthetic_coemulation_cores_factor,
                    bus_interconnect_pitch_um,
                )
            })
            .collect();

        let start = Instant::now();

        let metrics: Vec<QuantumDigitalTwinMetrics> = sweep_params
            .par_iter()
            .map(|p| {
                let solver = QuantumDigitalTwinSolver::new(*p);
                solver.evaluate_metrics()
            })
            .collect();

        let elapsed = start.elapsed().as_secs_f64();
        let throughput = (cycles as f64) / elapsed.max(1.0e-9);

        let mut sum_fid = 0.0;
        let mut min_fid = f64::MAX;
        let mut max_fid = f64::MIN;

        let mut sum_ret = 0.0;
        let mut min_ret = f64::MAX;
        let mut max_ret = f64::MIN;

        let mut sum_gap = 0.0;
        let mut min_gap = f64::MAX;
        let mut max_gap = f64::MIN;

        let mut sum_iso = 0.0;
        let mut min_iso = f64::MAX;
        let mut max_iso = f64::MIN;

        let mut sum_deph = 0.0;
        let mut min_deph = f64::MAX;
        let mut max_deph = f64::MIN;

        let mut compliant_count = 0usize;

        for m in &metrics {
            sum_fid += m.coemulation_fidelity;
            if m.coemulation_fidelity < min_fid {
                min_fid = m.coemulation_fidelity;
            }
            if m.coemulation_fidelity > max_fid {
                max_fid = m.coemulation_fidelity;
            }

            sum_ret += m.quantum_bus_state_retention_fraction;
            if m.quantum_bus_state_retention_fraction < min_ret {
                min_ret = m.quantum_bus_state_retention_fraction;
            }
            if m.quantum_bus_state_retention_fraction > max_ret {
                max_ret = m.quantum_bus_state_retention_fraction;
            }

            sum_gap += m.topological_protection_gap_mhz;
            if m.topological_protection_gap_mhz < min_gap {
                min_gap = m.topological_protection_gap_mhz;
            }
            if m.topological_protection_gap_mhz > max_gap {
                max_gap = m.topological_protection_gap_mhz;
            }

            sum_iso += m.inter_core_crosstalk_isolation_db;
            if m.inter_core_crosstalk_isolation_db < min_iso {
                min_iso = m.inter_core_crosstalk_isolation_db;
            }
            if m.inter_core_crosstalk_isolation_db > max_iso {
                max_iso = m.inter_core_crosstalk_isolation_db;
            }

            sum_deph += m.topological_mode_dephasing_rate_hz;
            if m.topological_mode_dephasing_rate_hz < min_deph {
                min_deph = m.topological_mode_dephasing_rate_hz;
            }
            if m.topological_mode_dephasing_rate_hz > max_deph {
                max_deph = m.topological_mode_dephasing_rate_hz;
            }

            if m.is_physically_compliant {
                compliant_count += 1;
            }
        }

        let n = cycles as f64;
        QuantumDigitalTwinBenchmarkResult {
            total_cycles: cycles,
            elapsed_seconds: elapsed,
            throughput_sweeps_per_sec: throughput,
            mean_coemulation_fidelity: sum_fid / n,
            min_coemulation_fidelity: min_fid,
            max_coemulation_fidelity: max_fid,
            mean_quantum_bus_state_retention_fraction: sum_ret / n,
            min_quantum_bus_state_retention_fraction: min_ret,
            max_quantum_bus_state_retention_fraction: max_ret,
            mean_topological_protection_gap_mhz: sum_gap / n,
            min_topological_protection_gap_mhz: min_gap,
            max_topological_protection_gap_mhz: max_gap,
            mean_inter_core_crosstalk_isolation_db: sum_iso / n,
            min_inter_core_crosstalk_isolation_db: min_iso,
            max_inter_core_crosstalk_isolation_db: max_iso,
            mean_topological_mode_dephasing_rate_hz: sum_deph / n,
            min_topological_mode_dephasing_rate_hz: min_deph,
            max_topological_mode_dephasing_rate_hz: max_deph,
            physical_compliance_fraction: compliant_count as f64 / n,
        }
    }
}
