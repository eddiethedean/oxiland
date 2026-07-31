//! UTF-8 BOM stripping for RDF readers.

use std::io::{self, Read};

const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// Reader that skips a leading UTF-8 BOM when present.
///
/// Returned as the inner reader type from [`crate::io::Parser::parse_reader`] /
/// [`crate::io::Parser::parse_path`]. BOM detection is deferred until the first
/// [`Read::read`] so constructing the wrapper does not perform I/O (and does
/// not change when I/O errors surface).
pub struct BomStrippingReader<R> {
    inner: R,
    /// `None` = not yet checked; `Some(offset)` = leftover probe bytes to emit.
    pending: Option<Pending>,
}

enum Pending {
    /// Still need to probe for a BOM.
    Check,
    /// Emit `buf[offset..len]` then resume reading `inner`.
    Prefix {
        buf: [u8; 3],
        len: usize,
        offset: usize,
    },
}

impl<R: Read> BomStrippingReader<R> {
    /// Wraps `reader`. No I/O is performed until the first read.
    #[must_use]
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            pending: Some(Pending::Check),
        }
    }
}

impl<R: Read> Read for BomStrippingReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        loop {
            match self.pending.take() {
                None => return self.inner.read(out),
                Some(Pending::Check) => {
                    let mut probe = [0u8; 3];
                    let mut filled = 0usize;
                    while filled < 3 {
                        match self.inner.read(&mut probe[filled..])? {
                            0 => break,
                            n => filled += n,
                        }
                    }
                    if filled == 3 && probe == UTF8_BOM {
                        // BOM consumed; continue with inner.
                        self.pending = None;
                        continue;
                    }
                    self.pending = Some(Pending::Prefix {
                        buf: probe,
                        len: filled,
                        offset: 0,
                    });
                }
                Some(Pending::Prefix {
                    buf,
                    len,
                    mut offset,
                }) => {
                    if offset >= len {
                        self.pending = None;
                        continue;
                    }
                    let available = len - offset;
                    let n = available.min(out.len());
                    out[..n].copy_from_slice(&buf[offset..offset + n]);
                    offset += n;
                    if offset < len {
                        self.pending = Some(Pending::Prefix { buf, len, offset });
                    } else {
                        self.pending = None;
                    }
                    return Ok(n);
                }
            }
        }
    }
}

/// Strips a leading Unicode BOM from a string slice.
#[must_use]
pub(crate) fn strip_utf8_bom_str(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// Strips a leading UTF-8 BOM from a byte slice.
#[must_use]
pub(crate) fn strip_utf8_bom_bytes(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(&UTF8_BOM).unwrap_or(bytes)
}
