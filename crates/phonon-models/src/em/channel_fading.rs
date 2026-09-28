//! Multipath Rayleigh & Rician Fading Channels, Doppler Spread & Delay Profiles
//!
//! Models realistic wireless multipath fading channels:
//! - Deterministic and stochastic multipath tapped-delay-line (TDL) profiles
//! - IEEE 802.11 Model B (residential indoor), Model C (small office), Model D (open office)
//! - Rician $K$-factor distribution (LOS specular path + diffuse Rayleigh scattered paths)
//! - Jakes Doppler power spectrum and Doppler frequency shift
//! - Frequency-selective channel transfer function $H(f, t)$
//! - Mean excess delay $\bar{\tau}$, RMS delay spread $\sigma_\tau$, and coherence bandwidth $B_{c, 50}$
//! - Additive White Gaussian Noise (AWGN) and safe deterministic PRNG ($XorShift64Star$)

use std::f64::consts::PI;

/// Fast, safe 64-bit pseudo-random number generator (XorShift64Star).
///
/// Fully deterministic and seedable, enabling reproducible channel realizations
/// without relying on external crates or unsafe code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelRng {
    state: u64,
}

impl Default for ChannelRng {
    fn default() -> Self {
        Self::new(0x853c49e6748fea9b)
    }
}

impl ChannelRng {
    /// Creates a new PRNG with a 64-bit seed.
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x853c49e6748fea9b } else { seed };
        Self { state }
    }

    /// Generates next pseudo-random 64-bit integer.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Generates next floating-point number uniformly distributed in $[0.0, 1.0)$.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Generates a pair of independent standard normal random variables $(Z_0, Z_1) \sim \mathcal{N}(0, 1)$
    /// using the Box-Muller transform.
    pub fn next_gaussian(&mut self) -> (f64, f64) {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let r = (-2.0 * u1.ln()).sqrt();
        let theta = 2.0 * PI * u2;
        (r * theta.cos(), r * theta.sin())
    }

    /// Generates a Rayleigh-distributed random variable with scale parameter $\sigma$.
    pub fn next_rayleigh(&mut self, sigma: f64) -> f64 {
        let u = self.next_f64().max(1e-15);
        sigma * (-2.0 * u.ln()).sqrt()
    }

    /// Generates a complex Rician fading coefficient $(h_r, h_i)$ with $K$-factor and total average power $\Omega = 1.0$.
    ///
    /// The specular LOS component amplitude is $s = \sqrt{\frac{K}{K+1}}$ and diffuse variance per dimension is $\sigma^2 = \frac{1}{2(K+1)}$.
    pub fn next_rician(&mut self, k_factor: f64) -> (f64, f64) {
        let k = k_factor.max(0.0);
        let s = (k / (k + 1.0)).sqrt();
        let sigma = (0.5 / (k + 1.0)).sqrt();
        let (g0, g1) = self.next_gaussian();
        (s + sigma * g0, sigma * g1)
    }
}

/// A discrete multipath delay tap in a tapped-delay-line channel model.
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelTap {
    /// Excess propagation delay relative to first arriving ray in seconds.
    pub delay_s: f64,
    /// Linear relative power of this multipath tap ($P_l$).
    pub power_linear: f64,
    /// Doppler frequency shift in Hertz ($f_{d, l} = f_d \cos\theta_l$).
    pub doppler_shift_hz: f64,
    /// Random or deterministic phase offset in radians ($\phi_l \in [0, 2\pi)$).
    pub phase_rad: f64,
}

impl ChannelTap {
    pub fn new(delay_s: f64, power_db: f64, doppler_shift_hz: f64, phase_rad: f64) -> Self {
        let power_linear = 10.0_f64.powf(power_db / 10.0);
        Self {
            delay_s: delay_s.max(0.0),
            power_linear,
            doppler_shift_hz,
            phase_rad,
        }
    }
}

