//! End-to-End Wi-Fi & SDR Physical/MAC Protocol Simulation Solver
//!
//! Bridges digital bitstream packets, OFDM modulation, RF channel degradation across
//! abstraction tiers, and CSMA/CA clear-channel assessment with CRC-32 verification.

use crate::em::rf_tier_engine::{
    LinkEvaluationParams, RfRealismTier, RfTierEngine, TierChannelResult,
};
use phonon_models::em::{
    ChannelRng, CsmaCaConfig, CsmaCaStation, DielectricWall, FadingChannel, MacAddress, MacFrame,
    ModulationScheme, OfdmConfig, PhysicalAntenna, RfNoiseModel, Vector3D,
};

/// Result of an end-to-end Wi-Fi packet transmission attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct PacketTransmissionResult {
    /// True if packet was received without bit errors and passed CRC-32 FCS check.
    pub success: bool,
    /// True if the 32-bit Frame Check Sequence (FCS) matched payload exactly.
    pub fcs_valid: bool,
    /// Total data bits transmitted in frame body.
    pub total_bits: usize,
    /// Number of bit decision errors encountered after demodulation.
    pub bit_errors: usize,
    /// Measured empirical Bit Error Rate (BER).
    pub empirical_ber: f64,
    /// Theoretical Bit Error Rate for the given modulation and SNR under AWGN.
    pub theoretical_ber: f64,
    /// Channel evaluation summary for this transmission.
    pub channel_result: TierChannelResult,
    /// Frame airtime duration in microseconds.
    pub airtime_us: f64,
    /// Effective PHY throughput in Megabits per second (Mbps).
    pub effective_throughput_mbps: f64,
    /// Transmission retries expended.
    pub retry_count: u32,
}

/// End-to-end Wi-Fi Link Simulator coupling transmitter, physical channel, and receiver.
#[derive(Debug, Clone)]
pub struct WifiLinkSimulator {
    /// Transmitting CSMA/CA Station.
    pub transmitter: CsmaCaStation,
    /// Receiving CSMA/CA Station.
    pub receiver: CsmaCaStation,
    /// 3D position of transmitter in meters.
    pub tx_position: Vector3D,
    /// 3D position of receiver in meters.
    pub rx_position: Vector3D,
    /// Transmitter antenna transducer.
    pub tx_antenna: PhysicalAntenna,
    /// Receiver antenna transducer.
    pub rx_antenna: PhysicalAntenna,
    /// Transmitter RF power output in Watts (e.g. 0.1 W = 20 dBm).
    pub tx_power_watts: f64,
    /// Obstacle dielectric walls.
    pub walls: Vec<DielectricWall>,
    /// Thermal and environmental RF noise model.
    pub noise_model: RfNoiseModel,
    /// Multipath fading channel profile.
    pub fading_channel: FadingChannel,
    /// Multi-tier RF realism abstraction engine.
    pub tier_engine: RfTierEngine,
    /// OFDM physical layer framing configuration.
    pub ofdm_config: OfdmConfig,
}

impl WifiLinkSimulator {
    /// Constructs a standard 5.2 GHz Wi-Fi link simulator between dual stations.
    pub fn new(
        tx_mac: MacAddress,
        rx_mac: MacAddress,
        tx_pos: Vector3D,
        rx_pos: Vector3D,
        modulation: ModulationScheme,
    ) -> Self {
        let bssid = rx_mac; // AP MAC is receiver
        let mac_cfg = CsmaCaConfig::wifi_802_11a();
        let transmitter = CsmaCaStation::new(tx_mac, bssid, mac_cfg.clone());
        let receiver = CsmaCaStation::new(rx_mac, bssid, mac_cfg);

        let freq_hz = 5.2e9; // 5.2 GHz
        let tx_antenna = PhysicalAntenna::quarter_wave_monopole("TxMonopole", freq_hz);
        let rx_antenna = PhysicalAntenna::quarter_wave_monopole("RxMonopole", freq_hz);

        let ofdm_config = OfdmConfig::wifi_802_11a_20mhz(modulation, 0.75); // 3/4 coding rate
        let fading_channel = FadingChannel::default();
        let noise_model = RfNoiseModel::new(20.0e6, 5.0); // 20 MHz, 5 dB NF

        Self {
            transmitter,
            receiver,
            tx_position: tx_pos,
            rx_position: rx_pos,
            tx_antenna,
            rx_antenna,
            tx_power_watts: 0.1, // 100 mW (+20 dBm)
            walls: Vec::new(),
            noise_model,
            fading_channel,
            tier_engine: RfTierEngine::new(RfRealismTier::Tier1RaytracedMultipath),
            ofdm_config,
        }
    }

    /// Sets the active realism tier.
    pub fn set_tier(&mut self, tier: RfRealismTier) {
        self.tier_engine.active_tier = tier;
    }

    /// Adds a dielectric obstacle wall to the propagation environment.
    pub fn add_wall(&mut self, wall: DielectricWall) {
        self.walls.push(wall);
    }

