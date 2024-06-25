//! Phy device
//!
//! Linux[include/linux/phy.h]
//!

use bitflags::bitflags;
use crate::error::Result;
use crate::net::phy::DeviceState;
use crate::net::phy::PhyDeviceOps;
use crate::net::phy::DuplexMode;

bitflags! {
    /// To determine what I2C functionality is present
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct PhyDeviceFlags: u32 {
        const IS_C45 = 0x0000_0001;                // 第1个bit
        const IS_INTERNAL = 0x0000_0002;           // 第2个bit
        const IS_PSEUDO_FIXED_LINK = 0x0000_0004;  // 第3个bit
        const IS_GIGABIT_CAPABLE = 0x0000_0008;    // 第4个bit
        const HAS_FIXUPS = 0x0000_0010;            // 第5个bit
        const SUSPENDED = 0x0000_0020;             // 第6个bit
        const SUSPENDED_BY_MDIO_BUS = 0x0000_0040; // 第7个bit
        const SYSFS_LINKS = 0x0000_0080;           // 第8个bit
        const LOOPBACK_ENABLED = 0x0000_0100;      // 第9个bit
        const DOWNSHIFTED_RATE = 0x0000_0200;      // 第10个bit
        const IS_ON_SFP_MODULE = 0x0000_0400;      // 第11个bit
        const MAC_MANAGED_PM = 0x0000_0800;        // 第12个bit
        const WOL_ENABLED = 0x0000_1000;           // 第13个bit
        const AUTONEG = 0x0000_2000;               // 第14个bit
        const LINK = 0x0000_4000;                  // 第15个bit
        const AUTONEG_COMPLETE = 0x0000_8000;      // 第16个bit
        const INTERRUPTS = 0x0001_0000;            // 第17个bit
        const IRQ_SUSPENDED = 0x0002_0000;         // 第18个bit
        const IRQ_RERUN = 0x0004_0000;             // 第19个bit
    }
}

pub struct PhyDevice {
  phy_id:u32,
  state: DeviceState,
  bitfiled: PhyDeviceFlags,
  speed: u32,
  duplex: DuplexMode,
}

impl PhyDeviceOps for PhyDevice {
    fn state(&self) -> DeviceState {
        self.state
    }
    fn phy_id(&self) -> u32 {
        self.phy_id
    }
    fn set_speed(&mut self, speed: u32) {
        self.speed = speed;
    }
    fn set_duplex(&mut self, mode: DuplexMode) {
        self.duplex = mode;
    }
    fn is_link_up(&self) -> bool {
        self.bitfiled.contains(PhyDeviceFlags::LINK) 
    }
    fn is_autoneg_enabled(&self) -> bool {
        self.bitfiled.contains(PhyDeviceFlags::AUTONEG) 
    }
    fn is_autoneg_completed(&self) -> bool {
        self.bitfiled.contains(PhyDeviceFlags::AUTONEG_COMPLETE) 
    }
    fn read(&mut self, regnum: u16) -> Result<u16> {
        Ok(0)
    }
    fn write(&mut self, regnum: u16, val: u16) -> Result {
        Ok(())
    }
    fn read_paged(&mut self, page: u16, regnum: u16) -> Result<u16> {
        Ok(0)
    }
    fn resolve_aneg_linkmode(&mut self) {
    }
    fn genphy_soft_reset(&mut self) -> Result {
        Ok(())
    }
    fn init_hw(&mut self) -> Result {
        Ok(())
    }
    fn start_aneg(&mut self) -> Result {
        Ok(())
    }
    fn genphy_resume(&mut self) -> Result {
        Ok(())
    }
    fn genphy_suspend(&mut self) -> Result {
        Ok(())
    }
    fn genphy_read_status(&mut self) -> Result<u16> {
        Ok(0)
    }
    fn genphy_update_link(&mut self) -> Result {
        Ok(())
    }
    fn genphy_read_lpa(&mut self) -> Result {
        Ok(())
    }
    fn genphy_read_abilities(&mut self) -> Result {
        Ok(())
    }
}

