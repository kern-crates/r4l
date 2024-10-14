// SPDX-License-Identifier: GPL-2.0

//! Rust dw_apb_i2c

#![no_std]

use kernel::{
    driver,
    irq,
    device::Device,
    module_platform_driver, of, platform,
    sync::Arc,
    i2c::{
        timing::{self, I2cTiming, I2cSpeedMode },
        msg::{self, I2cMsgFlags, I2cMsgInfo, GeneralI2cMsgInfo },
        functionality::I2cFuncFlags,
    },
    timekeeping::read_poll_timeout,
    sync::SpinLock,
    types::genmask,
    prelude::*,
    regmap,
    math,
    delay,
    new_spinlock,
    // completion::Completion,
};

module_platform_driver! {
      type: DwI2cDriver,
      name: "i2c_designware",
      license: "GPL",
      initcall: "subsys",
}

// Linux Raw id table
kernel::module_of_id_table!(DW_I2C_MOD_TABLE, DW_I2C_OF_MATCH_TABLE);
// R4L IdArray table
kernel::define_of_id_table! {DW_I2C_OF_MATCH_TABLE, (), [
    (of::DeviceId::Compatible("snps,designware-i2c"),None),
]}

struct DwI2cData();

impl driver::DeviceRemoval for DwI2cData {
    fn device_remove(&self) {
        pr_info!("unimplement DwI2cData Remove");
    }
}

struct DwI2cIrqHandler;
impl irq::Handler for DwI2cIrqHandler {
    type Data = i32;

    fn handle_irq(data: &i32) -> irq::Return {
        pr_info!("handled i2c irq get data {} ", data);
        irq::Return::Handled
    }
}

struct DwI2cDriver;
impl platform::Driver for DwI2cDriver {
    type Data = Arc<DwI2cData>;
    // Linux Raw id table
    kernel::driver_of_id_table!(DW_I2C_OF_MATCH_TABLE);

    fn probe(pdev: &mut platform::Device, _id_info: Option<&Self::IdInfo> ) -> Result<Self::Data> {
        let irq = pdev.irq_resource(0)?;
        let reg_base = pdev.ioremap_resource(0)?;
        let dev = Device::from_dev(pdev);
        let timing = I2cTiming::i2c_parse_fw_timings(&dev, I2cSpeedMode::StandMode, false);
        
        pr_info!("enter i2c platform probe func, get irq {}",irq);
        pr_info!("regbase is {:#x}",reg_base);

        // give a fixed value : clk.get_rate() = 100000000, clk_rate_khz = 100000
        let clk_rate_khz = (100000000 / 1000) as u32;

        // create master driver instance
        let driver_config = I2cDwDriverConfig::new(timing, clk_rate_khz);
        let mut i2c_master_driver = I2cDwMasterDriver::new(driver_config, reg_base);
        i2c_master_driver.setup()?;

        // Todo: create data

        Ok(Arc::new(DwI2cData()))
    }
}

/// I2cDwDriverConfig
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct I2cDwDriverConfig {
    timing: I2cTiming,
    clk_rate_khz: u32,
}

impl I2cDwDriverConfig {
    /// Create a dw-apb-i2c timing Config
    pub fn new(timing: I2cTiming, clk_rate_khz: u32) -> Self {
        Self {
            timing,
            clk_rate_khz,
        }
    }
}

/// The I2cDesignware Core Driver
#[allow(dead_code)]
pub(crate) struct I2cDwCoreDriver {
    /// I2c controller base address
    pub(crate) base: usize,
    /// Config From external
    pub(crate) ext_config: I2cDwDriverConfig,
    /// Corrected bus_freq_hz
    pub(crate) bus_freq_hz: u32,
    /// Corrected sda_hold_time
    pub(crate) sda_hold_time: u32,
    /// I2c functionality
    pub(crate) functionality: u32,
    /// I2c SpeedMode
    speed_mode: I2cSpeedMode,
}

unsafe impl Sync for I2cDwCoreDriver {}
unsafe impl Send for I2cDwCoreDriver {}


#[allow(dead_code)]
impl I2cDwCoreDriver {
    pub(crate) fn new(config: I2cDwDriverConfig, base_addr: usize) -> Self {
        Self {
            ext_config: config,
            bus_freq_hz: 0,
            sda_hold_time: 0,
            functionality: DW_I2C_DEFAULT_FUNCTIONALITY,
            speed_mode: I2cSpeedMode::StandMode,
            base: base_addr,
        }
    }

    pub(crate) fn speed_check(&mut self) -> Result<()> {
        let bus_freq_hz = self.ext_config.timing.get_bus_freq_hz();

        if !I2C_DESIGNWARE_SUPPORT_SPEED.contains(&bus_freq_hz) {
            pr_err!("{bus_freq_hz} Hz is unsupported, only 100kHz, 400kHz, 1MHz and 3.4MHz are supported");
            return Err(EINVAL);
        }
        self.bus_freq_hz = bus_freq_hz;

        // Check is high speed possible and fall back to fast mode if not
        let comp_param1 = self.read_ic_comp_param1();
        if comp_param1 & DW_IC_COMP_PARAM_1_SPEED_MODE_MASK != DW_IC_COMP_PARAM_1_SPEED_MODE_HIGH
            && self.bus_freq_hz == timing::I2C_MAX_HIGH_SPEED_MODE_FREQ
        {
            pr_err!("High Speed not supported! Fall back to fast mode");
            self.bus_freq_hz = timing::I2C_MAX_FAST_MODE_FREQ;
        }

        self.speed_mode = I2cSpeedMode::from_bus_freq(self.bus_freq_hz);
        Ok(())
    }

    pub(crate) fn com_type_check(&mut self) -> Result<()> {
        let com_type = regmap::reg_read(self.base, DW_IC_COMP_TYPE);
        if com_type == DW_IC_COMP_TYPE_VALUE {
            return Ok(())
        } else if com_type == DW_IC_COMP_TYPE_VALUE & 0x0000ffff {
            pr_err!("com_type check Failed, not support 16 bit system ");
            return Err(EINVAL);
        } else if com_type == DW_IC_COMP_TYPE_VALUE.to_be() {
            pr_err!("com_type check Failed, not support BE system ");
            return Err(EINVAL);
        } else {
            pr_err!(
                "com_type check failed, Unknown Synopsys component type: {:x}",
                com_type
            );
            return Err(EINVAL);
        }
    }

    #[inline]
    pub(crate) fn functionality_init(&mut self, functionality: u32) {
        self.functionality |= functionality;
    }

