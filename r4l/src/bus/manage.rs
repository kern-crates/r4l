//! Bus manage
//!
//! Linux reuesed kset manage bus and bus devices and drivers.
//! Now, we just use a simple linked_list to manage
//! TODO: replace linked_list with kset 
use crate::sync::Mutex;
use crate::bus::BusType;
use crate::sync::Arc;
use core::ops::Deref;
use crate::linked_list::{GetLinks, Links, List};

struct BusInner {
    name: &'static str,
    dev_name: &'static str,
}

pub enum BusMethodEnum {
    MATCH,
    PROBE,
}

pub trait BusTypeMethod {
    fn method_support(&self, method: BusMethodEnum) -> bool;
    fn bus_match(&self) {unimplemented!()}
    fn probe(&self) {unimplemented!()}
}

impl BusInner {
    fn new(name: &'static str, dev_name: &'static str) -> Self {
        BusType
    }
}

/// A task wrapper.
///
/// It add extra states to use in [`linked_list::List`].
pub struct BusType {
    inner: BusInner,
    links: Links<Self>,
}

impl GetLinks for BusType {
    type EntryType = Self;

    #[inline]
    fn get_links(t: &Self) -> &Links<Self> {
        &t.links
    }
}

impl BusType {
    /// Creates a new bus node.
    pub const fn new(name: &'static str, dev_name: &'static str) -> Self {
        Self {
            inner: BusInner::new(name, dev_name),
            links: Links::new(),
        }
    }
}

impl Deref for BusType {
    type Target = BusType;
    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// A simple FIFO wait task list
///
/// When a task is added to the list, it's placed at the end of the waitlist. 
/// When picking the next task to run, the head of the wait list is taken.
struct BusList {
    list: List<Arc<BusType>>,
}

impl BusList {
    /// Creates a new empty [WaitList].
    const fn new() -> Self {
        Self {
            list: List::new(),
        }
    }

    /// add node to list back
    fn add(&mut self, node: Arc<BusType>) {
        self.list.push_back(node);
    }

    /// Removes the given Node
    ///
    /// # Safety
    ///
    /// Callers must ensure that `data` is either on this list or in no list. It being on another
    /// list leads to memory unsafety.
    fn remove(&mut self, node: &Arc<BusType>) -> Option<Arc<BusType>> {
        unsafe { self.list.remove(node)}
    }
}

pub fn bus_register(bus: Arc<BusType>) {
    BUS_LIST.lock().add(bus);
}

pub fn bus_unregister(bus: &Arc<BusType>) {
    BUS_LIST.lock().remove(bus);
}
