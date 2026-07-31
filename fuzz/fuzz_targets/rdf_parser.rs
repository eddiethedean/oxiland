#![no_main]

use libfuzzer_sys::fuzz_target;
use oxiland::io::{Parser, Syntax};

fuzz_target!(|data: &[u8]| {
    for syntax in Syntax::all().iter().copied() {
        if let Ok(stream) = Parser::for_syntax(syntax).parse_slice(data) {
            for item in stream.take(256) {
                let _ = item;
            }
        }
    }
});
