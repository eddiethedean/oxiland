//! `librdf_hash` — opaque string-keyed HashMap.

use std::collections::HashMap;
use std::os::raw::c_char;
use std::ptr;

use crate::alloc::strdup_c;
use crate::error::{abort_on_panic, clear_last_error, set_last_error};
use crate::handles::io::{FILE, writeln_file};
use crate::handles::world::librdf_world;
use crate::handles::{
    TAG_HASH, TAG_WORLD, TypedHandle, borrow_handle, box_handle, cstr_optional, cstr_required,
    free_handle,
};

pub type librdf_hash = TypedHandle<HashInner>;

pub struct HashInner {
    pub map: HashMap<String, Vec<String>>,
}

fn parse_hash_string(map: &mut HashMap<String, Vec<String>>, string: &str) -> Result<(), String> {
    // Format: key1='value1', key2='value2'
    let mut rest = string.trim();
    while !rest.is_empty() {
        rest = rest.trim_start();
        let eq = rest
            .find('=')
            .ok_or_else(|| "hash string missing '='".to_string())?;
        let key = rest[..eq].trim().to_string();
        rest = rest[eq + 1..].trim_start();
        if !rest.starts_with('\'') {
            return Err("hash value must be single-quoted".into());
        }
        rest = &rest[1..];
        let mut value = String::new();
        let mut chars = rest.chars();
        while let Some(ch) = chars.next() {
            if ch == '\\' {
                if let Some(next) = chars.next() {
                    value.push(next);
                }
            } else if ch == '\'' {
                break;
            } else {
                value.push(ch);
            }
        }
        rest = chars.as_str().trim_start();
        if rest.starts_with(',') {
            rest = rest[1..].trim_start();
        }
        map.entry(key).or_default().push(value);
    }
    Ok(())
}

