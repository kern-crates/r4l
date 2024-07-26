//! Driver init

use core::ffi::{c_int, c_void};


struct InitcallAddrPair(*const u8, *const u8);

pub fn driver_framework_init() {
    subsys_fn_init();
    module_fn_init();
}

fn subsys_fn_init() {
    if let Err(e) = crate::of::of_platform_default_populate_init() {
        panic!("subsys fn init failed");
    }
}

fn initcall(pair: InitcallAddrPair) {
    let fn_ptr_size = core::mem::size_of::<*const extern "C" fn() -> c_int>();
    let start_addr = pair.0;
    let end_addr = pair.1;
    let mut current_addr = start_addr;

    crate::pr_info!(
        "enter driver_framework_init start_Addr {:p}  end_Addr{:p}",
        start_addr,
        end_addr
    );
    while current_addr < end_addr {
        let func_ptr_ptr: *const *const c_void = current_addr as *const *const c_void;
        let func_ptr_value: *const c_void = unsafe { *func_ptr_ptr };
        let func: extern "C" fn() -> c_int =
            unsafe { core::mem::transmute(func_ptr_value as *const extern "C" fn() -> c_int) };
        let result = func();
        crate::pr_info!("Function at address {:p} returned: {}",current_addr,result);
        if result < 0 {
            panic!("driver module init call failed");
        }
        current_addr = unsafe { current_addr.add(fn_ptr_size) };
    }
}

fn module_fn_init() {
    initcall(InitcallAddrPair(_initcall1 as *const u8, _initcall1_end as *const u8));
    initcall(InitcallAddrPair(_initcall2 as *const u8, _initcall2_end as *const u8));
    initcall(InitcallAddrPair(_initcall3 as *const u8, _initcall3_end as *const u8));
    initcall(InitcallAddrPair(_initcall4 as *const u8, _initcall4_end as *const u8));
    initcall(InitcallAddrPair(_initcall5 as *const u8, _initcall5_end as *const u8));
    initcall(InitcallAddrPair(_initcall6 as *const u8, _initcall6_end as *const u8));
    initcall(InitcallAddrPair(_initcall7 as *const u8, _initcall7_end as *const u8));
}

extern "C" {
    fn _initcall1();
    fn _initcall1_end();
    fn _initcall2();
    fn _initcall2_end();
    fn _initcall3();
    fn _initcall3_end();
    fn _initcall4();
    fn _initcall4_end();
    fn _initcall5();
    fn _initcall5_end();
    fn _initcall6();
    fn _initcall6_end();
    fn _initcall7();
    fn _initcall7_end();
}
