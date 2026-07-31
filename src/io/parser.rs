use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use oxigraph::io::{RdfParseError, RdfParser, RdfSyntaxError, ReaderQuadParser, SliceQuadParser};
use oxigraph::model::{GraphName, Quad};

use crate::io::Syntax;
use crate::io::bom::{BomStrippingReader, strip_utf8_bom_bytes, strip_utf8_bom_str};
use crate::io::location::SourceLocation;
use crate::{Error, Model, Result};

/// Configured RDF parser facade (ADR-007 / ADR-008).
///
/// Parsing always renames blank nodes so independent operations do not collide.
/// The streaming core yields fallible quads with explicit partial progress.
#[derive(Clone, Debug)]
pub struct Parser {
    syntax: Syntax,
    base_iri: Option<String>,
    graph_target: GraphTarget,
}

impl Parser {
    /// Creates a parser for an advertised [`Syntax`].
    #[must_use]
    pub fn for_syntax(syntax: Syntax) -> Self {
        Self {
            syntax,
            base_iri: None,
            graph_target: GraphTarget::DefaultGraph,
        }
    }

    /// Returns the configured syntax.
    #[must_use]
    pub fn syntax(&self) -> Syntax {
        self.syntax
    }

    /// Sets the base IRI used to resolve relative IRIs where the syntax supports it.
    pub fn base_iri(mut self, base_iri: impl Into<String>) -> Result<Self> {
        let base_iri = base_iri.into();
        // Validate early via Oxigraph so configuration fails before I/O.
        let _ = RdfParser::from_format(self.syntax.to_oxigraph())
            .with_base_iri(&base_iri)
            .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        self.base_iri = Some(base_iri);
        Ok(self)
    }

    /// Selects how triples and named graphs are mapped into quads.
    #[must_use]
    pub fn graph_target(mut self, target: GraphTarget) -> Self {
        self.graph_target = target;
        self
    }

    /// Streams quads from any [`Read`] source.
    ///
    /// A leading UTF-8 BOM is skipped when present.
    pub fn parse_reader<R: Read>(&self, reader: R) -> Result<QuadStream<BomStrippingReader<R>>> {
        Ok(QuadStream {
            inner: QuadStreamInner::Reader(
                self.build()?.for_reader(BomStrippingReader::new(reader)),
            ),
            graph_target: self.graph_target.clone(),
        })
    }

