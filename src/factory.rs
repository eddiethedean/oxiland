//! Baseline factory registration for Redland workflow parity (ADR-025).
//!
//! Registration covers the closed set of built-in parser, serializer, storage,
//! and query factories present in the pinned Redland baseline profiles.
//! Re-registering a built-in name is idempotent. Names outside that set return
//! [`Error::Unsupported`]. Third-party plugins absent from baseline profiles
//! remain outside the denominator.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

use crate::io::Syntax;
use crate::storage::supported_backends;
use crate::{Error, Result};

#[derive(Debug)]
struct FactoryRegistry {
    parsers: HashSet<String>,
    serializers: HashSet<String>,
    storages: HashSet<String>,
    queries: HashSet<String>,
}

impl FactoryRegistry {
    fn with_builtins() -> Self {
        let mut parsers = HashSet::new();
        let mut serializers = HashSet::new();
        for syntax in Syntax::all() {
            parsers.insert(syntax.name().to_owned());
            serializers.insert(syntax.name().to_owned());
        }
        let storages = supported_backends()
            .map(|descriptor| descriptor.name.to_owned())
            .collect();
        let mut queries = HashSet::new();
        queries.insert("sparql".to_owned());
        Self {
            parsers,
            serializers,
            storages,
            queries,
        }
    }
}

fn registry() -> &'static Mutex<FactoryRegistry> {
    static REGISTRY: OnceLock<Mutex<FactoryRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(FactoryRegistry::with_builtins()))
}

fn lock_registry() -> std::sync::MutexGuard<'static, FactoryRegistry> {
    registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn normalize(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

/// Registers a built-in parser factory name (ADR-025).
///
/// Known [`Syntax`] names (including aliases accepted by [`Syntax::from_name`])
/// succeed and are recorded under the canonical name. Unknown names return
/// [`Error::Unsupported`]. Re-registration is idempotent.
pub fn register_parser_factory(name: &str) -> Result<()> {
    let syntax = Syntax::from_name(name).map_err(|error| match error {
        Error::Unsupported(message) => Error::Unsupported(format!(
            "parser factory '{name}' is not a baseline built-in: {message}"
        )),
        other => other,
    })?;
    {
        let mut reg = lock_registry();
        reg.parsers.insert(syntax.name().to_owned());
        reg.parsers.insert(normalize(name));
    }
    Ok(())
}

/// Registers a built-in serializer factory name (ADR-025).
pub fn register_serializer_factory(name: &str) -> Result<()> {
    let syntax = Syntax::from_name(name).map_err(|error| match error {
        Error::Unsupported(message) => Error::Unsupported(format!(
            "serializer factory '{name}' is not a baseline built-in: {message}"
        )),
        other => other,
    })?;
    {
        let mut reg = lock_registry();
        reg.serializers.insert(syntax.name().to_owned());
        reg.serializers.insert(normalize(name));
    }
    Ok(())
}

/// Registers a first-party storage factory name (ADR-025).
///
/// Accepts identities from [`supported_backends`], including backends not
/// compiled into the current build. Legacy/plugin names fail observably.
pub fn register_storage_factory(name: &str) -> Result<()> {
    let normalized = normalize(name);
    let canonical = match normalized.as_str() {
        "mem" => "memory",
        other => other,
    };
    if supported_backends().any(|descriptor| descriptor.name == canonical) {
        {
            let mut reg = lock_registry();
            reg.storages.insert(canonical.to_owned());
            if normalized != canonical {
                reg.storages.insert(normalized);
            }
        }
        return Ok(());
    }
    Err(Error::Unsupported(format!(
        "storage factory '{name}' is not a baseline first-party backend; use closed StorageBackend discovery"
    )))
}

/// Registers the baseline SPARQL query factory (ADR-025).
pub fn register_query_factory(name: &str) -> Result<()> {
    let normalized = normalize(name);
    if normalized == "sparql" {
        lock_registry().queries.insert(normalized);
        return Ok(());
    }
    Err(Error::Unsupported(format!(
        "query factory '{name}' is not a baseline built-in; only 'sparql' is registered"
    )))
}

/// Returns whether a parser factory name is registered.
#[must_use]
pub fn parser_factory_registered(name: &str) -> bool {
    let reg = lock_registry();
    let normalized = normalize(name);
    reg.parsers.contains(&normalized)
        || Syntax::from_name(name).is_ok_and(|syntax| reg.parsers.contains(syntax.name()))
}

/// Returns whether a serializer factory name is registered.
#[must_use]
pub fn serializer_factory_registered(name: &str) -> bool {
    let reg = lock_registry();
    let normalized = normalize(name);
    reg.serializers.contains(&normalized)
        || Syntax::from_name(name).is_ok_and(|syntax| reg.serializers.contains(syntax.name()))
}

/// Returns whether a storage factory name is registered.
#[must_use]
pub fn storage_factory_registered(name: &str) -> bool {
    let normalized = normalize(name);
    let canonical = match normalized.as_str() {
        "mem" => "memory".to_owned(),
        other => other.to_owned(),
    };
    let reg = lock_registry();
    reg.storages.contains(&normalized) || reg.storages.contains(&canonical)
}

/// Returns whether a query factory name is registered.
#[must_use]
pub fn query_factory_registered(name: &str) -> bool {
    lock_registry().queries.contains(&normalize(name))
}