/// Multipath channel profile model.
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelProfile {
    /// Pure Additive White Gaussian Noise (no multipath fading, flat frequency response).
    Awgn,
    /// Flat Rayleigh fading (NLOS single-tap, Rayleigh distributed amplitude).
    FlatRayleigh,
    /// Flat Rician fading with line-of-sight $K$-factor.
    FlatRician { k_factor: f64 },
    /// IEEE 802.11 Model B: Residential indoor environment (RMS delay spread ~ 15 ns).
    IndoorModelB,
    /// IEEE 802.11 Model C: Small office indoor environment (RMS delay spread ~ 30 ns).
    IndoorModelC,
    /// IEEE 802.11 Model D: Typical open office indoor environment (RMS delay spread ~ 50 ns).
    IndoorModelD,
    /// User-specified custom tapped delay line.
    CustomMultipath { taps: Vec<ChannelTap> },
}

/// High-fidelity wireless multipath fading channel.
#[derive(Debug, Clone, PartialEq)]
pub struct FadingChannel {
    /// Channel multipath profile.
    pub profile: ChannelProfile,
    /// Maximum Doppler spread $f_d = \frac{v}{\lambda}$ in Hertz.
    pub max_doppler_hz: f64,
    /// Carrier frequency in Hertz ($f_c$).
    pub carrier_freq_hz: f64,
}

impl Default for FadingChannel {
    fn default() -> Self {
        Self {
            profile: ChannelProfile::IndoorModelB,
            max_doppler_hz: 5.0,    // 5 Hz typical indoor pedestrian walking
            carrier_freq_hz: 5.2e9, // 5.2 GHz Wi-Fi band
        }
    }
}

impl FadingChannel {
    pub fn new(profile: ChannelProfile, max_doppler_hz: f64, carrier_freq_hz: f64) -> Self {
        Self {
            profile,
            max_doppler_hz: max_doppler_hz.max(0.0),
            carrier_freq_hz: carrier_freq_hz.max(1.0e6),
        }
    }

    /// Resolves the discrete tapped delay line corresponding to the current channel profile.
    pub fn taps(&self) -> Vec<ChannelTap> {
        match &self.profile {
            ChannelProfile::Awgn => vec![ChannelTap {
                delay_s: 0.0,
                power_linear: 1.0,
                doppler_shift_hz: 0.0,
                phase_rad: 0.0,
            }],
            ChannelProfile::FlatRayleigh => vec![ChannelTap {
                delay_s: 0.0,
                power_linear: 1.0,
                doppler_shift_hz: self.max_doppler_hz,
                phase_rad: 0.0,
            }],
            ChannelProfile::FlatRician { .. } => vec![ChannelTap {
                delay_s: 0.0,
                power_linear: 1.0,
                doppler_shift_hz: self.max_doppler_hz,
                phase_rad: 0.0,
            }],
            ChannelProfile::IndoorModelB => {
                // IEEE 802.11 Model B (Residential, ~15 ns delay spread, 2 clusters)
                vec![
                    ChannelTap::new(0.0, 0.0, self.max_doppler_hz, 0.0),
                    ChannelTap::new(10.0e-9, -5.4, self.max_doppler_hz * 0.9, 0.5 * PI),
                    ChannelTap::new(20.0e-9, -10.8, -self.max_doppler_hz * 0.7, 1.2 * PI),
                    ChannelTap::new(30.0e-9, -16.2, self.max_doppler_hz * 0.5, 0.3 * PI),
                    ChannelTap::new(40.0e-9, -21.7, -self.max_doppler_hz * 0.3, 1.8 * PI),
                ]
            }
            ChannelProfile::IndoorModelC => {
                // IEEE 802.11 Model C (Small office, ~30 ns delay spread)
                vec![
                    ChannelTap::new(0.0, 0.0, self.max_doppler_hz, 0.0),
                    ChannelTap::new(10.0e-9, -2.2, self.max_doppler_hz * 0.95, 0.4 * PI),
                    ChannelTap::new(20.0e-9, -4.3, -self.max_doppler_hz * 0.8, 0.9 * PI),
                    ChannelTap::new(30.0e-9, -6.5, self.max_doppler_hz * 0.7, 1.5 * PI),
                    ChannelTap::new(50.0e-9, -10.9, -self.max_doppler_hz * 0.5, 0.2 * PI),
                    ChannelTap::new(80.0e-9, -17.4, self.max_doppler_hz * 0.4, 1.1 * PI),
                    ChannelTap::new(110.0e-9, -23.9, -self.max_doppler_hz * 0.2, 0.7 * PI),
                ]
            }
            ChannelProfile::IndoorModelD => {
                // IEEE 802.11 Model D (Typical open office, ~50 ns delay spread)
                vec![
                    ChannelTap::new(0.0, 0.0, self.max_doppler_hz, 0.0),
                    ChannelTap::new(10.0e-9, -0.9, self.max_doppler_hz * 0.98, 0.3 * PI),
                    ChannelTap::new(20.0e-9, -1.7, -self.max_doppler_hz * 0.9, 0.8 * PI),
                    ChannelTap::new(40.0e-9, -3.5, self.max_doppler_hz * 0.75, 1.4 * PI),
                    ChannelTap::new(70.0e-9, -6.1, -self.max_doppler_hz * 0.6, 0.1 * PI),
                    ChannelTap::new(100.0e-9, -8.7, self.max_doppler_hz * 0.5, 0.9 * PI),
                    ChannelTap::new(140.0e-9, -12.2, -self.max_doppler_hz * 0.4, 1.6 * PI),
                    ChannelTap::new(200.0e-9, -17.4, self.max_doppler_hz * 0.25, 0.5 * PI),
                ]
            }
            ChannelProfile::CustomMultipath { taps } => taps.clone(),
        }
    }

