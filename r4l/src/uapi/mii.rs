//! From Linux uapi/include/mii.h
//!
//!

pub const MII_BMCR: u32 = 0;
pub const MII_BMSR: u32 = 1;
pub const MII_PHYSID1: u32 = 2;
pub const MII_PHYSID2: u32 = 3;
pub const MII_ADVERTISE: u32 = 4;
pub const MII_LPA: u32 = 5;
pub const MII_EXPANSION: u32 = 6;
pub const MII_CTRL1000: u32 = 9;
pub const MII_STAT1000: u32 = 10;
pub const MII_MMD_CTRL: u32 = 13;
pub const MII_MMD_DATA: u32 = 14;
pub const MII_ESTATUS: u32 = 15;
pub const MII_DCOUNTER: u32 = 18;
pub const MII_FCSCOUNTER: u32 = 19;
pub const MII_NWAYTEST: u32 = 20;
pub const MII_RERRCOUNTER: u32 = 21;
pub const MII_SREVISION: u32 = 22;
pub const MII_RESV1: u32 = 23;
pub const MII_LBRERROR: u32 = 24;
pub const MII_PHYADDR: u32 = 25;
pub const MII_RESV2: u32 = 26;
pub const MII_TPISTATUS: u32 = 27;
pub const MII_NCONFIG: u32 = 28;

pub const BMCR_RESV: u32 = 63;
pub const BMCR_SPEED1000: u32 = 64;
pub const BMCR_CTST: u32 = 128;
pub const BMCR_FULLDPLX: u32 = 256;
pub const BMCR_ANRESTART: u32 = 512;
pub const BMCR_ISOLATE: u32 = 1024;
pub const BMCR_PDOWN: u32 = 2048;
pub const BMCR_ANENABLE: u32 = 4096;
pub const BMCR_SPEED100: u32 = 8192;
pub const BMCR_LOOPBACK: u32 = 16384;
pub const BMCR_RESET: u32 = 32768;
pub const BMCR_SPEED10: u32 = 0;
