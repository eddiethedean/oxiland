use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

use crate::Result;
use crate::factory;

/// A Redland-style feature value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FeatureValue {
    /// Textual feature value.
    String(String),
    /// Integer feature value.
    Integer(i64),
    /// Boolean feature value.
    Boolean(bool),
}

/// Shared IRI-keyed feature registry used by [`World`], models, parsers, and
/// serializers.
#[derive(Clone, Default)]
pub struct FeatureMap {
    inner: Arc<RwLock<HashMap<String, FeatureValue>>>,
}

impl fmt::Debug for FeatureMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let map = self
            .inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        f.debug_map().entries(map.iter()).finish()
    }
}

impl FeatureMap {
    /// Creates an empty feature map.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a feature identified by an IRI.
    pub fn set(&self, iri: impl Into<String>, value: FeatureValue) {
        self.inner
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(iri.into(), value);
    }

    /// Returns a cloned feature value, if configured.
    #[must_use]
    pub fn get(&self, iri: &str) -> Option<FeatureValue> {
        self.inner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(iri)
            .cloned()
    }
}

/// Feature registry used for Redland `librdf_storage_get/set_feature` mapping.
pub type StorageFeatures = FeatureMap;

/// Log severity for [`World`] logging (ADR-014).
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub enum LogLevel {
    /// Debug diagnostics.
    Debug,
    /// Informational messages.
    Info,
    /// Recoverable problems.
    #[default]
    Warn,
    /// Failures.
    Error,
}

impl LogLevel {
    /// Canonical lowercase name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

/// Logical log facility (ADR-014).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LogFacility {
    /// General / uncategorized.
    General,
    /// Model and storage operations.
    Model,
    /// Parser / serializer.
    Io,
    /// SPARQL query and update.
    Query,
    /// Utility helpers.
    Utility,
}

impl LogFacility {
    /// Canonical lowercase name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Model => "model",
            Self::Io => "io",
            Self::Query => "query",
            Self::Utility => "utility",
        }
    }
}

/// One log record delivered to handlers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LogRecord {
    /// Severity.
    pub level: LogLevel,
    /// Facility.
    pub facility: LogFacility,
    /// Message text.
    pub message: String,
}

impl fmt::Display for LogRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} {}] {}",
            self.level.name(),
            self.facility.name(),
            self.message
        )
    }
}

type LogHandler = Arc<dyn Fn(&LogRecord) + Send + Sync>;

/// Opaque embedding bridge token (ADR-026).
///
/// Safe Rust stores the value without dereferencing it. C ABI maps these to
/// `void *` handles for Raptor/Rasqal embedding parity.
pub type BridgeToken = usize;

/// Process-level configuration, feature registry, and logging (ADR-014).
///
/// Redland requires explicit world initialization. Oxiland resources are RAII
/// managed, so construction is sufficient and shutdown happens on drop.
///
/// `World` is cheap to clone: clones share the same feature registry, minimum
/// log level, log handler, and opaque bridge tokens. It is `Send` and `Sync`.
///
/// When the `tracing` Cargo feature is enabled, [`World::log`] also emits
/// `tracing` events, gated by the same minimum log level as the handler.
///
/// # Examples
///
/// ```
/// use oxiland::{FeatureValue, World};
///
/// let world = World::new();
/// world.set_feature("http://example.com/feature", FeatureValue::Boolean(true));
/// assert_eq!(
///     world.feature("http://example.com/feature"),
///     Some(FeatureValue::Boolean(true))
/// );
/// ```
#[derive(Clone, Default)]
pub struct World {
    features: FeatureMap,
    min_level: Arc<RwLock<LogLevel>>,
    handler: Arc<Mutex<Option<LogHandler>>>,
    raptor: Arc<RwLock<Option<BridgeToken>>>,
    raptor_init: Arc<RwLock<Option<BridgeToken>>>,
    rasqal: Arc<RwLock<Option<BridgeToken>>>,
    rasqal_init: Arc<RwLock<Option<BridgeToken>>>,
}

impl fmt::Debug for World {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("World")
            .field(
                "min_level",
                &*self
                    .min_level
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            )
            .field(
                "handler_set",
                &self
                    .handler
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .is_some(),
            )
            .field("raptor_set", &self.raptor().is_some())
            .field("rasqal_set", &self.rasqal().is_some())
            .finish_non_exhaustive()
    }
}

impl World {
    /// Creates an initialized world.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a feature identified by an IRI.
    pub fn set_feature(&self, iri: impl Into<String>, value: FeatureValue) {
        self.features.set(iri, value);
    }

