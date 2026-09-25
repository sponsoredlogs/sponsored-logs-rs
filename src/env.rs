//! Activation via the environment.
//!
//! Set `SPONSORED_LOGS` to onboard at startup without changing code, then layer
//! on `SPONSORED_LOGS_*` overrides. Environment activation and manual activation
//! coexist: reading the env never replaces the [`crate::layer`] API.

use crate::{Config, Selection};

/// Recognized truthy values for boolean env vars (case-insensitive).
const TRUTHY: [&str; 4] = ["1", "true", "yes", "on"];

fn truthy(value: &str) -> bool {
    TRUTHY.contains(&value.trim().to_ascii_lowercase().as_str())
}

/// Whether `SPONSORED_LOGS` requests activation, reading the real process env.
pub fn activate() -> bool {
    activate_with(|k| std::env::var(k).ok())
}

/// Build a [`Config`] from `SPONSORED_LOGS_*`, reading the real process env.
/// Only variables actually present override the defaults, so an unset var leaves
/// its default untouched.
pub fn config() -> Config {
    config_with(|k| std::env::var(k).ok())
}

/// [`activate`], but against a caller-supplied lookup. Testable without touching
/// global process state.
pub fn activate_with(get: impl Fn(&str) -> Option<String>) -> bool {
    get("SPONSORED_LOGS")
        .as_deref()
        .map(truthy)
        .unwrap_or(false)
}

/// [`config`], but against a caller-supplied lookup.
pub fn config_with(get: impl Fn(&str) -> Option<String>) -> Config {
    let mut config = Config::default();

    if let Some(v) = get("SPONSORED_LOGS_PROBABILITY") {
        if let Ok(p) = v.trim().parse::<f64>() {
            config.probability = p;
        }
    }
    if let Some(v) = get("SPONSORED_LOGS_PREFIX") {
        config.ad_prefix = v;
    }
    if let Some(v) = get("SPONSORED_LOGS_SELECTION") {
        config.selection = Selection::from_env_str(&v);
    }
    if let Some(v) = get("SPONSORED_LOGS_ASCII_ONLY") {
        config.ascii_only = truthy(&v);
    }

    config
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn lookup(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k: &str| map.get(k).cloned()
    }

    #[test]
    fn activate_honors_truthy_values() {
        for v in ["1", "true", "YES", " on "] {
            assert!(
                activate_with(lookup(&[("SPONSORED_LOGS", v)])),
                "{v:?} is truthy"
            );
        }
    }

    #[test]
    fn activate_is_false_when_unset_or_falsey() {
        assert!(!activate_with(lookup(&[])));
        assert!(!activate_with(lookup(&[("SPONSORED_LOGS", "0")])));
        assert!(!activate_with(lookup(&[("SPONSORED_LOGS", "nope")])));
    }

    #[test]
    fn unset_vars_leave_defaults_untouched() {
        let config = config_with(lookup(&[]));
        let default = Config::default();
        assert_eq!(config.probability, default.probability);
        assert_eq!(config.ad_prefix, default.ad_prefix);
        assert_eq!(config.selection, default.selection);
        assert!(!config.ascii_only);
    }

    #[test]
    fn present_vars_override_defaults() {
        let config = config_with(lookup(&[
            ("SPONSORED_LOGS_PROBABILITY", "0.25"),
            ("SPONSORED_LOGS_PREFIX", "SPONSORED:"),
            ("SPONSORED_LOGS_SELECTION", "cpm"),
            ("SPONSORED_LOGS_ASCII_ONLY", "true"),
        ]));
        assert_eq!(config.probability, 0.25);
        assert_eq!(config.ad_prefix, "SPONSORED:");
        assert_eq!(config.selection, Selection::Cpm);
        assert!(config.ascii_only);
    }

    #[test]
    fn unparseable_probability_keeps_the_default() {
        let config = config_with(lookup(&[("SPONSORED_LOGS_PROBABILITY", "banana")]));
        assert_eq!(config.probability, Config::default().probability);
    }

    #[test]
    fn unknown_selection_settles_to_weight() {
        let config = config_with(lookup(&[("SPONSORED_LOGS_SELECTION", "auction")]));
        assert_eq!(config.selection, Selection::Weight);
    }
}
