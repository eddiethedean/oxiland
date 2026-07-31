#![no_main]

use std::ffi::CString;

use libfuzzer_sys::fuzz_target;
use oxiland_capi::{
    librdf_free_node, librdf_free_uri, librdf_free_world, librdf_new_node_from_uri_string,
    librdf_new_uri, librdf_new_world, librdf_world_open,
};

fuzz_target!(|data: &[u8]| {
    let Ok(value) = CString::new(data) else {
        return;
    };
    let world = librdf_new_world();
    if world.is_null() {
        return;
    }
    librdf_world_open(world);
    let uri = librdf_new_uri(world, value.as_ptr().cast());
    let node = librdf_new_node_from_uri_string(world, value.as_ptr().cast());
    librdf_free_node(node);
    librdf_free_uri(uri);
    librdf_free_world(world);
});
