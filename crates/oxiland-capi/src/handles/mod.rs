//! Opaque tagged handles and live-pointer registry.

use std::cell::Cell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

use crate::error::set_last_error;

pub mod concepts;
pub mod digest;
pub mod files;
pub mod hash;
pub mod helpers;
pub mod heuristics;
pub mod io;
pub mod iterator;
pub mod list;
pub mod log_msg;
pub mod model;
pub mod node;
pub mod parser;
pub mod query;
pub mod serializer;
pub mod statement;
pub mod storage;
pub mod stream;
pub mod uri;
pub mod world;

pub const TAG_WORLD: u32 = 0x4F58_5701;
pub const TAG_STORAGE: u32 = 0x4F58_5702;
pub const TAG_MODEL: u32 = 0x4F58_5703;
pub const TAG_URI: u32 = 0x4F58_5704;
pub const TAG_NODE: u32 = 0x4F58_5705;
pub const TAG_STATEMENT: u32 = 0x4F58_5706;
pub const TAG_STREAM: u32 = 0x4F58_5707;
pub const TAG_PARSER: u32 = 0x4F58_5708;
pub const TAG_SERIALIZER: u32 = 0x4F58_5709;
pub const TAG_QUERY: u32 = 0x4F58_570A;
pub const TAG_QUERY_RESULTS: u32 = 0x4F58_570B;
pub const TAG_DIGEST: u32 = 0x4F58_570C;
pub const TAG_HASH: u32 = 0x4F58_570D;
pub const TAG_LIST: u32 = 0x4F58_570E;
pub const TAG_ITERATOR: u32 = 0x4F58_570F;
pub const TAG_QUERY_RESULTS_FORMATTER: u32 = 0x4F58_5710;
pub const TAG_IOSTREAM: u32 = 0x4F58_5711;
pub const TAG_FREED: u32 = 0xDEAD_F00D;

/// Identity hasher for the live-handle registry's already-uniform pointer keys.
///
/// The default SipHash is useful for attacker-controlled strings, but every
/// key here is an aligned address allocated by this process. Hashing the
/// pointer directly keeps validation cheap on hot C getters while the mutex
/// and registry continue to defend against stale or wrongly typed handles.
#[derive(Default)]
struct PointerHasher(u64);

impl Hasher for PointerHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut value = 0_u64;
        for (shift, byte) in bytes.iter().take(8).enumerate() {
            value |= u64::from(*byte) << (shift * 8);
        }
        self.0 = value;
    }

    fn write_usize(&mut self, value: usize) {
        self.0 = value as u64;
    }
}

type LiveHandles = HashMap<usize, u32, BuildHasherDefault<PointerHasher>>;

static LIVE: LazyLock<Mutex<LiveHandles>> = LazyLock::new(|| Mutex::new(LiveHandles::default()));
static LIVE_GENERATION: AtomicU64 = AtomicU64::new(1);

thread_local! {
    /// Last successfully validated handle for hot repeated calls. Any registry
    /// mutation advances `LIVE_GENERATION` and invalidates this entry.
    static LAST_VALIDATED: Cell<(usize, u32, u64)> = const { Cell::new((0, 0, 0)) };
}

/// Heap object shared by every opaque C handle.
#[repr(C)]
pub struct TypedHandle<T> {
    pub tag: u32,
    pub inner: T,
}

fn register(ptr: usize, tag: u32) {
    LIVE.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(ptr, tag);
    LIVE_GENERATION.fetch_add(1, Ordering::Release);
}

fn validate_live(ptr: usize, expected_tag: u32) -> bool {
    let generation = LIVE_GENERATION.load(Ordering::Acquire);
    if LAST_VALIDATED.with(|last| last.get() == (ptr, expected_tag, generation)) {
        return true;
    }
    let result = match LIVE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&ptr)
        .copied()
    {
        Some(tag) if tag == expected_tag => true,
        Some(tag) => {
            set_last_error(format!(
                "handle type tag mismatch (got {tag:#x}, expected {expected_tag:#x})"
            ));
            false
        }
        None => {
            set_last_error("freed or invalid handle");
            false
        }
    };
    if result {
        let generation = LIVE_GENERATION.load(Ordering::Acquire);
        LAST_VALIDATED.with(|last| last.set((ptr, expected_tag, generation)));
    }
    result
}

