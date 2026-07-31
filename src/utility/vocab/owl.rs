//! OWL vocabulary (`http://www.w3.org/2002/07/owl#`).

use super::node;
use oxigraph::model::NamedNode;

/// Namespace IRI.
pub const NS: &str = "http://www.w3.org/2002/07/owl#";

/// `owl:Class`.
#[must_use]
pub fn class() -> NamedNode {
    node("http://www.w3.org/2002/07/owl#Class")
}

/// `owl:Ontology`.
#[must_use]
pub fn ontology() -> NamedNode {
    node("http://www.w3.org/2002/07/owl#Ontology")
}

/// `owl:sameAs`.
#[must_use]
pub fn same_as() -> NamedNode {
    node("http://www.w3.org/2002/07/owl#sameAs")
}
