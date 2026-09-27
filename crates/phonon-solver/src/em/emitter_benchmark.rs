//! High-Throughput Parallel Emitter Synthesis & Transceiver Link Benchmark
//!
//! Validates first-principles discrete radio emitter synthesis and antenna transduction
//! across 10,000 parallel test cases using Rayon multi-threading.

use crate::em::rf_transceiver_solver::RfTransceiverSolver;
use phonon_models::em::{
    AmplifierClass, AntennaGeometry, DiscreteTransmitter, OscillatorType, PhysicalAntenna,
    Polarization, RfPowerAmplifier, Vector3D,
};
use rayon::prelude::*;
use std::time::Instant;

/// Report from executing the discrete emitter and transceiver link benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct EmitterBenchmarkReport {
    /// Total discrete RF emitter configurations synthesized.
    pub total_emitters_synthesized: usize,
    /// Total transceiver link budgets evaluated.
    pub total_transceiver_links_evaluated: usize,
    /// Total elapsed wall-clock computation time in milliseconds.
    pub elapsed_millis: f64,
    /// Overall throughput in evaluations per second.
    pub throughput_evaluations_per_sec: f64,
    /// Average relative discrepancy percentage against Friis transmission law.
    pub average_friis_error_percent: f64,
    /// Maximum observed relative discrepancy percentage against Friis transmission law.
    pub max_friis_error_percent: f64,
    /// Flag indicating whether all assertions and physical energy conservation constraints passed.
    pub all_valid: bool,
}

/// Benchmark runner for parallel discrete RF emitters.
#[derive(Debug, Clone, Default)]
pub struct EmitterBenchmarkRunner;

impl EmitterBenchmarkRunner {
    /// Creates a new emitter benchmark runner.
    pub fn new() -> Self {
        Self
    }

