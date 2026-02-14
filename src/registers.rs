use bitflags::bitflags;
use pmbus_adapter::{Linear11, ULinear16};

// ---------------------------------------------------------------------------
// TelemetryConfig (D0h, 6-byte block)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryPriority {
    A,
    B,
    C,
    Disabled,
}

impl TelemetryPriority {
    pub fn from_bits(val: u8) -> Self {
        match val & 0x03 {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            _ => Self::Disabled,
        }
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::Disabled => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryChannel {
    pub priority: TelemetryPriority,
    pub averaging: u8,
}

impl TelemetryChannel {
    pub fn from_raw(val: u8) -> Self {
        Self {
            priority: TelemetryPriority::from_bits(val & 0x03),
            averaging: (val >> 2) & 0x07,
        }
    }

    pub fn to_raw(self) -> u8 {
        (self.averaging & 0x07) << 2 | self.priority.to_bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryConfig {
    pub vout: TelemetryChannel,
    pub iout: TelemetryChannel,
    pub temp: TelemetryChannel,
    pub vin: TelemetryChannel,
}

impl TelemetryConfig {
    pub fn from_bytes(data: [u8; 6]) -> Self {
        // data[0] = block length (should be 4), data[1..5] = channel bytes
        Self {
            vout: TelemetryChannel::from_raw(data[0]),
            iout: TelemetryChannel::from_raw(data[1]),
            temp: TelemetryChannel::from_raw(data[2]),
            vin: TelemetryChannel::from_raw(data[3]),
        }
    }

    pub fn to_bytes(self) -> [u8; 6] {
        [
            self.vout.to_raw(),
            self.iout.to_raw(),
            self.temp.to_raw(),
            self.vin.to_raw(),
            0,
            0,
        ]
    }
}

// ---------------------------------------------------------------------------
// ReadAll (DAh, 14-byte block, read-only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadAll {
    pub status_word: u16,
    pub vout_raw: u16,
    pub iout_raw: u16,
    pub temperature_raw: u16,
    pub vin_raw: u16,
}

impl ReadAll {
    pub fn from_bytes(data: [u8; 14]) -> Self {
        Self {
            status_word: u16::from_le_bytes([data[0], data[1]]),
            vout_raw: u16::from_le_bytes([data[2], data[3]]),
            iout_raw: u16::from_le_bytes([data[4], data[5]]),
            temperature_raw: u16::from_le_bytes([data[6], data[7]]),
            vin_raw: u16::from_le_bytes([data[8], data[9]]),
        }
    }

    pub fn vout_f32(&self, exponent: i8) -> f32 {
        ULinear16::from_raw(self.vout_raw).to_f32(exponent)
    }

    pub fn iout_f32(&self) -> f32 {
        Linear11::from_raw(self.iout_raw).to_f32()
    }

    pub fn temperature_f32(&self) -> f32 {
        Linear11::from_raw(self.temperature_raw).to_f32()
    }

    pub fn vin_f32(&self) -> f32 {
        Linear11::from_raw(self.vin_raw).to_f32()
    }
}

// ---------------------------------------------------------------------------
// StatusAll (DBh, 7-byte block, read-only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusAll {
    pub vout: u8,
    pub iout: u8,
    pub input: u8,
    pub temperature: u8,
    pub cml: u8,
    pub other: u8,
    pub mfr: u8,
}

impl StatusAll {
    pub fn from_bytes(data: [u8; 7]) -> Self {
        Self {
            vout: data[0],
            iout: data[1],
            input: data[2],
            temperature: data[3],
            cml: data[4],
            other: data[5],
            mfr: data[6],
        }
    }
}

// ---------------------------------------------------------------------------
// StatusPhase (DCh, word)
// ---------------------------------------------------------------------------

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StatusPhase: u16 {
        const PH0 = 1 << 0;
        const PH1 = 1 << 1;
        const PH2 = 1 << 2;
        const PH3 = 1 << 3;
    }
}

impl StatusPhase {
    pub fn from_raw(raw: u16) -> Self {
        Self::from_bits_truncate(raw)
    }

    pub fn to_raw(self) -> u16 {
        self.bits()
    }
}

// ---------------------------------------------------------------------------
// SyncConfig (E4h, byte)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncDirection {
    Disabled,
    SyncOut,
    SyncIn,
    AutoDetect,
}

