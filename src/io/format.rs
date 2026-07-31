use oxigraph::io::RdfFormat;

use crate::{Error, Result};

/// Closed set of RDF syntaxes advertised by Oxiland (ADR-008).
///
/// Lookup is deterministic: unknown or unsupported aliases return
/// [`Error::Unsupported`] rather than guessing.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Syntax {
    /// Turtle (`text/turtle`, `.ttl`).
    Turtle,
    /// N-Triples (`application/n-triples`, `.nt`).
    NTriples,
    /// N-Quads (`application/n-quads`, `.nq`).
    NQuads,
    /// TriG (`application/trig`, `.trig`).
    TriG,
    /// RDF/XML (`application/rdf+xml`, `.rdf`).
    RdfXml,
}

impl Syntax {
    /// Returns every syntax advertised by this build.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Turtle,
            Self::NTriples,
            Self::NQuads,
            Self::TriG,
            Self::RdfXml,
        ]
    }

    /// Canonical Redland/Raptor-style short name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Turtle => "turtle",
            Self::NTriples => "ntriples",
            Self::NQuads => "nquads",
            Self::TriG => "trig",
            Self::RdfXml => "rdfxml",
        }
    }

    /// Canonical media type without parameters.
    #[must_use]
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Turtle => "text/turtle",
            Self::NTriples => "application/n-triples",
            Self::NQuads => "application/n-quads",
            Self::TriG => "application/trig",
            Self::RdfXml => "application/rdf+xml",
        }
    }

    /// Canonical file extension without a leading dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Turtle => "ttl",
            Self::NTriples => "nt",
            Self::NQuads => "nq",
            Self::TriG => "trig",
            Self::RdfXml => "rdf",
        }
    }

    /// Whether the syntax can represent named graphs.
    #[must_use]
    pub const fn supports_datasets(self) -> bool {
        matches!(self, Self::NQuads | Self::TriG)
    }

    /// Whether this build can parse the syntax.
    #[must_use]
    pub const fn can_parse(self) -> bool {
        true
    }

    /// Whether this build can serialize the syntax.
    #[must_use]
    pub const fn can_serialize(self) -> bool {
        true
    }

    /// Resolves a Redland-style syntax name or common alias.
    ///
    /// Note: the name alias `"xml"` resolves to RDF/XML, but
    /// [`Syntax::from_extension`] rejects `".xml"` as ambiguous. Prefer
    /// `"rdfxml"` / `".rdf"` when both lookup styles must agree.
    pub fn from_name(name: &str) -> Result<Self> {
        let normalized = normalize_token(name);
        match normalized.as_str() {
            "turtle" | "ttl" => Ok(Self::Turtle),
            "ntriples" | "n-triples" | "nt" => Ok(Self::NTriples),
            "nquads" | "n-quads" | "nq" => Ok(Self::NQuads),
            "trig" => Ok(Self::TriG),
            "rdfxml" | "rdf-xml" | "rdf/xml" | "xml" => Ok(Self::RdfXml),
            "n3" | "notation3" => Err(Error::Unsupported(
                "syntax 'n3' is not advertised by Oxiland; use turtle where N3-compatible input applies"
                    .into(),
            )),
            "jsonld" | "json-ld" | "ld+json" => Err(Error::Unsupported(
                "syntax 'jsonld' is deferred; Oxiland does not advertise JSON-LD".into(),
            )),
            "guess" | "auto" => Err(Error::Unsupported(
                "automatic syntax detection by content sniffing is unsupported; select an explicit Syntax"
                    .into(),
            )),
            other => Err(Error::Unsupported(format!(
                "unknown RDF syntax name '{other}'"
            ))),
        }
    }

    /// Resolves a media type, ignoring case and accepting `charset=utf-8` /
    /// `charset=ascii` parameters.
    pub fn from_media_type(media_type: &str) -> Result<Self> {
        let (base, params) = split_media_type(media_type);
        for param in params {
            if let Some((key, value)) = param.split_once('=') {
                if normalize_token(key) == "charset" {
                    let charset = normalize_token(value.trim_matches('"'));
                    if charset != "utf-8" && charset != "utf8" && charset != "ascii" {
                        return Err(Error::Unsupported(format!(
                            "unsupported charset '{value}' in media type '{media_type}'"
                        )));
                    }
                }
            }
        }

        match base.as_str() {
            "text/turtle" | "application/x-turtle" | "application/turtle" => Ok(Self::Turtle),
            "application/n-triples" | "application/x-ntriples" => Ok(Self::NTriples),
            "application/n-quads" | "text/x-nquads" | "application/x-nquads" => Ok(Self::NQuads),
            "application/trig" | "application/x-trig" => Ok(Self::TriG),
            "application/rdf+xml" => Ok(Self::RdfXml),
            "text/plain" => Err(Error::Unsupported(
                "media type 'text/plain' is ambiguous; use application/n-triples or an explicit Syntax"
                    .into(),
            )),
            "application/xml" | "text/xml" => Err(Error::Unsupported(
                "media type is ambiguous XML; use application/rdf+xml or an explicit Syntax".into(),
            )),
            "text/n3" | "text/rdf+n3" | "application/n3" => Err(Error::Unsupported(
                "media type maps to N3, which is not advertised by Oxiland".into(),
            )),
            "application/ld+json" | "application/jsonld" | "application/json" => {
                Err(Error::Unsupported(
                    "media type maps to JSON-LD, which is not advertised by Oxiland".into(),
                ))
            }
            other => Err(Error::Unsupported(format!(
                "unknown RDF media type '{other}'"
            ))),
        }
    }

    /// Resolves a file extension without requiring a leading dot.
    ///
    /// Ambiguous extensions such as `".xml"` and `".txt"` return
    /// [`Error::Unsupported`]. The Redland name alias `"xml"` is accepted by
    /// [`Syntax::from_name`] but not by this method.
    pub fn from_extension(extension: &str) -> Result<Self> {
        let normalized = normalize_token(extension.trim_start_matches('.'));
        match normalized.as_str() {
            "ttl" => Ok(Self::Turtle),
            "nt" => Ok(Self::NTriples),
            "nq" => Ok(Self::NQuads),
            "trig" => Ok(Self::TriG),
            "rdf" | "owl" => Ok(Self::RdfXml),
            "txt" => Err(Error::Unsupported(
                "extension '.txt' is ambiguous; use '.nt' or an explicit Syntax".into(),
            )),
            "xml" => Err(Error::Unsupported(
                "extension '.xml' is ambiguous; use '.rdf' or an explicit Syntax".into(),
            )),
            "n3" => Err(Error::Unsupported(
                "extension '.n3' maps to N3, which is not advertised by Oxiland".into(),
            )),
            "jsonld" | "json" => Err(Error::Unsupported(
                "extension maps to JSON-LD, which is not advertised by Oxiland".into(),
            )),
            other => Err(Error::Unsupported(format!(
                "unknown RDF file extension '{other}'"
            ))),
        }
    }

    pub(crate) fn to_oxigraph(self) -> RdfFormat {
        match self {
            Self::Turtle => RdfFormat::Turtle,
            Self::NTriples => RdfFormat::NTriples,
            Self::NQuads => RdfFormat::NQuads,
            Self::TriG => RdfFormat::TriG,
            Self::RdfXml => RdfFormat::RdfXml,
        }
    }
}