    /// Executes the 10,000-case parallel synthesis and transceiver link benchmark.
    pub fn run_benchmark(&self, num_emitters: usize, num_links: usize) -> EmitterBenchmarkReport {
        let n_tx = num_emitters.max(100);
        let n_links = num_links.max(100);

        let start_time = Instant::now();

        // 1. Parallel Synthesis of N Discrete RF Transmitters
        let transmitters: Vec<DiscreteTransmitter> = (0..n_tx)
            .into_par_iter()
            .map(|i| {
                let freq_tier = i % 5;
                let (freq_hz, osc, ant_geom, pol) = match freq_tier {
                    0 => {
                        // 433 MHz ISM Band Crystal Oscillator
                        let f = 433.92e6;
                        let lm = 15.0e-3;
                        let omega = 2.0 * std::f64::consts::PI * f;
                        let cm = 1.0 / (omega * omega * lm);
                        (
                            f,
                            OscillatorType::Crystal {
                                motional_inductance_h: lm,
                                motional_capacitance_f: cm,
                                motional_resistance_ohms: 20.0,
                                shunt_capacitance_f: 4.0e-12,
                                drive_level_watts: 0.001,
                            },
                            AntennaGeometry::Monopole {
                                height_m: 0.165,
                                wire_radius_m: 0.001,
                                conductivity: 5.8e7,
                            },
                            Polarization::LinearVertical,
                        )
                    }
                    1 => {
                        // 915 MHz US ISM Band Colpitts Oscillator
                        let f = 915.0e6;
                        (
                            f,
                            OscillatorType::Colpitts {
                                inductance_h: 8.0e-9,
                                c1_f: 7.5e-12,
                                c2_f: 7.5e-12,
                                transconductance_s: 0.04,
                                load_resistance_ohms: 50.0,
                                bias_current_a: 0.015,
                            },
                            AntennaGeometry::Dipole {
                                length_m: 0.155,
                                wire_radius_m: 0.0008,
                                conductivity: 5.8e7,
                            },
                            Polarization::LinearVertical,
                        )
                    }
                    2 => {
                        // 2.45 GHz Wi-Fi Colpitts with Microstrip Patch
                        let f = 2.45e9;
                        (
                            f,
                            OscillatorType::Colpitts {
                                inductance_h: 2.108e-9,
                                c1_f: 4.0e-12,
                                c2_f: 4.0e-12,
                                transconductance_s: 0.05,
                                load_resistance_ohms: 50.0,
                                bias_current_a: 0.02,
                            },
                            AntennaGeometry::MicrostripPatch {
                                length_m: 0.029,
                                width_m: 0.038,
                                substrate_er: 4.4,
                                substrate_height_m: 0.0016,
                                conductivity: 5.8e7,
                            },
                            Polarization::LinearHorizontal,
                        )
                    }
                    3 => {
                        // 5.8 GHz UNII Band LC Tank with Pyramidal Horn
                        let f = 5.8e9;
                        (
                            f,
                            OscillatorType::LCTank {
                                inductance_h: 1.0e-9,
                                capacitance_f: 0.75e-12,
                                series_resistance_ohms: 0.5,
                                peak_voltage_v: 2.0,
                            },
                            AntennaGeometry::PyramidalHorn {
                                aperture_a_m: 0.08,
                                aperture_b_m: 0.06,
                                axial_length_m: 0.12,
                                aperture_efficiency: 0.55,
                            },
                            Polarization::LinearVertical,
                        )
                    }
                    _ => {
                        // 28 GHz mmWave 2D Phased Array
                        let f = 28.0e9;
                        let lambda = 299_792_458.0 / f;
                        (
                            f,
                            OscillatorType::LCTank {
                                inductance_h: 0.2e-9,
                                capacitance_f: 0.16e-12,
                                series_resistance_ohms: 1.0,
                                peak_voltage_v: 1.2,
                            },
                            AntennaGeometry::PhasedArray2D {
                                num_elements_x: 8,
                                num_elements_y: 8,
                                spacing_x_m: lambda * 0.5,
                                spacing_y_m: lambda * 0.5,
                                steer_theta_rad: 0.2,
                                steer_phi_rad: 0.5,
                            },
                            Polarization::LinearVertical,
                        )
                    }
                };

                let pa = RfPowerAmplifier {
                    class: AmplifierClass::ClassAB,
                    gain_db: 20.0,
                    p1db_dbm: 23.0,
                    psat_dbm: 25.0,
                    supply_voltage_v: 3.3,
                    max_pae: 0.45,
                };

                let antenna = PhysicalAntenna::new(format!("Antenna_{i}"), freq_hz, ant_geom, pol);

                let x = (i as f64) * 5.0;
                let y = ((i % 50) as f64) * 3.0;
                let z = 1.5;

                DiscreteTransmitter::new(
                    format!("TX_{i}"),
                    Vector3D::new(x, y, z),
                    osc,
                    pa,
                    antenna,
                )
            })
            .collect();

        // 2. Parallel Evaluation of N Transceiver Links
        let solver = RfTransceiverSolver::new();

        let link_results: Vec<(f64, bool)> = (0..n_links)
            .into_par_iter()
            .map(|j| {
                let tx_idx = j % transmitters.len();
                let tx = &transmitters[tx_idx];

                // Receiver with matched antenna polarization and type
                let rx_antenna = tx.antenna.clone();
                let dist = 10.0 + ((j % 200) as f64) * 5.0; // 10 m to 1000 m
                let boresight_dir = match &tx.antenna.geometry {
                    AntennaGeometry::Dipole { .. } | AntennaGeometry::Monopole { .. } => {
                        Vector3D::new(1.0, 0.0, 0.0)
                    }
                    AntennaGeometry::MicrostripPatch { .. }
                    | AntennaGeometry::PyramidalHorn { .. }
                    | AntennaGeometry::ParabolicDish { .. } => Vector3D::new(0.0, 0.0, 1.0),
                    AntennaGeometry::PhasedArray2D {
                        steer_theta_rad,
                        steer_phi_rad,
                        ..
                    } => Vector3D::new(
                        steer_theta_rad.sin() * steer_phi_rad.cos(),
                        steer_theta_rad.sin() * steer_phi_rad.sin(),
                        steer_theta_rad.cos(),
                    ),
                    AntennaGeometry::Isotropic => Vector3D::new(1.0, 0.0, 0.0),
                };
                let rx_pos = tx.position + boresight_dir * dist;

                let r_ant =
                    rx_antenna.radiation_resistance_ohms() + rx_antenna.loss_resistance_ohms();
                let res = solver.solve_transceiver_link(
                    tx,
                    &rx_antenna,
                    rx_pos,
                    r_ant, // Matched load impedance
                    3.5,
                    20.0e6,
                );

                let err_percent = res.friis_discrepancy_ratio * 100.0;
                let valid = res.received_power_watts > 0.0
                    && res.incident_e_field_rms_v_per_m > 0.0
                    && res.snr_db.is_finite()
                    && err_percent < 5.0; // Electrodynamic link matches Friis within 5%

                (err_percent, valid)
            })
            .collect();

        let elapsed = start_time.elapsed().as_secs_f64() * 1000.0;
        let total_evals = n_tx + n_links;
        let throughput = if elapsed > 0.0 {
            (total_evals as f64) / (elapsed * 0.001)
        } else {
            0.0
        };

        let mut sum_err = 0.0;
        let mut max_err = 0.0;
        let mut all_valid = true;

        for (err, valid) in link_results {
            sum_err += err;
            if err > max_err {
                max_err = err;
            }
            if !valid {
                all_valid = false;
            }
        }

        let avg_err = if n_links > 0 {
            sum_err / (n_links as f64)
        } else {
            0.0
        };

        EmitterBenchmarkReport {
            total_emitters_synthesized: n_tx,
            total_transceiver_links_evaluated: n_links,
            elapsed_millis: elapsed,
            throughput_evaluations_per_sec: throughput,
            average_friis_error_percent: avg_err,
            max_friis_error_percent: max_err,
            all_valid,
        }
    }
}
