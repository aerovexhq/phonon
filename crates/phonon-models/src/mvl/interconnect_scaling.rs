//! Mathematical and physical interconnect scaling models for Multi-Valued Logic (MVL).
//!
//! Quantifies:
//! 1. Information-theoretic radix efficiency \(E(r) = \frac{\ln(r)}{r}\), proving radix-3 is optimal.
//! 2. Pin count reduction for 64-bit equivalent arithmetic datapaths (64 bits -> 41 trits, 35.9% reduction).
//! 3. Rent's rule interconnect wirelength and routing track congestion.
//! 4. Wiring capacitive power and dynamic energy scaling (\(E_{wire} \propto C_{wire} \cdot \Delta V^2\)).

/// Radix information theory metrics.
#[derive(Debug, Clone, Copy)]
pub struct RadixEfficiency {
    /// Radix base r
    pub radix: u32,
    /// Radix efficiency metric E(r) = ln(r) / r
    pub efficiency: f64,
    /// Hardware complexity factor = r / ln(r)
    pub hardware_complexity_factor: f64,
}

impl RadixEfficiency {
    /// Computes radix information efficiency for radix r.
    pub fn compute(r: u32) -> Self {
        let r_f = r.max(2) as f64;
        let efficiency = r_f.ln() / r_f;
        let hardware_complexity_factor = r_f / r_f.ln();

        Self {
            radix: r,
            efficiency,
            hardware_complexity_factor,
        }
    }
}

/// Rent's rule interconnect routing parameters.
#[derive(Debug, Clone, Copy)]
pub struct InterconnectRentModel {
    /// Rent's exponent p (typically 0.60 to 0.75 for dense processor datapaths)
    pub rent_exponent: f64,
    /// Metal interconnect line capacitance per meter [F/m] (typically ~0.2 pF/mm = 2.0e-10 F/m)
    pub capacitance_per_meter: f64,
    /// Minimum interconnect wire pitch [m] (e.g. 30 nm for M1/M2)
    pub wire_pitch_m: f64,
}

impl Default for InterconnectRentModel {
    fn default() -> Self {
        Self {
            rent_exponent: 0.65,
            capacitance_per_meter: 1.8e-10, // 0.18 pF/mm
            wire_pitch_m: 32.0e-9,          // 32 nm pitch
        }
    }
}

/// Comprehensive interconnect comparison metrics between logic radices.
#[derive(Debug, Clone, Copy)]
pub struct InterconnectComparison {
    /// Logic architecture name
    pub name: &'static str,
    /// Radix
    pub radix: u32,
    /// Number of pins / signal tracks
    pub pin_count: usize,
    /// Total representable states (log2 dynamic range)
    pub dynamic_range_bits: f64,
    /// Average wirelength [um]
    pub avg_wirelength_um: f64,
    /// Total routing track width / congestion [um]
    pub routing_width_um: f64,
    /// Total interconnect bus capacitance [fF]
    pub total_bus_capacitance_ff: f64,
    /// Dynamic interconnect switching energy per cycle [fJ] at supply Vdd
    pub switching_energy_fj: f64,
    /// Dynamic power dissipation [uW] at operating frequency
    pub dynamic_power_uw: f64,
    /// Percentage pin count reduction relative to 64-bit binary baseline
    pub pin_reduction_percent: f64,
    /// Percentage interconnect energy reduction relative to 64-bit binary baseline
    pub energy_reduction_percent: f64,
}

/// Evaluator for multi-valued interconnect scaling.
#[derive(Debug, Clone)]
pub struct InterconnectScalingEvaluator {
    rent: InterconnectRentModel,
    v_dd: f64,
    freq_ghz: f64,
}

impl InterconnectScalingEvaluator {
    /// Creates a new interconnect scaling evaluator.
    pub fn new(rent: InterconnectRentModel, v_dd: f64, freq_ghz: f64) -> Self {
        Self {
            rent,
            v_dd: v_dd.max(0.1),
            freq_ghz: freq_ghz.max(0.01),
        }
    }