    #[inline]
    pub(crate) fn read_ic_comp_param1(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_COMP_PARAM_1)
    }

    pub(crate) fn cfg_init_speed(&self, cfg: &mut u32) {
        match self.speed_mode {
            I2cSpeedMode::StandMode => *cfg |= DW_IC_CON_SPEED_STD,
            I2cSpeedMode::HighSpeedMode => *cfg |= DW_IC_CON_SPEED_HIGH,
            _ => *cfg |= DW_IC_CON_SPEED_FAST,
        }
    }

    #[inline]
    pub(crate) fn read_ic_con(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_CON)
    }

    #[inline]
    pub(crate) fn write_ic_con(&self, cfg: u32) {
        regmap::reg_write(self.base, DW_IC_CON, cfg);
    }

    #[inline]
    pub(crate) fn enable_10bitaddr(&self, enable: bool) {
        let ic_con = regmap::reg_read(self.base, DW_IC_CON);
        if enable {
            regmap::reg_write(self.base, DW_IC_CON, ic_con | DW_IC_CON_10BITADDR_MASTER);
        } else {
            regmap::reg_write(self.base, DW_IC_CON, ic_con & !DW_IC_CON_10BITADDR_MASTER);
        }
    }

    pub(crate) fn write_sda_hold_time(&self) {
        if self.sda_hold_time !=0 {
            regmap::reg_write(self.base, DW_IC_SDA_HOLD, self.sda_hold_time);
        }
    }

    pub(crate) fn sda_hold_time_init(&mut self) -> Result<()> {
        let comp_ver = regmap::reg_read(self.base, DW_IC_COMP_VERSION);
        let ext_sda_hold_ns = self.ext_config.timing.get_sda_hold_ns();

        if comp_ver < DW_IC_SDA_HOLD_MIN_VERS {
            pr_warn!("Hardware too old to adjust SDA hold time.");
            self.sda_hold_time = 0;
            return Ok(());
        }

        if ext_sda_hold_ns == 0 {
            self.sda_hold_time = regmap::reg_read(self.base, DW_IC_SDA_HOLD);
        } else {
            let sda_hold_time = math::div_round_closest_ull(
                    (self.ext_config.clk_rate_khz * ext_sda_hold_ns).into(),
                    math::MICRO) as u32;
            // Workaround for avoiding TX arbitration lost in case I2C
            // slave pulls SDA down "too quickly" after falling edge of
            // SCL by enabling non-zero SDA RX hold. Specification says it
            // extends incoming SDA low to high transition while SCL is
            // high but it appears to help also above issue.
            if (sda_hold_time & DW_IC_SDA_HOLD_RX_MASK) == 0 {
                self.sda_hold_time= sda_hold_time | (1 << 16);
            }
        }
        pr_debug!(
            "sda hold time is {}, and Tx:Rx = {}:{}",
            self.sda_hold_time,
            self.sda_hold_time & 0xFFFF,
            self.sda_hold_time >> 16,
        );
        pr_debug!("I2C  Bus Speed: {}", self.speed_mode);
        
        Ok(())
    }

    pub(crate) fn write_lhcnt(&self, lhcnt: &DwI2cSclLHCnt) {
        // Write standard speed timing parameters
        regmap::reg_write(self.base, DW_IC_SS_SCL_LCNT, lhcnt.ss_lcnt.into());
        regmap::reg_write(self.base, DW_IC_SS_SCL_HCNT, lhcnt.ss_hcnt.into());
        pr_debug!("write SCL_LCNT:HCNT  {}:{}", lhcnt.ss_lcnt, lhcnt.ss_hcnt);

        // Write fast mode/fast mode plus timing parameters
        regmap::reg_write(self.base, DW_IC_FS_SCL_LCNT, lhcnt.fs_lcnt.into());
        regmap::reg_write(self.base, DW_IC_FS_SCL_HCNT, lhcnt.fs_hcnt.into());
        pr_debug!("write FS_SCL_LCNT:HCNT  {}:{}", lhcnt.fs_lcnt, lhcnt.fs_hcnt);

        // Write high speed timing parameters if supported
        if self.speed_mode == I2cSpeedMode::HighSpeedMode {
            regmap::reg_write(self.base, DW_IC_HS_SCL_LCNT, lhcnt.hs_lcnt.into());
            regmap::reg_write(self.base, DW_IC_HS_SCL_HCNT, lhcnt.hs_hcnt.into());
            pr_debug!("write HS_SCL_LCNT:HCNT {}:{}", lhcnt.hs_lcnt, lhcnt.hs_hcnt);
        }
    }

    #[inline]
    pub(crate) fn write_fifo(&self, ic_tx: u32, ic_rx: u32) {
        regmap::reg_write(self.base, DW_IC_TX_TL, ic_tx);
        regmap::reg_write(self.base, DW_IC_RX_TL, ic_rx);
        pr_debug!("write fifo tx:rx {}:{}", ic_tx, ic_rx);
    }

    pub(crate) fn wait_bus_not_busy(&self) -> Result<()> {
        if let Err(e) = read_poll_timeout(
            || return regmap::reg_read(self.base, DW_IC_STATUS),
            move |x| x & DW_IC_STATUS_ACTIVITY == 0,
            1100,
            20000,
            false,
        ) {
            pr_err!("{:?} while waiting for bus ready", e);
            return  Err(EBUSY);
        }
        Ok(())

        //TODO: bus recovery
    }

    pub(crate) fn ic_enable(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_ENABLE)
    }

    #[inline]
    pub(crate) fn ic_enable_status(&self) {
        let _ = regmap::reg_read(self.base, DW_IC_ENABLE_STATUS);
    }

    pub(crate) fn read_and_clean_intrbits(
        &self,
        rx_outstanding: isize,
    ) -> (u32 , u32) {
        // The IC_INTR_STAT register just indicates "enabled" interrupts.
        // The unmasked raw version of interrupt status bits is available
        // in the IC_RAW_INTR_STAT register.
        //
        // That is,
        // stat = readl(IC_INTR_STAT);
        // equals to,
        // stat = readl(IC_RAW_INTR_STAT) & readl(IC_INTR_MASK);
        // The raw version might be useful for debugging purposes.
        let stat = regmap::reg_read(self.base, DW_IC_INTR_STAT);
        let mut abort_source = 0;

        // Do not use the IC_CLR_INTR register to clear interrupts, or
        // you'll miss some interrupts, triggered during the period from
        // readl(IC_INTR_STAT) to readl(IC_CLR_INTR).
        // Instead, use the separately-prepared IC_CLR_* registers.
        if stat & DW_IC_INTR_RX_UNDER != 0 {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_RX_UNDER);
        }
        if stat & DW_IC_INTR_RX_OVER != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_RX_OVER);
        }
        if stat & DW_IC_INTR_TX_OVER != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_TX_OVER);
        }
        if stat & DW_IC_INTR_RD_REQ != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_RD_REQ);
        }
        if stat & DW_IC_INTR_TX_ABRT != 0  {
            // The IC_TX_ABRT_SOURCE register is cleared whenever
            // the IC_CLR_TX_ABRT is read.  Preserve it beforehand.
            abort_source = regmap::reg_read(self.base, DW_IC_TX_ABRT_SOURCE);
            let _ = regmap::reg_read(self.base, DW_IC_CLR_TX_ABRT);
        }
        if stat & DW_IC_INTR_RX_DONE != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_RX_DONE);
        }
        if stat & DW_IC_INTR_ACTIVITY != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_ACTIVITY);
        }
        if stat & DW_IC_INTR_STOP_DET != 0  {
            if (rx_outstanding == 0) || (stat & DW_IC_INTR_RX_FULL !=0) {
                let _ = regmap::reg_read(self.base, DW_IC_CLR_STOP_DET);
            }
        }
        if stat & DW_IC_INTR_START_DET != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_START_DET);
        }
        if stat & DW_IC_INTR_GEN_CALL != 0  {
            let _ = regmap::reg_read(self.base, DW_IC_CLR_GEN_CALL);
        }
        (stat, abort_source)
    }

    #[inline]
    pub(crate) fn write_ic_tar(&self, tar: u32) {
        regmap::reg_write(self.base, DW_IC_TAR, tar);
    }

    #[inline]
    pub(crate) fn read_ic_data_cmd(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_DATA_CMD)
    }

    #[inline]
    pub(crate) fn write_ic_data_cmd(&self, cmd: u32) {
        regmap::reg_write(self.base, DW_IC_DATA_CMD, cmd);
    }

    #[inline]
    pub(crate) fn read_ic_txflr(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_TXFLR)
    }

    #[inline]
    pub(crate) fn read_ic_rxflr(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_RXFLR)
    }

    #[inline]
    pub(crate) fn read_raw_intr_stat(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_RAW_INTR_STAT)
    }

    #[inline]
    pub(crate) fn write_interrupt_mask(&self, mask: u32) {
        regmap::reg_write(self.base, DW_IC_INTR_MASK, mask);
    }

    #[inline]
    pub(crate) fn read_interrupt_mask(&self) -> u32 {
        regmap::reg_read(self.base, DW_IC_INTR_MASK)
    }

    #[inline]
    pub(crate) fn disable_all_interrupt(&self) {
        regmap::reg_write(self.base, DW_IC_INTR_MASK, 0);
    }

    #[inline]
    pub(crate) fn clear_all_interrupt(&self) {
        let _ = regmap::reg_read(self.base, DW_IC_CLR_INTR);
    }

    #[allow(dead_code)]
    pub(crate) fn disable(&self) {
        self.disable_controler();
        // Disable all interrupts
        self.disable_all_interrupt();
        self.clear_all_interrupt();
    }

    pub(crate) fn enable_controler(&self) {
        regmap::reg_write(self.base, DW_IC_ENABLE, 1);
    }

    pub(crate) fn disable_controler(&self) {
        let raw_int_stats = self.read_raw_intr_stat();
        let ic_enable = self.ic_enable();
        let need_aborted = raw_int_stats & DW_IC_INTR_MST_ON_HOLD;

        if need_aborted !=0 {
            let _ = regmap::reg_write(self.base, DW_IC_ENABLE, ic_enable | DW_IC_ENABLE_ABORT);
            if let Err(e) = read_poll_timeout(
                || regmap::reg_read(self.base, DW_IC_ENABLE),
                move |x| x & DW_IC_ENABLE_ABORT == 0 ,
                10,
                100,
                false,
            ) {
                pr_err!("{:?} while trying to abort current transfer", e);
            }
        }

        let mut try_cnt = 100;
        loop {
            self.disable_nowait();
            pr_debug!("==== usleep before ======");
            // delay::usleep(100);
            // check enable_status
            pr_debug!("==== usleep after ======");
            let status = regmap::reg_read(self.base, DW_IC_ENABLE_STATUS);
            if status & 1 == 0 {
                break;
            }
            try_cnt -= 1;
            if try_cnt == 0 {
                pr_err!("timeout in disabling i2c adapter");
                break;
            }
        }
    }

    fn disable_nowait(&self) {
        let _ = regmap::reg_write(self.base, DW_IC_ENABLE, 0);
    }
}

