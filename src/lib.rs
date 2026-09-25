//! # SponsoredLogs
//!
//! The world's first Log-Native Advertising Platform for Rust. For decades your
//! `tracing` output has been a pure cost center: emitted once, grepped never,
//! archived into oblivion. SponsoredLogs transforms your log stream into a
//! high-margin, programmatic revenue channel by inserting host-read sponsor
//! messages between your own log lines.
//!
//! Activation is opt-in, because consent is our moat. In Rust we cannot (and
//! would never) silently hijack your `println!`. Instead you add our
//! [`SponsoredLayer`] to your `tracing` subscriber and begin your monetization
//! journey:
//!
//! ```no_run
//! use tracing_subscriber::prelude::*;
//!
//! let exchange = sponsored_logs::layer();
//!
//! tracing_subscriber::registry()
//!     .with(tracing_subscriber::fmt::layer())
//!     .with(exchange.clone())
//!     .init();
//!
//! tracing::info!("service started");
//! // ... roughly 1 in 1000 log calls now carries a premium sponsor placement.
//!
//! let report = exchange.report();
//! println!("booked {} impressions, ${} spend", report.impressions, report.spend);
//! ```

pub mod advertisers;
mod layer;
pub mod ledger;

pub use advertisers::Ad;
pub use layer::SponsoredLayer;
pub use ledger::{AdReport, Ledger, Report};

use rand::distributions::{Distribution, WeightedIndex};
use rand::Rng;

/// How the exchange samples which creative fills a slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Selection {
    /// Pick by each ad's `weight`. Fill rate is king.
    #[default]
    Weight,
    /// Pick by each ad's `cpm`. The highest bidder wins more inventory.
    Cpm,
}

/// Enterprise-grade, self-serve campaign controls.
#[derive(Clone, Debug)]
pub struct Config {
    /// Fraction (0.0..=1.0) of intercepted log calls that carry an ad.
    pub probability: f64,
    /// The demand pool.
    pub ads: Vec<Ad>,
    /// How the pool is sampled.
    pub selection: Selection,
    /// Tag prepended to each message. Blank omits it.
    pub ad_prefix: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            probability: 0.001,
            ads: advertisers::default_pool(),
            selection: Selection::Weight,
            ad_prefix: "[AD]".to_string(),
        }
    }
}

/// Build the exchange with the default book and a conservative, brand-safe
/// 1-in-1000 fill rate.
pub fn layer() -> SponsoredLayer {
    SponsoredLayer::new(Config::default())
}

/// Build the exchange from a custom [`Config`]. Bring your own demand.
pub fn layer_with(config: Config) -> SponsoredLayer {
    SponsoredLayer::new(config)
}

/// Format one placement line, with the configured prefix.
pub(crate) fn render(ad: &Ad, prefix: &str) -> String {
    if prefix.is_empty() {
        ad.text.clone()
    } else {
        format!("{} {}", prefix, ad.text)
    }
}

/// The real-time, deterministic yield-optimization engine ("the exchange").
///
/// Stage two of selection: given an eligible pool, pick which creative fills the
/// slot. Returns `None` only when the pool is empty or every weight is zero.
pub(crate) fn pick<'a, R: Rng + ?Sized>(
    ads: &'a [Ad],
    selection: Selection,
    rng: &mut R,
) -> Option<&'a Ad> {
    if ads.is_empty() {
        return None;
    }

    // Choose the metric; :cpm falls back to weight when all cpm are zero,
    // because fill rate is king.
    let weights: Vec<f64> = match selection {
        Selection::Weight => ads.iter().map(|a| a.weight as f64).collect(),
        Selection::Cpm => {
            let cpm_sum: f64 = ads.iter().map(|a| a.cpm).sum();
            if cpm_sum > 0.0 {
                ads.iter().map(|a| a.cpm).collect()
            } else {
                ads.iter().map(|a| a.weight as f64).collect()
            }
        }
    };

    if weights.iter().sum::<f64>() <= 0.0 {
        return None;
    }

    let dist = WeightedIndex::new(&weights).ok()?;
    Some(&ads[dist.sample(rng)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn render_includes_prefix() {
        let ad = Ad::new("Contoso", 1, 22.0);
        assert_eq!(render(&ad, "[AD]"), "[AD] Contoso");
    }

    #[test]
    fn render_omits_blank_prefix() {
        let ad = Ad::new("Contoso", 1, 22.0);
        assert_eq!(render(&ad, ""), "Contoso");
    }

    #[test]
    fn pick_returns_none_for_empty_pool() {
        let mut rng = StdRng::seed_from_u64(1);
        assert!(pick(&[], Selection::Weight, &mut rng).is_none());
    }

    #[test]
    fn pick_returns_none_when_all_weights_zero() {
        let mut rng = StdRng::seed_from_u64(1);
        let ads = vec![Ad::new("a", 0, 0.0), Ad::new("b", 0, 0.0)];
        assert!(pick(&ads, Selection::Weight, &mut rng).is_none());
    }

    #[test]
    fn pick_never_chooses_zero_weight() {
        let mut rng = StdRng::seed_from_u64(42);
        let ads = vec![Ad::new("skip", 0, 0.0), Ad::new("keep", 5, 0.0)];
        for _ in 0..100 {
            assert_eq!(
                pick(&ads, Selection::Weight, &mut rng).unwrap().text,
                "keep"
            );
        }
    }

    #[test]
    fn cpm_selection_falls_back_to_weight_when_all_cpm_zero() {
        let mut rng = StdRng::seed_from_u64(7);
        let ads = vec![Ad::new("only", 3, 0.0)];
        assert_eq!(pick(&ads, Selection::Cpm, &mut rng).unwrap().text, "only");
    }

    #[test]
    fn cpm_selection_favors_the_highest_bidder() {
        let mut rng = StdRng::seed_from_u64(123);
        let ads = vec![Ad::new("cheap", 1, 1.0), Ad::new("dear", 1, 1000.0)];
        let mut dear = 0;
        for _ in 0..1000 {
            if pick(&ads, Selection::Cpm, &mut rng).unwrap().text == "dear" {
                dear += 1;
            }
        }
        assert!(
            dear > 900,
            "highest bidder should win most inventory, got {dear}"
        );
    }
}