    /// Streams quads from an in-memory byte slice.
    ///
    /// A leading UTF-8 BOM is skipped when present.
    pub fn parse_slice<'a>(
        &self,
        slice: &'a (impl AsRef<[u8]> + ?Sized),
    ) -> Result<SliceStream<'a>> {
        let bytes = strip_utf8_bom_bytes(slice.as_ref());
        Ok(SliceStream {
            inner: self.build()?.for_slice(bytes),
            graph_target: self.graph_target.clone(),
        })
    }

    /// Streams quads from a UTF-8 string.
    ///
    /// A leading Unicode BOM is skipped when present.
    pub fn parse_str<'a>(&self, input: &'a str) -> Result<SliceStream<'a>> {
        self.parse_slice(strip_utf8_bom_str(input).as_bytes())
    }

    /// Streams quads from a filesystem path.
    ///
    /// The path is diagnostic context only; the caller must select [`Syntax`]
    /// explicitly unless using [`Parser::parse_path_with_extension`].
    pub fn parse_path(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<QuadStream<BomStrippingReader<BufReader<File>>>> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|error| io_with_path(error, path))?;
        self.parse_reader(BufReader::new(file))
    }

    /// Parses a path after resolving [`Syntax`] from the file extension.
    ///
    /// Dataset syntaxes (N-Quads, TriG) default to [`GraphTarget::Dataset`] so
    /// typical named-graph files parse successfully. Graph-only syntaxes keep
    /// [`GraphTarget::DefaultGraph`].
    pub fn parse_path_with_extension(
        path: impl AsRef<Path>,
    ) -> Result<(Syntax, QuadStream<BomStrippingReader<BufReader<File>>>)> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| {
                Error::Unsupported(format!(
                    "path '{}' has no file extension for syntax detection",
                    path.display()
                ))
            })?;
        let syntax = Syntax::from_extension(extension)?;
        let mut parser = Parser::for_syntax(syntax);
        if syntax.supports_datasets() {
            parser = parser.graph_target(GraphTarget::Dataset);
        }
        let stream = parser.parse_path(path)?;
        Ok((syntax, stream))
    }

    /// Inserts parsed quads into `model` progressively (ADR-007).
    ///
    /// On parse or mid-stream I/O failure, already-inserted quads remain. The
    /// returned error documents how many quads this call newly inserted before
    /// failing.
    ///
    /// Returns the number of input quads successfully processed (including
    /// duplicates that were already present).
    ///
    /// Persistent (`Model::open`) models sync each successful insert, so a
    /// partial progressive load is durable. Prefer [`Parser::load_transactional`]
    /// when atomic import is required.
    pub fn load_into(&self, model: &Model, reader: impl Read) -> Result<usize> {
        let mut processed = 0usize;
        let mut newly_inserted = 0usize;
        for item in self.parse_reader(reader)? {
            let quad = item.map_err(|error| annotate_partial_load(error, newly_inserted))?;
            if model
                .insert_quad(quad)
                .map_err(|error| annotate_partial_load(error, newly_inserted))?
            {
                newly_inserted += 1;
            }
            processed += 1;
        }
        Ok(processed)
    }

    /// Parses the complete input into memory, then inserts only after the parse
    /// succeeds.
    ///
    /// If a later insert fails, quads newly inserted by this call are removed
    /// best-effort so the model returns to its pre-call contents for this batch.
    /// The name makes the buffering cost explicit. Prefer [`Parser::parse_reader`]
    /// for large streams when partial progress is acceptable.
    pub fn load_collecting(&self, model: &Model, reader: impl Read) -> Result<usize> {
        let quads = self.parse_reader(reader)?.collect::<Result<Vec<_>>>()?;
        let total = quads.len();
        let mut newly_inserted = Vec::new();
        for quad in quads {
            match model.insert_quad(quad.clone()) {
                Ok(true) => newly_inserted.push(quad),
                Ok(false) => {}
                Err(error) => {
                    for inserted in newly_inserted.into_iter().rev() {
                        let _ = model.remove_quad(&inserted);
                    }
                    return Err(error);
                }
            }
        }
        Ok(total)
    }

    /// Progressive load from a filesystem path.
    pub fn load_path_into(&self, model: &Model, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|error| io_with_path(error, path))?;
        self.load_into(model, BufReader::new(file))
    }

    /// Collecting load from a filesystem path.
    pub fn load_path_collecting(&self, model: &Model, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|error| io_with_path(error, path))?;
        self.load_collecting(model, BufReader::new(file))
    }

    /// Parses the complete input, then inserts inside [`Model::transaction`].
    ///
    /// Mid-parse failure leaves the model unchanged. On Fjall models, durability
    /// is synced only if the transaction commits (ADR-007 / 0.4).
    pub fn load_transactional(&self, model: &Model, reader: impl Read) -> Result<usize> {
        let quads = self.parse_reader(reader)?.collect::<Result<Vec<_>>>()?;
        let total = quads.len();
        model.transaction(|tx| {
            for quad in quads {
                tx.insert_quad(quad)?;
            }
            Ok(total)
        })
    }

    /// Transactional load from a filesystem path.
    pub fn load_path_transactional(&self, model: &Model, path: impl AsRef<Path>) -> Result<usize> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|error| io_with_path(error, path))?;
        self.load_transactional(model, BufReader::new(file))
    }

    fn build(&self) -> Result<RdfParser> {
        let mut parser = RdfParser::from_format(self.syntax.to_oxigraph()).rename_blank_nodes();
        if let Some(base_iri) = &self.base_iri {
            parser = parser
                .with_base_iri(base_iri)
                .map_err(|error| Error::InvalidRdf(error.to_string()))?;
        }
        match &self.graph_target {
            GraphTarget::DefaultGraph => {
                // Reject named-graph input even for TriG/N-Quads so DefaultGraph
                // and Dataset remain distinct (D-02-04).
                parser = parser
                    .with_default_graph(GraphName::DefaultGraph)
                    .without_named_graphs();
            }
            GraphTarget::Named(graph_name) => {
                // Remap the syntax default graph; same-named input quads are
                // allowed; foreign named graphs are rejected in the stream.
                parser = parser.with_default_graph(graph_name.clone());
            }
            GraphTarget::Dataset => {
                if !self.syntax.supports_datasets() {
                    return Err(Error::Unsupported(format!(
                        "syntax '{}' does not support dataset/named-graph input; use GraphTarget::DefaultGraph or Named",
                        self.syntax.name()
                    )));
                }
            }
        }
        Ok(parser)
    }
}