enum TransferResult  {
    // Unexpected irq
    UnExpectedInterrupt,
    // Recive IRQ abort
    Abort,
    // All msgs are process success
    Fininsh,
    // Still need next irq
    Continue,
}

/// Master driver transfer abstract
#[allow(dead_code)]
struct MasterXfer {
    /// XferData
    msgs: Vec<I2cMsgInfo>,
    /// run time hadware error code
    cmd_err: u32,
    /// the element index of the current rx message in the msgs array
    msg_read_idx: usize,
    /// the element index of the current tx message in the msgs array
    msg_write_idx: usize,
    /// error status of the current transfer
    msg_err: Result<()>,
    /// copy of the TX_ABRT_SOURCE register
    abort_source: u32,
    /// current master-rx elements in tx fifo
    rx_outstanding: isize,
    /// Driver Status
    status: u32,
}

impl Default for MasterXfer {
    /// Create an empty XferData
    fn default() -> Self {
        Self {
            msgs: Vec::new(),
            cmd_err: 0,
            msg_read_idx: 0,
            msg_write_idx: 0,
            msg_err: Ok(()),
            abort_source: 0,
            rx_outstanding: 0,
            status: 0,
        }
    }
}

impl MasterXfer {
    #[allow(dead_code)]
    fn init(&mut self, msgs: Vec<I2cMsgInfo>) {
        self.msgs = msgs;
        self.cmd_err = 0;
        self.msg_read_idx = 0;
        self.msg_write_idx = 0;
        self.msg_err = Ok(());
        self.abort_source = 0;
        self.rx_outstanding = 0;
        self.status = 0;
    }

    #[inline]
    pub(crate) fn is_empty_status(&self) -> bool {
        self.status == 0
    }

