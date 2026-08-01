use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use oxigraph::io::RdfSerializer;
use oxigraph::model::{GraphNameRef, Quad, QuadRef, Triple, TripleRef};

use crate::io::Syntax;
use crate::world::{FeatureMap, FeatureValue};
use crate::{Error, Model, Result, StatementPattern};

/// Configured RDF serializer facade.
///
/// Prefixes are accumulated on the builder and applied when serialization
/// begins. Prefixes are rejected for formats that do not support them.
#[derive(Clone, Debug)]
pub struct Serializer {
    syntax: Syntax,
    prefixes: Vec<(String, String)>,
    base_iri: Option<String>,
    features: FeatureMap,
}

impl Serializer {
    /// Creates a serializer for an advertised [`Syntax`].
    #[must_use]
    pub fn for_syntax(syntax: Syntax) -> Self {
        Self {
            syntax,
            prefixes: Vec::new(),
            base_iri: None,
            features: FeatureMap::new(),
        }
    }

    /// Sets a serializer feature (Redland `librdf_serializer_set_feature`).
    pub fn set_feature(&self, iri: impl Into<String>, value: FeatureValue) {
        self.features.set(iri, value);
    }

    /// Returns a serializer feature when set.
    #[must_use]
    pub fn feature(&self, iri: &str) -> Option<FeatureValue> {
        self.features.get(iri)
    }

    /// Returns the configured syntax.
    #[must_use]
    pub fn syntax(&self) -> Syntax {
        self.syntax
    }

    /// Adds a namespace prefix for syntaxes that support prefix declarations.
    pub fn with_prefix(
        mut self,
        prefix_name: impl Into<String>,
        prefix_iri: impl Into<String>,
    ) -> Result<Self> {
        if !matches!(self.syntax, Syntax::Turtle | Syntax::TriG | Syntax::RdfXml) {
            return Err(Error::Unsupported(format!(
                "syntax '{}' does not support namespace prefixes",
                self.syntax.name()
            )));
        }
        let prefix_name = prefix_name.into();
        let prefix_iri = prefix_iri.into();
        let _ = RdfSerializer::from_format(self.syntax.to_oxigraph())
            .with_prefix(&prefix_name, &prefix_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.prefixes.push((prefix_name, prefix_iri));
        Ok(self)
    }

    /// Sets a base IRI for syntaxes that emit relative IRIs.
    pub fn base_iri(mut self, base_iri: impl Into<String>) -> Result<Self> {
        if !matches!(self.syntax, Syntax::Turtle | Syntax::TriG | Syntax::RdfXml) {
            return Err(Error::Unsupported(format!(
                "syntax '{}' does not support serializer base IRIs",
                self.syntax.name()
            )));
        }
        let base_iri = base_iri.into();
        let _ = RdfSerializer::from_format(self.syntax.to_oxigraph())
            .with_base_iri(&base_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.base_iri = Some(base_iri);
        Ok(self)
    }

    /// Serializes quads to a [`Write`] destination.
    pub fn serialize_quads_to_writer<W: Write>(
        &self,
        writer: W,
        quads: impl IntoIterator<Item = Quad>,
    ) -> Result<W> {
        let mut serializer = self.build()?.for_writer(writer);
        for quad in quads {
            self.ensure_quad_compatible(&quad)?;
            serializer
                .serialize_quad(QuadRef::from(&quad))
                .map_err(map_write_error)?;
        }
        serializer.finish().map_err(map_write_error)
    }

    /// Serializes triples as default-graph statements.
    pub fn serialize_triples_to_writer<W: Write>(
        &self,
        writer: W,
        triples: impl IntoIterator<Item = Triple>,
    ) -> Result<W> {
        self.serialize_triples_fallible_to_writer(writer, triples.into_iter().map(Ok))
    }

    /// Serializes triples from a fallible iterator without buffering the full set.
    pub(crate) fn serialize_triples_fallible_to_writer<W: Write>(
        &self,
        writer: W,
        triples: impl IntoIterator<Item = Result<Triple>>,
    ) -> Result<W> {
        let mut serializer = self.build()?.for_writer(writer);
        for triple in triples {
            let triple = triple?;
            serializer
                .serialize_triple(TripleRef::from(&triple))
                .map_err(map_write_error)?;
        }
        serializer.finish().map_err(map_write_error)
    }

    /// Serializes an entire model by streaming its statement iterator.
    ///
    /// Dataset syntaxes preserve named graphs. Graph-only syntaxes require that
    /// every statement lives in the default graph.
    pub fn serialize_model_to_writer<W: Write>(&self, writer: W, model: &Model) -> Result<W> {
        let mut serializer = self.build()?.for_writer(writer);
        for item in model.find(StatementPattern::default()) {
            let quad = item?;
            self.ensure_quad_compatible(&quad)?;
            serializer
                .serialize_quad(QuadRef::from(&quad))
                .map_err(map_write_error)?;
        }
        serializer.finish().map_err(map_write_error)
    }

    /// Serializes quads into a UTF-8 string.
    pub fn serialize_quads_to_string(
        &self,
        quads: impl IntoIterator<Item = Quad>,
    ) -> Result<String> {
        let bytes = self.serialize_quads_to_writer(Vec::new(), quads)?;
        String::from_utf8(bytes).map_err(|error| Error::Serialize(error.to_string()))
    }

    /// Serializes a model into a UTF-8 string.
    pub fn serialize_model_to_string(&self, model: &Model) -> Result<String> {
        let bytes = self.serialize_model_to_writer(Vec::new(), model)?;
        String::from_utf8(bytes).map_err(|error| Error::Serialize(error.to_string()))
    }

    /// Serializes a model to a filesystem path.
    pub fn serialize_model_to_path(&self, model: &Model, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        let file = File::create(path).map_err(|error| {
            Error::Io(std::io::Error::new(
                error.kind(),
                format!("{}: {}", path.display(), error),
            ))
        })?;
        let writer = self.serialize_model_to_writer(BufWriter::new(file), model)?;
        writer
            .into_inner()
            .map_err(|error| map_write_error(error.into_error()))?;
        Ok(())
    }

    fn build(&self) -> Result<RdfSerializer> {
        let mut serializer = RdfSerializer::from_format(self.syntax.to_oxigraph());
        for (name, iri) in &self.prefixes {
            serializer = serializer
                .with_prefix(name, iri)
                .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        }
        if let Some(base_iri) = &self.base_iri {
            serializer = serializer
                .with_base_iri(base_iri)
                .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        }
        Ok(serializer)
    }

    fn ensure_quad_compatible(&self, quad: &Quad) -> Result<()> {
        if self.syntax.supports_datasets() {
            return Ok(());
        }
        if !matches!(quad.graph_name.as_ref(), GraphNameRef::DefaultGraph) {
            return Err(Error::Unsupported(format!(
                "syntax '{}' cannot serialize named-graph statements; use nquads or trig",
                self.syntax.name()
            )));
        }
        Ok(())
    }
}

fn map_write_error(error: std::io::Error) -> Error {
    if error.kind() == std::io::ErrorKind::InvalidInput {
        Error::Serialize(error.to_string())
    } else {
        Error::Io(error)
    }
}
