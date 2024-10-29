//! A pure-Rust #![no_std] crate for parsing Flattened Devicetrees,
//! with the goal of having a very ergonomic and idiomatic API.

#![no_std]
#![allow(missing_docs)]
pub struct MachineFdt<'a>(fdt::Fdt<'a>);
pub mod kernel_nodes;
pub use fdt::standard_nodes::Cpu;
pub use kernel_nodes::*;

pub type OfNode<'a> = fdt::node::FdtNode<'a, 'a>;

mod parsing;
mod phandle_arg;

pub use phandle_arg::OfPhandleArgs;
use phandle_arg::OfPhandleIterator;

use crate::parsing::BigEndianU32;

static mut MY_FDT_PTR: Option<*const u8> = None;

lazy_static::lazy_static! {
    static ref MY_MACHINE_FDT: MachineFdt<'static> =
        unsafe {init_from_ptr(MY_FDT_PTR.unwrap())};
}

pub fn get_fdt_ptr() -> Option<*const u8> {
    unsafe { MY_FDT_PTR }
}

/// # Safety
/// This function is unsafe because it dereferences a raw pointer.
pub unsafe fn init_fdt_ptr(virt_addr: *const u8) {
    MY_FDT_PTR = Some(virt_addr);
}

/// Init the DTB root, call after dtb finish mapping
unsafe fn init_from_ptr(virt_addr: *const u8) -> MachineFdt<'static> {
    MachineFdt(fdt::Fdt::from_ptr(virt_addr).unwrap())
}

/// Root Node found model or first compatible
pub fn machin_name() -> &'static str {
    let root_node = MY_MACHINE_FDT.0.root();
    let model = root_node
        .properties()
        .find(|p| p.name == "model")
        .and_then(|p| {
            core::str::from_utf8(p.value)
                .map(|s| s.trim_end_matches('\0'))
                .ok()
        });

    if let Some(name) = model {
        name
    } else {
        root_node.compatible().first()
    }
}

/// Searches for a node which contains a `compatible` property and contains
/// one of the strings inside of `with`
pub fn find_compatible_node(
    with: &'static [&'static str],
) -> impl Iterator<Item = OfNode<'static>> {
    MY_MACHINE_FDT.0.all_nodes().filter(|n| {
        n.compatible()
            .and_then(|compats| compats.all().find(|c| with.contains(c)))
            .is_some()
    })
}

pub fn of_device_is_available(node: OfNode<'static>) -> bool {
    let status = node.properties().find(|p| p.name == "status");
    let ret = match status {
        None => true,
        Some(st) => {
            let res: &'static str = core::str::from_utf8(st.value)
                .map(|s| s.trim_end_matches('\0'))
                .ok()
                .unwrap();
            if res.eq("okay") || res.eq("ok") {
                true
            } else {
                false
            }
        }
    };
    ret
}

pub fn of_property_read_u32(
    node: OfNode<'static>,
    name: &'static str,
    index: usize,
) -> Option<u32> {
    let property = node.property(name)?;
    let start_idx = index * 4;
    if start_idx + 4 > property.value.len() {
        return None;
    }
    Some(
        BigEndianU32::from_bytes(&property.value[start_idx..])
            .unwrap()
            .get(),
    )
}

pub fn bootargs() -> Option<&'static str> {
    MY_MACHINE_FDT.0.chosen().bootargs()
}

pub fn fdt_size() -> usize {
    MY_MACHINE_FDT.0.total_size()
}

pub fn memory_nodes() -> impl Iterator<Item = Memory> {
    MY_MACHINE_FDT
        .0
        .find_all_nodes("/memory")
        .map(|m| kernel_nodes::Memory { node: m })
}

pub fn pcsi() -> Option<kernel_nodes::Pcsi> {
    MY_MACHINE_FDT
        .0
        .find_node("/psci")
        .map(|n| kernel_nodes::Pcsi { node: n })
}

pub fn cpus() -> impl Iterator<Item = fdt::standard_nodes::Cpu<'static, 'static>> {
    MY_MACHINE_FDT.0.cpus()
}

pub fn find_phandle(phandle: u32) -> Option<OfNode<'static>> {
    MY_MACHINE_FDT.0.find_phandle(phandle)
}

pub fn of_parse_phandle_with_args(
    node: OfNode<'static>,
    list_name: &'static str,
    cell_name: Option<&'static str>,
    index: usize,
) -> Option<OfPhandleArgs> {
    let mut iter = OfPhandleIterator::new(node, list_name, cell_name, 0)?;
    iter.nth(index)
}
