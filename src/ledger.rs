//! The impression ledger: the source of financial truth.
//!
//! Each inserted message counts as one verified, viewable, fraud-free
//! impression for its ad. Accrued spend is `impressions / 1000 * cpm`. The
//! ledger holds raw tallies; spend is computed on top at report time.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::advertisers::Ad;

/// A single row of the live revenue dashboard.
#[derive(Clone, Debug, PartialEq)]
pub struct AdReport {
    pub text: String,
    pub impressions: u64,
    pub cpm: f64,
    pub spend: f64,
}

/// The full, board-deck-ready revenue snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub impressions: u64,
    pub spend: f64,
    pub ads: Vec<AdReport>,
}

/// Thread-safe, in-memory impression tally. Not persisted across restarts.
///
/// Keyed by ad text (copy is the key here, exactly how the Ruby ledger keys by
/// default). Stores raw impression counts plus the cpm seen for each creative.
#[derive(Debug, Default)]
pub struct Ledger {
    inner: Mutex<HashMap<String, (u64, f64)>>,
}

impl Ledger {
    /// Open a fresh, empty book.
    pub fn new() -> Self {
        Ledger {
            inner: Mutex::new(HashMap::new()),
        }
    }

    /// Book one verified impression against a creative.
    pub fn record(&self, ad: &Ad) {
        let mut map = self.inner.lock().expect("ledger poisoned");
        let entry = map.entry(ad.text.clone()).or_insert((0, ad.cpm));
        entry.0 += 1;
        entry.1 = ad.cpm;
    }

    /// Total verified impressions across every surface.
    pub fn total_impressions(&self) -> u64 {
        self.inner
            .lock()
            .expect("ledger poisoned")
            .values()
            .map(|(imp, _)| *imp)
            .sum()
    }

    /// Surface the live revenue dashboard as structured data, spend descending.
    pub fn report(&self) -> Report {
        let map = self.inner.lock().expect("ledger poisoned");
        let mut ads: Vec<AdReport> = map
            .iter()
            .map(|(text, (impressions, cpm))| {
                let spend = round_cents(*impressions as f64 / 1000.0 * cpm);
                AdReport {
                    text: text.clone(),
                    impressions: *impressions,
                    cpm: *cpm,
                    spend,
                }
            })
            .collect();

        ads.sort_by(|a, b| {
            b.spend
                .partial_cmp(&a.spend)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let impressions = ads.iter().map(|a| a.impressions).sum();
        let spend = round_cents(ads.iter().map(|a| a.spend).sum());

        Report {
            impressions,
            spend,
            ads,
        }
    }

    /// Clear the tally. Recognized revenue, unrecognized.
    pub fn reset(&self) {
        self.inner.lock().expect("ledger poisoned").clear();
    }
}

fn round_cents(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_and_totals_impressions() {
        let ledger = Ledger::new();
        let ad = Ad::new("Contoso", 1, 22.0);
        ledger.record(&ad);
        ledger.record(&ad);
        assert_eq!(ledger.total_impressions(), 2);
    }

    #[test]
    fn report_computes_spend() {
        let ledger = Ledger::new();
        let ad = Ad::new("Contoso", 1, 22.0);
        for _ in 0..1000 {
            ledger.record(&ad);
        }
        let report = ledger.report();
        assert_eq!(report.impressions, 1000);
        assert_eq!(report.spend, 22.0, "1000 impressions at 22.0 cpm = 22.0");
    }

    #[test]
    fn report_sorts_by_spend_descending() {
        let ledger = Ledger::new();
        let cheap = Ad::new("Initech", 1, 8.0);
        let dear = Ad::new("Contoso", 1, 22.0);
        for _ in 0..1000 {
            ledger.record(&cheap);
            ledger.record(&dear);
        }
        let report = ledger.report();
        assert_eq!(report.ads[0].text, "Contoso");
        assert_eq!(report.ads[1].text, "Initech");
    }

    #[test]
    fn reset_clears_the_book() {
        let ledger = Ledger::new();
        ledger.record(&Ad::new("Contoso", 1, 22.0));
        ledger.reset();
        assert_eq!(ledger.total_impressions(), 0);
    }
}
