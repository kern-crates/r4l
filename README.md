# R4L: Rust OS Compatible with Linux

## History
In the earliest design, we proposed OSL (Operating System Layer). The interfaces and organization
of OSL were not based on any single OS as a reference standard; rather, OSL defined its own more
general interfaces. During driver development, interaction with the OS had to follow OSL interface
specifications. This way, as long as the target OS was compatible with OSL, the driver would naturally
be compatible as well. This is what we called the cross-kernel driver framework and implementation.

The benefit of this approach is that hardware manufacturers could focus solely on the driver itself
without needing to learn various different OSs and then adapt to each one. 
The project maintenance address is: https://github.com/kern-crates/.github/pull/7/files.

However, this approach has a significant drawback: it requires acceptance from all hardware manufacturers.
In reality, hardware manufacturers are more inclined to adapt to mainstream OSs like Linux/Windows, and the
hardware ecosystem of these mainstream OSs is already relatively stable. Therefore, we proposed a second solution.
Based on the idea of the first solution, we let OSL follow the framework of a particular OS (Linux).
This is the basis of this project.

## How to use it in ArceOS
In your Arceos directory, clone this project 
```shell
cd $(path to arceos)/
git clone https://github.com/kern-crates/r4l.git
```

### Modify Arceos configurations
Modify the Arceos code to ensure that cross-kernel drivers can be used.”

1. modify modules code to add the cross-kernel drivers framework in Arceos

    If you use the i2c-designware driver, you need to add 
    
    in modules/axdriver/Cargo.toml:
    ``` shell
    # r4l driver
    r4l = { path = "../../r4l/r4l/" , features=["arceos"] }
    i2c_designware = { path = "../../r4l/drivers/i2c/busses/i2c-designware" , features=["arceos"] }
    ox3c_raw = { path = "../../r4l/drivers/i2c/ox3c-raw" , features=["arceos"] }
    ```
    in modules/axdriver/src/lib.rs
    ``` shell
    extern crate ox3c_raw;
    extern crate i2c_designware;

    # in init_drivers function 
    r4l::init::driver_framework_init(dtb_vaddr);
    ```
    For other modifications, please refer to the patch: ./r4l/patches/0001-for-arceos.patch.
    or refer to github commit: 
    
    https://github.com/happy-thw/arceos/commit/06b390d264df04ba49c904a00c96ed1ad21a4caf

    https://github.com/happy-thw/arceos/commit/2f7ee3e8d9237256fe2730e483ab9d5d2a24d237


2. modify `handler_table` crate 
    ``` shell
    diff --git a/src/lib.rs b/src/lib.rs
    index becd784..4d5aec6 100644
    --- a/src/lib.rs
    +++ b/src/lib.rs
    @@ -6,7 +6,7 @@ use core::sync::atomic::{AtomicUsize, Ordering};
    /// The type of an event handler.
    ///
    /// Currently no arguments and return values are supported.
    -pub type Handler = fn();
    +pub type Handler = fn(u32);
    
    /// A lock-free table of event handlers.
    ///
    @@ -40,7 +40,7 @@ impl<const N: usize> HandlerTable<N> {
            let handler = self.handlers[idx].load(Ordering::Acquire);
            if handler != 0 {
                let handler: Handler = unsafe { core::mem::transmute(handler) };
    -            handler();
    +            handler(idx as u32);
                true
            } else {
                false
    ```
3. if use it in A1000b
    - compile
    ``` shell
    make A=examples/shell PLATFORM=aarch64-bsta1000b LOG=debug SMP=8 FEATURES="driver-ramdisk,multitask,irq" fada
    ```
    - replace Img file:

        replace the generated arceos-fada.itb with the A1000b kernel image format.