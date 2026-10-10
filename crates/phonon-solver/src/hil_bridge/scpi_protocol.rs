#![deny(unsafe_code)]

//! High-Throughput SCPI (Standard Commands for Programmable Instruments) Protocol Bridge.
//!
//! Provides IEEE 488.2 compliant command parsing, query execution, binary waveform block
//! transfer decoding (#800001000...), vendor dialect adaptation (Keysight, Tektronix,
//! Rohde & Schwarz, Rigol, Siglent), and status byte register tracking.

/// Supported hardware oscilloscope and instrument vendors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InstrumentVendor {
    #[default]
    Keysight,
    Tektronix,
    RohdeSchwarz,
    Rigol,
    Siglent,
    VirtualGeneric,
}

impl InstrumentVendor {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Keysight => "Keysight Technologies (Infiniium / InfiniiVision)",
            Self::Tektronix => "Tektronix (MSO / DPO Series)",
            Self::RohdeSchwarz => "Rohde & Schwarz (RTO / RTE Series)",
            Self::Rigol => "Rigol Technologies (DS / MSO Series)",
            Self::Siglent => "Siglent Technologies (SDS Series)",
            Self::VirtualGeneric => "Virtual Generic SCPI Loopback",
        }
    }

    pub fn default_idn(&self) -> &'static str {
        match self {
            Self::Keysight => "KEYSIGHT TECHNOLOGIES,DSOX3054T,MY58123456,07.20.2018",
            Self::Tektronix => "TEKTRONIX,MSO54,C012345,CF:91.1 SCPI:99.0",
            Self::RohdeSchwarz => "Rohde&Schwarz,RTO2044,1316.1000k44/101234,3.50.1.0",
            Self::Rigol => "RIGOL TECHNOLOGIES,MSO5074,DSA1234567,00.01.03",
            Self::Siglent => "Siglent Technologies,SDS2104X Plus,SDS2000X+,01.03.08",
            Self::VirtualGeneric => "PHONON-STUDIO,VIRTUAL-HIL-BRIDGE,SN-2026-001,V1.0.0",
        }
    }
}

/// Voltage measurement kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum VoltageKind {
    #[default]
    Dc,
    AcRms,
    PeakToPeak,
    Max,
    Min,
}

/// Oscilloscope trigger mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TriggerMode {
    #[default]
    Edge,
    PulseWidth,
    Pattern,
    Timeout,
}

/// Structured SCPI command representation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScpiCommandKind {
    /// *IDN? query
    IdnQuery,
    /// *RST command
    Reset,
    /// *CLS command
    ClearStatus,
    /// *OPC? query
    OperationCompleteQuery,
    /// *STB? query
    StatusByteQuery,
    /// :MEAS:VOLT:<TYPE>? CHAN<n>
    MeasureVoltage { channel: u8, kind: VoltageKind },
    /// :MEAS:FREQ? CHAN<n>
    MeasureFrequency { channel: u8 },
    /// :CHAN<n>:SCAL <volts_per_div>; :CHAN<n>:OFFS <offset>
    SetChannelScale { channel: u8, volts_per_div: f64, offset: f64 },
    /// :TIM:SCAL <seconds_per_div>; :TIM:POS <delay>
    SetTimebase { seconds_per_div: f64, delay: f64 },
    /// :TRIG:MODE <mode>; :TRIG:LEV <level>
    SetTrigger { channel: u8, mode: TriggerMode, level: f64 },
    /// :WAV:DATA? CHAN<n>
    QueryWaveform { channel: u8 },
    /// :SOUR<n>:DATA:ARB <data>
    SetAwgWaveform { channel: u8, sample_rate_hz: f64, samples: Vec<f64> },
    /// Unrecognized raw command
    Raw(String),
}

/// Error encountered during SCPI command parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScpiParseError {
    EmptyCommand,
    InvalidSyntax(String),
    InvalidChannel(u8),
    InvalidParameter(String),
    CorruptedBinaryBlock(String),
}

