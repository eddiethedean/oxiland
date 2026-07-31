//! RDFS vocabulary (`http://www.w3.org/2000/01/rdf-schema#`).

use super::node;
use oxigraph::model::NamedNode;

/// Namespace IRI.
pub const NS: &str = "http://www.w3.org/2000/01/rdf-schema#";

/// `rdfs:label`.
#[must_use]
pub fn label() -> NamedNode {
    node("http://www.w3.org/2000/01/rdf-schema#label")
}

/// `rdfs:comment`.
#[must_use]
pub fn comment() -> NamedNode {
    node("http://www.w3.org/2000/01/rdf-schema#comment")
}

/// `rdfs:Class`.
#[must_use]
pub fn class() -> NamedNode {
    node("http://www.w3.org/2000/01/rdf-schema#Class")
}

/// `rdfs:subClassOf`.
#[must_use]
pub fn sub_class_of() -> NamedNode {
    node("http://www.w3.org/2000/01/rdf-schema#subClassOf")
}

/// `rdfs:subPropertyOf`.
#[must_use]
pub fn sub_property_of() -> NamedNode {
    node("http://www.w3.org/2000/01/rdf-schema#subPropertyOf")
}
