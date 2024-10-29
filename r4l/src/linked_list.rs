//! Defines the OS base link list.
//!
//! Every OS should provides:
//! linked_list

#[cfg(feature = "arceos")]
mod list {
    pub use linked_list::*;
}

pub use list::*;