    #[inline]
    pub(crate) fn clear_active(&mut self) {
        self.status &= !msg::STATUS_ACTIVE;
    }

    #[inline]
    pub(crate) fn set_active(&mut self) {
        self.status |= msg::STATUS_ACTIVE;
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        (self.status & msg::STATUS_ACTIVE) != 0 
    }

    #[inline]
    pub(crate) fn is_write_in_progress(&self) -> bool {
        (self.status & msg::STATUS_WRITE_IN_PROGRESS) != 0 
    }

    #[inline]
    pub(crate) fn set_write_in_progress(&mut self, set: bool) {
        if set {
            self.status |= msg::STATUS_WRITE_IN_PROGRESS;
        } else {
            self.status &= !msg::STATUS_WRITE_IN_PROGRESS;
        }
    }

    fn prepare(&mut self, msgs: Vec<I2cMsgInfo>, master_driver: &I2cDwMasterDriver) {
        self.init(msgs);
        let core_driver = &master_driver.driver;
        // disable the adapter
        master_driver.disable(false);

        let first_msg = &self.msgs[self.msg_write_idx as usize];
        let mut ic_tar = 0 ;
        // If the slave address is ten bit address, enable 10BITADDR
        if first_msg.flags() & I2cMsgFlags::I2C_ADDR_TEN != 0 {
            core_driver.enable_10bitaddr(true);
        } else {
            ic_tar = DW_IC_TAR_10BITADDR_MASTER;
            core_driver.enable_10bitaddr(false);
        }

        ic_tar |= first_msg.addr() as u32;
        core_driver.write_ic_tar(ic_tar);

        // Enforce disabled interrupts (due to HW issues) 
        core_driver.disable_all_interrupt();

        // Enable the adapter
        core_driver.enable_controler();
        self.set_active();
        // Dummy read to avoid the register getting stuck on Bay Trail
        let _ = core_driver.ic_enable_status();
    }

    fn irq_process(&mut self, master_driver: &I2cDwMasterDriver) -> TransferResult {
        let core_driver = &master_driver.driver;

        let (stat, abort_source) = 
            core_driver.read_and_clean_intrbits(self.rx_outstanding);
        self.abort_source = abort_source;

        // Unexpected interrupt in driver point of view. State
        // variables are either unset or stale so acknowledge and
        // disable interrupts for suppressing further interrupts if
        // interrupt really came from this HW (E.g. firmware has left
        // the HW active).
        if !self.is_active() {
            return TransferResult::UnExpectedInterrupt; 
        }

        if stat & DW_IC_INTR_TX_ABRT != 0 {
            self.cmd_err |= msg::DW_IC_ERR_TX_ABRT;
            self.status &= !msg::STATUS_MASK;
            return TransferResult::Abort; 
        }

        if stat & DW_IC_INTR_RX_FULL != 0 {
            self.read_msgs(&master_driver);
        }

        if stat & DW_IC_INTR_TX_EMPTY != 0 {
            self.write_msgs(&master_driver);
        }

        if  ((stat & DW_IC_INTR_STOP_DET != 0) || self.msg_err.is_err()) 
            && self.rx_outstanding == 0 {
                return TransferResult::Fininsh;
        }

        return TransferResult::Continue;
    }

    fn exit(&mut self, master_driver: &I2cDwMasterDriver) -> Result<()> {
        // We must disable the adapter before returning and signaling the end
        // of the current transfer. Otherwise the hardware might continue
        // generating interrupts which in turn causes a race condition with
        // the following transfer.  Needs some more investigation if the
        // additional interrupts are a hardware bug or this driver doesn't
        // handle them correctly yet.
        master_driver.disable(true);
        self.clear_active();

        match self.msg_err {
            Err(e) => {
                pr_err!("i2c dw transfer process msg error: {:?}",e);
                return Err(e);
            }
            Ok(_) => {},
        }

        if self.cmd_err == msg::DW_IC_ERR_TX_ABRT {
            pr_err!("i2c dw transfer recv tx_abort");
            self.handle_tx_abort()?;
        }

        if !self.is_empty_status() {
            pr_err!("transfer terminated early - interrupt latency too high?");
            return Err(EIO);
        }
        Ok(())
    }

    fn handle_tx_abort(&mut self) -> Result<()> {
        let abort_source = self.abort_source;
        if abort_source & DW_IC_TX_ABRT_NOACK != 0 {
            return Err(EIO);
        }
        if abort_source & DW_IC_TX_ARB_LOST != 0{
            return Err(EAGAIN);
        } else if abort_source & DW_IC_TX_ABRT_GCALL_READ != 0{
            return Err(EINVAL);
        } else {
            return Err(EIO);
        }
    }

