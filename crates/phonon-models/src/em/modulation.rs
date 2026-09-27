//! Digital Baseband PHY Modulation & Constellation Mapping
//!
//! Provides first-principles models for digital passband/baseband modulation:
//! - BPSK, QPSK, 16-QAM, 64-QAM, and 256-QAM Gray-coded constellations
//! - I/Q symbol mapping and minimum-distance Euclidean soft/hard demapping
//! - Exact and closed-form theoretical Bit Error Rate (BER) & Symbol Error Rate (SER) models
//!   under Additive White Gaussian Noise (AWGN) and flat Rayleigh fading
//! - OFDM subcarrier framing, cyclic prefix timing, and PHY data rate computation

/// Complementary error function $\text{erfc}(x) = \frac{2}{\sqrt{\pi}} \int_x^\infty e^{-t^2} dt$.
///
/// Implemented via the high-precision Chebyshev rational approximation (Abramowitz & Stegun 7.1.26)
/// with absolute approximation error $|\epsilon| < 1.5 \times 10^{-7}$.
pub fn erfc(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    // Constants for Abramowitz & Stegun 7.1.26
    let p = 0.3275911;
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;

    let t = 1.0 / (1.0 + p * x);
    let poly = t * (a1 + t * (a2 + t * (a3 + t * (a4 + t * a5))));
    poly * (-x * x).exp()
}

/// Gaussian Q-function $Q(x) = \frac{1}{\sqrt{2\pi}} \int_x^\infty e^{-u^2/2} du = \frac{1}{2} \text{erfc}\left(\frac{x}{\sqrt{2}}\right)$.
#[inline]
pub fn q_function(x: f64) -> f64 {
    if x <= 0.0 {
        1.0 - 0.5 * erfc(-x / std::f64::consts::SQRT_2)
    } else {
        0.5 * erfc(x / std::f64::consts::SQRT_2)
    }
}

/// Digital modulation scheme archetype.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModulationScheme {
    /// Binary Phase Shift Keying (1 bit per symbol).
    Bpsk,
    /// Quadrature Phase Shift Keying (2 bits per symbol).
    Qpsk,
    /// 16-state Quadrature Amplitude Modulation (4 bits per symbol).
    Qam16,
    /// 64-state Quadrature Amplitude Modulation (6 bits per symbol).
    Qam64,
    /// 256-state Quadrature Amplitude Modulation (8 bits per symbol).
    Qam256,
}

impl ModulationScheme {
    /// Number of digital bits packed per complex symbol $k = \log_2(M)$.
    #[inline]
    pub const fn bits_per_symbol(&self) -> usize {
        match self {
            Self::Bpsk => 1,
            Self::Qpsk => 2,
            Self::Qam16 => 4,
            Self::Qam64 => 6,
            Self::Qam256 => 8,
        }
    }

    /// Constellation alphabet cardinality $M = 2^k$.
    #[inline]
    pub const fn constellation_size(&self) -> usize {
        1 << self.bits_per_symbol()
    }

    /// Normalization scaling factor ensuring average symbol energy $E_s = 1.0$.
    #[inline]
    pub fn normalization_factor(&self) -> f64 {
        match self {
            Self::Bpsk => 1.0,
            Self::Qpsk => 1.0 / std::f64::consts::SQRT_2,
            Self::Qam16 => 1.0 / (10.0_f64).sqrt(),
            Self::Qam64 => 1.0 / (42.0_f64).sqrt(),
            Self::Qam256 => 1.0 / (170.0_f64).sqrt(),
        }
    }

    /// Maps an integer symbol index $0 \le s < M$ to normalized complex in-phase and quadrature coordinates $(I, Q)$.
    pub fn symbol_to_iq(&self, symbol_idx: usize) -> (f64, f64) {
        let norm = self.normalization_factor();
        match self {
            Self::Bpsk => {
                let bit = symbol_idx & 1;
                let i = if bit == 0 { -1.0 } else { 1.0 };
                (i * norm, 0.0)
            }
            Self::Qpsk => {
                let b0 = symbol_idx & 1;
                let b1 = (symbol_idx >> 1) & 1;
                let i = if b0 == 0 { -1.0 } else { 1.0 };
                let q = if b1 == 0 { -1.0 } else { 1.0 };
                (i * norm, q * norm)
            }
            Self::Qam16 => {
                // Gray-coded 4-PAM on each axis: 00 -> -3, 01 -> -1, 11 -> +1, 10 -> +3
                let pam_map = [-3.0, -1.0, 3.0, 1.0];
                let idx_i = symbol_idx & 0x03;
                let idx_q = (symbol_idx >> 2) & 0x03;
                (pam_map[idx_i] * norm, pam_map[idx_q] * norm)
            }
            Self::Qam64 => {
                // Gray-coded 8-PAM on each axis: -7, -5, -3, -1, 1, 3, 5, 7
                let pam_map = [-7.0, -5.0, -1.0, -3.0, 7.0, 5.0, 1.0, 3.0];
                let idx_i = symbol_idx & 0x07;
                let idx_q = (symbol_idx >> 3) & 0x07;
                (pam_map[idx_i] * norm, pam_map[idx_q] * norm)
            }
            Self::Qam256 => {
                // 16-PAM on each axis: -15, -13, ..., +15
                let pam_i = ((symbol_idx & 0x0F) as f64) * 2.0 - 15.0;
                let pam_q = (((symbol_idx >> 4) & 0x0F) as f64) * 2.0 - 15.0;
                (pam_i * norm, pam_q * norm)
            }
        }
    }