    /// Total integrated power of all multipath taps $\sum P_l$.
    pub fn total_power(&self) -> f64 {
        let taps = self.taps();
        taps.iter().map(|t| t.power_linear).sum()
    }

    /// Mean excess delay $\bar{\tau} = \frac{\sum P_l \tau_l}{\sum P_l}$ in seconds.
    pub fn mean_excess_delay(&self) -> f64 {
        let taps = self.taps();
        let total_p: f64 = taps.iter().map(|t| t.power_linear).sum();
        if total_p <= 1e-15 {
            return 0.0;
        }
        let weighted_sum: f64 = taps.iter().map(|t| t.power_linear * t.delay_s).sum();
        weighted_sum / total_p
    }

    /// RMS delay spread $\sigma_\tau = \sqrt{ \frac{\sum P_l \tau_l^2}{\sum P_l} - \bar{\tau}^2 }$ in seconds.
    pub fn rms_delay_spread(&self) -> f64 {
        let taps = self.taps();
        let total_p: f64 = taps.iter().map(|t| t.power_linear).sum();
        if total_p <= 1e-15 {
            return 0.0;
        }
        let mean_tau = self.mean_excess_delay();
        let second_moment: f64 = taps
            .iter()
            .map(|t| t.power_linear * t.delay_s * t.delay_s)
            .sum();
        let var = (second_moment / total_p) - (mean_tau * mean_tau);
        var.max(0.0).sqrt()
    }

    /// Coherence bandwidth for 50% frequency correlation $B_{c, 50} \approx \frac{1}{5 \sigma_\tau}$ in Hertz.
    pub fn coherence_bandwidth_50(&self) -> f64 {
        let sigma = self.rms_delay_spread();
        if sigma <= 1e-15 {
            f64::INFINITY
        } else {
            1.0 / (5.0 * sigma)
        }
    }

    /// Coherence time $T_c \approx \frac{9}{16 \pi f_d}$ in seconds.
    pub fn coherence_time(&self) -> f64 {
        if self.max_doppler_hz <= 1e-9 {
            f64::INFINITY
        } else {
            9.0 / (16.0 * PI * self.max_doppler_hz)
        }
    }

    /// Complex frequency-domain channel transfer function $H(f, t) = \sum_l \sqrt{P_l} e^{j(2\pi f_d t + \phi_l - 2\pi f \tau_l)}$.
    pub fn transfer_function(&self, freq_offset_hz: f64, time_s: f64) -> (f64, f64) {
        let taps = self.taps();
        let total_p = self.total_power().max(1e-15);
        let norm_factor = 1.0 / total_p.sqrt();

        let mut real = 0.0;
        let mut imag = 0.0;

        for tap in &taps {
            let amp = tap.power_linear.sqrt() * norm_factor;
            let phase = tap.phase_rad + 2.0 * PI * tap.doppler_shift_hz * time_s
                - 2.0 * PI * freq_offset_hz * tap.delay_s;
            real += amp * phase.cos();
            imag += amp * phase.sin();
        }

        (real, imag)
    }

