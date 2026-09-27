//! Boundary bridges for mixed-signal co-simulation:
//! Analog-to-Digital Converter (`A2dBridge`) and Digital-to-Analog Driver (`D2aBridge`).

use phonon_core::{DigitalNodeId, LogicLevel, NodeId};

/// Analog-to-Digital Converter Bridge (`A2dBridge`).
///
/// Monitors continuous differential voltage $V(t) = V(in\_pos) - V(in\_neg)$ and
/// maps it to discrete `LogicLevel` outputs with configurable Schmitt-trigger hysteresis.
#[derive(Debug, Clone)]
pub struct A2dBridge {
    /// Positive analog input node.
    pub in_pos: NodeId,
    /// Negative analog input node (typically `NodeId::GROUND`).
    pub in_neg: NodeId,
    /// Output digital signal node.
    pub out_dig: DigitalNodeId,
    /// Lower switching threshold (falling crossing triggers Low).
    pub vth_low: f64,
    /// Upper switching threshold (rising crossing triggers High).
    pub vth_high: f64,
    /// Analog input resistance loading the circuit ($\Omega$).
    pub r_in: f64,
    /// Input capacitance loading the circuit (F).
    pub c_in: f64,
    /// Current latched logic output.
    pub current_level: LogicLevel,
    /// Last sampled analog input voltage.
    pub last_v: f64,
}

impl A2dBridge {
    /// Creates a new `A2dBridge` with Schmitt-trigger thresholds.
    pub fn new(
        in_pos: NodeId,
        in_neg: NodeId,
        out_dig: DigitalNodeId,
        vth_low: f64,
        vth_high: f64,
    ) -> Self {
        Self {
            in_pos,
            in_neg,
            out_dig,
            vth_low,
            vth_high,
            r_in: 1e9,
            c_in: 0.0,
            current_level: LogicLevel::U,
            last_v: 0.0,
        }
    }

    /// Sets input impedance loading parameters.
    pub fn with_impedance(mut self, r_in: f64, c_in: f64) -> Self {
        self.r_in = r_in.max(1.0);
        self.c_in = c_in.max(0.0);
        self
    }

    /// Conductance presented to the continuous analog circuit ($G_{in} = 1 / R_{in}$).
    #[inline]
    pub fn conductance(&self) -> f64 {
        1.0 / self.r_in
    }

    /// Checks whether an analog voltage step $[t_{old}, t_{new}]$ with voltages $[v_{old}, v_{new}]$
    /// crossed a switching threshold.
    ///
    /// Returns `Some((t_cross, new_level))` if a crossing occurred, where $t_{cross}$ is calculated
    /// via linear interpolation.
    pub fn detect_crossing(
        &self,
        v_old: f64,
        v_new: f64,
        t_old: f64,
        t_new: f64,
    ) -> Option<(f64, LogicLevel)> {
        if (t_new - t_old).abs() < 1e-18 {
            return None;
        }

        // Rising crossing: old was below vth_high, new is at or above vth_high
        if v_new >= self.vth_high && v_old < self.vth_high && !self.current_level.is_high() {
            let frac = ((self.vth_high - v_old) / (v_new - v_old)).clamp(0.0, 1.0);
            let t_cross = t_old + frac * (t_new - t_old);
            return Some((t_cross, LogicLevel::One));
        }

        // Falling crossing: old was above vth_low, new is at or below vth_low
        if v_new <= self.vth_low && v_old > self.vth_low && !self.current_level.is_low() {
            let frac = ((v_old - self.vth_low) / (v_old - v_new)).clamp(0.0, 1.0);
            let t_cross = t_old + frac * (t_new - t_old);
            return Some((t_cross, LogicLevel::Zero));
        }

        None
    }

