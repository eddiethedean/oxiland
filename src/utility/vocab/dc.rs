//! Dublin Core Terms (`http://purl.org/dc/terms/`).

use super::node;
use oxigraph::model::NamedNode;

/// Namespace IRI.
pub const NS: &str = "http://purl.org/dc/terms/";

/// `dcterms:title`.
#[must_use]
pub fn title() -> NamedNode {
    node("http://purl.org/dc/terms/title")
}

/// `dcterms:creator`.
#[must_use]
pub fn creator() -> NamedNode {
    node("http://purl.org/dc/terms/creator")
}

/// `dcterms:description`.
#[must_use]
pub fn description() -> NamedNode {
    node("http://purl.org/dc/terms/description")
}