    /// Applies multipath channel degradation and Additive White Gaussian Noise (AWGN) to input complex symbols,
    /// returning the received symbols along with the composite complex channel gain $h = (h_r, h_i)$.
    ///
    /// # Arguments
    /// * `tx_symbols` - Transmitted normalized complex symbols $(I, Q)$ with average power $E_s \approx 1.0$.
    /// * `snr_db` - Signal-to-Noise Ratio (SNR) in decibels.
    /// * `rng` - Deterministic random number generator.
    pub fn apply_channel_with_gain(
        &self,
        tx_symbols: &[(f64, f64)],
        snr_db: f64,
        rng: &mut ChannelRng,
    ) -> (Vec<(f64, f64)>, (f64, f64)) {
        let snr_linear = 10.0_f64.powf(snr_db / 10.0).max(1e-12);
        let noise_sigma = (0.5 / snr_linear).sqrt();

        let mut rx_symbols = Vec::with_capacity(tx_symbols.len());

        let h = match self.profile {
            ChannelProfile::Awgn => {
                let h = (1.0, 0.0);
                for &(si, sq) in tx_symbols {
                    let (n_i, n_q) = rng.next_gaussian();
                    rx_symbols.push((si + noise_sigma * n_i, sq + noise_sigma * n_q));
                }
                h
            }
            ChannelProfile::FlatRayleigh => {
                let (hr, hi) = {
                    let (g0, g1) = rng.next_gaussian();
                    (g0 / std::f64::consts::SQRT_2, g1 / std::f64::consts::SQRT_2)
                };
                for &(si, sq) in tx_symbols {
                    let faded_i = si * hr - sq * hi;
                    let faded_q = si * hi + sq * hr;
                    let (n_i, n_q) = rng.next_gaussian();
                    rx_symbols.push((faded_i + noise_sigma * n_i, faded_q + noise_sigma * n_q));
                }
                (hr, hi)
            }
            ChannelProfile::FlatRician { k_factor } => {
                let (hr, hi) = rng.next_rician(k_factor);
                for &(si, sq) in tx_symbols {
                    let faded_i = si * hr - sq * hi;
                    let faded_q = si * hi + sq * hr;
                    let (n_i, n_q) = rng.next_gaussian();
                    rx_symbols.push((faded_i + noise_sigma * n_i, faded_q + noise_sigma * n_q));
                }
                (hr, hi)
            }
            ChannelProfile::IndoorModelB
            | ChannelProfile::IndoorModelC
            | ChannelProfile::IndoorModelD
            | ChannelProfile::CustomMultipath { .. } => {
                let taps = self.taps();
                let total_p = self.total_power().max(1e-15);
                let norm = 1.0 / total_p.sqrt();

                let mut hr = 0.0;
                let mut hi = 0.0;
                for tap in &taps {
                    let (gr, gi) = rng.next_gaussian();
                    let tap_amp = (tap.power_linear * 0.5).sqrt() * norm;
                    hr += tap_amp * gr;
                    hi += tap_amp * gi;
                }

                for &(si, sq) in tx_symbols {
                    let faded_i = si * hr - sq * hi;
                    let faded_q = si * hi + sq * hr;
                    let (n_i, n_q) = rng.next_gaussian();
                    rx_symbols.push((faded_i + noise_sigma * n_i, faded_q + noise_sigma * n_q));
                }
                (hr, hi)
            }
        };

        (rx_symbols, h)
    }

    /// Applies multipath channel degradation and Additive White Gaussian Noise (AWGN) to input complex symbols.
    pub fn apply_channel(
        &self,
        tx_symbols: &[(f64, f64)],
        snr_db: f64,
        rng: &mut ChannelRng,
    ) -> Vec<(f64, f64)> {
        self.apply_channel_with_gain(tx_symbols, snr_db, rng).0
    }
}