    /// Direct evaluation of logic state based on steady-state voltage (for DC operating point).
    pub fn evaluate_dc(&mut self, v_in: f64) -> LogicLevel {
        self.last_v = v_in;
        let level = if v_in >= self.vth_high {
            LogicLevel::One
        } else if v_in <= self.vth_low {
            LogicLevel::Zero
        } else {
            // In hysteresis dead-band: maintain previous or default to Zero
            if self.current_level == LogicLevel::U {
                if v_in >= 0.5 * (self.vth_low + self.vth_high) {
                    LogicLevel::One
                } else {
                    LogicLevel::Zero
                }
            } else {
                self.current_level
            }
        };
        self.current_level = level;
        level
    }
}

/// Instantaneous Norton companion model for a `D2aBridge` output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct D2aCompanion {
    /// Equivalent parallel Norton conductance ($G_{eq} = 1 / R_{out}$).
    pub g_eq: f64,
    /// Equivalent Norton current injected into `out_pos` ($-I_{eq}$ at `out_neg`).
    pub i_eq: f64,
    /// Instantaneous Thevenin driving voltage.
    pub v_thev: f64,
    /// Whether the output driver is currently in high-impedance mode (`Z`).
    pub is_high_z: bool,
}

/// Digital-to-Analog Driver Bridge (`D2aBridge`).
///
/// Drives continuous analog nodes from discrete `LogicLevel` inputs with finite
/// rise/fall times ($t_r, t_f$) and defined output drive impedance.
#[derive(Debug, Clone)]
pub struct D2aBridge {
    /// Input digital signal node.
    pub in_dig: DigitalNodeId,
    /// Positive analog output node.
    pub out_pos: NodeId,
    /// Negative analog output node (typically `NodeId::GROUND`).
    pub out_neg: NodeId,
    /// Output voltage corresponding to logic Low (0).
    pub v_low: f64,
    /// Output voltage corresponding to logic High (1).
    pub v_high: f64,
    /// Output driver impedance when active ($\Omega$).
    pub r_out: f64,
    /// Output impedance when in high-impedance (`Z`) state ($\Omega$).
    pub r_off: f64,
    /// 10%-90% or linear 0%-100% rise transition time (seconds).
    pub rise_time: f64,
    /// 90%-10% or linear 100%-0% fall transition time (seconds).
    pub fall_time: f64,

    // Dynamic state
    current_level: LogicLevel,
    v_start: f64,
    v_target: f64,
    t_start: f64,
    transition_duration: f64,
}

impl D2aBridge {
    /// Creates a new `D2aBridge`.
    pub fn new(
        in_dig: DigitalNodeId,
        out_pos: NodeId,
        out_neg: NodeId,
        v_low: f64,
        v_high: f64,
    ) -> Self {
        Self {
            in_dig,
            out_pos,
            out_neg,
            v_low,
            v_high,
            r_out: 50.0,
            r_off: 1e10,
            rise_time: 1e-9,
            fall_time: 1e-9,
            current_level: LogicLevel::Zero,
            v_start: v_low,
            v_target: v_low,
            t_start: 0.0,
            transition_duration: 1e-12,
        }
    }

    /// Sets drive timings and output impedances.
    pub fn with_timings(mut self, rise_time: f64, fall_time: f64, r_out: f64) -> Self {
        self.rise_time = rise_time.max(1e-15);
        self.fall_time = fall_time.max(1e-15);
        self.r_out = r_out.max(1e-6);
        self
    }

    /// Current target level.
    #[inline]
    pub fn current_level(&self) -> LogicLevel {
        self.current_level
    }

    /// Returns the target continuous voltage for a given logic level.
    #[inline]
    pub fn level_to_voltage(&self, level: LogicLevel) -> f64 {
        match level {
            LogicLevel::One | LogicLevel::H => self.v_high,
            LogicLevel::Zero | LogicLevel::L => self.v_low,
            LogicLevel::Z => (self.v_low + self.v_high) * 0.5,
            LogicLevel::U | LogicLevel::X | LogicLevel::W | LogicLevel::DontCare => {
                (self.v_low + self.v_high) * 0.5
            }
        }
    }