    /// Initiate (and continue) low level master read/write transaction.
    /// This function is only called from i2c_dw_isr, and pumping i2c_msg
    /// messages into the tx buffer.  Even if the size of i2c_msg data is
    /// longer than the size of the tx buffer, it handles everything.
    /// Todo: need to fix intr_mask
    fn write_msgs(&mut self, master_driver: &I2cDwMasterDriver) {
        let msg_len = self.msgs.len();
        let core_driver = &master_driver.driver;
        let mut intr_mask = DW_IC_INTR_MASTER_MASK;
        let addr = self.msgs[self.msg_write_idx].addr();
        let mut need_restart = false;
        loop {
            let write_idx = self.msg_write_idx;
            if write_idx >= msg_len {
                break;
            }

            if !self.is_write_in_progress() {
                //If both IC_EMPTYFIFO_HOLD_MASTER_EN and
                //IC_RESTART_EN are set, we must manually
                //set restart bit between messages.
                if (master_driver.cfg & DW_IC_CON_RESTART_EN !=0) && 
                    write_idx > 0
                {
                    need_restart = true;           
                }
            }

            let msg = &mut self.msgs[write_idx];

            if msg.addr() != addr {
                self.msg_err = Err(EINVAL);
                break;
            }

            let flr = core_driver.read_ic_txflr();
            let mut tx_limit = master_driver.tx_fifo_depth - flr;
                
            let flr = core_driver.read_ic_rxflr();
            let mut rx_limit = master_driver.rx_fifo_depth - flr;

            loop {
                if msg.send_end() || rx_limit <=0 || tx_limit <=0 {
                    break;
                }
                let mut cmd: u32 = 0 ;
                // If IC_EMPTYFIFO_HOLD_MASTER_EN is set we must
                // manually set the stop bit. However, it cannot be
                // detected from the registers so we set it always
                // when writing/reading the last byte.
                //
                // i2c-core always sets the buffer length of
                // I2C_FUNC_SMBUS_BLOCK_DATA to 1. The length will
                // be adjusted when receiving the first byte.
                // Thus we can't stop the transaction here.
                if write_idx == msg_len-1 &&
                    ( msg.flags() & I2cMsgFlags::I2C_MASTER_RECV_LEN == 0 ) && 
                    msg.send_left_last() 
                {
                    cmd |= DW_IC_DATA_CMD_STOP;
                }

                if need_restart {
                    cmd |= DW_IC_DATA_CMD_RESTART;
                    need_restart = false;
                }

                if msg.flags() & I2cMsgFlags::I2C_MASTER_READ != 0 {
                    /* Avoid rx buffer overrun */
                    if self.rx_outstanding >= 
                        master_driver.rx_fifo_depth.try_into().unwrap() {
                        break;
                    }
                    cmd |= DW_IC_DATA_CMD_CMD;
                    rx_limit -= 1;
                    self.rx_outstanding += 1;
                    msg.inc_recieve_cmd_cnt();
                } else {
                    let buf = msg.pop_front_byte() as u32;
                    cmd |= buf;
                }
                core_driver.write_ic_data_cmd(cmd);
                tx_limit -=1;
            }

            // Because we don't know the buffer length in the
            // I2C_FUNC_SMBUS_BLOCK_DATA case, we can't stop the
            // transaction here. Also disable the TX_EMPTY IRQ
            // while waiting for the data length byte to avoid the
            // bogus interrupts flood.
            if msg.flags() & I2cMsgFlags::I2C_MASTER_RECV_LEN !=0 {
                self.set_write_in_progress(true);
                intr_mask &= !DW_IC_INTR_TX_EMPTY;
                break;
            } else if !msg.send_end() {
                // wait next time TX_EMPTY interrupt
                self.set_write_in_progress(true);
                break;
            } else {
                self.set_write_in_progress(false);
            }
            self.msg_write_idx +=1;
        }
        
        // If i2c_msg index search is completed, we don't need TX_EMPTY
        // interrupt any more.
        if self.msg_write_idx == msg_len {
            intr_mask &= !DW_IC_INTR_TX_EMPTY;
        }

        if self.msg_err.is_err() {
            intr_mask = 0;
        }

        core_driver.write_interrupt_mask(intr_mask);
    }

    fn read_msgs(&mut self, master_driver: &I2cDwMasterDriver) {
        let msg_len = self.msgs.len();
        let core_driver = &master_driver.driver;

        loop {
            let read_idx = self.msg_read_idx;
            if read_idx >= msg_len {
                break;
            }

            let msg = &mut(self.msgs[read_idx]);

            if msg.flags() & I2cMsgFlags::I2C_MASTER_READ == 0 {
                self.msg_read_idx += 1;
                continue
            }

            let rx_valid = core_driver.read_ic_rxflr();

            for _ in 0..rx_valid {
                // check if buf can be write
                if msg.recieve_end() {
                    break;
                }

                let mut ic_data: u8 = core_driver.read_ic_data_cmd() as u8;

                ic_data &= DW_IC_DATA_CMD_DAT as u8;
                // Ensure length byte is a valid value
                if msg.flags() & I2cMsgFlags::I2C_MASTER_RECV_LEN !=0 {
                    // if IC_EMPTYFIFO_HOLD_MASTER_EN is set, which cannot be
                    // detected from the registers, the controller can be
                    // disabled if the STOP bit is set. But it is only set
                    // after receiving block data response length in
                    // I2C_FUNC_SMBUS_BLOCK_DATA case. That needs to read
                    // another byte with STOP bit set when the block data
                    // response length is invalid to complete the transaction.
                    if ic_data == 0 || ic_data > I2C_SMBUS_BLOCK_MAX {
                        ic_data = 1;
                    }
                    let mut buf_len = ic_data as usize;
                    // Adjust the buffer length and mask the flag 
                    // after receiving the first byte.
                    if msg.flags() & I2cMsgFlags::I2C_CLIENT_PEC != 0 {
                        buf_len+=2;
                    } else {
                        buf_len+=1;
                    };
                    msg.modify_recieve_threshold(buf_len);
                    // cacluate read_cmd_cnt
                    msg.modify_recieve_cmd_cnt(self.rx_outstanding.min(buf_len as isize));
                    msg.remove_flag(I2cMsgFlags::I2C_MASTER_RECV_LEN);
                    
                    // Received buffer length, re-enable TX_EMPTY interrupt
                    // to resume the SMBUS transaction.
                    // core_driver.enable_tx_empty_intr(true);
                    // let mut intr_mask = regmap::reg_read(base, DW_IC_INTR_MASK);
                    let mut intr_mask = core_driver.read_interrupt_mask();
                    intr_mask |= DW_IC_INTR_TX_EMPTY;
                    core_driver.write_interrupt_mask(intr_mask);
                }
                msg.push_byte(ic_data.try_into().unwrap());
                self.rx_outstanding -= 1;
            }
            
            if !msg.recieve_end() {
                // wait next time RX_FULL interrupt
                return
            } else {
                self.msg_read_idx +=1;
            }
        }
    }

}

/// The I2cDesignware Driver
pub struct I2cDwMasterDriver {
    /// I2c Config  register set value
    cfg: u32,
    /// core Driver
    driver: I2cDwCoreDriver,
    /// I2c scl_LHCNT
    lhcnt: DwI2cSclLHCnt,
    /// Fifo
    tx_fifo_depth: u32,
    rx_fifo_depth: u32,
    
    /// Arc completion 
    // cmd_complete: Arc<Completion>,

    /// Since xfer will be used in interrupt handler,
    /// the data needs a concurrent mechanism to ensure safety. 
    /// The driver will ensure that it will not be triggered
    /// by interrupts when using locks,
    /// so there is no need to use spin_noirq
    xfer: SpinLock<MasterXfer>,
}

impl I2cDwMasterDriver {
    /// Create a new I2cDesignwarDriver
    pub fn new(config: I2cDwDriverConfig, base_addr: usize) -> Self {
        Self {
            cfg: 0,
            driver: I2cDwCoreDriver::new(config, base_addr),
            lhcnt: DwI2cSclLHCnt::default(),
            tx_fifo_depth: 0,
            rx_fifo_depth: 0,
            // cmd_complete: Completion::new().unwrap(),
            xfer: new_spinlock!(MasterXfer::default()),
        }
    }

