//! Driver init

use core::ffi::{c_int, c_void};

pub fn driver_framework_init() {
    subsys_fn_init();
    module_fn_init();
}

fn subsys_fn_init() {
 //   crate::net::phy::mdio_bus_init();
}

fn module_fn_init(){
    let fn_ptr_size = core::mem::size_of::<*const extern "C" fn() -> c_int>();
    let start_addr = _initcall as *const u8;
    let end_addr = _initcall_end as *const u8;
    let mut current_addr = start_addr;

    crate::pr_debug!("enter driver_framework_init start_Addr {:p}  end_Addr{:p}", start_addr, end_addr);
    while current_addr < end_addr {

        let func_ptr_ptr: *const *const c_void = current_addr as *const *const c_void;
        let func_ptr_value: *const c_void = unsafe { *func_ptr_ptr };
        let func: extern "C" fn() -> c_int = unsafe {
            core::mem::transmute(func_ptr_value as *const extern "C" fn() -> c_int)
        };
        let result =  func();
        crate::pr_debug!("Function at address {:p} returned: {}", current_addr, result);

        current_addr = unsafe { current_addr.add(fn_ptr_size) };
    }
}

extern "C" {
    fn _initcall();
    fn _initcall_end();
}