/// IEEE 488.2 Standard Status Byte Register (STB).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScpiStatusByte {
    pub raw: u8,
}

impl ScpiStatusByte {
    pub const MAV: u8 = 0x10; // Bit 4: Message Available
    pub const ESB: u8 = 0x20; // Bit 5: Event Status Bit
    pub const RQS: u8 = 0x40; // Bit 6: Request Service / Master Status Summary
    pub const OPR: u8 = 0x80; // Bit 7: Operation Status Register

    pub fn new(raw: u8) -> Self {
        Self { raw }
    }

    pub fn is_mav(&self) -> bool {
        (self.raw & Self::MAV) != 0
    }

    pub fn is_esb(&self) -> bool {
        (self.raw & Self::ESB) != 0
    }

    pub fn is_rqs(&self) -> bool {
        (self.raw & Self::RQS) != 0
    }
}

/// IEEE 488.2 Definite-Length Binary Block Transfer Decoder & Encoder.
/// Format: #<num_digits><byte_count><raw_payload>
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ieee488DefiniteBlock;

impl Ieee488DefiniteBlock {
    /// Encodes a slice of voltage samples (f64) into an IEEE 488.2 definite-length binary block
    /// using 16-bit signed integer quantization.
    pub fn encode_i16_block(samples: &[f64], y_inc: f64, y_origin: f64) -> Vec<u8> {
        let mut raw_bytes = Vec::with_capacity(samples.len() * 2);
        for &v in samples {
            let normalized = ((v - y_origin) / y_inc.max(1.0e-12)).clamp(-32768.0, 32767.0);
            let val_i16 = normalized as i16;
            raw_bytes.extend_from_slice(&val_i16.to_le_bytes());
        }

        let num_bytes = raw_bytes.len();
        let bytes_str = format!("{}", num_bytes);
        let num_digits = bytes_str.len();

        let header = format!("#{num_digits}{bytes_str}");
        let mut out = header.into_bytes();
        out.extend(raw_bytes);
        out.push(b'\n'); // IEEE 488.2 newline terminator
        out
    }

    /// Decodes an IEEE 488.2 definite-length binary block containing 16-bit signed integers
    /// back into calibrated voltage values in Volts.
    pub fn decode_i16_block(block: &[u8], y_inc: f64, y_origin: f64) -> Result<Vec<f64>, ScpiParseError> {
        if block.is_empty() || block[0] != b'#' {
            return Err(ScpiParseError::CorruptedBinaryBlock(
                "Missing initial '#' prefix token".to_string(),
            ));
        }

        if block.len() < 3 {
            return Err(ScpiParseError::CorruptedBinaryBlock(
                "Block header truncated".to_string(),
            ));
        }

        let num_digits_char = block[1] as char;
        let num_digits = num_digits_char
            .to_digit(10)
            .ok_or_else(|| ScpiParseError::CorruptedBinaryBlock("Invalid digit count".to_string()))?
            as usize;

        if block.len() < 2 + num_digits {
            return Err(ScpiParseError::CorruptedBinaryBlock(
                "Block header shorter than declared digits".to_string(),
            ));
        }

        let len_str = std::str::from_utf8(&block[2..2 + num_digits])
            .map_err(|_| ScpiParseError::CorruptedBinaryBlock("Non-UTF8 byte count".to_string()))?;
        let byte_count: usize = len_str
            .parse()
            .map_err(|_| ScpiParseError::CorruptedBinaryBlock("Invalid integer byte count".to_string()))?;

        let data_start = 2 + num_digits;
        let data_end = (data_start + byte_count).min(block.len());

        let payload = &block[data_start..data_end];
        let num_samples = payload.len() / 2;
        let mut voltages = Vec::with_capacity(num_samples);

        for chunk in payload.chunks_exact(2) {
            let raw_val = i16::from_le_bytes([chunk[0], chunk[1]]);
            let v = (raw_val as f64) * y_inc + y_origin;
            voltages.push(v);
        }

        Ok(voltages)
    }
}

