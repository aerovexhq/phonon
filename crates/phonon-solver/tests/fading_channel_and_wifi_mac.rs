//! Integration Tests for Multipath Fading Channels, Delay Spread & Wi-Fi CSMA/CA MAC

use phonon_models::em::{
    ChannelProfile, ChannelRng, CsmaCaConfig, CsmaCaStation, FadingChannel, FrameType, MacAddress,
    MacFrame, StationState,
};

#[test]
fn test_channel_rng_gaussian_and_rayleigh_distributions() {
    let mut rng = ChannelRng::new(9999);
    let n = 20_000;

    let mut sum_g = 0.0;
    let mut sum_sq_g = 0.0;
    for _ in 0..n {
        let (g0, _) = rng.next_gaussian();
        sum_g += g0;
        sum_sq_g += g0 * g0;
    }
    let mean_g = sum_g / (n as f64);
    let var_g = (sum_sq_g / (n as f64)) - mean_g * mean_g;

    // Standard Normal: E[X] = 0, Var(X) = 1
    assert!(
        mean_g.abs() < 0.05,
        "Gaussian mean {} deviated from 0",
        mean_g
    );
    assert!(
        (var_g - 1.0).abs() < 0.05,
        "Gaussian variance {} deviated from 1",
        var_g
    );

    // Rayleigh distribution: with sigma = 1.0, mean = sqrt(pi / 2) approx 1.2533
    let mut sum_r = 0.0;
    for _ in 0..n {
        sum_r += rng.next_rayleigh(1.0);
    }
    let mean_r = sum_r / (n as f64);
    let expected_mean = (std::f64::consts::PI * 0.5).sqrt();
    assert!(
        (mean_r - expected_mean).abs() < 0.05,
        "Rayleigh mean {} deviated from {}",
        mean_r,
        expected_mean
    );
}

#[test]
fn test_indoor_multipath_profiles_delay_spread_and_coherence_bandwidth() {
    // Model B: Residential indoor (~15 ns RMS delay spread)
    let chan_b = FadingChannel::new(ChannelProfile::IndoorModelB, 5.0, 5.2e9);
    let sigma_b = chan_b.rms_delay_spread();
    assert!(
        sigma_b > 5.0e-9 && sigma_b < 25.0e-9,
        "Model B sigma {} out of range",
        sigma_b
    );
    let bc_50_b = chan_b.coherence_bandwidth_50();
    assert!(
        bc_50_b > 5.0e6,
        "Model B coherence bandwidth {} too narrow",
        bc_50_b
    );

    // Model D: Open office indoor (~50 ns RMS delay spread)
    let chan_d = FadingChannel::new(ChannelProfile::IndoorModelD, 5.0, 5.2e9);
    let sigma_d = chan_d.rms_delay_spread();
    assert!(
        sigma_d > 25.0e-9 && sigma_d < 70.0e-9,
        "Model D sigma {} out of range",
        sigma_d
    );
    let bc_50_d = chan_d.coherence_bandwidth_50();
    // Greater delay spread implies narrower coherence bandwidth
    assert!(bc_50_d < bc_50_b);

    // Doppler coherence time for 5 Hz Doppler: Tc approx 9 / (16*pi*5) approx 35.8 ms
    let tc = chan_b.coherence_time();
    assert!(
        (tc - 0.0358).abs() < 0.005,
        "Coherence time {} deviated",
        tc
    );
}

#[test]
fn test_ieee_crc32_and_mac_frame_construction() {
    let payload = b"Phonon High-Fidelity Wi-Fi Transmission Test Vector".to_vec();
    let ta = MacAddress([0x00, 0x14, 0x22, 0x01, 0x23, 0x45]);
    let ra = MacAddress([0x00, 0x14, 0x22, 0x99, 0x88, 0x77]);
    let bssid = ra;

    let frame = MacFrame::new_data(ta, ra, bssid, 42, payload.clone());

    assert_eq!(frame.frame_type, FrameType::Data);
    assert_eq!(frame.ra, ra);
    assert_eq!(frame.ta, ta);
    assert!(
        frame.verify_fcs(),
        "FCS failed to verify on newly constructed frame"
    );

    // Intentionally corrupt payload and verify FCS catches corruption
    let mut corrupt_frame = frame.clone();
    corrupt_frame.payload[0] ^= 0x01;
    assert!(
        !corrupt_frame.verify_fcs(),
        "FCS failed to detect single-bit corruption"
    );

    // Verify ACK frame
    let ack = MacFrame::new_ack(ta, 0);
    assert_eq!(ack.frame_type, FrameType::Ack);
    assert!(ack.verify_fcs());
}

#[test]
fn test_csma_ca_state_machine_and_exponential_backoff() {
    let ta = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    let ra = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x66]);
    let cfg = CsmaCaConfig::wifi_802_11a();
    let cw_min = cfg.cw_min; // 15
    let difs = cfg.difs_us; // 34 us

    let mut station = CsmaCaStation::new(ta, ra, cfg);
    assert_eq!(station.state, StationState::Idle);
    assert_eq!(station.current_cw, cw_min);

    // Enqueue packet
    station.enqueue_data(ra, b"Hello Packet 1".to_vec());
    assert_eq!(station.tx_queue.len(), 1);

    // Step with channel idle: transition to DifsWait
    let tx_frame = station.step(1.0, -85.0, 7);
    assert!(tx_frame.is_none());
    assert!(matches!(station.state, StationState::DifsWait { .. }));

    // Advance through remainder of DIFS
    let mut advanced_us = 1.0;
    let mut frame_out = None;
    while advanced_us <= difs + 10.0 {
        if let Some(f) = station.step(2.0, -85.0, 7) {
            frame_out = Some(f);
            break;
        }
        advanced_us += 2.0;
    }

    assert!(
        frame_out.is_some(),
        "Station did not transmit frame after DIFS deferral"
    );
    assert!(matches!(station.state, StationState::Transmitting { .. }));

    // Finish transmission
    let mut transmitted = false;
    for _ in 0..100 {
        station.step(5.0, -85.0, 7);
        if matches!(station.state, StationState::AwaitingAck { .. }) {
            transmitted = true;
            break;
        }
    }
    assert!(transmitted, "Station did not reach AwaitingAck state");

    // Simulate missed ACK (timeout): verify exponential backoff triggers
    // AwaitingAck timeout is ~100 us
    let mut retried = false;
    for _ in 0..100 {
        station.step(5.0, -85.0, 7);
        if matches!(station.state, StationState::Backoff { .. }) {
            retried = true;
            break;
        }
    }
    assert!(retried, "Station did not enter Backoff after ACK timeout");
    // Contention window doubled: 15 -> 31
    assert_eq!(station.current_cw, 31);
    assert_eq!(station.total_retries, 1);
}
