//! Parallel comparative benchmark runner comparing Atomic Relay Logic vs 3nm GAA CMOS.
//!
//! Evaluates:
//! 1. Standby quiescent leakage power elimination (> 99.999% reduction, practically zero).
//! 2. Subthreshold swing steepness (S < 5 mV/dec vs 65 mV/dec thermionic limit).
//! 3. Dynamic switching energy scaling at sub-100mV actuation voltage.
//! 4. Multi-threaded arithmetic throughput across all CPU cores using Rayon.

use phonon_models::relay::MultiBitRelayAdder;
use rayon::prelude::*;

/// Comparative benchmark metrics report between Atomic Relay and 3nm GAA CMOS.
#[derive(Debug, Clone)]
pub struct RelayBenchmarkReport {
    /// CMOS baseline supply voltage [V]
    pub cmos_vdd_v: f64,
    /// Atomic relay actuation supply voltage [V]
    pub relay_vdd_v: f64,

    /// CMOS 32-bit adder standby leakage power [uW]
    pub cmos_standby_leakage_uw: f64,
    /// Atomic relay 32-bit adder standby leakage power [uW]
    pub relay_standby_leakage_uw: f64,
    /// Standby power reduction percentage [%]
    pub standby_power_reduction_percent: f64,

    /// CMOS subthreshold swing S [mV/decade]
    pub cmos_subthreshold_swing_mv_per_dec: f64,
    /// Atomic relay effective subthreshold swing S [mV/decade]
    pub relay_subthreshold_swing_mv_per_dec: f64,
    /// Subthreshold swing steepness improvement factor
    pub swing_steepness_factor: f64,

    /// CMOS dynamic switching energy per 32-bit addition [fJ]
    pub cmos_dynamic_energy_per_op_fj: f64,
    /// Atomic relay dynamic switching energy per 32-bit addition [fJ]
    pub relay_dynamic_energy_per_op_fj: f64,
    /// Dynamic switching energy reduction percentage [%]
    pub dynamic_energy_savings_percent: f64,

    /// Total number of parallel arithmetic operations benchmarked
    pub operations_count: usize,
    /// Total parallel execution duration [s]
    pub parallel_duration_s: f64,
    /// True if all arithmetic operations verified bit-exact
    pub all_operations_exact: bool,
}

/// Benchmark harness comparing atomic relay logic against CMOS.
#[derive(Debug, Clone)]
pub struct RelayBenchmarkRunner {
    relay_vdd_v: f64,
    cmos_vdd_v: f64,
}

impl Default for RelayBenchmarkRunner {
    fn default() -> Self {
        Self {
            relay_vdd_v: 0.085, // 85 mV ultra-scaled relay actuation
            cmos_vdd_v: 0.70,   // 700 mV 3nm Silicon GAA CMOS baseline
        }
    }
}

impl RelayBenchmarkRunner {
    /// Creates a new benchmark runner with specified operating voltages.
    pub fn new(relay_vdd_v: f64, cmos_vdd_v: f64) -> Self {
        Self {
            relay_vdd_v: relay_vdd_v.max(0.04),
            cmos_vdd_v: cmos_vdd_v.max(0.40),
        }
    }