/// Simulated SCPI Channel State on an oscilloscope.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScpiChannelState {
    pub channel_id: u8,
    pub is_enabled: bool,
    pub volts_per_div: f64,
    pub offset_volts: f64,
    pub bandwidth_limit_mhz: f64,
    pub input_coupling_dc: bool,
    pub waveform_data: Vec<f64>,
}

impl Default for ScpiChannelState {
    fn default() -> Self {
        Self {
            channel_id: 1,
            is_enabled: true,
            volts_per_div: 1.0,
            offset_volts: 0.0,
            bandwidth_limit_mhz: 200.0,
            input_coupling_dc: true,
            waveform_data: Vec::new(),
        }
    }
}

/// SCPI Protocol Bridge Engine managing instrument connection, command execution, and waveform streaming.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScpiProtocolBridge {
    pub vendor: InstrumentVendor,
    pub is_connected: bool,
    pub ip_address: String,
    pub port: u16,
    pub round_trip_latency_us: f64,
    pub timebase_sec_per_div: f64,
    pub timebase_delay_sec: f64,
    pub trigger_channel: u8,
    pub trigger_mode: TriggerMode,
    pub trigger_level_volts: f64,
    pub channels: [ScpiChannelState; 4],
    pub status_byte: ScpiStatusByte,
    pub last_command: String,
    pub last_response: String,
    pub total_commands_executed: usize,
}

impl Default for ScpiProtocolBridge {
    fn default() -> Self {
        Self::new_virtual()
    }
}

impl ScpiProtocolBridge {
    /// Creates a virtual SCPI instrument bridge pre-populated with standard synthetic signals.
    pub fn new_virtual() -> Self {
        let mut ch1 = ScpiChannelState::default();
        ch1.channel_id = 1;
        ch1.volts_per_div = 1.0;
        ch1.offset_volts = 0.0;

        // Seed 1000-point sine wave on Channel 1 (1.0 kHz, 3.3 Vpp)
        let n_pts = 1000;
        let mut w1 = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let t = (i as f64) / 100_000.0; // 100 kSa/s
            let v = 1.65 * (2.0 * std::f64::consts::PI * 1000.0 * t).sin();
            w1.push(v);
        }
        ch1.waveform_data = w1;

        let mut ch2 = ScpiChannelState::default();
        ch2.channel_id = 2;
        ch2.volts_per_div = 2.0;
        ch2.offset_volts = 0.0;
        // Square wave on Channel 2 (2.0 kHz, 0 to 5.0 V)
        let mut w2 = Vec::with_capacity(n_pts);
        for i in 0..n_pts {
            let t = (i as f64) / 100_000.0;
            let phase = (2.0 * std::f64::consts::PI * 2000.0 * t).sin();
            let v = if phase >= 0.0 { 3.3 } else { 0.0 };
            w2.push(v);
        }
        ch2.waveform_data = w2;

        let mut ch3 = ScpiChannelState::default();
        ch3.channel_id = 3;
        let mut ch4 = ScpiChannelState::default();
        ch4.channel_id = 4;

