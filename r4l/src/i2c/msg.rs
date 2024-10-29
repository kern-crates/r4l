//! I2C msg abstraction
//!
//! Every OS should provide I2cMsg type and implement GeneralI2cMsg
//!
use crate::types::{bit, Bit, genmask};

/// I2cMsgFlags Struct
#[repr(C)]
pub struct I2cMsgFlags;
/// To determine what I2C functionality is present
impl I2cMsgFlags {
    /// read data (from slave to master). Guaranteed to be 0x0001
    pub const I2C_MASTER_READ:u16 = 0x0001;
    /// Use Packet Error Checking
    pub const I2C_CLIENT_PEC:u16  = 0x0004;
    /// this is a 10 bit chip address
    /// Only if I2C_FUNC_10BIT_ADDR is set
    pub const I2C_ADDR_TEN:u16 =  0x0010;
    /// we are the slave
    pub const I2C_CLIENT_SLAVE:u16 = 0x0020;
    /// We want to use I2C host notify
    pub const I2C_CLIENT_HOST_NOTIFY:u16 = 0x0040;
    /// for board_info; true if can wake
    pub const I2C_CLIENT_WAKE:u16 = 0x0080;
    /// Linux kernel
    pub const I2C_MASTER_DMA_SAFE:u16 = 0x0200;
    /// message length will be first received byte
    /// Only if I2C_FUNC_SMBUS_READ_BLOCK_DATA is set
    pub const I2C_MASTER_RECV_LEN:u16 = 0x0400;
    /// in a read message, master ACK/NACK bit is skipped
    pub const I2C_MASTER_NOREADACK:u16 = 0x0800;
    /// treat NACK from client as ACK
    pub const I2C_MASTER_IGN_NAK:u16 = 0x1000;
    /// toggles the Rd/Wr bit
    pub const I2C_MASTER_REVDIR:u16 = 0x2000;
    /// skip repeated start sequence
    pub const I2C_MASTER_NOSTART:u16 = 0x4000;
    /// force a STOP condition after the message
    pub const I2C_MASTER_STOP:u16 = 0x8000;

        // Multi-bit flags
    /// Use Omnivision SCCB protocol Must match I2C_M_STOP|IGNORE_NAK
    pub const I2C_CLIENT_SCCB:u16 = Self::I2C_MASTER_STOP | Self::I2C_MASTER_IGN_NAK;
}

/// GeneralI2cMsg data that Type I2cMsg must implement 
pub trait GeneralI2cMsgInfo:Default+Sync+Send {
    /// Create a new I2cMsg with addr and data that need to transfer
    fn new_send<const N: usize>(addr: u16, flags: u16, data: [u8;N]) -> Self;
    /// Create a new I2cMsg with addr and an empty buf that want to recive
    fn new_recieve(addr: u16, flags: u16, len: usize) -> Self;
    /// Get msg addr 
    fn addr(&self) -> u16;
    /// Get msg copy flags
    fn flags(&self) -> u16;
    /// int recieve cmd cnt
    fn inc_recieve_cmd_cnt(&mut self);
    /// Check whether the send msg is left last
    fn send_left_last(&self) -> bool;
    /// Check whether the send msg is end
    fn send_end(&self) -> bool;
    /// Check whether the recieve is end
    fn recieve_end(&self) -> bool ;
    /// Write 1byte to recieve msg
    fn push_byte(&mut self, byte: u8);
    /// Read 1byte from send msg front
    fn pop_front_byte(&mut self) -> u8;
    
    /// modify msg buf len, only used when flags contains I2cMasterRecvLen 
    fn modify_recieve_threshold(&mut self, buf_len: usize);
    /// modify recieve_cmd_cnt only flags contains I2cMasterRecvLen
    fn modify_recieve_cmd_cnt(&mut self, read_cmd_cnt: isize);
    /// remove one flag
    fn remove_flag(&mut self, flag: u16);
}

/// I2C Driver Sofware status flags.
/// active
pub const STATUS_ACTIVE : Bit<u32> =  bit(0);
/// Signal-driven I/O is enabled.
pub const STATUS_WRITE_IN_PROGRESS : Bit<u32> = bit(1);
/// Signal-driven I/O is enabled.
pub const STATUS_READ_IN_PROGRESS : Bit<u32> = bit(2);
/// GENMASK(2, 0)
pub const STATUS_MASK: u32 = genmask(2, 0);

///abrt cmd
pub const DW_IC_ERR_TX_ABRT:u32 = 0x1;

