//! Prefix ↔ base IRI namespace helper.

use crate::Result;
use crate::terms::{self, NamedNode};

/// A namespace prefix bound to a base IRI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Namespace {
    prefix: String,
    base: NamedNode,
}

impl Namespace {
    /// Creates a namespace from a prefix token and base IRI.
    pub fn new(prefix: impl Into<String>, base_iri: impl AsRef<str>) -> Result<Self> {
        let prefix = prefix.into();
        if prefix.is_empty()
            || !prefix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(crate::Error::InvalidRdf(format!(
                "namespace prefix '{prefix}' is invalid"
            )));
        }
        let base = terms::named_node(base_iri.as_ref())?;
        Ok(Self { prefix, base })
    }

    /// Returns the prefix token.
    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    /// Returns the base IRI.
    #[must_use]
    pub fn base(&self) -> &NamedNode {
        &self.base
    }

    /// Expands a local name under this namespace (`prefix:local` or bare local).
    pub fn expand(&self, local: &str) -> Result<NamedNode> {
        let local = local
            .strip_prefix(&format!("{}:", self.prefix))
            .unwrap_or(local);
        if local.is_empty() || local.contains(':') {
            return Err(crate::Error::InvalidRdf(format!(
                "local name '{local}' is invalid for namespace '{}'",
                self.prefix
            )));
        }
        terms::named_node(format!("{}{local}", self.base.as_str()))
    }
}
