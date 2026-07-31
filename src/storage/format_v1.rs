//! Shared Oxiland on-disk format v1 helpers (ADR-006 / ADR-022).

use oxigraph::io::{RdfFormat, RdfParser};
use oxigraph::model::Quad;
use oxigraph::store::Store;

use crate::{Error, Result};

/// Metadata key stored alongside durable quads.
pub(crate) const META_KEY: &str = "__oxiland/meta";
/// Current Oxiland durable format version.
pub(crate) const FORMAT_VERSION: u32 = 1;
/// Oxiland release that introduced format v1.
pub(crate) const FORMAT_OXILAND: &str = "0.4.0";

pub(crate) fn quad_key(quad: &Quad) -> String {
    format!("{quad} .")
}

pub(crate) fn parse_format_version(meta: &str) -> Result<u32> {
    // Minimal JSON parse: look for "format_version": <int>
    let key = "\"format_version\"";
    let Some(pos) = meta.find(key) else {
        return Err(Error::Storage(
            "format metadata missing format_version".into(),
        ));
    };
    let rest = &meta[pos + key.len()..];
    let rest = rest.trim_start().trim_start_matches(':').trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits
        .parse::<u32>()
        .map_err(|_| Error::Storage("format metadata has invalid format_version".into()))
}

pub(crate) fn parse_quad(key: &str) -> Result<Quad> {
    let mut parsed = RdfParser::from_format(RdfFormat::NQuads).for_reader(key.as_bytes());
    let quad = parsed
        .next()
        .ok_or_else(|| Error::Storage("persisted quad key was empty".into()))?
        .map_err(|error| Error::Storage(error.to_string()))?;
    if parsed.next().is_some() {
        return Err(Error::Storage(
            "persisted quad key contained multiple quads".into(),
        ));
    }
    Ok(quad)
}

/// RDF term equality as used by Oxigraph stores (value-equal typed literals).
pub(crate) fn quads_rdf_equal(left: &Quad, right: &Quad) -> Result<bool> {
    let probe = Store::new().map_err(|error| Error::Storage(error.to_string()))?;
    probe
        .insert(left)
        .map_err(|error| Error::Storage(error.to_string()))?;
    probe
        .contains(right.as_ref())
        .map_err(|error| Error::Storage(error.to_string()))
}

/// Returns the store's canonical quad matching `quad` under RDF equality.
pub(crate) fn stored_matching_quad(store: &Store, quad: &Quad) -> Result<Quad> {
    store
        .quads_for_pattern(
            Some(quad.subject.as_ref()),
            Some(quad.predicate.as_ref()),
            Some(quad.object.as_ref()),
            Some(quad.graph_name.as_ref()),
        )
        .next()
        .ok_or_else(|| {
            Error::Storage("matching quad missing from store after contains check".into())
        })?
        .map_err(|error| Error::Storage(error.to_string()))
}
