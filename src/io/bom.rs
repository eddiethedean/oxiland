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
                        match self.inner.read(&mut probe[filled..]) {
                            Ok(0) => break,
                            Ok(n) => filled += n,
                            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                            Err(error) => {
                                // Preserve probe progress so a retry does not drop bytes.
                                if filled > 0 {
                                    self.pending = Some(Pending::Prefix {
                                        buf: probe,
                                        len: filled,
                                        offset: 0,
                                    });
                                } else {
                                    self.pending = Some(Pending::Check);
                                }
                                return Err(error);
                            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, ErrorKind, Read};

    /// Yields one byte, then `Interrupted`, then the rest of the payload.
    struct InterruptAfterOne {
        data: Vec<u8>,
        pos: usize,
        interrupted: bool,
    }

    impl Read for InterruptAfterOne {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if buf.is_empty() {
                return Ok(0);
            }
            if self.pos == 1 && !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::from(ErrorKind::Interrupted));
            }
            if self.pos >= self.data.len() {
                return Ok(0);
            }
            buf[0] = self.data[self.pos];
            self.pos += 1;
            Ok(1)
        }
    }

    #[test]
    fn bom_probe_preserves_bytes_across_interrupted_read() {
        let mut payload = Vec::from(UTF8_BOM);
        payload.extend_from_slice(b"hello");
        let mut reader = BomStrippingReader::new(InterruptAfterOne {
            data: payload,
            pos: 0,
            interrupted: false,
        });
        let mut out = Vec::new();
        loop {
            let mut chunk = [0u8; 8];
            match reader.read(&mut chunk) {
                Ok(0) => break,
                Ok(n) => out.extend_from_slice(&chunk[..n]),
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => panic!("unexpected read error: {error}"),
            }
        }
        assert_eq!(out, b"hello");
    }

    #[test]
    fn bom_probe_preserves_partial_prefix_on_other_errors() {
        struct FailAfterOne {
            data: Vec<u8>,
            pos: usize,
        }
        impl Read for FailAfterOne {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                if self.pos >= 1 {
                    return Err(io::Error::from(ErrorKind::Other));
                }
                if buf.is_empty() || self.pos >= self.data.len() {
                    return Ok(0);
                }
                buf[0] = self.data[self.pos];
                self.pos += 1;
                Ok(1)
            }
        }
        let mut reader = BomStrippingReader::new(FailAfterOne {
            data: b"xyz".to_vec(),
            pos: 0,
        });
        let mut buf = [0u8; 4];
        assert_eq!(reader.read(&mut buf).unwrap_err().kind(), ErrorKind::Other);
        // First byte was probed; after the error it must still be readable.
        assert_eq!(reader.read(&mut buf).unwrap(), 1);
        assert_eq!(buf[0], b'x');
    }

    #[test]
    fn cursor_reader_strips_bom() {
        let mut reader = BomStrippingReader::new(Cursor::new(b"\xEF\xBB\xBFhi"));
        let mut out = String::new();
        reader.read_to_string(&mut out).unwrap();
        assert_eq!(out, "hi");
    }
}
