//! Unicode normalization helpers.

use unicode_normalization::UnicodeNormalization;

/// Returns the NFC normalization of `text`.
#[must_use]
pub fn normalize_nfc(text: &str) -> String {
    text.nfc().collect()
}

/// Returns the NFKC normalization of `text`.
#[must_use]
pub fn normalize_nfkc(text: &str) -> String {
    text.nfkc().collect()
}