    /// Returns a cloned feature value, if configured.
    #[must_use]
    pub fn feature(&self, iri: &str) -> Option<FeatureValue> {
        self.features.get(iri)
    }

    /// Sets the minimum level that will be delivered to the handler.
    pub fn set_log_level(&self, level: LogLevel) {
        *self
            .min_level
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = level;
    }

    /// Returns the configured minimum log level.
    #[must_use]
    pub fn log_level(&self) -> LogLevel {
        *self
            .min_level
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Registers a log handler. Replaces any previous handler.
    ///
    /// Handlers are invoked synchronously in call order of [`World::log`]. When
    /// multiple logical callbacks are needed, compose them in a single handler.
    pub fn set_log_handler<F>(&self, handler: F)
    where
        F: Fn(&LogRecord) + Send + Sync + 'static,
    {
        *self
            .handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Arc::new(handler));
    }

    /// Clears the log handler.
    pub fn clear_log_handler(&self) {
        *self
            .handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    }

    /// Emits a log record when `level` is at least the configured minimum.
    pub fn log(&self, level: LogLevel, facility: LogFacility, message: impl Into<String>) {
        if level < self.log_level() {
            return;
        }
        let record = LogRecord {
            level,
            facility,
            message: message.into(),
        };
        #[cfg(feature = "tracing")]
        {
            match level {
                LogLevel::Debug => {
                    tracing::debug!(facility = record.facility.name(), "{}", record.message)
                }
                LogLevel::Info => {
                    tracing::info!(facility = record.facility.name(), "{}", record.message)
                }
                LogLevel::Warn => {
                    tracing::warn!(facility = record.facility.name(), "{}", record.message)
                }
                LogLevel::Error => {
                    tracing::error!(facility = record.facility.name(), "{}", record.message)
                }
            }
        }
        if let Some(handler) = self
            .handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .cloned()
        {
            handler(&record);
        }
    }

    /// Stores an opaque Raptor world bridge token (ADR-026).
    ///
    /// Prefer this over raw pointers: the safe crate never dereferences the
    /// token. C ABI maps it to `void *`.
    pub fn set_raptor(&self, token: Option<BridgeToken>) {
        *self
            .raptor
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = token;
    }

    /// Returns the opaque Raptor bridge token, if set.
    #[must_use]
    pub fn raptor(&self) -> Option<BridgeToken> {
        *self
            .raptor
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Alias for [`World::set_raptor`].
    pub fn set_raptor_bridge(&self, token: Option<BridgeToken>) {
        self.set_raptor(token);
    }

    /// Alias for [`World::raptor`].
    #[must_use]
    pub fn raptor_bridge(&self) -> Option<BridgeToken> {
        self.raptor()
    }

    /// Stores an opaque Raptor init-handler token (ADR-026).
    pub fn set_raptor_init_handler(&self, token: Option<BridgeToken>) {
        *self
            .raptor_init
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = token;
    }

    /// Returns the opaque Raptor init-handler token, if set.
    #[must_use]
    pub fn raptor_init_handler(&self) -> Option<BridgeToken> {
        *self
            .raptor_init
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Stores an opaque Rasqal world bridge token (ADR-026).
    pub fn set_rasqal(&self, token: Option<BridgeToken>) {
        *self
            .rasqal
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = token;
    }

    /// Returns the opaque Rasqal bridge token, if set.
    #[must_use]
    pub fn rasqal(&self) -> Option<BridgeToken> {
        *self
            .rasqal
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Stores an opaque Rasqal init-handler token (ADR-026).
    pub fn set_rasqal_init_handler(&self, token: Option<BridgeToken>) {
        *self
            .rasqal_init
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = token;
    }

    /// Returns the opaque Rasqal init-handler token, if set.
    #[must_use]
    pub fn rasqal_init_handler(&self) -> Option<BridgeToken> {
        *self
            .rasqal_init
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Registers a baseline parser factory (ADR-025).
    pub fn register_parser_factory(&self, name: &str) -> Result<()> {
        factory::register_parser_factory(name)
    }

    /// Registers a baseline serializer factory (ADR-025).
    pub fn register_serializer_factory(&self, name: &str) -> Result<()> {
        factory::register_serializer_factory(name)
    }

    /// Registers a baseline storage factory (ADR-025).
    pub fn register_storage_factory(&self, name: &str) -> Result<()> {
        factory::register_storage_factory(name)
    }

    /// Registers a baseline query factory (ADR-025).
    pub fn register_query_factory(&self, name: &str) -> Result<()> {
        factory::register_query_factory(name)
    }
}