fn format_hash(map: &HashMap<String, Vec<String>>, filter: Option<&[*const c_char]>) -> String {
    let mut parts = Vec::new();
    for (key, values) in map {
        if let Some(filter) = filter {
            let skip = filter.iter().any(|p| {
                if p.is_null() {
                    false
                } else {
                    unsafe { std::ffi::CStr::from_ptr(*p) }
                        .to_str()
                        .ok()
                        .is_some_and(|s| s == key)
                }
            });
            if skip {
                continue;
            }
        }
        for value in values {
            let escaped = value.replace('\\', "\\\\").replace('\'', "\\'");
            parts.push(format!("{key}='{escaped}'"));
        }
    }
    parts.join(",")
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_hash(
    world: *mut librdf_world,
    _name: *const c_char,
) -> *mut librdf_hash {
    abort_on_panic(|| {
        clear_last_error();
        if unsafe { borrow_handle(world, TAG_WORLD) }.is_none() {
            return ptr::null_mut();
        }
        box_handle(
            TAG_HASH,
            HashInner {
                map: HashMap::new(),
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_hash_from_string(
    world: *mut librdf_world,
    name: *const c_char,
    string: *const c_char,
) -> *mut librdf_hash {
    abort_on_panic(|| {
        let hash = librdf_new_hash(world, name);
        if hash.is_null() {
            return ptr::null_mut();
        }
        if librdf_hash_from_string(hash, string) != 0 {
            librdf_free_hash(hash);
            return ptr::null_mut();
        }
        hash
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_hash_from_array_of_strings(
    world: *mut librdf_world,
    name: *const c_char,
    array: *const *const c_char,
) -> *mut librdf_hash {
    abort_on_panic(|| {
        clear_last_error();
        let hash = librdf_new_hash(world, name);
        if hash.is_null() {
            return ptr::null_mut();
        }
        if array.is_null() {
            return hash;
        }
        let Some(h) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return ptr::null_mut();
        };
        let mut i = 0usize;
        loop {
            let key_ptr = unsafe { *array.add(i) };
            if key_ptr.is_null() {
                break;
            }
            let val_ptr = unsafe { *array.add(i + 1) };
            let Some(key) = (unsafe { cstr_required(key_ptr, "key") }) else {
                return ptr::null_mut();
            };
            let value = if val_ptr.is_null() {
                String::new()
            } else {
                match unsafe { cstr_required(val_ptr, "value") } {
                    Some(v) => v.to_owned(),
                    None => return ptr::null_mut(),
                }
            };
            h.inner.map.entry(key.to_owned()).or_default().push(value);
            i += 2;
        }
        hash
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_new_hash_from_hash(old_hash: *mut librdf_hash) -> *mut librdf_hash {
    abort_on_panic(|| {
        clear_last_error();
        let Some(old) = (unsafe { borrow_handle(old_hash, TAG_HASH) }) else {
            return ptr::null_mut();
        };
        box_handle(
            TAG_HASH,
            HashInner {
                map: old.inner.map.clone(),
            },
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_free_hash(hash: *mut librdf_hash) {
    abort_on_panic(|| {
        clear_last_error();
        unsafe { free_handle(hash, TAG_HASH) };
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_get(hash: *mut librdf_hash, key: *const c_char) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return ptr::null_mut();
        };
        let Some(key) = (unsafe { cstr_required(key, "key") }) else {
            return ptr::null_mut();
        };
        match hash.inner.map.get(key).and_then(|v| v.first()) {
            Some(v) => strdup_c(v),
            None => ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_get_as_boolean(hash: *mut librdf_hash, key: *const c_char) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return -1;
        };
        let Some(key) = (unsafe { cstr_required(key, "key") }) else {
            return -1;
        };
        let Some(v) = hash.inner.map.get(key).and_then(|v| v.first()) else {
            return -1;
        };
        match v.to_ascii_lowercase().as_str() {
            "yes" | "true" | "1" | "on" => 1,
            "no" | "false" | "0" | "off" => 0,
            _ => -1,
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_get_as_long(
    hash: *mut librdf_hash,
    key: *const c_char,
) -> libc::c_long {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return -1;
        };
        let Some(key) = (unsafe { cstr_required(key, "key") }) else {
            return -1;
        };
        let Some(v) = hash.inner.map.get(key).and_then(|v| v.first()) else {
            return -1;
        };
        v.parse::<libc::c_long>().unwrap_or(-1)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_get_del(hash: *mut librdf_hash, key: *const c_char) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return ptr::null_mut();
        };
        let Some(key) = (unsafe { cstr_required(key, "key") }) else {
            return ptr::null_mut();
        };
        match hash
            .inner
            .map
            .remove(key)
            .and_then(|mut v| v.drain(..).next())
        {
            Some(v) => strdup_c(&v),
            None => ptr::null_mut(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_put_strings(
    hash: *mut librdf_hash,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return -1;
        };
        let Some(key) = (unsafe { cstr_required(key, "key") }) else {
            return -1;
        };
        let Some(value) = (unsafe { cstr_required(value, "value") }) else {
            return -1;
        };
        hash.inner
            .map
            .entry(key.to_owned())
            .or_default()
            .push(value.to_owned());
        0
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_print(hash: *mut librdf_hash, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return;
        };
        let text = format_hash(&hash.inner.map, None);
        let _ = writeln_file(fh, &text);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_print_keys(hash: *mut librdf_hash, fh: *mut FILE) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return;
        };
        for key in hash.inner.map.keys() {
            let _ = writeln_file(fh, key);
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_print_values(
    hash: *mut librdf_hash,
    key_string: *const c_char,
    fh: *mut FILE,
) {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return;
        };
        let Some(key) = (unsafe { cstr_required(key_string, "key_string") }) else {
            return;
        };
        if let Some(values) = hash.inner.map.get(key) {
            for value in values {
                let _ = writeln_file(fh, value);
            }
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_interpret_template(
    template_string: *const u8,
    dictionary: *mut librdf_hash,
    prefix: *const u8,
    suffix: *const u8,
) -> *mut u8 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(template) = (unsafe { cstr_required(template_string.cast(), "template_string") })
        else {
            return ptr::null_mut();
        };
        let Some(dict) = (unsafe { borrow_handle(dictionary, TAG_HASH) }) else {
            return ptr::null_mut();
        };
        let prefix = unsafe { cstr_optional(prefix.cast(), "prefix") }
            .ok()
            .flatten()
            .unwrap_or("${");
        let suffix = unsafe { cstr_optional(suffix.cast(), "suffix") }
            .ok()
            .flatten()
            .unwrap_or("}");
        let mut out = String::new();
        let mut rest = template;
        while let Some(start) = rest.find(prefix) {
            out.push_str(&rest[..start]);
            rest = &rest[start + prefix.len()..];
            if let Some(end) = rest.find(suffix) {
                let key = &rest[..end];
                if let Some(v) = dict.inner.map.get(key).and_then(|v| v.first()) {
                    out.push_str(v);
                }
                rest = &rest[end + suffix.len()..];
            } else {
                out.push_str(prefix);
                break;
            }
        }
        out.push_str(rest);
        strdup_c(&out).cast()
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_from_string(hash: *mut librdf_hash, string: *const c_char) -> i32 {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return -1;
        };
        let Some(string) = (unsafe { cstr_required(string, "string") }) else {
            return -1;
        };
        match parse_hash_string(&mut hash.inner.map, string) {
            Ok(()) => 0,
            Err(e) => {
                set_last_error(e);
                -1
            }
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn librdf_hash_to_string(
    hash: *mut librdf_hash,
    filter: *const *const c_char,
) -> *mut c_char {
    abort_on_panic(|| {
        clear_last_error();
        let Some(hash) = (unsafe { borrow_handle(hash, TAG_HASH) }) else {
            return ptr::null_mut();
        };
        let filter_slice = if filter.is_null() {
            None
        } else {
            let mut items = Vec::new();
            let mut i = 0usize;
            loop {
                let p = unsafe { *filter.add(i) };
                if p.is_null() {
                    break;
                }
                items.push(p);
                i += 1;
            }
            Some(items)
        };
        let text = format_hash(&hash.inner.map, filter_slice.as_deref());
        strdup_c(&text)
    })
}
