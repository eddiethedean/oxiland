//! Well-known RDF vocabulary IRIs (curated 0.5 set).

pub mod dc;
pub mod owl;
pub mod rdf;
pub mod rdfs;
pub mod xsd;

pub(crate) fn node(iri: &str) -> oxigraph::model::NamedNode {
    oxigraph::model::NamedNode::new_unchecked(iri)
}
