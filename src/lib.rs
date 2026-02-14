#![no_std]

pub mod registers;

use embedded_hal_async::i2c::I2c;
use heapless::Vec;
use pmbus_adapter::{
    Linear11, PmbusAdaptor, StatusByte, StatusCml, StatusInput, StatusIout, StatusOther,
    StatusTemperature, StatusVout, StatusWord, ULinear16, VoutMode, VoutModeType,
};

pub use registers::*;

pub const DEFAULT_ADDR: u8 = 0x24;
pub const DEFAULT_VOUT_EXPONENT: i8 = -9;

// MFR-specific command codes
const CMD_COMPENSATION_CONFIG: u8 = 0xB1;
const CMD_POWER_STAGE_CONFIG: u8 = 0xB5;
const CMD_TELEMETRY_CONFIG: u8 = 0xD0;
const CMD_READ_ALL: u8 = 0xDA;
const CMD_STATUS_ALL: u8 = 0xDB;
const CMD_STATUS_PHASE: u8 = 0xDC;
const CMD_SYNC_CONFIG: u8 = 0xE4;
const CMD_STACK_CONFIG: u8 = 0xEC;
const CMD_MISC_OPTIONS: u8 = 0xED;
const CMD_PIN_DETECT_OVERRIDE: u8 = 0xEE;
const CMD_SLAVE_ADDRESS: u8 = 0xEF;
const CMD_NVM_CHECKSUM: u8 = 0xF0;
const CMD_SIMULATE_FAULTS: u8 = 0xF1;
const CMD_FUSION_ID0: u8 = 0xFC;
const CMD_FUSION_ID1: u8 = 0xFD;

pub struct Tps546<BUS: I2c> {
    pmbus: PmbusAdaptor<BUS>,
    addr: u8,
    vout_exponent: i8,
}