    /// Initialize the designware I2C driver config
    pub fn setup(&mut self) -> Result<()> {
        // com and speed check must be the first step
        self.driver.com_type_check()?;
        self.driver.speed_check()?;
        // init config
        self.config_init()?;
        self.scl_lhcnt_init()?;
        self.driver.sda_hold_time_init()?;
        self.fifo_size_init();

        // Initialize the designware I2C master hardware
        self.master_setup();
        self.driver.disable_all_interrupt();
        Ok(())
    }

    /// functionality and cfg init
    fn config_init(&mut self) -> Result<()> {
        // init functionality
        let functionality = I2cFuncFlags::BIT_10_ADDR;
        self.driver.functionality_init(functionality);

        // init master cfg
        self.cfg = DW_IC_CON_MASTER | DW_IC_CON_SLAVE_DISABLE | DW_IC_CON_RESTART_EN;

        // On AMD pltforms BIOS advertises the bus clear feature
        // and enables the SCL/SDA stuck low. SMU FW does the
        // bus recovery process. Driver should not ignore this BIOS
        // advertisement of bus clear feature.
        let ic_con = self.driver.read_ic_con();
        if ic_con & DW_IC_CON_BUS_CLEAR_CTRL !=0 {
            self.cfg |= DW_IC_CON_BUS_CLEAR_CTRL;
        }

        self.driver.cfg_init_speed(&mut self.cfg);

        Ok(())
    }

    /// return  i2c functionality
    pub fn get_functionality(&self) -> u32 {
        self.driver.functionality
    }

    /// Prepare controller for a transaction and call xfer_msg
    pub fn master_transfer(&self, msgs: Vec<I2cMsgInfo>) -> Result<i32> {
        let msg_num = msgs.len();
        // reinit complete
        // self.cmd_complete.reinit();
        // wait bus free
        self.driver.wait_bus_not_busy()?;
        // transfer exit make sure interrupt is disabled 
        // so here lock is safety
        let mut transfer = self.xfer.lock();
        transfer.prepare(msgs, &self);
        drop(transfer);
        // Now, could enable interrupt
        self.driver.clear_all_interrupt();
        self.driver.write_interrupt_mask(DW_IC_INTR_MASTER_MASK);

        // // wait transfer complete
        // match self.cmd_complete.wait_for_completion_timeout_sec(1) {
        //     Err(e) => {
        //         pr_err!("wait complete timeout");
        //         //master_setup implicitly disables the adapter
        //         self.master_setup();
        //         self.driver.clear_all_interrupt();
        //         self.driver.disable_all_interrupt();
        //         return Err(e);
        //     }
        //     Ok(_) => (),
        // }

        // complete make sure interrupt is disable 
        // so here lock is safety
        let mut transfer = self.xfer.lock();
        transfer.exit(&self)?;

        Ok(msg_num.try_into().unwrap())
    }
    
    /// Interrupt service routine. This gets called whenever an I2C master interrupt
    /// occurs
    pub fn irq_handler(&self) -> irq::Return {
        let enable = self.driver.ic_enable();
        let stat = self.driver.read_raw_intr_stat();
        
        // check raw intr stat
        if enable == 0 || (stat & !0b100000000) == 0 {
            return irq::Return::None;
        }

        // master_transfer make sure when irq hanppend(irq enable)
        // no longer lock transfer, so here lock is safety
        pr_debug!("enter irq stat: {:#x}, enable: {:#x}", stat, enable);
        let mut transfer = self.xfer.lock();
        let result = transfer.irq_process(&self);
        drop(transfer);

        match result {
            TransferResult::UnExpectedInterrupt => {
                self.driver.disable_all_interrupt();
            },
            TransferResult::Abort => {
                // Anytime TX_ABRT is set, the contents of the tx/rx
                // buffers are flushed. Make sure to skip them.
                self.driver.disable_all_interrupt();
                // self.cmd_complete.complete();
            },
            TransferResult::Fininsh => {
                ()
                // self.cmd_complete.complete();
            },
            TransferResult::Continue => (),
        }

        return irq::Return::Handled;
    }

    fn master_setup(&self) {
        // Disable the adapter
        self.disable(false);
        // Write standard speed timing parameters
        self.driver.write_lhcnt(&self.lhcnt);
        // Write SDA hold time if supported
        self.driver.write_sda_hold_time();
        // Write fifo
        self.driver.write_fifo(self.tx_fifo_depth / 2, 0);
        // set IC_CON
        self.driver.write_ic_con(self.cfg);
    }

    fn disable(&self, fast: bool) {
        if fast {
            self.driver.disable_nowait();
        } else {
            self.driver.disable_controler();
        }
    }
    
    fn fifo_size_init(&mut self) {
        let param = self.driver.read_ic_comp_param1();
        self.tx_fifo_depth = ((param >> 16) & 0xff) + 1;
        self.rx_fifo_depth = ((param >> 8)  & 0xff) + 1;
        pr_debug!(
            "I2C fifo_depth RX:TX = {}: {}",
            self.rx_fifo_depth,
            self.tx_fifo_depth
        );
    }

    fn scl_lhcnt_init(&mut self) -> Result<()> {
        let driver = &mut self.driver;
        let ic_clk = driver.ext_config.clk_rate_khz;
        let mut scl_fall_ns = driver.ext_config.timing.get_scl_fall_ns();
        let mut sda_fall_ns = driver.ext_config.timing.get_sda_fall_ns();

        // Set standard and fast speed dividers for high/low periods
        if scl_fall_ns == 0 {
            scl_fall_ns = 300;
        }

        if sda_fall_ns == 0 {
            sda_fall_ns = 300;
        }

        // tLOW = 4.7 us and no offset
        self.lhcnt.ss_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 4700, scl_fall_ns, 0) as u16;
        // tHigh = 4 us and no offset DW default
        self.lhcnt.ss_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 4000, sda_fall_ns, false, 0) as u16;
        pr_debug!(
            "I2C dw Standard Mode HCNT:LCNT = {} : {}",
            self.lhcnt.ss_hcnt,
            self.lhcnt.ss_lcnt
        );

        let speed_mode = driver.speed_mode;
        if speed_mode == I2cSpeedMode::FastPlusMode {
            self.lhcnt.fs_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 500, scl_fall_ns, 0) as u16;
            self.lhcnt.fs_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 260, sda_fall_ns, false, 0) as u16;
            pr_debug!(
                "I2C Fast Plus Mode HCNT:LCNT = {} : {}",
                self.lhcnt.fs_hcnt,
                self.lhcnt.fs_lcnt
            );
        } else {
            self.lhcnt.fs_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 1300, scl_fall_ns, 0) as u16;
            self.lhcnt.fs_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 600, sda_fall_ns, false, 0) as u16;
            pr_debug!(
                "I2C Fast Mode HCNT:LCNT = {} : {}",
                self.lhcnt.fs_hcnt,
                self.lhcnt.fs_lcnt
            );
        }

        if speed_mode == I2cSpeedMode::HighSpeedMode {
            self.lhcnt.hs_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 320, scl_fall_ns, 0) as u16;
            self.lhcnt.hs_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 160, sda_fall_ns, false, 0) as u16;
            pr_debug!(
                "I2C High Speed Mode HCNT:LCNT = {} : {}",
                self.lhcnt.hs_hcnt,
                self.lhcnt.hs_lcnt
            );
        }
        Ok(())
    }

}