/// Destination policy for parsed triples and quads (D-02-04).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphTarget {
    /// Emit default-graph quads and reject named-graph input.
    DefaultGraph,
    /// Remap the syntax default graph into this named graph/context and reject
    /// input quads that name a different graph.
    Named(GraphName),
    /// Preserve named graphs from dataset syntaxes (TriG / N-Quads).
    Dataset,
}

/// Streaming quads from a [`Read`] source.
#[must_use]
pub struct QuadStream<R: Read> {
    inner: QuadStreamInner<R>,
    graph_target: GraphTarget,
}

enum QuadStreamInner<R: Read> {
    Reader(ReaderQuadParser<R>),
}

impl<R: Read> Iterator for QuadStream<R> {
    type Item = Result<Quad>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            QuadStreamInner::Reader(parser) => parser.next().map(|item| {
                map_parse_result(item)
                    .and_then(|quad| enforce_graph_target(quad, &self.graph_target))
            }),
        }
    }
}

/// Streaming quads from an in-memory slice.
#[must_use]
pub struct SliceStream<'a> {
    inner: SliceQuadParser<'a>,
    graph_target: GraphTarget,
}

impl Iterator for SliceStream<'_> {
    type Item = Result<Quad>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| {
            map_syntax_result(item).and_then(|quad| enforce_graph_target(quad, &self.graph_target))
        })
    }
}

fn enforce_graph_target(quad: Quad, target: &GraphTarget) -> Result<Quad> {
    match target {
        GraphTarget::DefaultGraph | GraphTarget::Dataset => Ok(quad),
        GraphTarget::Named(expected) => {
            if quad.graph_name == *expected {
                Ok(quad)
            } else {
                Err(Error::parse(
                    format!(
                        "named graph '{}' is not allowed for GraphTarget::Named('{expected}')",
                        quad.graph_name
                    ),
                    None,
                ))
            }
        }
    }
}

fn map_parse_result(result: std::result::Result<Quad, RdfParseError>) -> Result<Quad> {
    result.map_err(|error| match error {
        RdfParseError::Io(error) => Error::Io(error),
        RdfParseError::Syntax(error) => map_syntax_error(error),
    })
}

fn map_syntax_result(result: std::result::Result<Quad, RdfSyntaxError>) -> Result<Quad> {
    result.map_err(map_syntax_error)
}

fn map_syntax_error(error: RdfSyntaxError) -> Error {
    let location = error.location().map(SourceLocation::from_range);
    Error::parse(strip_embedded_location(error.to_string()), location)
}

/// Oxigraph often embeds "at line N column M" in the Display text. Strip that
/// when we already expose a structured [`SourceLocation`].
fn strip_embedded_location(message: String) -> String {
    const MARKERS: &[&str] = &[
        "Parser error at line ",
        "parser error at line ",
        "Error at line ",
        "error at line ",
    ];
    for marker in MARKERS {
        if let Some(rest) = message.strip_prefix(marker) {
            if let Some((_, semantic)) = rest.split_once(": ") {
                return semantic.to_owned();
            }
        }
    }
    // Also handle messages that start with other text then " at line ".
    if let Some(idx) = message.find(" at line ") {
        if let Some((_, semantic)) = message[idx..].split_once(": ") {
            return semantic.to_owned();
        }
    }
    message
}

fn annotate_partial_load(error: Error, newly_inserted: usize) -> Error {
    if newly_inserted == 0 {
        return error;
    }
    let note = format!(
        "partial load newly inserted {newly_inserted} statement(s) that remain in the model (ADR-007 progressive load)"
    );
    match error {
        Error::Parse(mut parse) => {
            parse.message = format!("{}; {note}", parse.message);
            Error::Parse(parse)
        }
        Error::Io(io_error) => Error::Io(std::io::Error::new(
            io_error.kind(),
            format!("{io_error}; {note}"),
        )),
        Error::Storage(message) => Error::Storage(format!("{message}; {note}")),
        Error::InvalidRdf(message) => Error::InvalidRdf(format!("{message}; {note}")),
        Error::Serialize(message) => Error::Serialize(format!("{message}; {note}")),
        Error::Unsupported(message) => Error::Unsupported(format!("{message}; {note}")),
        Error::SparqlParse(message) => Error::SparqlParse(format!("{message}; {note}")),
        Error::SparqlEvaluation(message) => Error::SparqlEvaluation(format!("{message}; {note}")),
        Error::OpenStore { path, message } => Error::OpenStore {
            path,
            message: format!("{message}; {note}"),
        },
    }
}

fn io_with_path(error: std::io::Error, path: &Path) -> Error {
    Error::Io(std::io::Error::new(
        error.kind(),
        format!("{}: {}", path.display(), error),
    ))
}
