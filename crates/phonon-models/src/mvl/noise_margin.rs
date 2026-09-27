//! Static Noise Margin (SNM) and multi-state stability analysis for Multi-Valued Logic (MVL).
//!
//! Evaluates the 3-state Voltage Transfer Characteristic (VTC) of ternary logic cells, extracting:
//! 1. Critical unity-gain points (\(dV_{out}/dV_{in} = -1.0\)): \(V_{IL1}, V_{IH1}, V_{IL2}, V_{IH2}\).
//! 2. Multi-state noise margins: \(NM_{L0}, NM_{H0}, NM_{L1}, NM_{H1}\).
//! 3. Overall Ternary Static Noise Margin:
//!    \[\text{TSNM} = \min(NM_{L0}, NM_{H0}, NM_{L1}, NM_{H1})\]
//! 4. Thermal retention stability against thermal voltage \(V_T(T) = k_B T / q\).

const Q_E: f64 = 1.602_176_634e-19;
const K_B: f64 = 1.380_649e-23;

/// Noise margin metrics for a 3-state ternary inverter cell.
#[derive(Debug, Clone, Copy)]
pub struct TernaryNoiseMargins {
    /// Low noise margin for transition 1 (Low -> Mid) [V]
    pub nm_l0: f64,
    /// High noise margin for transition 1 (Low -> Mid) [V]
    pub nm_h0: f64,
    /// Low noise margin for transition 2 (Mid -> High) [V]
    pub nm_l1: f64,
    /// High noise margin for transition 2 (Mid -> High) [V]
    pub nm_h1: f64,
    /// Overall worst-case Ternary Static Noise Margin (TSNM) [V]
    pub tsnm: f64,
    /// Output low voltage Vol [V]
    pub v_ol: f64,
    /// Output intermediate plateau voltage Vom [V]
    pub v_om: f64,
    /// Output high voltage Voh [V]
    pub v_oh: f64,
    /// First input low voltage Vil1 (dVout/dVin = -1) [V]
    pub v_il1: f64,
    /// First input high voltage Vih1 (dVout/dVin = -1) [V]
    pub v_ih1: f64,
    /// Second input low voltage Vil2 (dVout/dVin = -1) [V]
    pub v_il2: f64,
    /// Second input high voltage Vih2 (dVout/dVin = -1) [V]
    pub v_ih2: f64,
}

/// Thermal stability retention report across temperature.
#[derive(Debug, Clone, Copy)]
pub struct ThermalRetentionReport {
    /// Operating temperature [K]
    pub temperature_k: f64,
    /// Thermal voltage Vt = kB * T / q [V]
    pub thermal_voltage_v: f64,
    /// Critical 3-sigma thermal noise threshold (3 * Vt) [V]
    pub thermal_noise_threshold_v: f64,
    /// Evaluated TSNM at this temperature [V]
    pub tsnm_v: f64,
    /// Stability safety factor = TSNM / (3 * Vt)
    pub safety_factor: f64,
    /// Is logic state physically stable against thermal fluctuations?
    pub is_stable: bool,
}

/// Analyzer for ternary voltage transfer curves and static noise margins.
#[derive(Debug, Clone)]
pub struct TernaryNoiseMarginAnalyzer {
    v_dd: f64,
}

impl TernaryNoiseMarginAnalyzer {
    /// Creates a new analyzer for supply voltage Vdd.
    pub fn new(v_dd: f64) -> Self {
        Self {
            v_dd: v_dd.max(0.1),
        }
    }

    /// Evaluates canonical Simple Ternary Inverter (STI) transfer function:
    /// High output (Vdd) when Vin < Vdd/3,
    /// Intermediate plateau (Vdd/2) when Vdd/3 <= Vin <= 2Vdd/3,
    /// Low output (0 V) when Vin > 2Vdd/3,
    /// smoothed with physical Fermi-Dirac sigmoid transitions.
    pub fn evaluate_sti_vtc(&self, v_in: f64) -> f64 {
        let v_dd = self.v_dd;
        let v1 = (1.0 / 3.0) * v_dd;
        let v2 = (2.0 / 3.0) * v_dd;
        let sharpness = 40.0 / v_dd; // Transition steepness

        let sig1 = 1.0 / (1.0 + (-sharpness * (v_in - v1)).exp());
        let sig2 = 1.0 / (1.0 + (-sharpness * (v_in - v2)).exp());

        // At Vin = 0: sig1=0, sig2=0 -> Vout = Vdd
        // At Vin = Vdd/2: sig1=1, sig2=0 -> Vout = Vdd/2
        // At Vin = Vdd: sig1=1, sig2=1 -> Vout = 0
        v_dd - 0.5 * v_dd * sig1 - 0.5 * v_dd * sig2
    }

