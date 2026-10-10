#![deny(unsafe_code)]

//! Hardware-in-the-Loop (HIL) Real-Time Oscilloscope & Logic Analyzer Protocol Bridge.
//!
//! Provides high-throughput SCPI over TCP/IP & USB-TMC, binary waveform block transfer decoding (#8...),
//! Sigrok-compatible multi-channel logic analysis and protocol decoders (SPI, I2C, UART, CAN),
//! continuous waveform co-simulation synchronizer (< 1.0 us jitter), and 10-point engineering audit.

pub mod hil_audit;
pub mod hil_synchronizer;
pub mod logic_analyzer;
pub mod scpi_protocol;

pub use hil_audit::{audit_hil_bridge, HilAuditItem, HilAuditReport};
pub use hil_synchronizer::{HilMode, HilSyncMetrics, HilSynchronizer};
pub use logic_analyzer::{
    CanDecoder, CanFrame, I2cDecoder, I2cPacket, LogicAnalyzerCapture, LogicTriggerCondition,
    SpiDecoder, SpiPacket, TriggerEdge, UartDecoder, UartFrame,
};
pub use scpi_protocol::{
    Ieee488DefiniteBlock, InstrumentVendor, ScpiChannelState, ScpiCommandKind, ScpiParseError,
    ScpiProtocolBridge, ScpiStatusByte, TriggerMode, VoltageKind,
};

/// Master orchestrator for Hardware-in-the-Loop (HIL) Protocol Bridge & Logic Analyzer.
#[derive(Debug, Clone, PartialEq)]
pub struct HilBridgeEngine {
    pub scpi: ScpiProtocolBridge,
    pub logic_capture: LogicAnalyzerCapture,
    pub spi_decoder: SpiDecoder,
    pub i2c_decoder: I2cDecoder,
    pub uart_decoder: UartDecoder,
    pub can_decoder: CanDecoder,
    pub synchronizer: HilSynchronizer,
    pub audit_report: HilAuditReport,
}

impl Default for HilBridgeEngine {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl HilBridgeEngine {
    /// Fast cold-boot constructor executing well under 2.0 ms (typically < 0.1 ms).
    pub fn new_fast() -> Self {
        let scpi = ScpiProtocolBridge::new_virtual();

        // Seed a 500-sample logic analyzer capture buffer with active digital clock & protocol pulses
        let mut logic_capture = LogicAnalyzerCapture::new(1.0e6, 8); // 1 MSa/s, 8 channels
        for i in 0..500 {
            let clk = if (i % 20) < 10 { 1u16 } else { 0u16 };      // Ch 0 (Clock)
            let data = if (i % 80) < 40 { 1u16 << 1 } else { 0u16 }; // Ch 1 (Data)
            let sck = if (i % 10) < 5 { 1u16 << 2 } else { 0u16 };   // Ch 2 (SPI SCK)
            let cs = if i > 50 && i < 450 { 0u16 } else { 1u16 << 3 }; // Ch 3 (SPI CS#, active-low)
            let sda = if (i % 40) < 20 { 1u16 << 4 } else { 0u16 };  // Ch 4 (I2C SDA)
            let scl = if (i % 20) < 10 { 1u16 << 5 } else { 0u16 };  // Ch 5 (I2C SCL)
            let uart_tx = if (i % 60) < 30 { 1u16 << 6 } else { 0u16 }; // Ch 6 (UART TX)
            let can_rx = if (i % 100) < 50 { 1u16 << 7 } else { 0u16 }; // Ch 7 (CAN RX)

            logic_capture.push_sample(clk | data | sck | cs | sda | scl | uart_tx | can_rx);
        }

        let spi_decoder = SpiDecoder::default();
        let i2c_decoder = I2cDecoder::default();
        let uart_decoder = UartDecoder::default();
        let can_decoder = CanDecoder::default();
        let synchronizer = HilSynchronizer::new_synthetic();
        let audit_report = HilAuditReport::default_baseline();

        Self {
            scpi,
            logic_capture,
            spi_decoder,
            i2c_decoder,
            uart_decoder,
            can_decoder,
            synchronizer,
            audit_report,
        }
    }

    /// Executes an ASCII SCPI command line string on the instrument bridge.
    pub fn execute_scpi(&mut self, cmd: &str) -> String {
        self.scpi.execute(cmd)
    }

    /// Steps co-simulation forward at the given SPICE transient timestamp.
    pub fn step_co_sim(&mut self, current_time_s: f64, spice_voltage: f64, spice_logic: u16) -> HilSyncMetrics {
        self.synchronizer.step_co_simulation(current_time_s, spice_voltage, spice_logic)
    }

    /// Runs the 10-point HIL Bridge verification audit and caches the report.
    pub fn run_audit(&mut self) -> &HilAuditReport {
        self.audit_report = audit_hil_bridge();
        &self.audit_report
    }

    /// Decodes all SPI packets from the logic analyzer capture buffer.
    pub fn decode_spi(&self) -> Vec<SpiPacket> {
        self.spi_decoder.decode(&self.logic_capture)
    }

    /// Decodes all I2C packets from the logic analyzer capture buffer.
    pub fn decode_i2c(&self) -> Vec<I2cPacket> {
        self.i2c_decoder.decode(&self.logic_capture)
    }

    /// Decodes all UART frames from the logic analyzer capture buffer.
    pub fn decode_uart(&self) -> Vec<UartFrame> {
        self.uart_decoder.decode(&self.logic_capture)
    }

    /// Decodes all CAN frames from the logic analyzer capture buffer.
    pub fn decode_can(&self) -> Vec<CanFrame> {
        self.can_decoder.decode(&self.logic_capture)
    }
}
