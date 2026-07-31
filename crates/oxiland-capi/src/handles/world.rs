//! `librdf_world` handle.

use oxiland::World;

use crate::error::{abort_on_panic, clear_last_error};
use crate::handles::{TAG_WORLD, TypedHandle, box_handle, free_handle};

pub type librdf_world = TypedHandle<WorldInner>;

pub struct WorldInner {
    pub world: World,
    pub opened: bool,
}

/// Creates a new world handle.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_world() -> *mut librdf_world {
    abort_on_panic(|| {
        clear_last_error();
        box_handle(
            TAG_WORLD,
            WorldInner {
                world: World::new(),
                opened: false,
            },
        )
    })
}

/// Frees a world. Null is a no-op.
#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_world(world: *mut librdf_world) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle from librdf_new_world.
        unsafe { free_handle(world, TAG_WORLD) };
    });
}

/// Opens the world (preview: marks opened; construction already initializes).
#[unsafe(no_mangle)]
pub extern "C" fn librdf_world_open(world: *mut librdf_world) {
    abort_on_panic(|| {
        clear_last_error();
        // SAFETY: world is null or a live world handle.
        let Some(handle) = (unsafe { crate::handles::borrow_handle(world, TAG_WORLD) }) else {
            return;
        };
        handle.inner.opened = true;
    });
}