    /// Extracts static noise margins from a discrete sampled VTC curve (v_in, v_out).
    pub fn extract_noise_margins(
        &self,
        v_in: &[f64],
        v_out: &[f64],
    ) -> Option<TernaryNoiseMargins> {
        if v_in.len() < 10 || v_in.len() != v_out.len() {
            return None;
        }

        let n = v_in.len();
        let mut slopes = Vec::with_capacity(n);

        // Compute numerical derivative dVout / dVin
        for i in 0..n {
            let slope = if i == 0 {
                (v_out[1] - v_out[0]) / (v_in[1] - v_in[0]).max(1e-12)
            } else if i == n - 1 {
                (v_out[n - 1] - v_out[n - 2]) / (v_in[n - 1] - v_in[n - 2]).max(1e-12)
            } else {
                (v_out[i + 1] - v_out[i - 1]) / (v_in[i + 1] - v_in[i - 1]).max(1e-12)
            };
            slopes.push(slope);
        }

        // Nominal levels
        let v_oh = v_out[0];
        let v_ol = v_out[n - 1];

        // Mid-point index
        let mid_idx = n / 2;
        let v_om = v_out[mid_idx];

        // Find unity gain points: dVout/dVin = -1
        // Transition 1 is in index range [0, mid_idx]
        let mut v_il1 = (0.25 / 3.0) * self.v_dd;
        let mut v_ih1 = (1.25 / 3.0) * self.v_dd;

        for i in 0..mid_idx {
            if slopes[i] <= -1.0 {
                v_il1 = v_in[i];
                break;
            }
        }
        for i in (0..mid_idx).rev() {
            if slopes[i] <= -1.0 {
                v_ih1 = v_in[i];
                break;
            }
        }

        // Transition 2 is in index range [mid_idx, n-1]
        let mut v_il2 = (1.75 / 3.0) * self.v_dd;
        let mut v_ih2 = (2.75 / 3.0) * self.v_dd;

        for i in mid_idx..n {
            if slopes[i] <= -1.0 {
                v_il2 = v_in[i];
                break;
            }
        }
        for i in (mid_idx..n).rev() {
            if slopes[i] <= -1.0 {
                v_ih2 = v_in[i];
                break;
            }
        }

        // Noise margins:
        // Transition 1: Low input to Mid input
        let nm_l0 = (v_il1 - v_ol).max(0.0);
        let nm_h0 = (v_oh - v_ih1).max(0.0);

        // Transition 2: Mid input to High input
        let nm_l1 = (v_il2 - v_om).max(0.0);
        let nm_h1 = (self.v_dd - v_ih2).max(0.0);

        let tsnm = nm_l0.min(nm_h0).min(nm_l1).min(nm_h1);

        Some(TernaryNoiseMargins {
            nm_l0,
            nm_h0,
            nm_l1,
            nm_h1,
            tsnm,
            v_ol,
            v_om,
            v_oh,
            v_il1,
            v_ih1,
            v_il2,
            v_ih2,
        })
    }

    /// Computes analytical noise margins for the canonical STI cell.
    pub fn compute_sti_margins(&self, points: usize) -> Option<TernaryNoiseMargins> {
        let n = points.max(100);
        let mut v_in = Vec::with_capacity(n);
        let mut v_out = Vec::with_capacity(n);

        let dv = self.v_dd / ((n - 1) as f64);
        for i in 0..n {
            let v = (i as f64) * dv;
            v_in.push(v);
            v_out.push(self.evaluate_sti_vtc(v));
        }

        self.extract_noise_margins(&v_in, &v_out)
    }

    /// Evaluates thermal retention stability at a given temperature.
    pub fn evaluate_thermal_retention(&self, tsnm: f64, temp_k: f64) -> ThermalRetentionReport {
        let v_t = K_B * temp_k / Q_E;
        let threshold = 3.0 * v_t;
        let safety_factor = tsnm / threshold.max(1e-6);

        ThermalRetentionReport {
            temperature_k: temp_k,
            thermal_voltage_v: v_t,
            thermal_noise_threshold_v: threshold,
            tsnm_v: tsnm,
            safety_factor,
            is_stable: safety_factor >= 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_noise_margin_sti() {
        let analyzer = TernaryNoiseMarginAnalyzer::new(0.9);
        let margins = analyzer
            .compute_sti_margins(300)
            .expect("Failed to compute margins");

        assert!(margins.tsnm > 0.05, "TSNM must exceed 50 mV");
        assert!(margins.nm_l0 > 0.05);
        assert!(margins.nm_h0 > 0.05);
        assert!(margins.nm_l1 > 0.05);
        assert!(margins.nm_h1 > 0.05);
    }

    #[test]
    fn test_thermal_retention_across_temperature() {
        let analyzer = TernaryNoiseMarginAnalyzer::new(0.9);
        let margins = analyzer.compute_sti_margins(300).unwrap();

        // Check at room temperature 300 K
        let rep_300 = analyzer.evaluate_thermal_retention(margins.tsnm, 300.0);
        assert!(rep_300.is_stable, "Must be stable at 300 K");
        assert!(rep_300.safety_factor > 1.0);

        // Check at 380 K: safety factor degrades due to increased thermal fluctuations
        let rep_380 = analyzer.evaluate_thermal_retention(margins.tsnm, 380.0);
        assert!(rep_380.safety_factor < rep_300.safety_factor);
    }
}
