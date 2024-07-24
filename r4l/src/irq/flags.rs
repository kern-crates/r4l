// SPDX-License-Identifier: GPL-2.0

use bitflags::bitflags;

bitflags! {
    /// Container for interrupt flags.
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct Flags: u32 {
        /// Use the interrupt line as already configured.
        const TRIGGER_NONE = 1;
        /// The interrupt is triggered when the signal goes from low to high.
        const TRIGGER_RISING = 1 << 1;
        /// Allow sharing the irq among several devices.
        const SHARED = 1 << 2;
        /// Do not disable this IRQ during suspend. Does not guarantee that this interrupt will wake
        /// the system from a suspended state.
        const NO_SUSPEND = 1 << 3;
        /// If the IRQ is shared with a NO_SUSPEND user, execute this interrupt handler after
        /// suspending interrupts. For system wakeup devices users need to implement wakeup detection
        /// in their interrupt handlers.
        const COND_SUSPEND = 1 << 4;
        /// Interrupt is per cpu.
        const PERCPU = 1 << 5;
    }
}
