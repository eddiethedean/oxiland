//! Temporary file name helper.

use std::ptr;

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};

#[unsafe(no_mangle)]
pub extern "C" fn librdf_files_temporary_file_name() -> *mut std::os::raw::c_char {
    abort_on_panic(|| {
        clear_last_error();
        let path = std::env::temp_dir().join(format!("oxiland-{}", std::process::id()));
        match path.to_str() {
            Some(s) => strdup_c(s),
            None => {
                set_last_error("temp path is not UTF-8");
                ptr::null_mut()
            }
        }
    })
}
