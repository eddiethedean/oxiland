//! Opaque tagged handles and live-pointer registry.

use std::cell::Cell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::os::raw::c_char;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

#[cfg(target_os = "windows")]
use std::sync::atomic::AtomicUsize;

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

#[cfg(target_os = "windows")]
struct ProcessHotHandle {
    address: AtomicUsize,
    generation: AtomicU64,
}

#[cfg(target_os = "windows")]
impl ProcessHotHandle {
    const fn new() -> Self {
        Self {
            address: AtomicUsize::new(0),
            generation: AtomicU64::new(0),
        }
    }
}

/// Windows TLS access is noticeably more expensive across a DLL boundary.
/// Keep the most recently validated address for each fixed handle tag in a
/// process-wide lock-free cache, with the registry generation as its lifetime
/// proof. Thread-local validation remains the fallback for contention.
#[cfg(target_os = "windows")]
static PROCESS_HOT: [ProcessHotHandle; 18] = [const { ProcessHotHandle::new() }; 18];

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
    // Registering a new address cannot invalidate any previously validated
    // live handle. Address reuse is preceded by unregister(), which advances
    // the generation before the old allocation is released.
}

#[cfg(target_os = "windows")]
fn process_hot_index(tag: u32) -> Option<usize> {
    let index = usize::try_from(tag & 0xff).ok()?;
    (index < PROCESS_HOT.len()).then_some(index)
}

#[cfg(target_os = "windows")]
fn process_hot_hit(ptr: usize, tag: u32, generation: u64) -> bool {
    let Some(index) = process_hot_index(tag) else {
        return false;
    };
    let entry = &PROCESS_HOT[index];
    entry.generation.load(Ordering::Acquire) == generation
        && entry.address.load(Ordering::Relaxed) == ptr
}

#[cfg(target_os = "windows")]
fn update_process_hot(ptr: usize, tag: u32, generation: u64) {
    if let Some(index) = process_hot_index(tag) {
        let entry = &PROCESS_HOT[index];
        entry.address.store(ptr, Ordering::Relaxed);
        entry.generation.store(generation, Ordering::Release);
    }
}

fn validate_live(ptr: usize, expected_tag: u32) -> bool {
    let generation = LIVE_GENERATION.load(Ordering::Acquire);
    #[cfg(target_os = "windows")]
    if process_hot_hit(ptr, expected_tag, generation) {
        return true;
    }
    if LAST_VALIDATED.with(|last| last.get() == (ptr, expected_tag, generation)) {
        return true;
    }
    let registered_tag = {
        let live = LIVE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        live.get(&ptr).copied()
    };
    let result = match registered_tag {
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
        #[cfg(target_os = "windows")]
        update_process_hot(ptr, expected_tag, generation);
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
            drop(live);
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

/// Hot-path borrow for repeated getters that already passed validation.
///
/// Skips mutex registry lookups when the thread-local last-validated entry
/// still matches. Does not clear or set last-error; callers that miss should
/// fall back to [`borrow_handle`].
///
/// # Safety
/// Same requirements as [`borrow_handle`].
#[inline]
pub unsafe fn borrow_handle_hot<'a, T>(
    ptr: *mut TypedHandle<T>,
    expected_tag: u32,
) -> Option<&'a mut TypedHandle<T>> {
    if ptr.is_null() {
        return None;
    }
    let addr = ptr as usize;
    let generation = LIVE_GENERATION.load(Ordering::Acquire);
    #[cfg(target_os = "windows")]
    if process_hot_hit(addr, expected_tag, generation) {
        // SAFETY: the generation-bound process cache was populated only after
        // registry validation for this fixed tag.
        let handle = unsafe { &mut *ptr };
        return (handle.tag == expected_tag).then_some(handle);
    }
    let tls_hit = LAST_VALIDATED.with(|last| last.get() == (addr, expected_tag, generation));
    if !tls_hit && !validate_live(addr, expected_tag) {
        return None;
    }
    // SAFETY: registry (or TLS hit after a prior registry success) affirms live tag.
    let handle = unsafe { &mut *ptr };
    if handle.tag != expected_tag {
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

/// Marks a handle freed and drops its inner value while retaining the tagged
/// allocation as a tombstone. Used for the model-size direct fast path; the
/// owner world later deallocates the tombstone.
///
/// # Safety
/// `ptr` follows the same contract as [`free_handle`].
pub unsafe fn retire_handle<T>(ptr: *mut TypedHandle<T>, expected_tag: u32) -> bool {
    if ptr.is_null() {
        return false;
    }
    if !unregister(ptr as usize, expected_tag) {
        return false;
    }
    // SAFETY: unregister restored exclusive ownership of this live handle.
    let handle = unsafe { &mut *ptr };
    handle.tag = TAG_FREED;
    // SAFETY: inner is initialized and is dropped exactly once here. The outer
    // allocation remains as a tag-only tombstone until its world is freed.
    unsafe { std::ptr::drop_in_place(std::ptr::addr_of_mut!((*ptr).inner)) };
    true
}

/// Deallocates a previously retired tag-only handle without dropping its
/// already-dropped inner value again.
///
/// # Safety
/// `ptr` must have been successfully retired by [`retire_handle`].
pub unsafe fn deallocate_retired_handle<T>(ptr: *mut TypedHandle<T>) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: box_handle allocates this exact layout with the global allocator;
    // retire_handle has already run the inner destructor.
    unsafe {
        std::alloc::dealloc(
            ptr.cast::<u8>(),
            std::alloc::Layout::new::<TypedHandle<T>>(),
        );
    }
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