#[allow(dead_code)]
#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct DwI2cSclLHCnt {
    /// standard speed HCNT value
    pub(crate) ss_hcnt: u16,
    /// standard speed LCNT value
    pub(crate) ss_lcnt: u16,
    /// Fast Speed HCNT value
    pub(crate) fs_hcnt: u16,
    /// Fast Speed LCNT value
    pub(crate) fs_lcnt: u16,
    /// Fast Speed Plus HCNT value
    pub(crate) fp_hcnt: u16,
    /// Fast Speed Plus LCNT value
    pub(crate) fp_lcnt: u16,
    /// High Speed HCNT value
    pub(crate) hs_hcnt: u16,
    /// High Speed LCNT value
    pub(crate) hs_lcnt: u16,
}

#[allow(dead_code)]
impl DwI2cSclLHCnt {
    /// Conditional expression:
    ///  
    ///  IC_[FS]S_SCL_LCNT + 1 >= IC_CLK * (tLOW + tf)
    ///
    /// DW I2C core starts counting the SCL CNTs for the LOW period
    /// of the SCL clock (tLOW) as soon as it pulls the SCL line.
    /// In order to meet the tLOW timing spec, we need to take into
    /// account the fall time of SCL signal (tf).  Default tf value
    /// should be 0.3 us, for safety.
    pub(crate) fn scl_lcnt(ic_clk: u32, tlow: u32, tf: u32, offset: u32) -> u32 {
        pr_debug!(
            "scl_lcnt: ic_clk: {} , tlow:{}  tf:{} , offset:{}",
            ic_clk,
            tlow,
            tf,
            offset
        );
        let right: u64 = ic_clk as u64 * (tlow as u64 + tf as u64);
        (math::div_round_closest_ull(right, math::MICRO) - 1 + offset as u64).try_into().unwrap()
    }

    /// DesignWare I2C core doesn't seem to have solid strategy to meet
    /// the tHD;STA timing spec.  Configuring _HCNT based on tHIGH spec
    /// will result in violation of the tHD;STA spec.
    /// Conditional expression1:
    /// IC_[FS]S_SCL_HCNT + (1+4+3) >= IC_CLK * tHIGH
    /// This is based on the DW manuals, and represents an ideal
    /// configuration.  The resulting I2C bus speed will be
    /// If your hardware is free from tHD;STA issue, try this one.
    ///
    /// Conditional expression2:
    /// IC_[FS]S_SCL_HCNT + 3 >= IC_CLK * (tHD;STA + tf)
    /// This is just experimental rule; the tHD;STA period turned
    /// out to be proportinal to (_HCNT + 3).  With this setting
    /// we could meet both tHIGH and tHD;STA timing specs.
    /// If unsure, you'd better to take this alternative.
    ///
    /// The reason why we need to take into account "tf" here,
    /// is the same as described in i2c_dw_scl_lcnt().
    pub(crate) fn scl_hcnt(ic_clk: u32, tsymbol: u32, tf: u32, cond: bool, offset: u32) -> u32 {
        if cond {
            let right: u64 = ic_clk as u64 * tsymbol as u64;
            (math::div_round_closest_ull(right, math::MICRO) - 8 + offset as u64).try_into().unwrap()
        } else {
            let right: u64 = ic_clk as u64 * (tsymbol as u64 + tf as u64);
            (math::div_round_closest_ull(right, math::MICRO) - 3 + offset as u64).try_into().unwrap()
        }
    }
}


/// Distributor Control Register Offset.
pub(crate) const DW_IC_CON: usize = 0x00;
pub(crate) const DW_IC_TAR: usize = 0x04;
#[allow(dead_code)]
pub(crate) const DW_IC_SAR: usize = 0x08;
pub(crate) const DW_IC_DATA_CMD:    usize = 0x10;
pub(crate) const DW_IC_SS_SCL_HCNT: usize = 0x14;
pub(crate) const DW_IC_SS_SCL_LCNT: usize = 0x18;
pub(crate) const DW_IC_FS_SCL_HCNT: usize = 0x1c;
pub(crate) const DW_IC_FS_SCL_LCNT: usize = 0x20;
pub(crate) const DW_IC_HS_SCL_HCNT: usize = 0x24;
pub(crate) const DW_IC_HS_SCL_LCNT: usize = 0x28;
pub(crate) const DW_IC_INTR_STAT:   usize = 0x2c;
pub(crate) const DW_IC_INTR_MASK:   usize = 0x30;
pub(crate) const DW_IC_RAW_INTR_STAT: usize = 0x34;
pub(crate) const DW_IC_RX_TL:    usize = 0x38;
pub(crate) const DW_IC_TX_TL:    usize = 0x3c;
pub(crate) const DW_IC_CLR_INTR: usize = 0x40;
pub(crate) const DW_IC_CLR_RX_UNDER: usize = 0x44;
pub(crate) const DW_IC_CLR_RX_OVER:  usize = 0x48;
pub(crate) const DW_IC_CLR_TX_OVER:  usize = 0x4c;
pub(crate) const DW_IC_CLR_RD_REQ:   usize = 0x50;
pub(crate) const DW_IC_CLR_TX_ABRT:  usize = 0x54;
pub(crate) const DW_IC_CLR_RX_DONE:  usize = 0x58;
pub(crate) const DW_IC_CLR_ACTIVITY: usize = 0x5c;
pub(crate) const DW_IC_CLR_STOP_DET: usize = 0x60;
pub(crate) const DW_IC_CLR_START_DET:usize = 0x64;
pub(crate) const DW_IC_CLR_GEN_CALL: usize = 0x68;
pub(crate) const DW_IC_ENABLE: usize = 0x6c;
pub(crate) const DW_IC_STATUS: usize = 0x70;
pub(crate) const DW_IC_TXFLR:  usize = 0x74;
pub(crate) const DW_IC_RXFLR:  usize = 0x78;
pub(crate) const DW_IC_SDA_HOLD: usize = 0x7c;
pub(crate) const DW_IC_TX_ABRT_SOURCE:  usize = 0x80;
pub(crate) const DW_IC_ENABLE_STATUS:   usize = 0x9c;
#[allow(dead_code)]
pub(crate) const DW_IC_CLR_RESTART_DET: usize = 0xa8;
pub(crate) const DW_IC_COMP_PARAM_1: usize = 0xf4;
pub(crate) const DW_IC_COMP_VERSION: usize = 0xf8;
pub(crate) const DW_IC_COMP_TYPE: usize = 0xfc;
/// Designware Component Type number = 0x44_57_01_40. This
/// assigned unique hex value is constant and is derived from the two
/// ASCII letters “DW” followed by a 16-bit unsigned number.
/// "DW" + 0x0140
pub(crate) const DW_IC_COMP_TYPE_VALUE: u32 = 0x44570140;
/// "111" = v1.11
pub(crate) const DW_IC_SDA_HOLD_MIN_VERS: u32 = 0x3131312A;