    /// Maps a raw bitstream into normalized $(I, Q)$ complex symbols.
    pub fn map_bits(&self, bits: &[bool]) -> Vec<(f64, f64)> {
        let k = self.bits_per_symbol();
        let num_symbols = bits.len().div_ceil(k);
        let mut symbols = Vec::with_capacity(num_symbols);

        for chunk in bits.chunks(k) {
            let mut idx = 0;
            for (bit_pos, &bit) in chunk.iter().enumerate() {
                if bit {
                    idx |= 1 << bit_pos;
                }
            }
            symbols.push(self.symbol_to_iq(idx));
        }

        symbols
    }

    /// Demaps noisy received $(I, Q)$ symbols back to bit decisions via minimum Euclidean distance.
    pub fn demap_symbols(&self, received_symbols: &[(f64, f64)]) -> Vec<bool> {
        let k = self.bits_per_symbol();
        let m = self.constellation_size();
        let mut bits = Vec::with_capacity(received_symbols.len() * k);

        // Precompute constellation points
        let mut alphabet = Vec::with_capacity(m);
        for s in 0..m {
            alphabet.push((s, self.symbol_to_iq(s)));
        }

        for &(rx_i, rx_q) in received_symbols {
            let mut best_sym = 0;
            let mut min_dist_sq = f64::INFINITY;

            for &(s, (ci, cq)) in &alphabet {
                let di = rx_i - ci;
                let dq = rx_q - cq;
                let d2 = di * di + dq * dq;
                if d2 < min_dist_sq {
                    min_dist_sq = d2;
                    best_sym = s;
                }
            }

            for bit_pos in 0..k {
                bits.push((best_sym & (1 << bit_pos)) != 0);
            }
        }

        bits
    }

    /// Theoretical Bit Error Rate (BER) under AWGN as a function of $E_b / N_0$ (linear scale).
    pub fn theoretical_ber_awgn(&self, eb_n0_linear: f64) -> f64 {
        let eb_n0 = eb_n0_linear.max(0.0);
        match self {
            Self::Bpsk => q_function((2.0 * eb_n0).sqrt()),
            Self::Qpsk => q_function((2.0 * eb_n0).sqrt()),
            Self::Qam16 => {
                // For M-QAM: P_b ~ (4 / k) * (1 - 1/sqrt(M)) * Q(sqrt( (3*k / (M-1)) * (Eb/N0) ))
                let k = 4.0;
                let m = 16.0;
                let arg = ((3.0 * k / (m - 1.0)) * eb_n0).sqrt();
                (4.0 / k) * (1.0 - 1.0 / m.sqrt()) * q_function(arg)
            }
            Self::Qam64 => {
                let k = 6.0;
                let m = 64.0;
                let arg = ((3.0 * k / (m - 1.0)) * eb_n0).sqrt();
                (4.0 / k) * (1.0 - 1.0 / m.sqrt()) * q_function(arg)
            }
            Self::Qam256 => {
                let k = 8.0;
                let m = 256.0;
                let arg = ((3.0 * k / (m - 1.0)) * eb_n0).sqrt();
                (4.0 / k) * (1.0 - 1.0 / m.sqrt()) * q_function(arg)
            }
        }
    }

    /// Theoretical Symbol Error Rate (SER) under AWGN.
    pub fn theoretical_ser_awgn(&self, es_n0_linear: f64) -> f64 {
        let es_n0 = es_n0_linear.max(0.0);
        match self {
            Self::Bpsk => q_function((2.0 * es_n0).sqrt()),
            Self::Qpsk => {
                let q = q_function(es_n0.sqrt());
                2.0 * q - q * q
            }
            Self::Qam16 | Self::Qam64 | Self::Qam256 => {
                let m = self.constellation_size() as f64;
                let q = q_function(((3.0 / (m - 1.0)) * es_n0).sqrt());
                let factor = 1.0 - 1.0 / m.sqrt();
                4.0 * factor * q - 4.0 * factor * factor * q * q
            }
        }
    }

