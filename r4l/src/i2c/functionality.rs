//! To determine i2c what functionality is present
//!
//! From linux/include/uapi/linux/i2c.h

/// I2cFuncFlags Struct
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct I2cFuncFlags;
///  To determine what I2C functionality is present
impl I2cFuncFlags {
    /// Support I2C
    pub const I2C:u32                    = 0x00000001;
    /// required for I2C_M_TEN
    pub const BIT_10_ADDR:u32           = 0x00000002;
    /// required for I2C_M_IGNORE_NAK etc.
    pub const PROTOCOL_MANGLING:u32      = 0x00000004;
    /// Support smbus_pec
    pub const SMBUS_PEC:u32              = 0x00000008;
    /// Support Nostart
    pub const NOSTART:u32                = 0x00000010;
    /// Support Slave
    pub const SLAVE:u32                  = 0x00000020;
    /// Fill Doc
    pub const SMBUS_BLOCK_PROC_CALL:u32  = 0x00008000; /* SMBus 2.0 or later */
    /// Fill Doc
    pub const SMBUS_QUICK:u32            = 0x00010000;
    /// Fill Doc
    pub const SMBUS_READ_BYTE:u32        = 0x00020000;
    /// Fill Doc
    pub const SMBUS_WRITE_BYTE:u32       = 0x00040000;
    /// Fill Doc
    pub const SMBUS_READ_BYTE_DATA:u32   = 0x00080000;
    /// Fill Doc
    pub const SMBUS_WRITE_BYTE_DATA:u32  = 0x00100000;
    /// Fill Doc
    pub const SMBUS_READ_WORD_DATA:u32   = 0x00200000;
    /// Fill Doc
    pub const SMBUS_WRITE_WORD_DATA:u32  = 0x00400000;
    /// Fill Doc
    pub const SMBUS_PROC_CALL:u32        = 0x00800000;
    /// required for I2C_M_RECV_LEN
    pub const SMBUS_READ_BLOCK_DATA:u32  = 0x01000000;
    /// Fill Doc
    pub const SMBUS_WRITE_BLOCK_DATA:u32 = 0x02000000;
    /// I2C-like block xfer
    pub const SMBUS_READ_I2C_BLOCK:u32   = 0x04000000;
    /// w/ 1-byte reg. addr.
    pub const SMBUS_WRITE_I2C_BLOCK:u32  = 0x08000000;
    /// SMBus 2.0 or later
    pub const SMBUS_HOST_NOTIFY:u32      = 0x10000000;

    // Multi-bit flags
    /// Fill Doc
    pub const SMBUS_BYTE:u32 =  Self::SMBUS_READ_BYTE | Self::SMBUS_WRITE_BYTE;
    /// Fill Doc
    pub const SMBUS_BYTE_DATA:u32 =  Self::SMBUS_READ_BYTE_DATA | Self::SMBUS_WRITE_BYTE_DATA;
    /// Fill Doc
    pub const SMBUS_WORD_DATA:u32 = Self::SMBUS_READ_WORD_DATA | Self::SMBUS_WRITE_WORD_DATA;
    /// Fill Doc
    pub const SMBUS_BLOCK_DATA:u32 = Self::SMBUS_READ_BLOCK_DATA | Self::SMBUS_WRITE_BLOCK_DATA;
    /// Fill Doc
    pub const SMBUS_I2C_BLOCK:u32 = Self::SMBUS_READ_I2C_BLOCK | Self::SMBUS_WRITE_I2C_BLOCK;

}