    /// Schedules a transition to a new logic level starting at time `t`.
    pub fn set_level(&mut self, level: LogicLevel, t: f64) {
        let v_curr = self.instantaneous_thevenin_voltage(t);
        self.current_level = level;
        self.v_start = v_curr;
        self.v_target = self.level_to_voltage(level);
        self.t_start = t;
        self.transition_duration = if self.v_target >= self.v_start {
            self.rise_time
        } else {
            self.fall_time
        };
    }

    /// Instantaneous Thevenin driver voltage at time `t`.
    pub fn instantaneous_thevenin_voltage(&self, t: f64) -> f64 {
        if self.transition_duration <= 1e-15 || t <= self.t_start {
            return self.v_start;
        }
        let alpha = ((t - self.t_start) / self.transition_duration).clamp(0.0, 1.0);
        self.v_start + alpha * (self.v_target - self.v_start)
    }

    /// Evaluates Norton companion stamp at continuous time `t`.
    pub fn evaluate(&self, t: f64) -> D2aCompanion {
        let is_high_z = self.current_level.is_high_z();
        let r = if is_high_z { self.r_off } else { self.r_out };
        let g_eq = 1.0 / r;
        let v_thev = self.instantaneous_thevenin_voltage(t);
        let i_eq = v_thev * g_eq;

        D2aCompanion {
            g_eq,
            i_eq,
            v_thev,
            is_high_z,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a2d_bridge_crossing() {
        let a2d = A2dBridge::new(
            NodeId::new(1),
            NodeId::GROUND,
            DigitalNodeId::new(0),
            1.0,
            2.0,
        );

        // Rising crossing from 0.5V to 2.5V between t=0.0 and t=1.0s
        let res = a2d.detect_crossing(0.5, 2.5, 0.0, 1.0);
        assert!(res.is_some());
        let (t_cross, level) = res.unwrap();
        assert_eq!(level, LogicLevel::One);
        // Crossing of 2.0V occurs at (2.0 - 0.5) / 2.0 = 0.75s
        assert!((t_cross - 0.75).abs() < 1e-6);

        // Falling crossing from 2.5V to 0.5V
        let mut a2d_high = a2d.clone();
        a2d_high.current_level = LogicLevel::One;
        let res_fall = a2d_high.detect_crossing(2.5, 0.5, 1.0, 2.0);
        assert!(res_fall.is_some());
        let (t_cross_f, level_f) = res_fall.unwrap();
        assert_eq!(level_f, LogicLevel::Zero);
        // Falling crossing of 1.0V: (2.5 - 1.0) / 2.0 = 0.75 -> t = 1.0 + 0.75 = 1.75s
        assert!((t_cross_f - 1.75).abs() < 1e-6);
    }

    #[test]
    fn test_d2a_bridge_ramp() {
        let mut d2a = D2aBridge::new(
            DigitalNodeId::new(0),
            NodeId::new(1),
            NodeId::GROUND,
            0.0,
            5.0,
        )
        .with_timings(10e-9, 10e-9, 50.0);

        // Initial state at t=0
        let comp0 = d2a.evaluate(0.0);
        assert!((comp0.v_thev - 0.0).abs() < 1e-6);
        assert!((comp0.g_eq - 1.0 / 50.0).abs() < 1e-6);

        // Trigger High transition at t=10ns
        d2a.set_level(LogicLevel::One, 10e-9);

        // At midpoint t=15ns (5ns into 10ns rise)
        let comp_mid = d2a.evaluate(15e-9);
        assert!((comp_mid.v_thev - 2.5).abs() < 1e-6);

        // At t=20ns (completion)
        let comp_end = d2a.evaluate(20e-9);
        assert!((comp_end.v_thev - 5.0).abs() < 1e-6);
    }
}
