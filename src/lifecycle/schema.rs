use serde::de::DeserializeOwned;

use crate::model::schema::{EntityTypeSchema, Transition};

use super::LifecycleError;

/// Reusable schema parsers; domains can extend the deserialized schema.
pub trait LifecycleSchema: DeserializeOwned {
    fn from_json(text: &str) -> Result<Self, LifecycleError> {
        serde_json::from_str(text).map_err(|error| LifecycleError::InvalidSchema(error.to_string()))
    }

    fn from_toml(text: &str) -> Result<Self, LifecycleError> {
        toml::from_str(text).map_err(|error| LifecycleError::InvalidSchema(error.to_string()))
    }
}

/// Schema-driven defaults retain the kernel's existing undirected edge policy.
pub trait LifecycleDefinition {
    fn states(&self) -> &[String];
    fn transitions(&self) -> &[Transition];

    fn entry_state(&self) -> Option<&str> {
        self.states().first().map(String::as_str)
    }

    fn allows_transition(&self, from: &str, to: &str) -> bool {
        self.transitions().iter().any(|edge| {
            (edge.from == from && edge.to == to) || (edge.from == to && edge.to == from)
        })
    }

    fn allowed_next_states(&self, from: &str) -> Vec<String> {
        let mut states = std::collections::BTreeSet::new();
        for edge in self.transitions() {
            if edge.from == from {
                states.insert(edge.to.clone());
            }
            if edge.to == from {
                states.insert(edge.from.clone());
            }
        }
        states.into_iter().collect()
    }
}

impl LifecycleSchema for EntityTypeSchema {}

impl LifecycleDefinition for EntityTypeSchema {
    fn states(&self) -> &[String] {
        &self.states
    }

    fn transitions(&self) -> &[Transition] {
        &self.transitions
    }
}