    /// Simulates a single packet transmission from transmitter to receiver.
    ///
    /// Transmits raw payload bytes, modulates into I/Q symbols, degrades over RF channel,
    /// demodulates, counts bit errors, and evaluates CRC-32 Frame Check Sequence.
    pub fn transmit_payload(
        &mut self,
        payload: &[u8],
        rng: &mut ChannelRng,
    ) -> PacketTransmissionResult {
        // 1. Construct IEEE 802.11 MAC Data Frame with CRC-32 FCS
        let frame = MacFrame::new_data(
            self.transmitter.mac_address,
            self.receiver.mac_address,
            self.transmitter.bssid,
            self.transmitter.seq_counter,
            payload.to_vec(),
        );
        self.transmitter.seq_counter = (self.transmitter.seq_counter + 1) & 0x0FFF;

        // 2. Evaluate physical RF channel across the configured realism tier
        let params = LinkEvaluationParams {
            tier: self.tier_engine.active_tier,
            tx_pos: self.tx_position,
            rx_pos: self.rx_position,
            tx_power_watts: self.tx_power_watts,
            tx_antenna: &self.tx_antenna,
            rx_antenna: &self.rx_antenna,
            walls: &self.walls,
            noise_model: &self.noise_model,
            fading: Some(&self.fading_channel),
        };
        let chan_result = self.tier_engine.evaluate_link(&params, rng);

        // 3. Serialize frame to bitstream
        let frame_bytes = frame.serialize_header_and_payload();
        let mut tx_bits = Vec::with_capacity(frame_bytes.len() * 8 + 32);
        for &byte in &frame_bytes {
            for bit_pos in 0..8 {
                tx_bits.push((byte & (1 << bit_pos)) != 0);
            }
        }
        // Append 32-bit FCS
        for bit_pos in 0..32 {
            tx_bits.push((frame.fcs & (1 << bit_pos)) != 0);
        }

        // 4. Modulate bits to normalized I/Q complex symbols
        let mod_scheme = self.ofdm_config.modulation;
        let tx_symbols = mod_scheme.map_bits(&tx_bits);

        // 5. Apply multipath channel fading and AWGN noise scaled by computed SNR
        let (rx_symbols_raw, h_channel) =
            self.fading_channel
                .apply_channel_with_gain(&tx_symbols, chan_result.snr_db, rng);

        // OFDM pilot-assisted channel equalization: r_eq = r / h = (r * h*) / |h|^2
        let h_mag_sq = h_channel.0 * h_channel.0 + h_channel.1 * h_channel.1;
        let rx_symbols: Vec<(f64, f64)> = if h_mag_sq > 1e-12 {
            rx_symbols_raw
                .into_iter()
                .map(|(ri, rq)| {
                    let eq_i = (ri * h_channel.0 + rq * h_channel.1) / h_mag_sq;
                    let eq_q = (rq * h_channel.0 - ri * h_channel.1) / h_mag_sq;
                    (eq_i, eq_q)
                })
                .collect()
        } else {
            rx_symbols_raw
        };

        // 6. Demap received noisy symbols via minimum Euclidean distance
        let rx_bits = mod_scheme.demap_symbols(&rx_symbols);

        // 7. Measure bit errors
        let total_bits = tx_bits.len();
        let mut bit_errors = 0;
        for i in 0..total_bits.min(rx_bits.len()) {
            if tx_bits[i] != rx_bits[i] {
                bit_errors += 1;
            }
        }
        let empirical_ber = (bit_errors as f64) / (total_bits as f64).max(1.0);

        // 8. Reconstruct frame bytes from decoded bits and verify CRC-32 FCS
        let mut rx_bytes = Vec::with_capacity(frame_bytes.len());
        for byte_idx in 0..frame_bytes.len() {
            let mut byte_val = 0u8;
            for bit_pos in 0..8 {
                let bit_idx = byte_idx * 8 + bit_pos;
                if bit_idx < rx_bits.len() && rx_bits[bit_idx] {
                    byte_val |= 1 << bit_pos;
                }
            }
            rx_bytes.push(byte_val);
        }

        let mut rx_fcs = 0u32;
        let fcs_bit_offset = frame_bytes.len() * 8;
        for bit_pos in 0..32 {
            let bit_idx = fcs_bit_offset + bit_pos;
            if bit_idx < rx_bits.len() && rx_bits[bit_idx] {
                rx_fcs |= 1 << bit_pos;
            }
        }

        let calculated_fcs = phonon_models::em::compute_crc32(&rx_bytes);
        let fcs_valid = (rx_fcs == calculated_fcs) && (bit_errors == 0);
        let success = fcs_valid;

        // 9. Frame airtime & throughput calculation
        let phy_rate = self.ofdm_config.phy_data_rate_mbps();
        let airtime_us = frame.airtime_us(phy_rate, 20.0);
        let effective_throughput_mbps = if success {
            ((payload.len() * 8) as f64) / airtime_us
        } else {
            0.0
        };

        // Theoretical BER for modulation scheme
        let k = mod_scheme.bits_per_symbol() as f64;
        let snr_lin = 10.0_f64.powf(chan_result.snr_db / 10.0);
        let eb_n0_lin = snr_lin / k;
        let theoretical_ber = mod_scheme.theoretical_ber_awgn(eb_n0_lin);

        if success {
            self.transmitter.total_tx_success += 1;
            self.receiver.handle_received_frame(frame);
        } else {
            self.transmitter.total_retries += 1;
        }

        PacketTransmissionResult {
            success,
            fcs_valid,
            total_bits,
            bit_errors,
            empirical_ber,
            theoretical_ber,
            channel_result: chan_result,
            airtime_us,
            effective_throughput_mbps,
            retry_count: 0,
        }
    }
}