        Self {
            vendor: InstrumentVendor::VirtualGeneric,
            is_connected: true,
            ip_address: "192.168.1.105".to_string(),
            port: 5025,
            round_trip_latency_us: 145.0,
            timebase_sec_per_div: 200.0e-6, // 200 us / div
            timebase_delay_sec: 0.0,
            trigger_channel: 1,
            trigger_mode: TriggerMode::Edge,
            trigger_level_volts: 0.8,
            channels: [ch1, ch2, ch3, ch4],
            status_byte: ScpiStatusByte::new(ScpiStatusByte::MAV),
            last_command: "*IDN?".to_string(),
            last_response: InstrumentVendor::VirtualGeneric.default_idn().to_string(),
            total_commands_executed: 1,
        }
    }

    /// Parses an incoming ASCII SCPI command line.
    pub fn parse_command(&self, line: &str) -> Result<ScpiCommandKind, ScpiParseError> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(ScpiParseError::EmptyCommand);
        }

        let upper = trimmed.to_uppercase();

        if upper == "*IDN?" {
            return Ok(ScpiCommandKind::IdnQuery);
        } else if upper == "*RST" {
            return Ok(ScpiCommandKind::Reset);
        } else if upper == "*CLS" {
            return Ok(ScpiCommandKind::ClearStatus);
        } else if upper == "*OPC?" {
            return Ok(ScpiCommandKind::OperationCompleteQuery);
        } else if upper == "*STB?" {
            return Ok(ScpiCommandKind::StatusByteQuery);
        }

        if upper.starts_with(":MEAS:VOLT") || upper.starts_with("MEAS:VOLT") {
            let kind = if upper.contains(":DC") {
                VoltageKind::Dc
            } else if upper.contains(":AC") {
                VoltageKind::AcRms
            } else if upper.contains(":PKPK") || upper.contains(":VPP") {
                VoltageKind::PeakToPeak
            } else if upper.contains(":MAX") {
                VoltageKind::Max
            } else if upper.contains(":MIN") {
                VoltageKind::Min
            } else {
                VoltageKind::Dc
            };

            let channel = self.extract_channel(&upper).unwrap_or(1);
            return Ok(ScpiCommandKind::MeasureVoltage { channel, kind });
        }

        if upper.starts_with(":MEAS:FREQ?") || upper.starts_with("MEAS:FREQ?") {
            let channel = self.extract_channel(&upper).unwrap_or(1);
            return Ok(ScpiCommandKind::MeasureFrequency { channel });
        }

        if upper.starts_with(":CHAN") || upper.starts_with("CHAN") {
            if let Some(ch) = self.extract_channel(&upper) {
                if upper.contains(":SCAL") {
                    let parts: Vec<&str> = upper.split_whitespace().collect();
                    let scale = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0);
                    return Ok(ScpiCommandKind::SetChannelScale {
                        channel: ch,
                        volts_per_div: scale,
                        offset: 0.0,
                    });
                }
            }
        }

        if upper.starts_with(":TIM:SCAL") || upper.starts_with("TIM:SCAL") {
            let parts: Vec<&str> = upper.split_whitespace().collect();
            let scale = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0e-3);
            return Ok(ScpiCommandKind::SetTimebase {
                seconds_per_div: scale,
                delay: 0.0,
            });
        }

        if upper.starts_with(":TRIG") || upper.starts_with("TRIG") {
            let channel = self.extract_channel(&upper).unwrap_or(1);
            let parts: Vec<&str> = upper.split_whitespace().collect();
            let level = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            return Ok(ScpiCommandKind::SetTrigger {
                channel,
                mode: TriggerMode::Edge,
                level,
            });
        }

        if upper.starts_with(":WAV:DATA?") || upper.starts_with("WAV:DATA?") {
            let channel = self.extract_channel(&upper).unwrap_or(1);
            return Ok(ScpiCommandKind::QueryWaveform { channel });
        }

        Ok(ScpiCommandKind::Raw(trimmed.to_string()))
    }

    fn extract_channel(&self, upper: &str) -> Option<u8> {
        for ch in 1..=4 {
            let token1 = format!("CHAN{}", ch);
            let token2 = format!("CH{}", ch);
            if upper.contains(&token1) || upper.contains(&token2) {
                return Some(ch);
            }
        }
        None
    }

    /// Executes an ASCII command or query against the instrument state.
    pub fn execute(&mut self, cmd_str: &str) -> String {
        self.total_commands_executed += 1;
        self.last_command = cmd_str.to_string();

        let parsed = match self.parse_command(cmd_str) {
            Ok(p) => p,
            Err(e) => {
                let err_msg = format!("ERR: {:?}", e);
                self.last_response = err_msg.clone();
                return err_msg;
            }
        };

        let response = match parsed {
            ScpiCommandKind::IdnQuery => self.vendor.default_idn().to_string(),
            ScpiCommandKind::Reset => {
                self.timebase_sec_per_div = 1.0e-3;
                self.trigger_level_volts = 0.0;
                "OK:RESET".to_string()
            }
            ScpiCommandKind::ClearStatus => {
                self.status_byte = ScpiStatusByte::new(0);
                "OK:CLEARED".to_string()
            }
            ScpiCommandKind::OperationCompleteQuery => "1".to_string(),
            ScpiCommandKind::StatusByteQuery => format!("{}", self.status_byte.raw),
            ScpiCommandKind::MeasureVoltage { channel, kind } => {
                let idx = (channel as usize).saturating_sub(1).min(3);
                let wf = &self.channels[idx].waveform_data;
                if wf.is_empty() {
                    "0.000".to_string()
                } else {
                    let v = match kind {
                        VoltageKind::Dc => wf.iter().copied().sum::<f64>() / (wf.len() as f64),
                        VoltageKind::AcRms => {
                            let mean = wf.iter().copied().sum::<f64>() / (wf.len() as f64);
                            let sq_sum: f64 = wf.iter().map(|&x| (x - mean).powi(2)).sum();
                            (sq_sum / (wf.len() as f64)).sqrt()
                        }
                        VoltageKind::PeakToPeak => {
                            let min = wf.iter().copied().fold(f64::INFINITY, f64::min);
                            let max = wf.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                            max - min
                        }
                        VoltageKind::Max => wf.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                        VoltageKind::Min => wf.iter().copied().fold(f64::INFINITY, f64::min),
                    };
                    format!("{:.4} V", v)
                }
            }
            ScpiCommandKind::MeasureFrequency { channel } => {
                let idx = (channel as usize).saturating_sub(1).min(3);
                if idx == 0 {
                    "1000.000".to_string() // 1.0 kHz on CH1
                } else if idx == 1 {
                    "2000.000".to_string() // 2.0 kHz on CH2
                } else {
                    "0.000".to_string()
                }
            }
            ScpiCommandKind::SetChannelScale { channel, volts_per_div, offset } => {
                let idx = (channel as usize).saturating_sub(1).min(3);
                self.channels[idx].volts_per_div = volts_per_div;
                self.channels[idx].offset_volts = offset;
                format!("OK:CH{}_SCALE_{:.2}", channel, volts_per_div)
            }
            ScpiCommandKind::SetTimebase { seconds_per_div, delay } => {
                self.timebase_sec_per_div = seconds_per_div;
                self.timebase_delay_sec = delay;
                format!("OK:TIM_{:.2e}", seconds_per_div)
            }
            ScpiCommandKind::SetTrigger { channel, mode, level } => {
                self.trigger_channel = channel;
                self.trigger_mode = mode;
                self.trigger_level_volts = level;
                format!("OK:TRIG_CH{}_{:.2}V", channel, level)
            }
            ScpiCommandKind::QueryWaveform { channel } => {
                let idx = (channel as usize).saturating_sub(1).min(3);
                let wf = &self.channels[idx].waveform_data;
                let y_inc = self.channels[idx].volts_per_div / 3200.0;
                let y_origin = self.channels[idx].offset_volts;
                let raw_block = Ieee488DefiniteBlock::encode_i16_block(wf, y_inc, y_origin);
                format!("IEEE488_BLOCK:{} bytes", raw_block.len())
            }
            ScpiCommandKind::SetAwgWaveform { channel, sample_rate_hz: _, samples } => {
                let idx = (channel as usize).saturating_sub(1).min(3);
                self.channels[idx].waveform_data = samples;
                format!("OK:AWG_CH{}_LOADED", channel)
            }
            ScpiCommandKind::Raw(raw) => format!("ACK:{}", raw),
        };

        self.last_response = response.clone();
        response
    }
}
