//! XML Schema datatypes (`http://www.w3.org/2001/XMLSchema#`).

use super::node;
use oxigraph::model::NamedNode;

/// Namespace IRI.
pub const NS: &str = "http://www.w3.org/2001/XMLSchema#";

/// `xsd:string`.
#[must_use]
pub fn string() -> NamedNode {
    node("http://www.w3.org/2001/XMLSchema#string")
}

/// `xsd:boolean`.
#[must_use]
pub fn boolean() -> NamedNode {
    node("http://www.w3.org/2001/XMLSchema#boolean")
}

/// `xsd:integer`.
#[must_use]
pub fn integer() -> NamedNode {
    node("http://www.w3.org/2001/XMLSchema#integer")
}

/// `xsd:decimal`.
#[must_use]
pub fn decimal() -> NamedNode {
    node("http://www.w3.org/2001/XMLSchema#decimal")
}

/// `xsd:dateTime`.
#[must_use]
pub fn date_time() -> NamedNode {
    node("http://www.w3.org/2001/XMLSchema#dateTime")
}
