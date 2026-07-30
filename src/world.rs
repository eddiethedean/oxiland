use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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

/// Process-level configuration and feature registry.
///
/// Redland requires explicit world initialization. Oxiland resources are RAII
/// managed, so construction is sufficient and shutdown happens on drop.
///
/// `World` is cheap to clone: clones share the same feature registry. It is
/// `Send` and `Sync`.
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
#[derive(Clone, Debug, Default)]
pub struct World {
    features: Arc<RwLock<HashMap<String, FeatureValue>>>,
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
            .expect("world feature lock poisoned")
            .insert(iri.into(), value);
    }

    /// Returns a cloned feature value, if configured.
    #[must_use]
    pub fn feature(&self, iri: &str) -> Option<FeatureValue> {
        self.features
            .read()
            .expect("world feature lock poisoned")
            .get(iri)
            .cloned()
    }
}
