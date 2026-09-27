//! First-Principles Discrete RF Transceiver & Antenna Transduction Solver
//!
//! Models end-to-end discrete radio links from circuit oscillators and power amplifiers
//! through physical antenna transducers, 3D space propagation, polarization coupling,
//! to receiving antenna terminal load voltages and Friis transmission law verification.

use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::em::{
    DiscreteTransmitter, PhysicalAntenna, Vector3D, BOLTZMANN_CONSTANT, REFERENCE_TEMP_K,
    VACUUM_IMPEDANCE,
};
use std::f64::consts::PI;

/// Result of an end-to-end transceiver link budget calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct TransceiverLinkResult {
    /// Separation distance between transmitter and receiver in meters.
    pub distance_m: f64,
    /// Electromagnetic propagation delay $\tau = d / c$ in seconds.
    pub propagation_delay_s: f64,
    /// RF power delivered into the transmitting antenna in Watts.
    pub tx_power_watts: f64,
    /// Transmitting antenna directivity gain along line-of-sight (linear scale).
    pub tx_gain_linear: f64,
    /// Receiving antenna directivity gain along line-of-sight (linear scale).
    pub rx_gain_linear: f64,
    /// Polarization mismatch factor $M_{pol} \in [0.0, 1.0]$.
    pub polarization_mismatch_factor: f64,
    /// Free-Space Path Loss (FSPL) in linear scale.
    pub fspl_linear: f64,
    /// Free-Space Path Loss (FSPL) in dB.
    pub fspl_db: f64,
    /// Incident electric field RMS amplitude at the receiving antenna in V/m.
    pub incident_e_field_rms_v_per_m: f64,
    /// Induced open-circuit terminal voltage $V_{oc} = E_{inc} \cdot h_{eff}$ in Volts.
    pub induced_open_circuit_voltage_v: f64,
    /// Received RF power delivered to receiver load resistance in Watts.
    pub received_power_watts: f64,
    /// Received RF power in dBm.
    pub received_power_dbm: f64,
    /// Analytical Friis transmission formula power $P_r = P_t G_t G_r (\lambda / 4\pi d)^2 M_{pol}$ in Watts.
    pub analytical_friis_power_watts: f64,
    /// Discrepancy ratio $|P_{rx} - P_{friis}| / P_{friis}$.
    pub friis_discrepancy_ratio: f64,
    /// Receiver thermal noise floor in dBm ($k_B T_{sys} B$).
    pub noise_floor_dbm: f64,
    /// Signal-to-Noise Ratio (SNR) in dB.
    pub snr_db: f64,
}

/// Transient RF burst packet waveform simulation result.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientBurstResult {
    /// Sample timestamps in seconds.
    pub time_points_s: Vec<f64>,
    /// Instantaneous transmitter antenna current $I_{tx}(t)$ in Amperes.
    pub tx_current_waveform_a: Vec<f64>,
    /// Instantaneous received load voltage $V_{rx}(t)$ in Volts.
    pub rx_voltage_waveform_v: Vec<f64>,
    /// Channel propagation delay $\tau = d / c$ in seconds.
    pub propagation_delay_s: f64,
    /// Peak envelope voltage at receiver load in Volts.
    pub peak_received_voltage_v: f64,
}

/// Solver for physical transceiver links and continuous antenna transduction.
#[derive(Debug, Clone, Default)]
pub struct RfTransceiverSolver;