/// DW_IC_CON functionality
pub(crate) const DW_IC_CON_MASTER: u32 = 1 << 0;  
pub(crate) const DW_IC_CON_SPEED_STD: u32 = 1 << 1;
pub(crate) const DW_IC_CON_SPEED_FAST: u32 = 2 << 1;
pub(crate) const DW_IC_CON_SPEED_HIGH: u32 = 3 << 1;
pub(crate) const DW_IC_CON_10BITADDR_MASTER: u32 = 1 << 4;  
pub(crate) const DW_IC_CON_RESTART_EN: u32 = 1 << 5; 
pub(crate) const DW_IC_CON_SLAVE_DISABLE: u32 = 1 << 6; 
pub(crate) const DW_IC_CON_BUS_CLEAR_CTRL: u32 = 1 << 11; 

/// DW_IC_INTR_RX functionality
pub(crate) const DW_IC_INTR_RX_UNDER:u32 = 1 << 0 ;
pub(crate) const DW_IC_INTR_RX_OVER: u32 = 1 << 1 ;
pub(crate) const DW_IC_INTR_RX_FULL: u32 = 1 << 2 ;
pub(crate) const DW_IC_INTR_TX_OVER: u32 = 1 << 3 ;
pub(crate) const DW_IC_INTR_TX_EMPTY:u32 = 1 << 4 ;
pub(crate) const DW_IC_INTR_RD_REQ:  u32 = 1 << 5 ;
pub(crate) const DW_IC_INTR_TX_ABRT: u32 = 1 << 6 ;
pub(crate) const DW_IC_INTR_RX_DONE: u32 = 1 << 7 ;
pub(crate) const DW_IC_INTR_ACTIVITY:u32 = 1 << 8 ;
pub(crate) const DW_IC_INTR_STOP_DET:u32 = 1 << 9 ;
pub(crate) const DW_IC_INTR_START_DET:  u32 = 1 << 10 ;
pub(crate) const DW_IC_INTR_GEN_CALL:   u32 = 1 << 11 ;
#[allow(dead_code)]
pub(crate) const DW_IC_INTR_RESTART_DET:u32 = 1 << 12 ;
pub(crate) const DW_IC_INTR_MST_ON_HOLD:u32 = 1 << 13 ;

pub(crate) const DW_IC_INTR_MASTER_MASK:u32 = 
    DW_IC_INTR_RX_FULL  | 
    DW_IC_INTR_TX_EMPTY | 
    DW_IC_INTR_TX_ABRT  | 
    DW_IC_INTR_STOP_DET ;

/// DW_IC_ENABLE_ABORT functionality
pub(crate) const DW_IC_ENABLE_ABORT: u32 = 1 << 1;  

/// DW_IC_STATUS functionality
pub(crate) const DW_IC_STATUS_ACTIVITY:u32 = 1 << 0; 

/// DW_IC_COMP_PARAM_1 functionality
pub(crate) const DW_IC_COMP_PARAM_1_SPEED_MODE_HIGH:u32 = (1 << 2) | (1 << 3);
pub(crate) const DW_IC_COMP_PARAM_1_SPEED_MODE_MASK:u32 = genmask(3, 2);

/// DW_IC_SDA_HOLD_RX_MASK 
pub const DW_IC_SDA_HOLD_RX_MASK: u32 = genmask(23, 16);
/// DW_IC_TAR_10BITADDR_MASTER
pub(crate) const DW_IC_TAR_10BITADDR_MASTER: u32 = 1 << 12;  

/// DW_IC_DATA functionality
pub(crate) const DW_IC_DATA_CMD_DAT: u32 = genmask(7, 0);
pub(crate) const DW_IC_DATA_CMD_CMD: u32 = 1 << 8;    
pub(crate) const DW_IC_DATA_CMD_STOP: u32 = 1 << 9;
pub(crate) const DW_IC_DATA_CMD_RESTART: u32 = 1 << 10;

/// DW_IC_TX_ABRT functionality
pub(crate) const DW_IC_TX_ABRT_GCALL_READ: u32 = 1 << 5;
pub(crate) const DW_IC_TX_ARB_LOST: u32 = 1 << 12;
pub(crate) const DW_IC_TX_ABRT_NOACK:u32 = (1<<0) | (1<<1) | (1<<2) | (1<<3) | (1<<4) ;

///  dw-i2c-defualt functionality
pub const DW_I2C_DEFAULT_FUNCTIONALITY: u32 = I2cFuncFlags::I2C |
    I2cFuncFlags::SMBUS_BYTE       |
    I2cFuncFlags::SMBUS_BYTE_DATA  |
    I2cFuncFlags::SMBUS_WORD_DATA  |
    I2cFuncFlags::SMBUS_BLOCK_DATA |
    I2cFuncFlags::SMBUS_I2C_BLOCK  ;

/// support mode speed
const I2C_DESIGNWARE_SUPPORT_SPEED: [u32; 4] = [
    timing::I2C_MAX_STANDARD_MODE_FREQ,
    timing::I2C_MAX_FAST_MODE_FREQ,
    timing::I2C_MAX_FAST_MODE_PLUS_FREQ,
    timing::I2C_MAX_HIGH_SPEED_MODE_FREQ,
];

/// Data for SMBus Messages
pub const I2C_SMBUS_BLOCK_MAX:u8 = 32;