impl SyncDirection {
    pub fn from_bits(val: u8) -> Self {
        match val & 0x03 {
            0 => Self::Disabled,
            1 => Self::SyncOut,
            2 => Self::SyncIn,
            _ => Self::AutoDetect,
        }
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Self::Disabled => 0,
            Self::SyncOut => 1,
            Self::SyncIn => 2,
            Self::AutoDetect => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncConfig {
    pub direction: SyncDirection,
    pub rising_edge: bool,
}

impl SyncConfig {
    pub fn from_raw(raw: u8) -> Self {
        Self {
            direction: SyncDirection::from_bits((raw >> 6) & 0x03),
            rising_edge: (raw >> 5) & 1 != 0,
        }
    }

    pub fn to_raw(self) -> u8 {
        (self.direction.to_bits() << 6) | ((self.rising_edge as u8) << 5) | 0x10
    }
}

// ---------------------------------------------------------------------------
// StackConfig (ECh, word)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseCount {
    Single,
    TwoPhase,
    ThreePhase,
    FourPhase,
}

impl PhaseCount {
    pub fn from_bits(val: u8) -> Self {
        match val & 0x0F {
            0 => Self::Single,
            1 => Self::TwoPhase,
            2 => Self::ThreePhase,
            3 => Self::FourPhase,
            _ => Self::Single,
        }
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Self::Single => 0,
            Self::TwoPhase => 1,
            Self::ThreePhase => 2,
            Self::FourPhase => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackConfig {
    pub phase_count: PhaseCount,
}

impl StackConfig {
    pub fn from_raw(raw: u16) -> Self {
        Self {
            phase_count: PhaseCount::from_bits((raw & 0x0F) as u8),
        }
    }

    pub fn to_raw(self) -> u16 {
        self.phase_count.to_bits() as u16
    }
}

// ---------------------------------------------------------------------------
// MiscOptions (EDh, word)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdcResolution {
    Bit12,
    Bit10,
    Bit8,
    Bit6,
}

impl AdcResolution {
    pub fn from_bits(val: u8) -> Self {
        match val & 0x03 {
            0 => Self::Bit12,
            1 => Self::Bit10,
            2 => Self::Bit8,
            _ => Self::Bit6,
        }
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Self::Bit12 => 0,
            Self::Bit10 => 1,
            Self::Bit8 => 2,
            Self::Bit6 => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MiscOptions {
    pub pin_detect_stack: bool,
    pub pin_detect_sync: bool,
    pub pin_detect_comp: bool,
    pub pin_detect_address: bool,
    pub pin_detect_interleave: bool,
    pub pec: u8,
    pub fault_counter_reset: bool,
    pub adc_resolution: AdcResolution,
}

impl MiscOptions {
    pub fn from_raw(raw: u16) -> Self {
        Self {
            pin_detect_stack: (raw >> 12) & 1 != 0,
            pin_detect_sync: (raw >> 11) & 1 != 0,
            pin_detect_comp: (raw >> 9) & 1 != 0,
            pin_detect_address: (raw >> 8) & 1 != 0,
            pin_detect_interleave: (raw >> 5) & 1 != 0,
            pec: ((raw >> 3) & 0x03) as u8,
            fault_counter_reset: (raw >> 2) & 1 != 0,
            adc_resolution: AdcResolution::from_bits((raw & 0x03) as u8),
        }
    }

    pub fn to_raw(self) -> u16 {
        ((self.pin_detect_stack as u16) << 12)
            | ((self.pin_detect_sync as u16) << 11)
            | ((self.pin_detect_comp as u16) << 9)
            | ((self.pin_detect_address as u16) << 8)
            | ((self.pin_detect_interleave as u16) << 5)
            | (((self.pec & 0x03) as u16) << 3)
            | ((self.fault_counter_reset as u16) << 2)
            | (self.adc_resolution.to_bits() as u16)
    }
}

// ---------------------------------------------------------------------------
// PinDetectOverride (EEh, word)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinDetectOverride {
    pub stack_config: bool,
    pub sync_config: bool,
    pub comp_config: bool,
    pub address: bool,
    pub interleave: bool,
    pub vout: bool,
    raw_spare: u16,
}

const PDO_KNOWN_MASK: u16 =
    (1 << 12) | (1 << 11) | (1 << 9) | (1 << 8) | (1 << 5) | (1 << 0);

impl PinDetectOverride {
    pub fn from_raw(raw: u16) -> Self {
        Self {
            stack_config: (raw >> 12) & 1 != 0,
            sync_config: (raw >> 11) & 1 != 0,
            comp_config: (raw >> 9) & 1 != 0,
            address: (raw >> 8) & 1 != 0,
            interleave: (raw >> 5) & 1 != 0,
            vout: raw & 1 != 0,
            raw_spare: raw & !PDO_KNOWN_MASK,
        }
    }

