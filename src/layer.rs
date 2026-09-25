//! The patent-pending insertion architecture.
//!
//! A [`tracing_subscriber::Layer`] that rides alongside your existing telemetry
//! with negligible overhead. Each intercepted event runs normally, then the
//! exchange consults the configured probability and, on a hit, books an
//! impression and emits a sponsor placement into the same log stream.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rand::Rng;
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::ledger::{Ledger, Report};
use crate::{pick, render, Config};

/// The exchange. Clone it freely: clones share one ledger and one on/off flag,
/// so `report()` from any handle sees the whole book.
#[derive(Clone)]
pub struct SponsoredLayer {
    inner: Arc<Inner>,
}

struct Inner {
    config: Config,
    ledger: Ledger,
    active: AtomicBool,
}

impl SponsoredLayer {
    /// Stand up the exchange from a [`Config`]. Active on creation.
    pub fn new(config: Config) -> Self {
        SponsoredLayer {
            inner: Arc::new(Inner {
                config,
                ledger: Ledger::new(),
                active: AtomicBool::new(true),
            }),
        }
    }

    /// Resume monetization. Placements ship again on the configured fill rate.
    pub fn sponsor(&self) {
        self.inner.active.store(true, Ordering::Relaxed);
    }

    /// Pause the revenue firehose. The layer stays resident but inert.
    pub fn unsponsor(&self) {
        self.inner.active.store(false, Ordering::Relaxed);
    }

    /// Whether the exchange is currently filling inventory.
    pub fn active(&self) -> bool {
        self.inner.active.load(Ordering::Relaxed)
    }

    /// The live revenue dashboard as structured data.
    pub fn report(&self) -> Report {
        self.inner.ledger.report()
    }

    /// Total verified, viewable, fraud-free impressions booked.
    pub fn impressions(&self) -> u64 {
        self.inner.ledger.total_impressions()
    }

    /// Clear the tally.
    pub fn reset_ledger(&self) {
        self.inner.ledger.reset();
    }

    /// The core auction. Rolls the fill dice; on a hit books an impression and
    /// returns the rendered placement line. Separated from the tracing plumbing
    /// so it can be tested without a subscriber.
    fn maybe_fill<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<String> {
        if !self.active() {
            return None;
        }

        // Stage one: whether to show a message at all.
        if rng.gen::<f64>() >= self.inner.config.probability {
            return None;
        }

        // Stage two: which creative fills the slot.
        let ad = pick(&self.inner.config.ads, self.inner.config.selection, rng)?;
        self.inner.ledger.record(ad);
        Some(render(
            ad,
            &self.inner.config.ad_prefix,
            self.inner.config.ascii_only,
        ))
    }
}

impl<S: Subscriber> Layer<S> for SponsoredLayer {
    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut rng = rand::thread_rng();
        if let Some(placement) = self.maybe_fill(&mut rng) {
            // Emit the placement into the same stream, at the same level the
            // audience is already reading, at their moment of peak attention.
            //
            tracing::info!(sponsored = true, "{placement}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::advertisers::Ad;
    use crate::Selection;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn always_on_config() -> Config {
        Config {
            probability: 1.0,
            ads: vec![Ad::new("Contoso", 1, 22.0)],
            selection: Selection::Weight,
            ad_prefix: "[AD]".to_string(),
            ascii_only: false,
        }
    }

    #[test]
    fn fills_and_books_when_probability_is_one() {
        let layer = SponsoredLayer::new(always_on_config());
        let mut rng = StdRng::seed_from_u64(1);
        let placement = layer.maybe_fill(&mut rng);
        assert_eq!(placement, Some("[AD] Contoso".to_string()));
        assert_eq!(layer.impressions(), 1);
    }

    #[test]
    fn never_fills_at_zero_probability() {
        let mut config = always_on_config();
        config.probability = 0.0;
        let layer = SponsoredLayer::new(config);
        let mut rng = StdRng::seed_from_u64(1);
        for _ in 0..1000 {
            assert!(layer.maybe_fill(&mut rng).is_none());
        }
        assert_eq!(layer.impressions(), 0);
    }

    #[test]
    fn unsponsor_pauses_the_firehose() {
        let layer = SponsoredLayer::new(always_on_config());
        layer.unsponsor();
        let mut rng = StdRng::seed_from_u64(1);
        assert!(layer.maybe_fill(&mut rng).is_none());
        assert!(!layer.active());

        layer.sponsor();
        assert!(layer.active());
        assert!(layer.maybe_fill(&mut rng).is_some());
    }

    #[test]
    fn clones_share_one_ledger() {
        let layer = SponsoredLayer::new(always_on_config());
        let handle = layer.clone();
        let mut rng = StdRng::seed_from_u64(1);
        layer.maybe_fill(&mut rng);
        assert_eq!(handle.impressions(), 1, "clone sees the same book");
    }

    #[test]
    fn reset_clears_the_book() {
        let layer = SponsoredLayer::new(always_on_config());
        let mut rng = StdRng::seed_from_u64(1);
        layer.maybe_fill(&mut rng);
        layer.reset_ledger();
        assert_eq!(layer.impressions(), 0);
    }
}