impl RfTransceiverSolver {
    /// Creates a new RF transceiver solver.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates an end-to-end discrete RF link between a transmitter and a receiving antenna.
    pub fn solve_transceiver_link(
        &self,
        transmitter: &DiscreteTransmitter,
        rx_antenna: &PhysicalAntenna,
        rx_position: Vector3D,
        rx_load_impedance_ohms: f64,
        rx_noise_figure_db: f64,
        channel_bandwidth_hz: f64,
    ) -> TransceiverLinkResult {
        let displacement = rx_position - transmitter.position;
        let distance = displacement.magnitude().max(0.001);
        let prop_delay = distance / SPEED_OF_LIGHT;
        let los_dir = displacement.normalize();

        // Carrier frequency and wavelength
        let freq_hz = transmitter.carrier_frequency_hz();
        let lambda = SPEED_OF_LIGHT / freq_hz;

        // 1. Transmitter radiation angles (theta, phi) from transmitter coordinate frame
        // Theta: angle from +Z
        let theta_tx = los_dir.z.clamp(-1.0, 1.0).acos();
        let phi_tx = los_dir.y.atan2(los_dir.x);
        let phi_tx = if phi_tx < 0.0 {
            phi_tx + 2.0 * PI
        } else {
            phi_tx
        };

        let tx_gain = transmitter.antenna.gain_linear(theta_tx, phi_tx).max(1e-6);
        let p_tx = transmitter.power_into_antenna_watts().max(1e-12);

        // 2. Receiver arrival gain along arrival line-of-sight
        let arrival_dir = -los_dir;
        // Directional antennas (Patch, Horn, Dish, PhasedArray) are aligned pointing towards the incoming wave.
        // Wire antennas (Dipole, Monopole) are vertically oriented along Z axis.
        let rx_gain = match &rx_antenna.geometry {
            phonon_models::em::AntennaGeometry::Dipole { .. }
            | phonon_models::em::AntennaGeometry::Monopole { .. } => {
                let theta_rx = arrival_dir.z.clamp(-1.0, 1.0).acos();
                let phi_rx = arrival_dir.y.atan2(arrival_dir.x);
                let phi_rx = if phi_rx < 0.0 {
                    phi_rx + 2.0 * PI
                } else {
                    phi_rx
                };
                rx_antenna.gain_linear(theta_rx, phi_rx)
            }
            phonon_models::em::AntennaGeometry::Isotropic => 1.0,
            _ => {
                // Directional aperture antenna pointed towards transmitter (boresight arrival)
                rx_antenna.max_gain_linear()
            }
        }
        .max(1e-6);

        // 3. Polarization mismatch
        let pol_factor = transmitter
            .antenna
            .polarization
            .mismatch_factor(&rx_antenna.polarization);

        // 4. Free space path loss
        let fspl_lin = (4.0 * PI * distance / lambda).powi(2);
        let fspl_db = 10.0 * fspl_lin.log10();

        // 5. Analytical Friis transmission formula
        let p_friis = (p_tx * tx_gain * rx_gain / fspl_lin) * pol_factor;

        // 6. First-principles electrodynamic induction:
        // Poynting flux at receiver: S = P_tx * G_tx / (4 * pi * d^2)
        let s_inc = (p_tx * tx_gain) / (4.0 * PI * distance * distance);
        // Incident electric field RMS: E = sqrt(eta_0 * S)
        let e_inc = (VACUUM_IMPEDANCE * s_inc).sqrt();

        // Receiver input impedance: R_ant = R_rad + R_loss
        let r_ant = rx_antenna.radiation_resistance_ohms() + rx_antenna.loss_resistance_ohms();

        // Directed effective length of RX antenna along arrival vector
        let h_eff = (r_ant * rx_gain * lambda * lambda / (PI * VACUUM_IMPEDANCE)).sqrt();
        let v_oc = e_inc * h_eff * pol_factor.sqrt();
        let r_load = rx_load_impedance_ohms.max(0.1);

        // Terminal circuit voltage divider: V_load = V_oc * (R_load / (R_ant + R_load))
        let v_load = v_oc * (r_load / (r_ant + r_load));
        // Power dissipated in load: P_load = V_load^2 / R_load
        let p_rx_load = (v_load * v_load) / r_load;

        let p_rx_dbm = 10.0 * (p_rx_load * 1000.0).max(1e-20).log10();

        // Circuit impedance mismatch factor M_imp = 4 * R_ant * R_load / (R_ant + R_load)^2
        let impedance_factor = 4.0 * r_ant * r_load / (r_ant + r_load).powi(2);
        let p_friis_expected = p_friis * impedance_factor;

        // Discrepancy comparison against Friis
        let discrepancy = if p_friis_expected > 0.0 {
            (p_rx_load - p_friis_expected).abs() / p_friis_expected
        } else {
            0.0
        };

        // 7. Receiver noise floor and SNR
        let f_linear = 10.0_f64.powf(rx_noise_figure_db / 10.0);
        let t_sys = REFERENCE_TEMP_K * f_linear;
        let bw = channel_bandwidth_hz.max(1.0);
        let p_noise_watts = BOLTZMANN_CONSTANT * t_sys * bw;
        let p_noise_dbm = 10.0 * (p_noise_watts * 1000.0).max(1e-25).log10();
        let snr_db = p_rx_dbm - p_noise_dbm;

        TransceiverLinkResult {
            distance_m: distance,
            propagation_delay_s: prop_delay,
            tx_power_watts: p_tx,
            tx_gain_linear: tx_gain,
            rx_gain_linear: rx_gain,
            polarization_mismatch_factor: pol_factor,
            fspl_linear: fspl_lin,
            fspl_db,
            incident_e_field_rms_v_per_m: e_inc,
            induced_open_circuit_voltage_v: v_oc,
            received_power_watts: p_rx_load,
            received_power_dbm: p_rx_dbm,
            analytical_friis_power_watts: p_friis,
            friis_discrepancy_ratio: discrepancy,
            noise_floor_dbm: p_noise_dbm,
            snr_db,
        }
    }