    pub fn to_raw(self) -> u16 {
        ((self.stack_config as u16) << 12)
            | ((self.sync_config as u16) << 11)
            | ((self.comp_config as u16) << 9)
            | ((self.address as u16) << 8)
            | ((self.interleave as u16) << 5)
            | (self.vout as u16)
            | self.raw_spare
    }
}

// ---------------------------------------------------------------------------
// SimulateFaults (F1h, word)
// ---------------------------------------------------------------------------

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct SimulateFaults: u16 {
        const FAULT_PERSIST = 1 << 15;
        const TEMP_OTF      = 1 << 14;
        const VIN_OVF       = 1 << 12;
        const VIN_OFF       = 1 << 11;
        const VOUT_UVF      = 1 << 10;
        const VOUT_OVF      = 1 << 9;
        const IOUT_OCF      = 1 << 8;
        const WARN_PERSIST  = 1 << 7;
        const TEMP_OTW      = 1 << 5;
        const VIN_UVW       = 1 << 4;
        const VOUT_UVW      = 1 << 3;
        const VOUT_OVW      = 1 << 2;
        const IOUT_OCW      = 1 << 1;
    }
}

impl SimulateFaults {
    pub fn from_raw(raw: u16) -> Self {
        Self::from_bits_truncate(raw)
    }

