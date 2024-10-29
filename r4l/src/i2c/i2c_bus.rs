// SPDX-License-Identifier: GPL-2.0

use super::{I2cClient, I2cDriver,I2cAdapter,I2cAlgo};
use alloc::collections::VecDeque;
use axlog::info;

use crate::bus::BusType;
use crate::device::DeviceOps;
use crate::of::DeviceId::Compatible;
use crate::prelude::Vec;
use crate::sync::{Arc, Mutex};
use crate::{error::*, pr_debug, pr_info};

pub struct I2cBus {
    i2c_clients: VecDeque<Arc<Mutex<I2cClient>>>,
    i2c_drivers: VecDeque<Arc<Mutex<I2cDriver>>>,
}

impl I2cBus {
    const fn new() -> Self {
        I2cBus {
            i2c_clients: VecDeque::new(),
            i2c_drivers: VecDeque::new(),
        }
    }
}

unsafe impl Send for I2cBus {}
unsafe impl Sync for I2cBus {}

impl BusType for I2cBus {
    const NAME: &'static str = "i2c";
    type Device = Arc<Mutex<I2cClient>>;
    type Driver = Arc<Mutex<I2cDriver>>;

    fn bus_driver_match(&self, i2cdrv: Self::Driver) -> Vec<Self::Device> {
        let mut matched_i2cdrv: Vec<_> = Vec::new();
        let table = i2cdrv
            .lock()
            .id_table()
            .expect("i2c driver not define Compatible Table");
        for clinet in self.i2c_clients.iter() {
            for id in table {
                match id {
                    Compatible(id) => {
                        if clinet.lock().compatible_match(id) {
                            pr_info!("i2c driver : {} i2c client matched", id);
                            matched_i2cdrv.push(clinet.clone());
                        }
                    }
                    _ => panic!("invalid id table"),
                }
            }    
        }
        matched_i2cdrv
    }

    fn add_device(&mut self, i2cdev: Self::Device) -> Result {
        self.i2c_clients.push_back(i2cdev);
        Ok(())
    }

    fn add_driver(&mut self, i2cdrv: Self::Driver) -> Result {
        self.i2c_drivers.push_back(i2cdrv);
        Ok(())
    }

}

static I2C_BUS: Mutex<I2cBus> = Mutex::new(I2cBus::new());

pub fn i2c_register_device(client: Arc<Mutex<I2cClient>>) -> Result {
    I2C_BUS.lock().add_device(client)?;
    Ok(())
}

pub fn i2c_register_driver(i2cdrv: Arc<Mutex<I2cDriver>>) -> Result {
    let mut i2c_bus = I2C_BUS.lock();
    i2c_bus.add_driver(i2cdrv.clone())?;
    let matched_i2cdrv = i2c_bus.bus_driver_match(i2cdrv.clone());
    // before probe, unlock bus
    drop(i2c_bus);
    for clinet in matched_i2cdrv {
        match i2cdrv.lock().probe {
            Some(fn_probe) => fn_probe(clinet)?,
            None => panic!("pdev not have probe call back"),
        }
    }
    Ok(())
}

pub fn i2c_register_adapter<T: I2cAlgo>(adpter: I2cAdapter, data: T::Data) -> Result 
    where <T as I2cAlgo>::Data: 'static
{
    let adpt_of_node = adpter.of_node;
    let mut child = adpt_of_node.children().peekable();
    if child.peek().is_some() {
        for c in child {
            if !of_fdt::of_device_is_available(c) {
                continue;
            }
            let client = Arc::new(
                Mutex::new(I2cClient::new(c, adpter.clone()))
            );
            client.lock().set_adpt_data(data.clone());
            I2C_BUS.lock().add_device(client.clone())?;
            pr_info!("i2c clinet {} register ok ", c.name); 
        }
    } else {
            pr_debug!("there is no childen in {}", adpt_of_node.name);
    } 
    Ok(())
}