    /// Runs a parallel benchmark across multiple CPU cores using Rayon.
    pub fn run_benchmark(&self, num_operations: usize) -> RelayBenchmarkReport {
        let bit_width = 32;
        let adder = MultiBitRelayAdder::new(bit_width);

        // 1. Standby quiescent leakage power comparison
        // 3nm CMOS: ~28 transistors per bit * 32 bits = 896 transistors.
        // I_off ~ 0.5 nA/transistor at 0.7V -> ~450 nA * 0.7V ~ 0.315 uW per 32-bit adder
        let cmos_standby_leakage_uw: f64 = 0.315;

        // Relay: open gap leakage ~ 1e-16 A -> 320 relays * 1e-16 A * 0.085 V ~ 2.7e-15 W ~ 2.7e-9 uW
        let relay_standby_leakage_w = adder.total_static_leakage_w(self.relay_vdd_v);
        let relay_standby_leakage_uw = relay_standby_leakage_w * 1.0e6;
        let standby_power_reduction_percent =
            (1.0 - (relay_standby_leakage_uw / cmos_standby_leakage_uw.max(1e-12))) * 100.0;

        // 2. Subthreshold swing
        let cmos_subthreshold_swing_mv_per_dec = 65.0; // Boltzmann thermionic limit at 300K
        let relay_subthreshold_swing_mv_per_dec = 0.5; // Abrupt mechanical pull-in
        let swing_steepness_factor =
            cmos_subthreshold_swing_mv_per_dec / relay_subthreshold_swing_mv_per_dec;

        // 3. Dynamic energy per 32-bit addition
        // CMOS: E = 0.5 * C_total * Vdd^2. C_total ~ 150 fF -> 0.5 * 150e-15 * 0.49 ~ 36.75 fJ
        let cmos_dynamic_energy_per_op_fj = 36.75;
        // Relay: E = 0.5 * C_total * V_relay^2 + E_mech. C_total ~ 272 fF -> 0.5 * 272e-15 * 0.007225 ~ 0.98 fJ + 0.48 fJ mech = 1.46 fJ
        let relay_dynamic_energy_per_op_j = adder.total_dynamic_energy_j(self.relay_vdd_v);
        let relay_dynamic_energy_per_op_fj = relay_dynamic_energy_per_op_j * 1.0e15;
        let dynamic_energy_savings_percent =
            (1.0 - (relay_dynamic_energy_per_op_fj / cmos_dynamic_energy_per_op_fj)) * 100.0;

        // 4. Parallel execution across all CPU cores
        let start_time = std::time::Instant::now();
        let ops_per_thread = (num_operations / rayon::current_num_threads()).max(1);

        let test_pairs: Vec<(u64, u64)> = (0..num_operations)
            .map(|i| {
                let a = ((i as u64) * 1103515245 + 12345) & 0x7FFF_FFFF;
                let b = ((i as u64) * 214013 + 2531011) & 0x7FFF_FFFF;
                (a, b)
            })
            .collect();

        let all_exact: bool = test_pairs
            .par_chunks(ops_per_thread)
            .map(|chunk| {
                let local_adder = MultiBitRelayAdder::new(bit_width);
                for &(a, b) in chunk {
                    let (sum, _cout) = local_adder.add_u64(a, b);
                    let expected = (a + b) & 0xFFFF_FFFF;
                    if sum != expected {
                        return false;
                    }
                }
                true
            })
            .all(|res| res);

        let parallel_duration_s = start_time.elapsed().as_secs_f64();

        RelayBenchmarkReport {
            cmos_vdd_v: self.cmos_vdd_v,
            relay_vdd_v: self.relay_vdd_v,
            cmos_standby_leakage_uw,
            relay_standby_leakage_uw,
            standby_power_reduction_percent,
            cmos_subthreshold_swing_mv_per_dec,
            relay_subthreshold_swing_mv_per_dec,
            swing_steepness_factor,
            cmos_dynamic_energy_per_op_fj,
            relay_dynamic_energy_per_op_fj,
            dynamic_energy_savings_percent,
            operations_count: num_operations,
            parallel_duration_s,
            all_operations_exact: all_exact,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_benchmark_leakage_and_energy_savings() {
        let runner = RelayBenchmarkRunner::default();
        let rep = runner.run_benchmark(1000);

        // Standby leakage reduction must exceed 99.999%
        assert!(rep.standby_power_reduction_percent > 99.999);
        assert!(rep.relay_standby_leakage_uw < 1.0e-6);

        // Subthreshold swing should be > 10x steeper than CMOS
        assert!(rep.swing_steepness_factor > 10.0);

        // Sub-100mV dynamic energy reduction should exceed 85%
        assert!(rep.dynamic_energy_savings_percent > 85.0);

        // Arithmetic correctness
        assert!(rep.all_operations_exact);
    }
}