/// an I2C transaction segment beginning with START 
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct I2cMsgInfo{
    ///  Slave address, either 7 or 10 bits. When this is a 10 bit address,
    ///  I2C_M_TEN must be set in @flags and the adapter must support I2C_FUNC_10BIT_ADDR
    addr: u16,
    /// msg flags:
    flags: u16,
    /// The buffer into which data is read, or from which it's written
    buf: *mut u8,
    /// The buffer length
    buf_len: usize,
    /// record current recieve idx
    recieve_idx: isize,
    /// record current send idx
    send_idx: isize,
    /// Only used when msg is used to recieve,
    /// before recieve data, first need to send recieve cmd
    /// so this is used to record recieve data cmd cnt
    recieve_cmd_cnt: isize,
}

impl Default for I2cMsgInfo {
    fn default() -> Self {
        Self {
            addr: 0,
            flags: 0,
            buf: core::ptr::null_mut(),
            buf_len: 0,
            send_idx: 0,
            recieve_idx: 0,
            recieve_cmd_cnt: 0,
        }
    }
}

unsafe impl Send for I2cMsgInfo {}
unsafe impl Sync for I2cMsgInfo {}

impl I2cMsgInfo {
    /// Create a new I2cMsg from linux
    pub fn new_raw(addr: u16, flags: u16, buf: *mut u8, buf_len: usize) -> Self {
        I2cMsgInfo { addr, flags, buf, buf_len, recieve_idx: 0, send_idx: 0,  recieve_cmd_cnt: 0}
    }
}

impl GeneralI2cMsgInfo for I2cMsgInfo {
    /// Create a new I2cMsg with addr and data that need to transfer
    fn new_send<const N: usize>(_addr: u16, _flags: u16, _data: [u8;N]) -> Self {
        unimplemented!("Linux now use binding:i2c_msg");
    }

    /// Create a new I2cMsg with addr and an empty buf that want to recive
    fn new_recieve(_addr: u16, _flags: u16, _len: usize) -> Self {
        unimplemented!("Linux now use binding:i2c_msg");
    }

    /// Get msg addr 
    fn addr(&self) -> u16 {
        self.addr
    }

    /// Get msg copy flags
    fn flags(&self) -> u16 {
        self.flags
    }

    /// inc recieve_cmd_cnt
    fn inc_recieve_cmd_cnt(&mut self) {
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_READ != 0 );
        self.recieve_cmd_cnt +=1;
    }

    /// Check whether the send msg is left last
    fn send_left_last(&self) -> bool {
        // MasterRead means msg can be write
        if self.flags & I2cMsgFlags::I2C_MASTER_READ != 0 {
            self.recieve_cmd_cnt as usize == self.buf_len -1
        } else {
            self.send_idx as usize == self.buf_len - 1
        }
    }

    /// Check whether the send msg is end
    fn send_end(&self) -> bool {
        // MasterRead means msg can be write
        if self.flags & I2cMsgFlags::I2C_MASTER_READ != 0 {
            self.recieve_cmd_cnt as usize == self.buf_len
        } else {
            self.send_idx as usize == self.buf_len
        }
    }

    /// Check whether the recieve is end
    fn recieve_end(&self) -> bool {
        // MasterRead means msg can be write
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_READ != 0);
        self.recieve_idx as usize == self.buf_len
    }

    /// Write 1byte to recieve msg
    fn push_byte(&mut self, byte: u8) {
        // MasterRead means msg can be write
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_READ != 0);

        if self.recieve_end() {
            panic!("access buf overfllow");
        }
        unsafe{*self.buf.offset(self.recieve_idx) = byte};
        self.recieve_idx +=1;
    }

    /// Read 1byte from send msg front
    fn pop_front_byte(&mut self) -> u8 {
        // MasterRead means msg can be write, don't alow read 
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_READ == 0);

        if self.send_end() {
            panic!("access buf overfllow");
        }

        let byte = unsafe{*self.buf.offset(self.send_idx)};
        self.send_idx +=1;
        byte
    }

    /// modify msg buf len, only used when flags contains I2cMasterRecvLen 
    fn modify_recieve_threshold(&mut self, buf_len: usize) {
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_RECV_LEN !=0 );
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_READ != 0);
        self.buf_len = buf_len;
    }

    /// modify recieve_cmd_cnt only flags contains I2cMasterRecvLen
    fn modify_recieve_cmd_cnt(&mut self, read_cmd_cnt: isize) {
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_RECV_LEN !=0 );
        assert!(self.flags & I2cMsgFlags::I2C_MASTER_READ !=0 );
        self.recieve_cmd_cnt = read_cmd_cnt;
    }

    /// remove one flag
    fn remove_flag(&mut self, flag: u16) {
        self.flags &= !flag;
    }
}