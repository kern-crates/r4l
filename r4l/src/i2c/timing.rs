//! I2C Time configuration
use core::fmt;
use crate::device::Device;
use crate::c_str;
use crate::str::CStr;

/// i2c Speed mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum I2cSpeedMode {
    /// Standard Speed Mode.
    StandMode = 0,
    /// Fast Speed Mode.
    FastMode,
    /// Fast Plus Mode.
    FastPlusMode,
    /// TURBO Mode.
    TurboMode,
    /// High Speed.
    HighSpeedMode,
    /// ULTRA_FAST.
    UltraFastMode,
    /// Unknown.
    UnknownMode,
}

/// I2C standard mode max bus frequency in hz
pub const I2C_MAX_STANDARD_MODE_FREQ: u32 = 100000;
/// I2C fast mode max bus frequency in hz
pub const I2C_MAX_FAST_MODE_FREQ: u32 = 400000;
/// I2C fast plus mode max bus frequency in hz
pub const I2C_MAX_FAST_MODE_PLUS_FREQ: u32 = 1000000;
/// I2C turbo mode max bus frequency in hz
pub const I2C_MAX_TURBO_MODE_FREQ: u32 = 1400000;
/// I2C high speed mode max bus frequency in hz
pub const I2C_MAX_HIGH_SPEED_MODE_FREQ: u32 = 3400000;
/// I2C ultra fast mode max bus frequency in hz
pub const I2C_MAX_ULTRA_FAST_MODE_FREQ: u32 = 5000000;

impl I2cSpeedMode {
    /// From a u32 bus_freq_hz to SpeedMode
    pub fn from_bus_freq(bus_freq: u32) -> Self {
        match bus_freq {
            I2C_MAX_STANDARD_MODE_FREQ =>   I2cSpeedMode::StandMode,
            I2C_MAX_FAST_MODE_FREQ =>       I2cSpeedMode::FastMode,
            I2C_MAX_FAST_MODE_PLUS_FREQ =>  I2cSpeedMode::FastPlusMode,
            I2C_MAX_TURBO_MODE_FREQ =>      I2cSpeedMode::TurboMode,
            I2C_MAX_HIGH_SPEED_MODE_FREQ => I2cSpeedMode::HighSpeedMode,
            I2C_MAX_ULTRA_FAST_MODE_FREQ => I2cSpeedMode::UltraFastMode,
            _ => I2cSpeedMode::UnknownMode,
        }
    }
}

impl fmt::Display for I2cSpeedMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// I2C timing config for all i2c driver
///
/// An instance of `I2cTiming` include can be used for any i2c driver to describe
/// the bus frequency in Hz
/// time SCL signal takes to rise in ns; t(r) in the I2C specification
/// time SCL signal takes to fall in ns; t(f) in the I2C specification
/// time IP core additionally needs to setup SCL in ns
/// time SDA signal takes to fall in ns; t(f) in the I2C specification
/// time IP core additionally needs to hold SDA in ns
/// width in ns of spikes on i2c lines that the IP core digital filter can filter out
/// threshold frequency for the low pass IP core analog filter
#[derive(Clone,Copy,PartialEq, Eq, Debug)]
pub struct I2cTiming {
    bus_freq_hz: u32,
    scl_rise_ns: u32,
    scl_fall_ns: u32,
    scl_int_delay_ns: u32,
    sda_fall_ns: u32,
    sda_hold_ns: u32,
    digital_filter_width_ns: u32,
    analog_filter_cutoff_freq_hz: u32,
}

impl Default for I2cTiming {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

impl I2cTiming {
    /// New An Default of timing configuration for a special SpeedMode
    pub fn new(mode: I2cSpeedMode, use_default: bool) -> Self {
        // SAFETY: The variables will be fully initialized later.
        let t = Self::default();

        if use_default {
            match mode {
                I2cSpeedMode::StandMode => t
                    .with_base_config_enable(I2C_MAX_STANDARD_MODE_FREQ,1000,300),
                I2cSpeedMode::FastMode => t
                    .with_base_config_enable(I2C_MAX_FAST_MODE_FREQ, 300, 300),
                I2cSpeedMode::FastPlusMode => t
                    .with_base_config_enable(I2C_MAX_FAST_MODE_PLUS_FREQ, 120, 120),
                I2cSpeedMode::TurboMode => t
                    .with_base_config_enable(I2C_MAX_TURBO_MODE_FREQ, 120, 120),
                I2cSpeedMode::HighSpeedMode => t
                    .with_base_config_enable(I2C_MAX_HIGH_SPEED_MODE_FREQ, 120, 120),
                I2cSpeedMode::UltraFastMode => t
                    .with_base_config_enable(I2C_MAX_ULTRA_FAST_MODE_FREQ, 120, 120),
                _ => panic!("unknown mode"),
            };
        }
        t
    }

