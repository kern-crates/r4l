//! Phy driver
//!
//! Linux[include/linux/phy.h]
//!
//!
//!

use crate::net::phy::{DeviceId, PhyDriverFlags};
use crate::error::Result;
use crate::str::CStr;
use super::PhyDevice;


pub struct PhyDriver {
  pub name: &'static CStr,
  pub flags: PhyDriverFlags, 
  pub deviceid: DeviceId,
  pub soft_reset: Option<fn(&mut PhyDevice)-> Result>,
  pub get_features: Option<fn(&mut PhyDevice)-> Result>,
  pub match_phy_device:  Option<fn(&PhyDevice)-> bool>,
  pub config_aneg: Option<fn(&mut PhyDevice)-> Result>,
  pub read_status: Option<fn(&mut PhyDevice)-> Result<u16>>,
  pub suspend: Option<fn(&mut PhyDevice)-> Result>,
  pub resume: Option<fn(&mut PhyDevice)-> Result>,
  pub read_mmd: Option<fn(&mut PhyDevice, u8, u16)-> Result<u16>>,
  pub write_mmd: Option<fn(&mut PhyDevice, u8, u16, u16)-> Result>,
  pub link_change_notify: Option<fn(&mut PhyDevice)>,
}
