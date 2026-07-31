//! RDF vocabulary (`http://www.w3.org/1999/02/22-rdf-syntax-ns#`).

use super::node;
use oxigraph::model::NamedNode;

/// Namespace IRI.
pub const NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

/// `rdf:type`.
#[must_use]
pub fn type_() -> NamedNode {
    node("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
}

/// `rdf:Property`.
#[must_use]
pub fn property() -> NamedNode {
    node("http://www.w3.org/1999/02/22-rdf-syntax-ns#Property")
}

/// `rdf:Statement`.
#[must_use]
pub fn statement() -> NamedNode {
    node("http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement")
}

/// `rdf:subject`.
#[must_use]
pub fn subject() -> NamedNode {
    node("http://www.w3.org/1999/02/22-rdf-syntax-ns#subject")
}

/// `rdf:predicate`.
#[must_use]
pub fn predicate() -> NamedNode {
    node("http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate")
}

/// `rdf:object`.
#[must_use]
pub fn object() -> NamedNode {
    node("http://www.w3.org/1999/02/22-rdf-syntax-ns#object")
}