fn unregister(ptr: usize, expected_tag: u32) -> bool {
    let mut live = LIVE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    match live.get(&ptr).copied() {
        Some(tag) if tag == expected_tag => {
            live.remove(&ptr);
            LIVE_GENERATION.fetch_add(1, Ordering::Release);
            true
        }
        Some(tag) => {
            set_last_error(format!(
                "handle type tag mismatch on free (got {tag:#x}, expected {expected_tag:#x})"
            ));
            false
        }
        None => {
            set_last_error("double free or invalid handle");
            false
        }
    }
}

/// Boxes `inner` and returns a raw handle pointer registered as live.
pub fn box_handle<T>(tag: u32, inner: T) -> *mut TypedHandle<T> {
    let handle = Box::new(TypedHandle { tag, inner });
    let ptr = Box::into_raw(handle);
    register(ptr as usize, tag);
    ptr
}

/// Null-safe typed borrow. Sets last-error on failure.
///
/// # Safety
/// `ptr` must be null or a live handle previously returned by [`box_handle`]
/// for the same `T` and `expected_tag`.
pub unsafe fn borrow_handle<'a, T>(
    ptr: *mut TypedHandle<T>,
    expected_tag: u32,
) -> Option<&'a mut TypedHandle<T>> {
    if ptr.is_null() {
        set_last_error("null handle");
        return None;
    }
    if !validate_live(ptr as usize, expected_tag) {
        return None;
    }
    // SAFETY: caller guarantees `ptr` is a live TypedHandle<T> from this crate.
    let handle = unsafe { &mut *ptr };
    if handle.tag != expected_tag {
        set_last_error(format!(
            "handle type tag mismatch (got {:#x}, expected {:#x})",
            handle.tag, expected_tag
        ));
        return None;
    }
    Some(handle)
}

/// Frees a handle. Null is a no-op. Double-free of an unregistered pointer
/// records an error and returns without dropping again.
///
/// # Safety
/// `ptr` must be null or a pointer from [`box_handle`] for this `T`.
pub unsafe fn free_handle<T>(ptr: *mut TypedHandle<T>, expected_tag: u32) {
    if ptr.is_null() {
        return;
    }
    let addr = ptr as usize;
    if !unregister(addr, expected_tag) {
        return;
    }
    // SAFETY: address was live in the registry; exclusive ownership restored.
    let handle = unsafe { &mut *ptr };
    if handle.tag != expected_tag {
        set_last_error(format!(
            "handle type tag mismatch on free (got {:#x}, expected {:#x})",
            handle.tag, expected_tag
        ));
    }
    handle.tag = TAG_FREED;
    // SAFETY: unique ownership; memory came from Box::into_raw.
    drop(unsafe { Box::from_raw(ptr) });
}

/// Reads a required NUL-terminated UTF-8 C string.
///
/// # Safety
/// `ptr` must be null or a valid NUL-terminated C string for the duration.
pub unsafe fn cstr_required<'a>(ptr: *const c_char, field: &str) -> Option<&'a str> {
    if ptr.is_null() {
        set_last_error(format!("{field} is null"));
        return None;
    }
    // SAFETY: caller provides a valid C string pointer.
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    match cstr.to_str() {
        Ok(s) => Some(s),
        Err(_) => {
            set_last_error(format!("{field} is not valid UTF-8"));
            None
        }
    }
}

/// Reads an optional NUL-terminated UTF-8 C string (`NULL` → `None`).
///
/// # Safety
/// Same as [`cstr_required`] when non-null.
pub unsafe fn cstr_optional<'a>(ptr: *const c_char, field: &str) -> Result<Option<&'a str>, ()> {
    if ptr.is_null() {
        return Ok(None);
    }
    // SAFETY: caller provides a valid C string pointer.
    let cstr = unsafe { std::ffi::CStr::from_ptr(ptr) };
    match cstr.to_str() {
        Ok(s) => Ok(Some(s)),
        Err(_) => {
            set_last_error(format!("{field} is not valid UTF-8"));
            Err(())
        }
    }
}