    /// Setup the base config of  i2c speed mode
    pub fn with_base_config_enable(
        mut self,
        bus_freq_hz: u32,
        scl_rise_ns: u32,
        scl_fall_ns: u32,
    ) -> Self {
        self.bus_freq_hz = bus_freq_hz;
        self.scl_rise_ns = scl_rise_ns;
        self.scl_fall_ns = scl_fall_ns;
        self
    }

    /// Create i2c timing config from Device
    pub fn i2c_parse_fw_timings(dev: &Device, mode: I2cSpeedMode , use_default: bool) -> I2cTiming {
        let mut builder = Self::new(mode, use_default);
        i2c_parse_timing(dev, c_str!("clock-frequency"), |x| {
            builder.with_bus_freq_hz(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-scl-rising-time-ns"), |x| {
            builder.with_scl_rise_ns(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-scl-falling-time-ns"), |x| {
            builder.with_scl_fall_ns(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-scl-internal-delay-ns"), |x| {
            builder.with_scl_int_delay_ns(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-sda-falling-time-ns"), |x| {
            builder.with_sda_fall_ns(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-sda-hold-time-ns"), |x| {
            builder.with_sda_hold_ns(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-digital-filter-width-ns"), |x| {
            builder.with_digital_filter_width_ns(x)
        });
        i2c_parse_timing(dev, c_str!("i2c-analog-filter-cutoff-frequency"), |x| {
            builder.with_analog_filter_cutoff_freq_hz(x)
        });
        builder
    }

    /// Setup bus freq HZ
    #[inline]
    pub fn with_bus_freq_hz(&mut self, bus_freq_hz:u32) -> &mut Self {
        self.bus_freq_hz = bus_freq_hz;
        self
    }

    /// Setup scl rise ns
    #[inline]
    pub fn with_scl_rise_ns(&mut self, scl_fall_ns:u32) -> &mut Self {
        self.scl_fall_ns = scl_fall_ns;
        self
    }

    /// Setup scl fall ns
    #[inline]
    pub fn with_scl_fall_ns(&mut self, scl_rise_ns:u32) -> &mut Self {
        self.scl_rise_ns = scl_rise_ns;
        self
    }

    /// Setup scl int delay ns
    #[inline]
    pub fn with_scl_int_delay_ns(&mut self, scl_int_delay_ns:u32) -> &mut Self {
        self.scl_int_delay_ns = scl_int_delay_ns;
        self
    }

    /// Setup sda fall ns
    #[inline]
    pub fn with_sda_fall_ns(&mut self, sda_fall_ns:u32) -> &mut Self {
        self.sda_fall_ns = sda_fall_ns;
        self
    }

    /// Setup sda hold ns
    #[inline]
    pub fn with_sda_hold_ns(&mut self, sda_hold_ns:u32) -> &mut Self {
        self.sda_hold_ns = sda_hold_ns;
        self
    }

    /// Setup digital filter width ns
    #[inline]
    pub fn with_digital_filter_width_ns(&mut self, digital_filter_width_ns:u32) -> &mut Self {
        self.digital_filter_width_ns = digital_filter_width_ns;
        self
    }

    /// Setup analog filter cutoff freq_hz
    #[inline]
    pub fn with_analog_filter_cutoff_freq_hz(&mut self, analog_filter_cutoff_freq_hz:u32) -> &mut Self {
        self.analog_filter_cutoff_freq_hz = analog_filter_cutoff_freq_hz;
        self
    }

    /// get bus freq HZ
    #[inline]
    pub fn get_bus_freq_hz(&self) -> u32 {
        self.bus_freq_hz
    }

    /// get sda fall ns
    #[inline]
    pub fn get_sda_fall_ns(&self) -> u32 {
        self.sda_fall_ns
    }

    /// get scl fall ns
    #[inline]
    pub fn get_scl_fall_ns(&self) -> u32 {
        self.scl_fall_ns
    }

    /// get sda hold time
    #[inline]
    pub fn get_sda_hold_ns(&self) -> u32 {
        self.sda_hold_ns
    }
}

fn i2c_parse_timing<'a, F: FnOnce(u32) -> &'a mut I2cTiming>(
    dev: &Device,
    propname: &'static CStr,
    f: F,
) {
    // SAFETY: val always valid
    if let Ok(val) = dev.device_property_read_u32(propname)  {
        f(val); 
    }
}
