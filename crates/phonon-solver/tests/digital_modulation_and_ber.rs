//! Integration Tests for Digital Baseband PHY Modulation, Constellation Mapping & AWGN BER

use phonon_models::em::{
    erfc, q_function, ChannelProfile, ChannelRng, FadingChannel, ModulationScheme, OfdmConfig,
};

#[test]
fn test_erfc_and_q_function_analytical_precision() {
    // Q(0) = 0.5
    assert!((q_function(0.0) - 0.5).abs() < 1e-6);
    // Q(1.0) approx 0.158655
    assert!((q_function(1.0) - 0.158655).abs() < 1e-4);
    // Q(3.0) approx 0.0013499
    assert!((q_function(3.0) - 0.0013499).abs() < 1e-5);
    // erfc(0) = 1.0
    assert!((erfc(0.0) - 1.0).abs() < 1e-6);
    // erfc(1.0) approx 0.157299
    assert!((erfc(1.0) - 0.157299).abs() < 1e-4);
}

#[test]
fn test_constellation_cardinality_and_energy_normalization() {
    let schemes = [
        (ModulationScheme::Bpsk, 1, 2),
        (ModulationScheme::Qpsk, 2, 4),
        (ModulationScheme::Qam16, 4, 16),
        (ModulationScheme::Qam64, 6, 64),
        (ModulationScheme::Qam256, 8, 256),
    ];

    for (scheme, bits_per_sym, const_size) in schemes {
        assert_eq!(scheme.bits_per_symbol(), bits_per_sym);
        assert_eq!(scheme.constellation_size(), const_size);

        // Verify average symbol energy Es = E[I^2 + Q^2] approx 1.0
        let mut total_energy = 0.0;
        for s in 0..const_size {
            let (i, q) = scheme.symbol_to_iq(s);
            total_energy += i * i + q * q;
        }
        let avg_energy = total_energy / (const_size as f64);
        assert!(
            (avg_energy - 1.0).abs() < 0.05,
            "Scheme {:?} average energy {} deviated from 1.0",
            scheme,
            avg_energy
        );
    }
}

#[test]
fn test_noiseless_bit_mapping_and_demapping_fidelity() {
    let schemes = [
        ModulationScheme::Bpsk,
        ModulationScheme::Qpsk,
        ModulationScheme::Qam16,
        ModulationScheme::Qam64,
        ModulationScheme::Qam256,
    ];

    let mut rng = ChannelRng::new(42);

    for scheme in schemes {
        let k = scheme.bits_per_symbol();
        let num_bits = 480 * k; // Multiple of symbol sizes
        let test_bits: Vec<bool> = (0..num_bits).map(|_| rng.next_f64() > 0.5).collect();

        let tx_symbols = scheme.map_bits(&test_bits);
        assert_eq!(tx_symbols.len(), num_bits / k);

        // Demap without noise
        let rx_bits = scheme.demap_symbols(&tx_symbols);
        assert_eq!(rx_bits.len(), test_bits.len());

        let mut errors = 0;
        for (i, (&b_tx, &b_rx)) in test_bits.iter().zip(rx_bits.iter()).enumerate() {
            if b_tx != b_rx {
                errors += 1;
                if errors <= 5 {
                    eprintln!("Error at bit {} for {:?}", i, scheme);
                }
            }
        }
        assert_eq!(
            errors, 0,
            "Noiseless mapping/demapping for {:?} had {} errors",
            scheme, errors
        );
    }
}

#[test]
fn test_awgn_ber_waterfall_behavior() {
    let scheme = ModulationScheme::Qpsk;
    let k = scheme.bits_per_symbol();
    let num_bits = 10_000;
    let mut rng = ChannelRng::new(12345);
    let test_bits: Vec<bool> = (0..num_bits).map(|_| rng.next_f64() > 0.5).collect();
    let tx_symbols = scheme.map_bits(&test_bits);

    let channel = FadingChannel::new(ChannelProfile::Awgn, 0.0, 5.2e9);

    // Test across increasing SNR levels: 0 dB, 6 dB, 12 dB
    let snr_levels = [0.0, 6.0, 12.0];
    let mut measured_bers = Vec::new();

    for &snr_db in &snr_levels {
        let rx_symbols = channel.apply_channel(&tx_symbols, snr_db, &mut rng);
        let rx_bits = scheme.demap_symbols(&rx_symbols);

        let errors = test_bits
            .iter()
            .zip(rx_bits.iter())
            .filter(|(&a, &b)| a != b)
            .count();
        let ber = (errors as f64) / (num_bits as f64);
        measured_bers.push(ber);

        let eb_n0_lin = 10.0_f64.powf(snr_db / 10.0) / (k as f64);
        let theoretical_ber = scheme.theoretical_ber_awgn(eb_n0_lin);

        println!(
            "SNR: {:.1} dB | Measured BER: {:.4} | Theoretical: {:.4}",
            snr_db, ber, theoretical_ber
        );
    }

    // Monotonic decrease with SNR
    assert!(measured_bers[0] > measured_bers[1]);
    assert!(measured_bers[1] >= measured_bers[2]);
    // At 12 dB SNR QPSK, BER should be nearly zero (< 0.001)
    assert!(measured_bers[2] < 0.01);
}

#[test]
fn test_ofdm_channel_framing_and_phy_data_rates() {
    // IEEE 802.11a 20 MHz (64 FFT, 48 data subcarriers, 4.0 us symbol)
    let ofdm_64qam = OfdmConfig::wifi_802_11a_20mhz(ModulationScheme::Qam64, 0.75); // 64-QAM 3/4
    assert_eq!(ofdm_64qam.num_fft_subcarriers, 64);
    assert_eq!(ofdm_64qam.num_data_subcarriers, 48);
    assert!((ofdm_64qam.subcarrier_spacing_hz() - 312_500.0).abs() < 1.0);
    assert!((ofdm_64qam.fft_duration_s() - 3.2e-6).abs() < 1e-9);
    assert!((ofdm_64qam.guard_interval_s() - 0.8e-6).abs() < 1e-9);
    assert!((ofdm_64qam.total_symbol_duration_s() - 4.0e-6).abs() < 1e-9);

    // 48 subcarriers * 6 bits * 0.75 code rate = 216 data bits/symbol
    assert!((ofdm_64qam.data_bits_per_symbol() - 216.0).abs() < 1e-6);
    // 216 bits / 4 us = 54.0 Mbps (standard maximum 802.11a rate!)
    assert!((ofdm_64qam.phy_data_rate_mbps() - 54.0).abs() < 0.01);

    // BPSK 1/2 rate: 48 * 1 * 0.5 = 24 bits / 4 us = 6.0 Mbps
    let ofdm_bpsk = OfdmConfig::wifi_802_11a_20mhz(ModulationScheme::Bpsk, 0.5);
    assert!((ofdm_bpsk.phy_data_rate_mbps() - 6.0).abs() < 0.01);

    // IEEE 802.11n 40 MHz channel (128 FFT, 108 data subcarriers)
    let ofdm_n_64qam = OfdmConfig::wifi_802_11n_40mhz(ModulationScheme::Qam64, 5.0 / 6.0);
    // 108 subcarriers * 6 bits * (5/6) = 540 bits / 4 us = 135.0 Mbps
    assert!((ofdm_n_64qam.phy_data_rate_mbps() - 135.0).abs() < 0.1);
}