    pub fn to_raw(self) -> u16 {
        self.bits()
    }
}

// ---------------------------------------------------------------------------
// PowerStageConfig (B5h, byte)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vdd5Voltage {
    V3_9,
    V4_1,
    V4_3,
    V4_5,
    V4_7,
    V4_9,
    V5_1,
    V5_3,
}

impl Vdd5Voltage {
    pub fn from_bits(val: u8) -> Self {
        match val {
            0 => Self::V3_9,
            1 => Self::V4_1,
            2 => Self::V4_3,
            3 => Self::V4_5,
            4 => Self::V4_7,
            5 => Self::V4_9,
            6 => Self::V5_1,
            7 => Self::V5_3,
            _ => Self::V5_3,
        }
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Self::V3_9 => 0,
            Self::V4_1 => 1,
            Self::V4_3 => 2,
            Self::V4_5 => 3,
            Self::V4_7 => 4,
            Self::V4_9 => 5,
            Self::V5_1 => 6,
            Self::V5_3 => 7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerStageConfig {
    pub vdd5: Vdd5Voltage,
}

impl PowerStageConfig {
    pub fn from_raw(raw: u8) -> Self {
        Self {
            vdd5: Vdd5Voltage::from_bits((raw >> 4) & 0x0F),
        }
    }

    pub fn to_raw(self) -> u8 {
        self.vdd5.to_bits() << 4
    }
}

// ---------------------------------------------------------------------------
// CompensationConfig (B1h, 5-byte block)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompensationConfig(pub [u8; 5]);

impl CompensationConfig {
    pub fn from_bytes(data: [u8; 5]) -> Self {
        Self(data)
    }

    pub fn to_bytes(self) -> [u8; 5] {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Operation (01h, byte)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarginMode {
    Off,
    MarginLowIgnore,
    MarginLowAct,
    MarginHighIgnore,
    MarginHighAct,
}

impl MarginMode {
    pub fn from_bits(val: u8) -> Self {
        match val & 0x0F {
            0b0000 => Self::Off,
            0b0100 => Self::MarginLowIgnore,
            0b0101 => Self::MarginLowAct,
            0b1000 => Self::MarginHighIgnore,
            0b1001 => Self::MarginHighAct,
            _ => Self::Off,
        }
    }

    pub fn to_bits(self) -> u8 {
        match self {
            Self::Off => 0b0000,
            Self::MarginLowIgnore => 0b0100,
            Self::MarginLowAct => 0b0101,
            Self::MarginHighIgnore => 0b1000,
            Self::MarginHighAct => 0b1001,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation {
    pub on: bool,
    pub soft_off: bool,
    pub margin: MarginMode,
    raw_lower: u8, // bits [5:0] raw, preserves unknown layouts
}

impl Operation {
    pub fn from_raw(raw: u8) -> Self {
        Self {
            on: (raw >> 7) & 1 != 0,
            soft_off: (raw >> 6) & 1 != 0,
            margin: MarginMode::from_bits((raw >> 2) & 0x0F),
            raw_lower: raw & 0x3F,
        }
    }

    pub fn to_raw(self) -> u8 {
        ((self.on as u8) << 7) | ((self.soft_off as u8) << 6) | self.raw_lower
    }

    /// Set the margin mode, updating the lower bits accordingly.
    pub fn with_margin(mut self, margin: MarginMode) -> Self {
        self.raw_lower = (self.raw_lower & 0x03) | (margin.to_bits() << 2);
        self.margin = margin;
        self
    }
}

// ---------------------------------------------------------------------------
// OnOffConfig (02h, byte)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OnOffConfig {
    pub power_up_control: bool,
    pub cmd_enable: bool,
    pub control_pin_enable: bool,
    pub control_pin_active_high: bool,
    pub immediate_off: bool,
}

impl OnOffConfig {
    pub fn from_raw(raw: u8) -> Self {
        Self {
            power_up_control: (raw >> 4) & 1 != 0,
            cmd_enable: (raw >> 3) & 1 != 0,
            control_pin_enable: (raw >> 2) & 1 != 0,
            control_pin_active_high: (raw >> 1) & 1 != 0,
            immediate_off: raw & 1 != 0,
        }
    }

    pub fn to_raw(self) -> u8 {
        ((self.power_up_control as u8) << 4)
            | ((self.cmd_enable as u8) << 3)
            | ((self.control_pin_enable as u8) << 2)
            | ((self.control_pin_active_high as u8) << 1)
            | (self.immediate_off as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_round_trip() {
        let raw = 0x04u8;
        let op = Operation::from_raw(raw);
        assert!(!op.on);
        assert!(!op.soft_off);
        assert_eq!(op.to_raw(), raw);
    }

    #[test]
    fn on_off_config_round_trip() {
        let raw = 0x17u8;
        let cfg = OnOffConfig::from_raw(raw);
        assert!(cfg.power_up_control);
        assert!(!cfg.cmd_enable);
        assert!(cfg.control_pin_enable);
        assert!(cfg.control_pin_active_high);
        assert!(cfg.immediate_off);
        assert_eq!(cfg.to_raw(), raw);
    }

    #[test]
    fn sync_config_round_trip() {
        let raw = 0xF0u8;
        let cfg = SyncConfig::from_raw(raw);
        assert_eq!(cfg.direction as u8, SyncDirection::AutoDetect as u8);
        assert!(cfg.rising_edge);
        assert_eq!(cfg.to_raw(), raw);
    }

    #[test]
    fn stack_config_round_trip() {
        let raw = 0x0000u16;
        let cfg = StackConfig::from_raw(raw);
        assert_eq!(cfg.phase_count as u8, PhaseCount::Single as u8);
        assert_eq!(cfg.to_raw(), raw);
    }

    #[test]
    fn misc_options_round_trip() {
        let raw = 0x0000u16;
        let opts = MiscOptions::from_raw(raw);
        assert!(!opts.pin_detect_stack);
        assert!(!opts.pin_detect_sync);
        assert_eq!(opts.to_raw(), raw);
    }

    #[test]
    fn pin_detect_override_round_trip() {
        let raw = 0x1F2Fu16;
        let pdo = PinDetectOverride::from_raw(raw);
        assert!(pdo.stack_config);
        assert!(pdo.sync_config);
        assert!(pdo.comp_config);
        assert!(pdo.address);
        assert!(pdo.interleave);
        assert!(pdo.vout);
        assert_eq!(pdo.to_raw(), raw);
    }

    #[test]
    fn power_stage_config_round_trip() {
        let raw = 0x70u8;
        let cfg = PowerStageConfig::from_raw(raw);
        assert_eq!(cfg.vdd5 as u8, Vdd5Voltage::V5_3 as u8);
        assert_eq!(cfg.to_raw(), raw);
    }

    #[test]
    fn read_all_parsing() {
        let mut data = [0u8; 14];
        // status_word = 0x0000
        data[0] = 0x00;
        data[1] = 0x00;
        // vout_raw = 0x0100 (ULinear16)
        data[2] = 0x00;
        data[3] = 0x01;
        // iout_raw = 0xD00A => Linear11
        data[4] = 0x0A;
        data[5] = 0xD0;
        // temperature_raw = 0x0C19 => Linear11
        data[6] = 0x19;
        data[7] = 0x0C;
        // vin_raw = 0xE860 => Linear11
        data[8] = 0x60;
        data[9] = 0xE8;

        let ra = ReadAll::from_bytes(data);
        assert_eq!(ra.status_word, 0x0000);
        assert_eq!(ra.vout_raw, 0x0100);
        assert_eq!(ra.iout_raw, 0xD00A);
        assert_eq!(ra.temperature_raw, 0x0C19);
        assert_eq!(ra.vin_raw, 0xE860);
    }

    #[test]
    fn status_all_parsing() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let sa = StatusAll::from_bytes(data);
        assert_eq!(sa.vout, 0x01);
        assert_eq!(sa.iout, 0x02);
        assert_eq!(sa.input, 0x03);
        assert_eq!(sa.temperature, 0x04);
        assert_eq!(sa.cml, 0x05);
        assert_eq!(sa.other, 0x06);
        assert_eq!(sa.mfr, 0x07);
    }

    #[test]
    fn telemetry_config_round_trip() {
        let data = [0x03, 0x03, 0x03, 0x03, 0x00, 0x00];
        let cfg = TelemetryConfig::from_bytes(data);
        assert_eq!(cfg.to_bytes(), data);
    }
}
