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
    /// Resolved once at construction: whether to gild the `[AD]` tag. The TTY
    /// and `NO_COLOR` state are read here, not per-event, so a hot log path
    /// never touches the environment.
    gild: bool,
}

impl SponsoredLayer {
    /// Stand up the exchange from a [`Config`]. Active on creation.
    pub fn new(config: Config) -> Self {
        let gild = crate::color::gild(config.color);
        SponsoredLayer {
            inner: Arc::new(Inner {
                config,
                ledger: Ledger::new(),
                active: AtomicBool::new(true),
                gild,
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
    /// returns the rendered [`Placement`]. Separated from the tracing plumbing
    /// so it can be tested without a subscriber.
    fn maybe_fill<R: Rng + ?Sized>(&self, rng: &mut R) -> Option<Placement> {
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
        let text = render(
            ad,
            &self.inner.config.ad_prefix,
            self.inner.config.ascii_only,
            self.inner.gild,
        );
        Some(Placement {
            text,
            is_banner: ad.format == crate::Format::Banner,
            gilded: self.inner.gild,
        })
    }
}

/// One rendered placement, plus whether it is a multi-line banner unit and
/// whether it carries gilding. The emit path branches on both: a banner needs
/// its own blank-line framing, and any gilded placement must travel as a
/// `Display` field, since the fmt subscriber escapes control characters in the
/// event message and would print our ANSI as the literal text `\x1b`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Placement {
    text: String,
    is_banner: bool,
    gilded: bool,
}

impl<S: Subscriber> Layer<S> for SponsoredLayer {
    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        let mut rng = rand::thread_rng();
        let Some(placement) = self.maybe_fill(&mut rng) else {
            return;
        };

        // Gilded placements carry raw ANSI, which the fmt subscriber escapes in
        // the message but preserves in a Display field. So anything gilded is
        // emitted as the `sponsored` field; plain placements keep the cleaner
        // message path. Banners frame themselves with leading/trailing blank
        // lines so the top border starts at column zero either way.
        //
        match (placement.is_banner, placement.gilded) {
            (true, true) => tracing::info!(sponsored = %format!("\n{}\n", placement.text)),
            (true, false) => tracing::info!("\n{}\n", placement.text),
            (false, true) => tracing::info!(sponsored = %placement.text),
            (false, false) => tracing::info!(sponsored = true, "{}", placement.text),
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
            color: crate::Color::Never,
        }
    }

    #[test]
    fn fills_and_books_when_probability_is_one() {
        let layer = SponsoredLayer::new(always_on_config());
        let mut rng = StdRng::seed_from_u64(1);
        let placement = layer.maybe_fill(&mut rng).expect("should fill");
        assert_eq!(placement.text, "[AD] Contoso");
        assert!(!placement.is_banner, "a line creative is not a banner");
        assert!(!placement.gilded, "Color::Never does not gild");
        assert_eq!(layer.impressions(), 1);
    }

    #[test]
    fn gilded_placement_carries_a_real_escape_byte_and_is_flagged() {
        // Regression: the fmt subscriber escapes control chars in the message,
        // so a gilded ad must be flagged (to route through the Display field
        // path) and must carry a real ESC byte, not the literal text "\x1b".
        let config = Config {
            probability: 1.0,
            ads: vec![Ad::new("Contoso", 1, 22.0)],
            selection: Selection::Weight,
            ad_prefix: "[AD]".to_string(),
            ascii_only: false,
            color: crate::Color::Always,
        };
        let layer = SponsoredLayer::new(config);
        let mut rng = StdRng::seed_from_u64(1);
        let placement = layer.maybe_fill(&mut rng).expect("should fill");
        assert!(placement.gilded, "Color::Always gilds");
        assert!(
            placement.text.contains('\u{1b}'),
            "carries a real ESC byte, not the literal string backslash-x-1-b"
        );
        assert!(
            !placement.text.contains("\\x1b"),
            "must not contain the literal escape text"
        );
    }

    #[test]
    fn banner_creatives_are_flagged_for_the_banner_emit_path() {
        use crate::Box;
        let config = Config {
            probability: 1.0,
            ads: vec![Ad::new("Own the viewport.", 1, 0.0).banner(Box::Double)],
            selection: Selection::Weight,
            ad_prefix: "[AD]".to_string(),
            ascii_only: false,
            color: crate::Color::Never,
        };
        let layer = SponsoredLayer::new(config);
        let mut rng = StdRng::seed_from_u64(1);
        let placement = layer.maybe_fill(&mut rng).expect("should fill");
        assert!(
            placement.is_banner,
            "a banner creative routes to the banner path"
        );
        assert!(placement.text.starts_with("╔═ [AD] "));
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