    /// Evaluates interconnect metrics for a given bus configuration.
    pub fn evaluate_bus(
        &self,
        name: &'static str,
        radix: u32,
        pin_count: usize,
        binary_baseline_pins: usize,
        binary_baseline_energy_fj: f64,
    ) -> InterconnectComparison {
        let p = self.rent.rent_exponent;
        let pins_f = pin_count as f64;

        // Rent's rule average wirelength scaling: L_avg ~ pins^(p - 0.5) * L_0
        let l_base_um = 15.0; // Base unit cell span in um
        let avg_wirelength_um = l_base_um * pins_f.powf(p - 0.5);

        // Routing bus track width = pin_count * pitch
        let routing_width_um = pins_f * (self.rent.wire_pitch_m * 1.0e6);

        // Total bus capacitance = pin_count * C_per_meter * L_avg
        let avg_wirelength_m = avg_wirelength_um * 1.0e-6;
        let bus_cap_farads = pins_f * self.rent.capacitance_per_meter * avg_wirelength_m;
        let total_bus_capacitance_ff = bus_cap_farads * 1.0e15;

        // Dynamic energy per cycle:
        // In binary (radix 2), swing is rail-to-rail (Vdd): E = 0.5 * alpha * C * Vdd^2
        // In balanced ternary (radix 3), transitions occur with steps of Vdd/2 (half-swing),
        // with probability 2/3 of 0.5*Vdd swing and 1/3 of full Vdd swing.
        // Effective voltage step factor:
        let voltage_step_factor = match radix {
            2 => 1.0,                    // full Vdd swing
            3 => 0.5 * 0.5 + 0.5 * 0.25, // average weighted voltage squared swing ~ 0.375
            4 => 0.333 * 0.333,          // 1/9 ~ 0.111
            _ => 1.0,
        };

        let activity_factor = 0.35; // typical datapath switching factor
        let switching_energy_j =
            0.5 * activity_factor * bus_cap_farads * (self.v_dd * self.v_dd) * voltage_step_factor;
        let switching_energy_fj = switching_energy_j * 1.0e15;
        let dynamic_power_uw = switching_energy_fj * self.freq_ghz; // fJ * GHz = 1e-15 J * 1e9 s^-1 = 1e-6 W = uW

        // Representable states in log2 bits
        let dynamic_range_bits = pins_f * (radix as f64).log2();

        // Reductions vs baseline
        let pin_reduction_percent = if binary_baseline_pins > 0 {
            100.0 * (1.0 - (pin_count as f64) / (binary_baseline_pins as f64))
        } else {
            0.0
        };

        let energy_reduction_percent = if binary_baseline_energy_fj > 0.0 {
            100.0 * (1.0 - switching_energy_fj / binary_baseline_energy_fj)
        } else {
            0.0
        };

        InterconnectComparison {
            name,
            radix,
            pin_count,
            dynamic_range_bits,
            avg_wirelength_um,
            routing_width_um,
            total_bus_capacitance_ff,
            switching_energy_fj,
            dynamic_power_uw,
            pin_reduction_percent,
            energy_reduction_percent,
        }
    }

    /// Generates a comprehensive comparison suite for standard word-width architectures:
    /// 1. 64-bit Binary (Baseline)
    /// 2. 41-trit Balanced Ternary (Full 64-bit dynamic range equivalent: 3^41 > 2^64)
    /// 3. 32-trit Balanced Ternary (High-density arithmetic: 3^32 ~ 1.85e15 ~ 50.7 bits)
    /// 4. 32-quat Quaternary (4^32 = 2^64 equivalent)
    pub fn compare_all(&self) -> Vec<InterconnectComparison> {
        let bin = self.evaluate_bus("64-bit Binary Baseline", 2, 64, 64, 0.0);
        let baseline_energy = bin.switching_energy_fj;

        let ter_41 = self.evaluate_bus("41-trit Balanced Ternary", 3, 41, 64, baseline_energy);
        let ter_32 = self.evaluate_bus("32-trit Balanced Ternary", 3, 32, 64, baseline_energy);
        let quat_32 = self.evaluate_bus("32-quat Quaternary", 4, 32, 64, baseline_energy);

        vec![bin, ter_41, ter_32, quat_32]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radix_information_efficiency() {
        let r2 = RadixEfficiency::compute(2);
        let r3 = RadixEfficiency::compute(3);
        let r4 = RadixEfficiency::compute(4);

        // Radix 3 has higher information efficiency than radix 2 and radix 4
        assert!(r3.efficiency > r2.efficiency);
        assert!(r3.efficiency > r4.efficiency);
        // Radix 3 has lower hardware complexity factor than radix 2
        assert!(r3.hardware_complexity_factor < r2.hardware_complexity_factor);
    }

    #[test]
    fn test_interconnect_pin_and_energy_savings() {
        let evaluator =
            InterconnectScalingEvaluator::new(InterconnectRentModel::default(), 0.9, 3.0);
        let comparisons = evaluator.compare_all();

        let bin = &comparisons[0];
        let ter41 = &comparisons[1];
        let ter32 = &comparisons[2];

        assert_eq!(bin.pin_count, 64);
        assert_eq!(ter41.pin_count, 41);
        assert_eq!(ter32.pin_count, 32);

        // 41 trits covers > 64 bits of dynamic range
        assert!(ter41.dynamic_range_bits >= 64.0);

        // Pin reduction >= 35% for 41-trit
        assert!(ter41.pin_reduction_percent > 35.0);

        // Pin reduction == 50% for 32-trit
        assert!((ter32.pin_reduction_percent - 50.0).abs() < 1e-3);

        // Energy reduction should exceed 40% due to fewer pins and reduced voltage swing
        assert!(ter41.energy_reduction_percent > 40.0);
        assert!(ter32.energy_reduction_percent > 50.0);
    }
}
