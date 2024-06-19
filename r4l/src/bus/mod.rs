//! Bus implement
//!
//! Linux has a module called bus.
//! Each bus type in the kernel (PCI, USB, etc) should
//! declare one static object of this type. They must
//! initialize the name field, and may optionally initialize
//! the match callback
//!
//! Drivers and device attached on a bus_type

