// SPDX-License-Identifier: GPL-2.0

//! I2c kernel interface

pub mod timing;
pub mod functionality;
pub mod msg;

mod i2c_bus;
mod i2c_client;
mod i2c_drv;

pub use i2c_bus::*;
pub use i2c_client::*;
pub use i2c_drv::*;

use crate::{prelude::*, device};
use core::marker::PhantomData;
use of_fdt::OfNode;
use msg::I2cMsgInfo;
use crate::sync::{Arc,Mutex};
use crate::driver;

use macros::vtable;

/// A i2cAdapter.
#[vtable]
pub trait I2cAlgo {
    /// Context data associated with the gpio chip.
    ///
    /// It determines the type of the context data passed to each of the methods of the trait.
    type Data: Send + Sync + Clone ;

    /// master xfer 
    fn master_xfer(_data: &Self::Data, _msg: &I2cMsg, _msg_len: usize) -> Result<i32>;

    /// functonality
    fn functionality(_data: &Self::Data) -> u32;
}

/// Wraps the kernel's struct of i2c_msg.
pub struct I2cMsg(Vec<I2cMsgInfo>);

// SAFETY: `msg` only holds a pointer to a C i2c_msg, which is safe to be used from any thread.
unsafe impl Send for I2cMsg {}

// SAFETY: `Device` only holds a pointer to a C i2c_msg, references to which are safe to be used
// from any thread.
unsafe impl Sync for I2cMsg {}


impl I2cMsg {
    /// From arceos msgs to user msg array
    pub fn into_array(&self) -> Result<Vec<I2cMsgInfo>>
    {
        Ok(self.0.clone())
    }
}

#[derive(Clone)]
pub struct I2cAlgorithm {
    pub master_xfer: Option<fn(client: &mut I2cClient, msg: &I2cMsg, msg_len: usize) -> Result<i32>>,
    pub functionality: Option<fn(client: &mut I2cClient) -> Result<u32>>,
}

impl Default for I2cAlgorithm {
    fn default() -> Self {
        let mut s = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

///
/// `Struct i2c_adapter`.
///
#[derive(Clone)]
pub struct I2cAdapter{
    name: &'static str,
    owner: &'static ThisModule,
    of_node: OfNode<'static>,
    parent: device::Device,
    pub alogrithm: I2cAlgorithm,
}

unsafe impl Sync for I2cAdapter {}
unsafe impl Send for I2cAdapter {}


impl I2cAdapter {
    // Creates a new [`I2cAdapter`] but does not register it yet.
    pub fn new <T: I2cAlgo>(
        name: &'static CStr,
        owner: &'static ThisModule, 
        of_node: OfNode<'static>,
        device: device::Device,
    ) -> Self 
    where <T as I2cAlgo>::Data: 'static {
        let mut instance = Self {
            name: name,
            owner: owner,
            of_node: of_node,
            parent: device,
            alogrithm: I2cAlgorithm::default(),
        };        
        
        let algo = &mut instance.alogrithm;

        if T::HAS_MASTER_XFER {
            algo.master_xfer = Some(master_xfer_callback::<T>);
        }
        if T::HAS_FUNCTIONALITY {
            algo.functionality = Some(functionality_callback::<T>);
        }
        instance
    }

    pub fn get_node(&self) -> OfNode<'static> {
        self.of_node
    }

    // Add the number of adapter,and register adapter into clinet
    pub fn add_numbered_adapter<T: I2cAlgo>(&self, data: T::Data) -> Result 
        where <T as I2cAlgo>::Data: 'static
    {
        i2c_register_adapter::<T>( self.clone(), data)?;
        Ok(())
    }

}

impl Drop for I2cAdapter {
    fn drop(&mut self) {
        pr_warn!("i2c adapter dropped");
    }
}

pub fn master_xfer_callback<T: I2cAlgo>(client: &mut I2cClient, msg: &I2cMsg, msg_len: usize) -> Result<i32> 
    where <T as I2cAlgo>::Data: 'static
{
    let data =  client.get_adpt_data::<T::Data>().unwrap();
    T::master_xfer(data, msg, msg_len)
}

pub fn functionality_callback<T: I2cAlgo>(client: &mut I2cClient) -> Result<u32>  
    where <T as I2cAlgo>::Data: 'static
{
    let data =  client.get_adpt_data::<T::Data>().unwrap();
    Ok(T::functionality(data))
}