    /// Theoretical Bit Error Rate (BER) under flat Rayleigh fading as a function of average $E_b / N_0$ (linear).
    pub fn theoretical_ber_rayleigh(&self, avg_eb_n0_linear: f64) -> f64 {
        let gamma = avg_eb_n0_linear.max(1e-12);
        match self {
            Self::Bpsk | Self::Qpsk => {
                // Exact: P_b = 0.5 * (1 - sqrt(gamma / (1 + gamma)))
                0.5 * (1.0 - (gamma / (1.0 + gamma)).sqrt())
            }
            Self::Qam16 | Self::Qam64 | Self::Qam256 => {
                // High SNR asymptotic approximation: P_b ~ C / gamma
                let k = self.bits_per_symbol() as f64;
                let m = self.constellation_size() as f64;
                let factor = (m - 1.0) / (6.0 * k);
                (factor / gamma).clamp(0.0, 0.5)
            }
        }
    }
}

/// Orthogonal Frequency Division Multiplexing (OFDM) channel framing parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct OfdmConfig {
    /// Total FFT points / subcarriers (e.g. 64, 128, 256, 512).
    pub num_fft_subcarriers: usize,
    /// Number of active data payload subcarriers (e.g. 48 in 802.11a/g).
    pub num_data_subcarriers: usize,
    /// Number of pilot subcarriers for phase/frequency tracking (e.g. 4 in 802.11a/g).
    pub num_pilot_subcarriers: usize,
    /// Cyclic Prefix duration fraction (e.g. 0.25 for standard 800 ns guard interval).
    pub cyclic_prefix_ratio: f64,
    /// Total analog channel bandwidth in Hz (e.g. 20.0 MHz, 40.0 MHz, 80.0 MHz).
    pub channel_bandwidth_hz: f64,
    /// Forward Error Correction (FEC) convolutional / LDPC coding rate (e.g. 0.5, 0.75, 0.833).
    pub fec_coding_rate: f64,
    /// Baseband modulation scheme per data subcarrier.
    pub modulation: ModulationScheme,
}

impl OfdmConfig {
    /// Creates the standard IEEE 802.11a/g 20 MHz OFDM framing configuration.
    pub fn wifi_802_11a_20mhz(modulation: ModulationScheme, fec_rate: f64) -> Self {
        Self {
            num_fft_subcarriers: 64,
            num_data_subcarriers: 48,
            num_pilot_subcarriers: 4,
            cyclic_prefix_ratio: 0.25, // 16 samples out of 64 -> 800 ns GI
            channel_bandwidth_hz: 20.0e6,
            fec_coding_rate: fec_rate.clamp(0.1, 1.0),
            modulation,
        }
    }

    /// Creates the IEEE 802.11n/ac 40 MHz high-throughput OFDM configuration.
    pub fn wifi_802_11n_40mhz(modulation: ModulationScheme, fec_rate: f64) -> Self {
        Self {
            num_fft_subcarriers: 128,
            num_data_subcarriers: 108,
            num_pilot_subcarriers: 6,
            cyclic_prefix_ratio: 0.25,
            channel_bandwidth_hz: 40.0e6,
            fec_coding_rate: fec_rate.clamp(0.1, 1.0),
            modulation,
        }
    }

    /// Subcarrier frequency spacing $\Delta f = B / N_{FFT}$ in Hz (e.g. 312.5 kHz in 802.11a).
    #[inline]
    pub fn subcarrier_spacing_hz(&self) -> f64 {
        self.channel_bandwidth_hz / (self.num_fft_subcarriers as f64)
    }

    /// Useful FFT symbol duration $T_{FFT} = 1 / \Delta f$ in seconds (e.g. 3.2 microseconds).
    #[inline]
    pub fn fft_duration_s(&self) -> f64 {
        1.0 / self.subcarrier_spacing_hz()
    }

    /// Cyclic prefix / Guard Interval duration $T_{CP} = T_{FFT} \times \text{ratio}$ in seconds (e.g. 800 ns).
    #[inline]
    pub fn guard_interval_s(&self) -> f64 {
        self.fft_duration_s() * self.cyclic_prefix_ratio
    }

    /// Total OFDM symbol duration $T_{sym} = T_{FFT} + T_{CP}$ in seconds (e.g. 4.0 microseconds).
    #[inline]
    pub fn total_symbol_duration_s(&self) -> f64 {
        self.fft_duration_s() + self.guard_interval_s()
    }

    /// Number of raw data bits carried per OFDM symbol: $N_{DBPS} = N_{data} \times k \times R_{code}$.
    #[inline]
    pub fn data_bits_per_symbol(&self) -> f64 {
        let k = self.modulation.bits_per_symbol() as f64;
        (self.num_data_subcarriers as f64) * k * self.fec_coding_rate
    }

    /// Effective PHY Layer throughput in Megabits per second (Mbps).
    #[inline]
    pub fn phy_data_rate_mbps(&self) -> f64 {
        let bits_per_sec = self.data_bits_per_symbol() / self.total_symbol_duration_s();
        bits_per_sec * 1e-6
    }
}
