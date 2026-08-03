//! Internal state objects for the C model adapter.
//!
//! These types keep cache and transaction invariants out of the FFI entry
//! points. They intentionally expose behavior rather than writable fields.

use std::ffi::c_void;
use std::ptr;

pub struct CardinalityCache(Option<usize>);

impl CardinalityCache {
    pub fn known_empty() -> Self {
        Self(Some(0))
    }

    pub fn unknown() -> Self {
        Self(None)
    }

    pub fn get(&self) -> Option<usize> {
        self.0
    }

    pub fn store(&mut self, value: usize) {
        self.0 = Some(value);
    }

    pub fn invalidate(&mut self) {
        self.0 = None;
    }
}

pub struct TransactionState {
    active: bool,
    handle: *mut c_void,
}

impl TransactionState {
    pub fn idle() -> Self {
        Self {
            active: false,
            handle: ptr::null_mut(),
        }
    }

    pub fn begin(&mut self) -> bool {
        if self.active {
            return false;
        }
        self.active = true;
        true
    }

    pub fn begin_with_handle(&mut self, handle: *mut c_void) {
        self.active = true;
        self.handle = handle;
    }

    pub fn finish(&mut self) {
        self.active = false;
        self.handle = ptr::null_mut();
    }

    pub fn handle(&self) -> *mut c_void {
        self.handle
    }
}
