use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, RwLock};

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

/// Process-level configuration, feature registry, and logging (ADR-014).
///
/// Redland requires explicit world initialization. Oxiland resources are RAII
/// managed, so construction is sufficient and shutdown happens on drop.
///
/// `World` is cheap to clone: clones share the same feature registry and log
/// handler. It is `Send` and `Sync`.
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
    features: Arc<RwLock<HashMap<String, FeatureValue>>>,
    min_level: Arc<RwLock<LogLevel>>,
    handler: Arc<Mutex<Option<LogHandler>>>,
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
        self.features
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(iri.into(), value);
    }

    /// Returns a cloned feature value, if configured.
    #[must_use]
    pub fn feature(&self, iri: &str) -> Option<FeatureValue> {
        self.features
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(iri)
            .cloned()
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
}
