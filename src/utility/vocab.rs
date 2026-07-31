//! Well-known RDF vocabulary IRIs (curated 0.5 set).

use oxigraph::model::NamedNode;

fn node(iri: &str) -> NamedNode {
    NamedNode::new_unchecked(iri)
}

/// RDF vocabulary (`http://www.w3.org/1999/02/22-rdf-syntax-ns#`).
pub mod rdf {
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
}

/// RDFS vocabulary (`http://www.w3.org/2000/01/rdf-schema#`).
pub mod rdfs {
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
}

/// XML Schema datatypes (`http://www.w3.org/2001/XMLSchema#`).
pub mod xsd {
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
}

/// OWL vocabulary (`http://www.w3.org/2002/07/owl#`).
pub mod owl {
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
}

/// Dublin Core Terms (`http://purl.org/dc/terms/`).
pub mod dc {
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
}