impl<BUS: I2c + 'static> Tps546<BUS> {
    pub fn new(pmbus: PmbusAdaptor<BUS>, addr: u8) -> Self {
        Self {
            pmbus,
            addr,
            vout_exponent: DEFAULT_VOUT_EXPONENT,
        }
    }

    pub async fn init(&mut self) -> Result<(), BUS::Error> {
        let mode = self.pmbus.get_vout_mode(self.addr).await?;
        if let VoutModeType::ULinear16 { exponent } = mode.mode {
            self.vout_exponent = exponent;
        }
        Ok(())
    }

    pub fn inner(&mut self) -> &mut PmbusAdaptor<BUS> {
        &mut self.pmbus
    }

    pub fn release(self) -> PmbusAdaptor<BUS> {
        self.pmbus
    }

    pub fn addr(&self) -> u8 {
        self.addr
    }

    pub fn vout_exponent(&self) -> i8 {
        self.vout_exponent
    }

    pub fn set_vout_exponent(&mut self, exp: i8) {
        self.vout_exponent = exp;
    }

    // -----------------------------------------------------------------------
    // Send-byte commands
    // -----------------------------------------------------------------------

    pub async fn clear_faults(&mut self) -> Result<(), BUS::Error> {
        self.pmbus.clear_faults(self.addr).await
    }

    pub async fn store_user_all(&mut self) -> Result<(), BUS::Error> {
        self.pmbus.store_user_all(self.addr).await
    }

    pub async fn restore_user_all(&mut self) -> Result<(), BUS::Error> {
        self.pmbus.restore_user_all(self.addr).await
    }

    // -----------------------------------------------------------------------
    // Byte R/W (raw u8)
    // -----------------------------------------------------------------------

    pub async fn get_phase(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_phase(self.addr).await
    }

    pub async fn set_phase(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_phase(self.addr, val).await
    }

    pub async fn get_write_protect(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_write_protect(self.addr).await
    }

    pub async fn set_write_protect(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_write_protect(self.addr, val).await
    }

    pub async fn get_capability(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_capability(self.addr).await
    }

    pub async fn get_pmbus_revision(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_pmbus_revision(self.addr).await
    }

    // Fault response byte accessors
    pub async fn get_vout_ov_fault_response(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_vout_ov_fault_response(self.addr).await
    }

    pub async fn set_vout_ov_fault_response(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_vout_ov_fault_response(self.addr, val).await
    }

    pub async fn get_vout_uv_fault_response(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_vout_uv_fault_response(self.addr).await
    }

    pub async fn set_vout_uv_fault_response(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_vout_uv_fault_response(self.addr, val).await
    }

    pub async fn get_iout_oc_fault_response(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_iout_oc_fault_response(self.addr).await
    }

    pub async fn set_iout_oc_fault_response(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_iout_oc_fault_response(self.addr, val).await
    }

    pub async fn get_ot_fault_response(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_ot_fault_response(self.addr).await
    }

    pub async fn set_ot_fault_response(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_ot_fault_response(self.addr, val).await
    }

    pub async fn get_vin_ov_fault_response(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_vin_ov_fault_response(self.addr).await
    }

    pub async fn set_vin_ov_fault_response(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_vin_ov_fault_response(self.addr, val).await
    }

    pub async fn get_ton_max_fault_response(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_ton_max_fault_response(self.addr).await
    }

    pub async fn set_ton_max_fault_response(&mut self, val: u8) -> Result<(), BUS::Error> {
        self.pmbus.set_ton_max_fault_response(self.addr, val).await
    }

    // -----------------------------------------------------------------------
    // Byte R/W (typed)
    // -----------------------------------------------------------------------

    pub async fn get_operation(&mut self) -> Result<Operation, BUS::Error> {
        let raw = self.pmbus.get_operation(self.addr).await?;
        Ok(Operation::from_raw(raw))
    }

    pub async fn set_operation(&mut self, op: Operation) -> Result<(), BUS::Error> {
        self.pmbus.set_operation(self.addr, op.to_raw()).await
    }

    pub async fn get_on_off_config(&mut self) -> Result<OnOffConfig, BUS::Error> {
        let raw = self.pmbus.get_on_off_config(self.addr).await?;
        Ok(OnOffConfig::from_raw(raw))
    }

    pub async fn set_on_off_config(&mut self, cfg: OnOffConfig) -> Result<(), BUS::Error> {
        self.pmbus.set_on_off_config(self.addr, cfg.to_raw()).await
    }

    // -----------------------------------------------------------------------
    // VOUT_MODE (special)
    // -----------------------------------------------------------------------

    pub async fn get_vout_mode(&mut self) -> Result<VoutMode, BUS::Error> {
        self.pmbus.get_vout_mode(self.addr).await
    }

    pub async fn set_vout_mode(&mut self, mode: VoutMode) -> Result<(), BUS::Error> {
        self.pmbus.set_vout_mode(self.addr, mode).await?;
        if let VoutModeType::ULinear16 { exponent } = mode.mode {
            self.vout_exponent = exponent;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // ULinear16 R/W (f32 Volts, uses cached exponent)
    // -----------------------------------------------------------------------

    async fn get_ulinear16_word(
        &mut self,
        getter: impl AsyncFnOnce(&mut PmbusAdaptor<BUS>, u8) -> Result<u16, BUS::Error>,
    ) -> Result<f32, BUS::Error> {
        let raw = getter(&mut self.pmbus, self.addr).await?;
        Ok(ULinear16::from_raw(raw).to_f32(self.vout_exponent))
    }

    async fn set_ulinear16_word(
        &mut self,
        value: f32,
        setter: impl AsyncFnOnce(&mut PmbusAdaptor<BUS>, u8, u16) -> Result<(), BUS::Error>,
    ) -> Result<(), BUS::Error> {
        let raw = ULinear16::from_f32(value, self.vout_exponent)
            .unwrap_or(ULinear16::from_raw(0))
            .raw();
        setter(&mut self.pmbus, self.addr, raw).await
    }

    pub async fn get_vout_command(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_command).await
    }

    pub async fn set_vout_command(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_command).await
    }

    pub async fn get_vout_trim(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_trim).await
    }

    pub async fn set_vout_trim(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_trim).await
    }

    pub async fn get_vout_max(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_max).await
    }

    pub async fn set_vout_max(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_max).await
    }

    pub async fn get_vout_margin_high(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_margin_high).await
    }

    pub async fn set_vout_margin_high(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_margin_high).await
    }

    pub async fn get_vout_margin_low(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_margin_low).await
    }

    pub async fn set_vout_margin_low(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_margin_low).await
    }

    pub async fn get_vout_min(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_min).await
    }

    pub async fn set_vout_min(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_min).await
    }

    pub async fn get_vout_ov_fault_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_ov_fault_limit).await
    }

    pub async fn set_vout_ov_fault_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_ov_fault_limit).await
    }

    pub async fn get_vout_ov_warn_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_ov_warn_limit).await
    }

    pub async fn set_vout_ov_warn_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_ov_warn_limit).await
    }

    pub async fn get_vout_uv_warn_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_uv_warn_limit).await
    }

    pub async fn set_vout_uv_warn_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_uv_warn_limit).await
    }

    pub async fn get_vout_uv_fault_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::get_vout_uv_fault_limit).await
    }

    pub async fn set_vout_uv_fault_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_ulinear16_word(v, PmbusAdaptor::set_vout_uv_fault_limit).await
    }

    // ULinear16 read-only
    pub async fn read_vout(&mut self) -> Result<f32, BUS::Error> {
        self.get_ulinear16_word(PmbusAdaptor::read_vout).await
    }

    // -----------------------------------------------------------------------
    // Linear11 R/W (f32)
    // -----------------------------------------------------------------------

    async fn get_linear11_word(
        &mut self,
        getter: impl AsyncFnOnce(&mut PmbusAdaptor<BUS>, u8) -> Result<u16, BUS::Error>,
    ) -> Result<f32, BUS::Error> {
        let raw = getter(&mut self.pmbus, self.addr).await?;
        Ok(Linear11::from_raw(raw).to_f32())
    }

    async fn set_linear11_word(
        &mut self,
        value: f32,
        setter: impl AsyncFnOnce(&mut PmbusAdaptor<BUS>, u8, u16) -> Result<(), BUS::Error>,
    ) -> Result<(), BUS::Error> {
        let raw = Linear11::from_f32(value)
            .unwrap_or(Linear11::from_raw(0))
            .raw();
        setter(&mut self.pmbus, self.addr, raw).await
    }

    pub async fn get_vout_transition_rate(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_vout_transition_rate).await
    }

    pub async fn set_vout_transition_rate(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_vout_transition_rate).await
    }

    pub async fn get_vout_scale_loop(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_vout_scale_loop).await
    }

    pub async fn set_vout_scale_loop(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_vout_scale_loop).await
    }

    pub async fn get_frequency_switch(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_frequency_switch).await
    }

    pub async fn set_frequency_switch(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_frequency_switch).await
    }

    pub async fn get_vin_on(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_vin_on).await
    }

    pub async fn set_vin_on(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_vin_on).await
    }

    pub async fn get_vin_off(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_vin_off).await
    }

    pub async fn set_vin_off(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_vin_off).await
    }

    pub async fn get_iout_cal_gain(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_iout_cal_gain).await
    }

    pub async fn set_iout_cal_gain(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_iout_cal_gain).await
    }

    pub async fn get_iout_cal_offset(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_iout_cal_offset).await
    }

    pub async fn set_iout_cal_offset(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_iout_cal_offset).await
    }

    pub async fn get_iout_oc_fault_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_iout_oc_fault_limit).await
    }

    pub async fn set_iout_oc_fault_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_iout_oc_fault_limit).await
    }

    pub async fn get_iout_oc_warn_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_iout_oc_warn_limit).await
    }

    pub async fn set_iout_oc_warn_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_iout_oc_warn_limit).await
    }

    pub async fn get_ot_fault_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_ot_fault_limit).await
    }

    pub async fn set_ot_fault_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_ot_fault_limit).await
    }

    pub async fn get_ot_warn_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_ot_warn_limit).await
    }

    pub async fn set_ot_warn_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_ot_warn_limit).await
    }

    pub async fn get_vin_ov_fault_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_vin_ov_fault_limit).await
    }

    pub async fn set_vin_ov_fault_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_vin_ov_fault_limit).await
    }

    pub async fn get_vin_uv_warn_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_vin_uv_warn_limit).await
    }

    pub async fn set_vin_uv_warn_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_vin_uv_warn_limit).await
    }

    pub async fn get_ton_delay(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_ton_delay).await
    }

    pub async fn set_ton_delay(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_ton_delay).await
    }

    pub async fn get_ton_rise(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_ton_rise).await
    }

    pub async fn set_ton_rise(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_ton_rise).await
    }

    pub async fn get_ton_max_fault_limit(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_ton_max_fault_limit).await
    }

    pub async fn set_ton_max_fault_limit(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_ton_max_fault_limit).await
    }

    pub async fn get_toff_delay(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_toff_delay).await
    }

    pub async fn set_toff_delay(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_toff_delay).await
    }

    pub async fn get_toff_fall(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::get_toff_fall).await
    }

    pub async fn set_toff_fall(&mut self, v: f32) -> Result<(), BUS::Error> {
        self.set_linear11_word(v, PmbusAdaptor::set_toff_fall).await
    }

    // Linear11 read-only
    pub async fn read_vin(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::read_vin).await
    }

    pub async fn read_iout(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::read_iout).await
    }

    pub async fn read_temperature(&mut self) -> Result<f32, BUS::Error> {
        self.get_linear11_word(PmbusAdaptor::read_temperature_1).await
    }

    // -----------------------------------------------------------------------
    // Word R/W (raw u16)
    // -----------------------------------------------------------------------

    pub async fn get_interleave(&mut self) -> Result<u16, BUS::Error> {
        self.pmbus.get_interleave(self.addr).await
    }

    pub async fn set_interleave(&mut self, val: u16) -> Result<(), BUS::Error> {
        self.pmbus.set_interleave(self.addr, val).await
    }

    pub async fn get_smbalert_mask(&mut self, status_register: u8) -> Result<u8, BUS::Error> {
        self.pmbus.get_smbalert_mask(self.addr, status_register).await
    }

    pub async fn set_smbalert_mask(&mut self, data: u16) -> Result<(), BUS::Error> {
        self.pmbus.set_smbalert_mask(self.addr, data).await
    }

    // -----------------------------------------------------------------------
    // Status reads (typed)
    // -----------------------------------------------------------------------

    pub async fn get_status_byte(&mut self) -> Result<StatusByte, BUS::Error> {
        self.pmbus.get_status_byte(self.addr).await
    }

    pub async fn get_status_word(&mut self) -> Result<StatusWord, BUS::Error> {
        self.pmbus.get_status_word(self.addr).await
    }

    pub async fn get_status_vout(&mut self) -> Result<StatusVout, BUS::Error> {
        self.pmbus.get_status_vout(self.addr).await
    }

    pub async fn get_status_iout(&mut self) -> Result<StatusIout, BUS::Error> {
        self.pmbus.get_status_iout(self.addr).await
    }

    pub async fn get_status_input(&mut self) -> Result<StatusInput, BUS::Error> {
        self.pmbus.get_status_input(self.addr).await
    }

    pub async fn get_status_temperature(&mut self) -> Result<StatusTemperature, BUS::Error> {
        self.pmbus.get_status_temperature(self.addr).await
    }

    pub async fn get_status_cml(&mut self) -> Result<StatusCml, BUS::Error> {
        self.pmbus.get_status_cml(self.addr).await
    }

    pub async fn get_status_other(&mut self) -> Result<StatusOther, BUS::Error> {
        self.pmbus.get_status_other(self.addr).await
    }

    pub async fn get_status_mfr_specific(&mut self) -> Result<u8, BUS::Error> {
        self.pmbus.get_status_mfr_specific(self.addr).await
    }

    // -----------------------------------------------------------------------
    // Block R/W (MFR strings)
    // -----------------------------------------------------------------------

    pub async fn get_mfr_id(&mut self) -> Result<Vec<u8, 32>, BUS::Error> {
        self.pmbus.get_mfr_id(self.addr).await
    }

    pub async fn set_mfr_id(&mut self, data: &[u8]) -> Result<(), BUS::Error> {
        self.pmbus.set_mfr_id(self.addr, data).await
    }

    pub async fn get_mfr_model(&mut self) -> Result<Vec<u8, 32>, BUS::Error> {
        self.pmbus.get_mfr_model(self.addr).await
    }

    pub async fn set_mfr_model(&mut self, data: &[u8]) -> Result<(), BUS::Error> {
        self.pmbus.set_mfr_model(self.addr, data).await
    }

    pub async fn get_mfr_revision(&mut self) -> Result<Vec<u8, 32>, BUS::Error> {
        self.pmbus.get_mfr_revision(self.addr).await
    }

    pub async fn set_mfr_revision(&mut self, data: &[u8]) -> Result<(), BUS::Error> {
        self.pmbus.set_mfr_revision(self.addr, data).await
    }

    pub async fn get_mfr_serial(&mut self) -> Result<Vec<u8, 32>, BUS::Error> {
        self.pmbus.get_mfr_serial(self.addr).await
    }

    pub async fn set_mfr_serial(&mut self, data: &[u8]) -> Result<(), BUS::Error> {
        self.pmbus.set_mfr_serial(self.addr, data).await
    }

    // Block read-only
    pub async fn get_ic_device_id(&mut self) -> Result<Vec<u8, 32>, BUS::Error> {
        self.pmbus.get_ic_device_id(self.addr).await
    }

    pub async fn get_ic_device_rev(&mut self) -> Result<Vec<u8, 32>, BUS::Error> {
        self.pmbus.get_ic_device_rev(self.addr).await
    }

    // -----------------------------------------------------------------------
    // MFR-specific commands
    // -----------------------------------------------------------------------

    // TelemetryConfig (D0h, block 6B)
    pub async fn get_telemetry_config(&mut self) -> Result<TelemetryConfig, BUS::Error> {
        let data = self.pmbus.raw_block_read(self.addr, CMD_TELEMETRY_CONFIG).await?;
        let mut buf = [0u8; 6];
        let len = data.len().min(6);
        buf[..len].copy_from_slice(&data[..len]);
        Ok(TelemetryConfig::from_bytes(buf))
    }

    pub async fn set_telemetry_config(&mut self, cfg: TelemetryConfig) -> Result<(), BUS::Error> {
        let bytes = cfg.to_bytes();
        self.pmbus.raw_block_write(self.addr, CMD_TELEMETRY_CONFIG, &bytes).await
    }

    // ReadAll (DAh, block 14B, read-only)
    pub async fn read_all(&mut self) -> Result<ReadAll, BUS::Error> {
        let data = self.pmbus.raw_block_read(self.addr, CMD_READ_ALL).await?;
        let mut buf = [0u8; 14];
        let len = data.len().min(14);
        buf[..len].copy_from_slice(&data[..len]);
        Ok(ReadAll::from_bytes(buf))
    }

    // StatusAll (DBh, block 7B, read-only)
    pub async fn status_all(&mut self) -> Result<StatusAll, BUS::Error> {
        let data = self.pmbus.raw_block_read(self.addr, CMD_STATUS_ALL).await?;
        let mut buf = [0u8; 7];
        let len = data.len().min(7);
        buf[..len].copy_from_slice(&data[..len]);
        Ok(StatusAll::from_bytes(buf))
    }

    // StatusPhase (DCh, word)
    pub async fn get_status_phase(&mut self) -> Result<StatusPhase, BUS::Error> {
        let raw = self.pmbus.raw_read_word(self.addr, CMD_STATUS_PHASE).await?;
        Ok(StatusPhase::from_raw(raw))
    }

    pub async fn set_status_phase(&mut self, val: StatusPhase) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_word(self.addr, CMD_STATUS_PHASE, val.to_raw()).await
    }

    // SyncConfig (E4h, byte)
    pub async fn get_sync_config(&mut self) -> Result<SyncConfig, BUS::Error> {
        let raw = self.pmbus.raw_read_byte(self.addr, CMD_SYNC_CONFIG).await?;
        Ok(SyncConfig::from_raw(raw))
    }

    pub async fn set_sync_config(&mut self, cfg: SyncConfig) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_byte(self.addr, CMD_SYNC_CONFIG, cfg.to_raw()).await
    }

    // StackConfig (ECh, word)
    pub async fn get_stack_config(&mut self) -> Result<StackConfig, BUS::Error> {
        let raw = self.pmbus.raw_read_word(self.addr, CMD_STACK_CONFIG).await?;
        Ok(StackConfig::from_raw(raw))
    }

    pub async fn set_stack_config(&mut self, cfg: StackConfig) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_word(self.addr, CMD_STACK_CONFIG, cfg.to_raw()).await
    }

    // MiscOptions (EDh, word)
    pub async fn get_misc_options(&mut self) -> Result<MiscOptions, BUS::Error> {
        let raw = self.pmbus.raw_read_word(self.addr, CMD_MISC_OPTIONS).await?;
        Ok(MiscOptions::from_raw(raw))
    }

    pub async fn set_misc_options(&mut self, opts: MiscOptions) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_word(self.addr, CMD_MISC_OPTIONS, opts.to_raw()).await
    }

    // PinDetectOverride (EEh, word)
    pub async fn get_pin_detect_override(&mut self) -> Result<PinDetectOverride, BUS::Error> {
        let raw = self.pmbus.raw_read_word(self.addr, CMD_PIN_DETECT_OVERRIDE).await?;
        Ok(PinDetectOverride::from_raw(raw))
    }

    pub async fn set_pin_detect_override(&mut self, val: PinDetectOverride) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_word(self.addr, CMD_PIN_DETECT_OVERRIDE, val.to_raw()).await
    }

    // SlaveAddress (EFh, byte)
    pub async fn get_slave_address(&mut self) -> Result<u8, BUS::Error> {
        let raw = self.pmbus.raw_read_byte(self.addr, CMD_SLAVE_ADDRESS).await?;
        Ok(raw & 0x7F)
    }

    pub async fn set_slave_address(&mut self, addr: u8) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_byte(self.addr, CMD_SLAVE_ADDRESS, addr & 0x7F).await
    }

    // NVM Checksum (F0h, word, read-only)
    pub async fn get_nvm_checksum(&mut self) -> Result<u16, BUS::Error> {
        self.pmbus.raw_read_word(self.addr, CMD_NVM_CHECKSUM).await
    }

    // SimulateFaults (F1h, word)
    pub async fn get_simulate_faults(&mut self) -> Result<SimulateFaults, BUS::Error> {
        let raw = self.pmbus.raw_read_word(self.addr, CMD_SIMULATE_FAULTS).await?;
        Ok(SimulateFaults::from_raw(raw))
    }

    pub async fn set_simulate_faults(&mut self, val: SimulateFaults) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_word(self.addr, CMD_SIMULATE_FAULTS, val.to_raw()).await
    }

    // FusionID0 (FCh, word, read-only)
    pub async fn get_fusion_id0(&mut self) -> Result<u16, BUS::Error> {
        self.pmbus.raw_read_word(self.addr, CMD_FUSION_ID0).await
    }

    // FusionID1 (FDh, block 6B, read-only)
    pub async fn get_fusion_id1(&mut self) -> Result<[u8; 6], BUS::Error> {
        let data = self.pmbus.raw_block_read(self.addr, CMD_FUSION_ID1).await?;
        let mut buf = [0u8; 6];
        let len = data.len().min(6);
        buf[..len].copy_from_slice(&data[..len]);
        Ok(buf)
    }

    // CompensationConfig (B1h, block 5B)
    pub async fn get_compensation_config(&mut self) -> Result<CompensationConfig, BUS::Error> {
        let data = self.pmbus.raw_block_read(self.addr, CMD_COMPENSATION_CONFIG).await?;
        let mut buf = [0u8; 5];
        let len = data.len().min(5);
        buf[..len].copy_from_slice(&data[..len]);
        Ok(CompensationConfig::from_bytes(buf))
    }

    pub async fn set_compensation_config(&mut self, cfg: CompensationConfig) -> Result<(), BUS::Error> {
        self.pmbus.raw_block_write(self.addr, CMD_COMPENSATION_CONFIG, &cfg.to_bytes()).await
    }

    // PowerStageConfig (B5h, byte)
    pub async fn get_power_stage_config(&mut self) -> Result<PowerStageConfig, BUS::Error> {
        let raw = self.pmbus.raw_read_byte(self.addr, CMD_POWER_STAGE_CONFIG).await?;
        Ok(PowerStageConfig::from_raw(raw))
    }

    pub async fn set_power_stage_config(&mut self, cfg: PowerStageConfig) -> Result<(), BUS::Error> {
        self.pmbus.raw_write_byte(self.addr, CMD_POWER_STAGE_CONFIG, cfg.to_raw()).await
    }
}
