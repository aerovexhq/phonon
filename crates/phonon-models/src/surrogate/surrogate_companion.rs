//! MNA companion models for Neural Surrogate devices.

use super::mlp::MultilayerPerceptron;

/// Operating mode / terminal topology of the neural surrogate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurrogateDeviceType {
    /// 2-terminal device (e.g. non-linear diode, memristor, varactor): input [V_D], output [I_D].
    TwoTerminalDiode,
    /// 3/4-terminal field-effect device (e.g. MOSFET, HEMT, FinFET): input [V_DS, V_GS, V_BS], output [I_DS].
    ThreeTerminalTransistor,
}

/// A trained neural surrogate device companion ready for monolithic MNA matrix stamping.
#[derive(Debug, Clone, PartialEq)]
pub struct NeuralSurrogateCompanion {
    pub name: String,
    pub device_type: SurrogateDeviceType,
    pub network: MultilayerPerceptron,
}

impl NeuralSurrogateCompanion {
    /// Creates a 2-terminal diode neural surrogate companion.
    pub fn new_diode(name: &str, network: MultilayerPerceptron) -> Self {
        assert_eq!(
            network.in_features(),
            1,
            "Diode surrogate must take 1 input (V_D)"
        );
        assert_eq!(
            network.out_features(),
            1,
            "Diode surrogate must produce 1 output (I_D)"
        );
        Self {
            name: name.to_string(),
            device_type: SurrogateDeviceType::TwoTerminalDiode,
            network,
        }
    }

    /// Creates a 3/4-terminal transistor neural surrogate companion.
    pub fn new_transistor(name: &str, network: MultilayerPerceptron) -> Self {
        assert!(
            network.in_features() == 2 || network.in_features() == 3,
            "Transistor surrogate must take 2 inputs (V_DS, V_GS) or 3 inputs (V_DS, V_GS, V_BS)"
        );
        assert_eq!(
            network.out_features(),
            1,
            "Transistor surrogate must produce 1 output (I_DS)"
        );
        Self {
            name: name.to_string(),
            device_type: SurrogateDeviceType::ThreeTerminalTransistor,
            network,
        }
    }

    /// Evaluates 2-terminal diode response: returns (I_D, dynamic conductance g_d = dI_D/dV_D).
    pub fn evaluate_diode(&self, v_d: f64) -> (f64, f64) {
        let (out, jac) = self.network.forward_with_jacobian(&[v_d]);
        let i_d = out[0];
        let g_d = jac[0][0].max(1e-12); // Keep positive small conductance for numerical stability
        (i_d, g_d)
    }

    /// Evaluates 3/4-terminal transistor response: returns (I_DS, g_m = dI/dV_GS, g_ds = dI/dV_DS, g_mbs = dI/dV_BS).
    pub fn evaluate_transistor(&self, v_ds: f64, v_gs: f64, v_bs: f64) -> (f64, f64, f64, f64) {
        let inputs = if self.network.in_features() == 2 {
            vec![v_ds, v_gs]
        } else {
            vec![v_ds, v_gs, v_bs]
        };

        let (out, jac) = self.network.forward_with_jacobian(&inputs);
        let i_ds = out[0];
        let g_ds = jac[0][0].max(1e-12);
        let g_m = jac[0][1].max(0.0);
        let g_mbs = if inputs.len() > 2 {
            jac[0][2].max(0.0)
        } else {
            0.0
        };

        (i_ds, g_m, g_ds, g_mbs)
    }
}