    /// Simulates transient RF burst packet transmission, synthesizing time-domain antenna currents and received voltages.
    pub fn simulate_transient_burst(
        &self,
        transmitter: &DiscreteTransmitter,
        rx_antenna: &PhysicalAntenna,
        rx_position: Vector3D,
        pulse_duration_s: f64,
        num_samples: usize,
    ) -> TransientBurstResult {
        let displacement = rx_position - transmitter.position;
        let distance = displacement.magnitude().max(0.001);
        let prop_delay = distance / SPEED_OF_LIGHT;

        let link =
            self.solve_transceiver_link(transmitter, rx_antenna, rx_position, 50.0, 3.0, 20.0e6);

        let n = num_samples.max(100);
        let total_sim_time = pulse_duration_s + prop_delay * 1.5;
        let dt = total_sim_time / (n as f64);

        let freq_hz = transmitter.carrier_frequency_hz();
        let omega = 2.0 * PI * freq_hz;
        let i_tx_peak = transmitter.antenna_terminal_current_a();

        // Effective load peak voltage: V_load_peak = sqrt(2 * P_rx * R_load)
        let v_rx_peak = (2.0 * link.received_power_watts * 50.0).sqrt();

        let mut time_points = Vec::with_capacity(n);
        let mut tx_current = Vec::with_capacity(n);
        let mut rx_voltage = Vec::with_capacity(n);

        for step in 0..n {
            let t = (step as f64) * dt;
            time_points.push(t);

            // TX current is active during [0, pulse_duration] with smooth Hann ramp-up/ramp-down
            let i_t = if t <= pulse_duration_s {
                let envelope = 0.5 * (1.0 - (2.0 * PI * t / pulse_duration_s).cos());
                i_tx_peak * envelope * (omega * t).cos()
            } else {
                0.0
            };
            tx_current.push(i_t);

            // RX voltage is delayed by prop_delay
            let t_delayed = t - prop_delay;
            let v_t = if t_delayed >= 0.0 && t_delayed <= pulse_duration_s {
                let envelope = 0.5 * (1.0 - (2.0 * PI * t_delayed / pulse_duration_s).cos());
                v_rx_peak * envelope * (omega * t_delayed).cos()
            } else {
                0.0
            };
            rx_voltage.push(v_t);
        }

        TransientBurstResult {
            time_points_s: time_points,
            tx_current_waveform_a: tx_current,
            rx_voltage_waveform_v: rx_voltage,
            propagation_delay_s: prop_delay,
            peak_received_voltage_v: v_rx_peak,
        }
    }
}