fn normalize_token(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn split_media_type(media_type: &str) -> (String, Vec<&str>) {
    let mut parts = media_type.split(';');
    let base = normalize_token(parts.next().unwrap_or_default());
    let params = parts
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    (base, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_aliases_resolve_deterministically() {
        assert_eq!(Syntax::from_name("Turtle").unwrap(), Syntax::Turtle);
        assert_eq!(Syntax::from_name("n-triples").unwrap(), Syntax::NTriples);
        assert_eq!(Syntax::from_name("RDF/XML").unwrap(), Syntax::RdfXml);
        assert!(matches!(
            Syntax::from_name("jsonld"),
            Err(Error::Unsupported(_))
        ));
        assert!(matches!(
            Syntax::from_name("guess"),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn media_types_accept_utf8_charset() {
        assert_eq!(
            Syntax::from_media_type("text/turtle; charset=utf-8").unwrap(),
            Syntax::Turtle
        );
        assert!(matches!(
            Syntax::from_media_type("text/turtle; charset=iso-8859-1"),
            Err(Error::Unsupported(_))
        ));
    }

    #[test]
    fn extensions_cover_advertised_syntaxes() {
        for syntax in Syntax::all() {
            assert_eq!(Syntax::from_extension(syntax.extension()).unwrap(), *syntax);
            assert!(syntax.can_parse());
            assert!(syntax.can_serialize());
        }
    }
}
